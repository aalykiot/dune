use crate::errors::generic_error;
use crate::modules::ModulePath;
use crate::modules::ModuleSource;
use crate::modules::CORE_MODULES;
use crate::typescript;
use anyhow::bail;
use anyhow::Result;
use colored::*;
use lazy_static::lazy_static;
use path_absolutize::*;
use regex::Regex;
use sha::sha1::Sha1;
use sha::utils::Digest;
use sha::utils::DigestExt;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

/// Defines the interface of a module loader.
pub trait ModuleLoader {
    fn load(&self, specifier: &str) -> Result<ModuleSource>;
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath>;
}

static EXTENSIONS: &[&str] = &["js", "ts", "tsx", "json"];

#[derive(Default)]
pub struct FsModuleLoader;

impl FsModuleLoader {
    /// Transforms PathBuf into String.
    fn transform(&self, path: PathBuf) -> String {
        path.into_os_string().into_string().unwrap()
    }

    /// Checks if path is a JSON file.
    fn is_json_import(&self, path: &Path) -> bool {
        match path.extension() {
            Some(value) => value == "json",
            None => false,
        }
    }

    /// Wraps JSON data into an ES module (using v8's built in objects).
    fn wrap_json(&self, source: &str) -> String {
        format!("export default JSON.parse(`{}`);", source)
    }

    /// Loads contents from a file.
    fn load_source(&self, path: &Path) -> Result<ModuleSource> {
        let source = fs::read_to_string(path)?;
        let source = match self.is_json_import(path) {
            true => self.wrap_json(source.as_str()),
            false => source,
        };

        Ok(source)
    }

    /// Loads import as file.
    fn load_as_file(&self, path: &Path) -> Result<ModuleSource> {
        // 1. Check if path is already a valid file.
        if path.is_file() {
            return self.load_source(path);
        }

        // 2. Check if we need to add an extension.
        if path.extension().is_none() {
            for ext in EXTENSIONS {
                let path = &path.with_extension(ext);
                if path.is_file() {
                    return self.load_source(path);
                }
            }
        }

        // 3. Bail out with an error.
        let err_message = format!("Module not found \"{}\"", path.display());
        bail!(generic_error(err_message));
    }

    /// Loads import as directory using the 'index.[ext]' convention.
    fn load_as_directory(&self, path: &Path) -> Result<ModuleSource> {
        for ext in EXTENSIONS {
            let path = &path.join(format!("index.{}", ext));
            if path.is_file() {
                return self.load_source(path);
            }
        }
        let err_message = format!("Module not found \"{}\"", path.display());
        bail!(generic_error(err_message));
    }
}

impl ModuleLoader for FsModuleLoader {
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath> {
        // Windows platform full path regex.
        lazy_static! {
            static ref WINDOWS_REGEX: Regex = Regex::new(r"^[a-zA-Z]:\\").unwrap();
        }

        // Resolve absolute import.
        if specifier.starts_with('/') || WINDOWS_REGEX.is_match(specifier) {
            return Ok(self.transform(Path::new(specifier).absolutize()?.to_path_buf()));
        }

        // Resolve relative import.
        let cwd = &env::current_dir().unwrap();
        let base = base.map(|v| Path::new(v).parent().unwrap()).unwrap_or(cwd);

        if specifier.starts_with("./") || specifier.starts_with("../") {
            return Ok(self.transform(base.join(specifier).absolutize()?.to_path_buf()));
        }

        bail!(generic_error(format!("Module not found \"{}\"", specifier)))
    }

    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Load source.
        let path = Path::new(specifier);
        let maybe_source = self
            .load_as_file(path)
            .or_else(|_| self.load_as_directory(path));

        // Append default extension (if none specified).
        let path = match path.extension() {
            Some(_) => path.into(),
            None => path.with_extension("js"),
        };

        let source = match maybe_source {
            Ok(source) => source,
            Err(_) => bail!(generic_error(format!(
                "Module not found \"{}\"",
                path.display()
            ))),
        };

        let path_extension = path.extension().unwrap();

        // Check if it's a TypeScript import.
        match path_extension == "ts" || path_extension == "tsx" {
            true => typescript::transpile(path.to_str(), &source)
                .map_err(|e| generic_error(e.to_string())),
            false => Ok(source),
        }
    }
}

static DUNE_ROOT: &str = ".dune";
static DUNE_CACHE_DIR: &str = "cache";

#[derive(Default)]
/// Loader supporting URL imports.
pub struct UrlModuleLoader {
    // Ignores the cache and re-downloads the dependency.
    pub skip_cache: bool,
}

impl ModuleLoader for UrlModuleLoader {
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath> {
        // 1. Check if specifier is a valid URL.
        if let Ok(url) = Url::parse(specifier) {
            return Ok(url.into());
        }

        // 2. Check if the requester is a valid URL.
        if let Some(base) = base {
            if let Ok(base) = Url::parse(base) {
                let options = Url::options();
                let url = options.base_url(Some(&base));
                let url = url.parse(specifier)?;

                return Ok(url.as_str().to_string());
            }
        }

        // Possibly unreachable error.
        bail!(generic_error("Base is not a valid URL"));
    }

    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Create a .cache directory.
        let cache_dir = &dirs::home_dir()
            .unwrap()
            .join(DUNE_ROOT)
            .join(DUNE_CACHE_DIR);

        if fs::create_dir_all(&cache_dir).is_err() {
            bail!(generic_error("Failed to create module caching directory"))
        }

        // Hash URL using sha1.
        let hash = Sha1::default().digest(specifier.as_bytes()).to_hex();
        let module_path = cache_dir.join(&hash);

        if !self.skip_cache {
            // Check cache, and load file.
            if module_path.is_file() {
                let source = fs::read_to_string(&module_path).unwrap();
                return Ok(source);
            }
        }

        println!("{} {}", "Downloading".green(), specifier);

        // Download file and, save it to cache.
        let source = match ureq::get(specifier).call()?.into_string() {
            Ok(source) => source,
            Err(_) => bail!(generic_error(format!("Module not found \"{}\"", specifier))),
        };

        // Check if it's a TypeScript import.
        let source = match specifier.ends_with(".ts") || specifier.ends_with(".tsx") {
            true => typescript::transpile(Some(specifier), &source)?,
            false => source,
        };

        fs::write(&module_path, &source)?;

        Ok(source)
    }
}

#[derive(Default)]
pub struct CoreModuleLoader;

impl ModuleLoader for CoreModuleLoader {
    fn resolve(&self, _: Option<&str>, specifier: &str) -> Result<ModulePath> {
        match CORE_MODULES.get(specifier) {
            Some(_) => Ok(specifier.to_string()),
            None => bail!(generic_error(format!("Module not found \"{}\"", specifier))),
        }
    }
    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Since any errors will be caught at the resolve stage, we can
        // go ahead an unwrap the value with no worries.
        Ok(CORE_MODULES.get(specifier).unwrap().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_resolve_fs_imports() {
        // Tests to run later on.
        let tests = vec![
            (
                None,
                "/dev/core/tests/005_more_imports.js",
                "/dev/core/tests/005_more_imports.js",
            ),
            (
                Some("/dev/core/tests/005_more_imports.js"),
                "./006_more_imports.js",
                "/dev/core/tests/006_more_imports.js",
            ),
            (
                Some("/dev/core/tests/005_more_imports.js"),
                "../006_more_imports.js",
                "/dev/core/006_more_imports.js",
            ),
            (
                Some("/dev/core/tests/005_more_imports.js"),
                "/dev/core/tests/006_more_imports.js",
                "/dev/core/tests/006_more_imports.js",
            ),
        ];

        // Run tests.
        let loader = FsModuleLoader::default();

        for (base, specifier, expected) in tests {
            let path = loader.resolve(base, specifier).unwrap();
            let expected = if cfg!(target_os = "windows") {
                String::from(Path::new(expected).absolutize().unwrap().to_str().unwrap())
            } else {
                expected.into()
            };

            assert_eq!(path, expected);
        }
    }

    #[test]
    fn test_load_fs_imports() {
        // Crate temp dir.
        let temp_dir = assert_fs::TempDir::new().unwrap();

        const SRC: &str = r"
            export function sayHello() {
                console.log('Hello, World!');
            }
        ";

        let source_files = vec![
            "./core/tests/005_more_imports.js",
            "./core/tests/006_more_imports/index.js",
        ];

        // Create source files.
        source_files.iter().for_each(|file| {
            let path = Path::new(file);
            let path = temp_dir.child(path);

            path.touch().unwrap();
            fs::write(path, SRC).unwrap();
        });

        // Group of tests to be run.
        let tests = vec![
            "./core/tests/005_more_imports",
            "./core/tests/005_more_imports.js",
            "./core/tests/006_more_imports/",
        ];

        // Run tests.
        let loader = FsModuleLoader::default();

        for specifier in tests {
            let path = format!("{}", temp_dir.child(specifier).display());
            let source = loader.load(&path);

            assert!(source.is_ok());
            assert_eq!(source.unwrap(), SRC);
        }
    }

    #[test]
    fn test_resolve_url_imports() {
        // Group of tests to be run.
        let tests = vec![
            (
                None,
                "http://github.com/x/core/tests/006_url_imports.js",
                "http://github.com/x/core/tests/006_url_imports.js",
            ),
            (
                Some("http://github.com/x/core/tests/006_url_imports.js"),
                "./005_more_imports.js",
                "http://github.com/x/core/tests/005_more_imports.js",
            ),
            (
                Some("http://github.com/x/core/tests/006_url_imports.js"),
                "../005_more_imports.js",
                "http://github.com/x/core/005_more_imports.js",
            ),
            (
                Some("http://github.com/x/core/tests/006_url_imports.js"),
                "http://github.com/x/core/tests/005_more_imports.js",
                "http://github.com/x/core/tests/005_more_imports.js",
            ),
        ];

        // Run tests.
        let loader = UrlModuleLoader::default();

        for (base, specifier, expected) in tests {
            let url = loader.resolve(base, specifier).unwrap();
            assert_eq!(url, expected);
        }
    }
}
