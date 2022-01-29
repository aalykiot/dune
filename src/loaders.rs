use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Error};
use path_clean::PathClean;

// Defining the behavior for any kind of module loader.
pub trait ModuleLoader {
    fn load(&self, specifier: &str) -> Result<String, Error>;
    fn resolve(&self, referrer: &str, specifier: &str) -> Result<String, Error>;
}

static EXTENSIONS: &[&str] = &["js", "json"];

#[derive(Default)]
pub struct FsModuleLoader;

impl FsModuleLoader {
    // Helper method to "clean" messy path strings and convert PathBuf to String.
    fn clean(&self, path: PathBuf) -> Result<String, Error> {
        Ok(path.clean().into_os_string().into_string().unwrap())
    }

    fn is_json_import(&self, path: &str) -> bool {
        let path = Path::new(path);
        match path.extension() {
            Some(value) => value == "json",
            None => false,
        }
    }

    fn wrap_json(&self, source: &str) -> String {
        format!("export default JSON.parse(`{}`);", source)
    }

    // If import is a file, load it as JavaScript text.
    fn resolve_as_file(&self, path: &Path) -> Result<PathBuf, Error> {
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
        bail!("Failed to find module \"{}\"", path.display());
    }

    // If import is a directory, load it using the 'index.[ext]' convention.
    fn resolve_as_directory(&self, path: &Path) -> Result<PathBuf, Error> {
        for ext in EXTENSIONS {
            let path = path.join(format!("index.{}", ext));
            if path.is_file() {
                return Ok(path);
            }
        }
        bail!("Failed to find module \"{}\"", path.display());
    }
}

impl ModuleLoader for FsModuleLoader {
    fn load(&self, specifier: &str) -> Result<String, Error> {
        let source = fs::read_to_string(specifier)?;
        let source = match self.is_json_import(specifier) {
            true => self.wrap_json(source.as_str()),
            false => source,
        };
        Ok(source)
    }

    fn resolve(&self, referrer: &str, specifier: &str) -> Result<String, Error> {
        // Resolving absolute define imports.
        if specifier.starts_with('/') {
            let base_directory = &Path::new("/");
            let path = base_directory.join(specifier);

            return self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path))
                .and_then(|path| self.clean(path));
        }

        // Resolving relative defined imports.
        let cwd = &Path::new(".");
        let mut base_dir = Path::new(referrer).parent().unwrap_or(cwd);

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
            return self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path))
                .and_then(|path| self.clean(path));
        }

        bail!("Failed to resolve module \"{}\"", specifier);
    }
}
