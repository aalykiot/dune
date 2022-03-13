use crate::errors::generic_error;
use crate::modules::ModulePath;
use crate::modules::ModuleSource;
use crate::modules::CORE_MODULES;
use anyhow::bail;
use anyhow::Result;
use colored::*;
use path_clean::PathClean;
use sha::sha1::Sha1;
use sha::utils::Digest;
use sha::utils::DigestExt;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

// Defines the behavior of a module loader.
pub trait ModuleLoader {
    fn load(&self, specifier: &str) -> Result<ModuleSource>;
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath>;
}

static EXTENSIONS: &[&str] = &["js", "json"];

#[derive(Default)]
pub struct FsModuleLoader;

impl FsModuleLoader {
    // Helper method to "clean" messy path strings and convert PathBuf to String.
    fn clean(&self, path: PathBuf) -> String {
        path.clean().into_os_string().into_string().unwrap()
    }

    // Simple function that checks if import is a JSON file.
    fn is_json_import(&self, path: &str) -> bool {
        let path = Path::new(path);
        match path.extension() {
            Some(value) => value == "json",
            None => false,
        }
    }

    // Handle JSON imports using v8's built in methods.
    fn wrap_json(&self, source: &str) -> String {
        format!("export default JSON.parse(`{}`);", source)
    }

    // If import is a file, load it as simple text.
    fn resolve_as_file(&self, path: &Path) -> Result<PathBuf> {
        // 1. Check if path is already a valid file.
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        // 2. Check if we need to add an extension.
        for ext in EXTENSIONS {
            let path = path.with_extension(ext);
            if path.is_file() {
                return Ok(path);
            }
        }
        // 3. Bail out with an error.
        let path = self.clean(path.to_path_buf());
        let err_message = format!("Module not found \"{}\"", path);
        bail!(generic_error(err_message));
    }
    // If import is a directory, load it using the 'index.[ext]' convention.
    fn resolve_as_directory(&self, path: &Path) -> Result<PathBuf> {
        for ext in EXTENSIONS {
            let path = path.join(format!("index.{}", ext));
            if path.is_file() {
                return Ok(path);
            }
        }
        let path = self.clean(path.to_path_buf());
        let err_message = format!("Module not found \"{}\"", path);
        bail!(generic_error(err_message));
    }
}

impl ModuleLoader for FsModuleLoader {
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath> {
        // 1. Try to resolve specifier as an absolute import.
        if specifier.starts_with('/') {
            let base_directory = &Path::new("/");
            let path = base_directory.join(specifier);

            return self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path))
                .map(|path| self.clean(path));
        }
        // 2. Try to resolve specifier as a relative import.
        let cwd = &Path::new(".");
        let mut base_dir = base.map(|v| Path::new(v).parent().unwrap()).unwrap_or(cwd);

        if specifier.starts_with("./") || specifier.starts_with("../") {
            let win_target;
            let target = if cfg!(target_os = "windows") {
                #[allow(clippy::manual_strip)]
                let t = if specifier.starts_with("./") {
                    &specifier[2..]
                } else {
                    base_dir = base_dir.parent().unwrap();
                    &specifier[3..]
                };
                win_target = t.replace("/", "\\");
                &*win_target
            } else {
                specifier
            };

            let path = base_dir.join(target);
            let path = std::env::current_dir().unwrap().join(path);

            return self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path))
                .map(|path| self.clean(path));
        }

        bail!(generic_error(format!("Module not found \"{}\"", specifier)))
    }

    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Load source from path.
        let source = fs::read_to_string(specifier)?;
        let source = match self.is_json_import(specifier) {
            true => self.wrap_json(source.as_str()),
            false => source,
        };

        Ok(source)
    }
}

#[derive(Default)]
// Support importing URLs because...why not?
pub struct UrlModuleLoader;

impl ModuleLoader for UrlModuleLoader {
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath> {
        // 1. Check if the specifier is a valid URL.
        if let Ok(url) = Url::parse(specifier) {
            return Ok(url.into());
        }
        // 2. Check if the caller provided a valid base URL.
        if let Some(base) = base {
            if let Ok(base) = Url::parse(base) {
                let options = Url::options();
                let url = options.base_url(Some(&base));
                let url = url.parse(specifier)?;
                return Ok(url.as_str().to_string());
            }
        }

        // This error shouldn't be showing up often.
        bail!(generic_error("Base is not a valid URL"));
    }

    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Create a .cache directory if it does not exist.
        let cache_dir = env::current_dir()?.join(".cache");

        if fs::create_dir_all(&cache_dir).is_err() {
            bail!(generic_error("Failed to create module caching directory"))
        }

        // Every URL module is hashed into a unique path.
        let hash = Sha1::default().digest(specifier.as_bytes()).to_hex();
        let module_path = cache_dir.join(&hash);

        // If the file is already in cache, just load it.
        if module_path.is_file() {
            let source = fs::read_to_string(&module_path).unwrap();
            return Ok(source);
        }

        println!("{} {}", "Downloading".green(), specifier);

        // Not in cache, so we'll download it.
        let source = match reqwest::blocking::get(specifier)
            .and_then(|response| response.bytes())
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        {
            Ok(source) => source,
            Err(_) => bail!(generic_error(format!("Module not found \"{}\"", specifier))),
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
    use tempfile::tempdir;

    #[test]
    fn test_file_import_resolution() {
        // Create a directory inside of `std::env::temp_dir()`.
        let temp_dir = tempdir().unwrap();

        let filenames = vec![
            ("tests/core/", "005_more_imports.js"),
            ("tests/core/", "006_more_imports.js"),
            ("tests/", "006_more_imports.js"),
            ("tests/core/007_more_imports", "index.js"),
        ];

        filenames.iter().for_each(|(base, name)| {
            std::fs::create_dir_all(temp_dir.path().join(base)).unwrap();
            std::fs::File::create(temp_dir.path().join(base).join(name)).unwrap();
        });

        let wrap = |filename: &str| format!("{}", temp_dir.path().join(filename).display());

        // tests = Vec<(Base, Specifier, Expected_Result)>
        let tests = vec![
            (
                None,
                wrap("tests/core/005_more_imports.js"),
                wrap("tests/core/005_more_imports.js"),
            ),
            (
                Some(wrap("tests/core/005_more_imports.js")),
                "./006_more_imports.js".into(),
                wrap("tests/core/006_more_imports.js"),
            ),
            (
                Some(wrap("tests/core/005_more_imports.js")),
                "./006_more_imports".into(),
                wrap("tests/core/006_more_imports.js"),
            ),
            (
                Some(wrap("tests/core/005_more_imports.js")),
                "../006_more_imports.js".into(),
                wrap("tests/006_more_imports.js"),
            ),
            (
                Some(wrap("tests/core/005_more_imports.js")),
                "../006_more_imports".into(),
                wrap("tests/006_more_imports.js"),
            ),
            (
                None,
                wrap("tests/core/007_more_imports"),
                wrap("tests/core/007_more_imports/index.js"),
            ),
        ];

        let loader = FsModuleLoader::default();

        for (base, specifier, expected) in tests {
            let path = match base {
                Some(base) => loader.resolve(Some(&base), specifier.as_str()),
                None => loader.resolve(None, specifier.as_str()),
            };

            assert!(path.is_ok());
            assert_eq!(path.unwrap(), expected);
        }
    }

    #[test]
    fn test_url_import_resolution() {
        // Tests to run later on.
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
