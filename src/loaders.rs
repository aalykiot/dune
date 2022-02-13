use crate::errors::CustomError;
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

pub type ModuleSpecifier = String;
pub type ModuleSource = String;

// Defines the behavior of a module loader.
pub trait ModuleLoader {
    fn load(&self, specifier: &str) -> Result<ModuleSource>;
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModuleSpecifier>;
}

static EXTENSIONS: &[&str] = &["js", "json"];

#[derive(Default)]
pub struct FsModuleLoader;

impl FsModuleLoader {
    // Helper method to "clean" messy path strings and convert PathBuf to String.
    fn clean(&self, path: PathBuf) -> Result<String> {
        Ok(path.clean().into_os_string().into_string().unwrap())
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
        bail!(CustomError::generic(format!(
            "Module not found \"{}\"",
            path.display()
        )));
    }
    // If import is a directory, load it using the 'index.[ext]' convention.
    fn resolve_as_directory(&self, path: &Path) -> Result<PathBuf> {
        for ext in EXTENSIONS {
            let path = path.join(format!("index.{}", ext));
            if path.is_file() {
                return Ok(path);
            }
        }
        bail!(CustomError::generic(format!(
            "Module not found \"{}\"",
            path.display()
        )));
    }
}

impl ModuleLoader for FsModuleLoader {
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModuleSpecifier> {
        // 1. Try to resolve specifier as a relative import.
        if specifier.starts_with('/') {
            let base_directory = &Path::new("/");
            let path = base_directory.join(specifier);
            return self.clean(path);
        }
        // 2. Try to resolve specifier as an absolute import.
        let cwd = &Path::new(".");
        let mut base = base.map(|v| Path::new(v).parent().unwrap()).unwrap_or(cwd);

        if specifier.starts_with("./") || specifier.starts_with("../") {
            let win_target;
            let target = if cfg!(target_os = "windows") {
                #[allow(clippy::manual_strip)]
                let t = if specifier.starts_with("./") {
                    &specifier[2..]
                } else {
                    base = base.parent().unwrap();
                    &specifier[3..]
                };
                win_target = t.replace("/", "\\");
                &*win_target
            } else {
                specifier
            };

            let path = base.join(target);
            let path = std::env::current_dir().unwrap().join(path);

            // Use `.js` as the default extension.
            let path = match path.extension() {
                Some(_) => path,
                None => PathBuf::from(format!("{}.js", path.display())),
            };

            return self.clean(path);
        }

        bail!(CustomError::generic(format!(
            "Module not found \"{}\"",
            specifier
        )));
    }

    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Check is specifier references a file, folder, etc.
        let path = Path::new(specifier);
        let path = self
            .resolve_as_file(path)
            .or_else(|_| self.resolve_as_directory(path))
            .and_then(|path| self.clean(path))?;

        // Load source from path.
        let source = fs::read_to_string(&path)?;
        let source = match self.is_json_import(&path) {
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
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModuleSpecifier> {
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

        bail!(CustomError::generic("Base is not a valid URL"));
    }

    fn load(&self, specifier: &str) -> Result<ModuleSource> {
        // Create a .cache directory if it does not exist.
        let cache_dir = env::current_dir()?.join(".cache");

        if fs::create_dir_all(&cache_dir).is_err() {
            bail!(CustomError::generic(
                "Failed to create module caching directory"
            ));
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
            Err(_) => bail!(CustomError::generic(format!(
                "Module not found \"{}\"",
                specifier
            ))),
        };

        fs::write(&module_path, &source)?;

        Ok(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_imports() {
        // Tests to run later on.
        let tests = vec![
            (
                None,
                "/dev/core/tests/005_more_imports.ts",
                "/dev/core/tests/005_more_imports.ts",
            ),
            (
                Some("/dev/core/tests/005_more_imports.ts"),
                "./006_more_imports.ts",
                "/dev/core/tests/006_more_imports.ts",
            ),
            (
                Some("/dev/core/tests/005_more_imports.ts"),
                "../006_more_imports.ts",
                "/dev/core/006_more_imports.ts",
            ),
            (
                Some("/dev/core/tests/005_more_imports.ts"),
                "/dev/core/tests/006_more_imports.ts",
                "/dev/core/tests/006_more_imports.ts",
            ),
        ];

        // Run tests.
        let loader = FsModuleLoader::default();

        for (base, specifier, expected) in tests {
            let path = loader.resolve(base, specifier).unwrap();
            assert_eq!(path, expected);
        }
    }

    #[test]
    fn test_url_imports() {
        // Tests to run later on.
        let tests = vec![
            (
                None,
                "http://github.com/x/core/tests/006_url_imports.ts",
                "http://github.com/x/core/tests/006_url_imports.ts",
            ),
            (
                Some("http://github.com/x/core/tests/006_url_imports.ts"),
                "./005_more_imports.ts",
                "http://github.com/x/core/tests/005_more_imports.ts",
            ),
            (
                Some("http://github.com/x/core/tests/006_url_imports.ts"),
                "../005_more_imports.ts",
                "http://github.com/x/core/005_more_imports.ts",
            ),
            (
                Some("http://github.com/x/core/tests/006_url_imports.ts"),
                "http://github.com/x/core/tests/005_more_imports.ts",
                "http://github.com/x/core/tests/005_more_imports.ts",
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
