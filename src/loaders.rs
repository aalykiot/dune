use anyhow::{bail, Context, Error};
use colored::*;
use path_clean::PathClean;
use reqwest::Url;
use sha::sha1::Sha1;
use sha::utils::{Digest, DigestExt};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
    // Basic file-system loader.
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

#[derive(Default)]
pub struct WebModuleLoader;

impl WebModuleLoader {
    fn resolve_as_url(&self, referrer: &str, specifier: &str) -> Result<Url, Error> {
        // Check if the referrer is a valid URL.
        if let Ok(referrer) = Url::parse(referrer) {
            let options = Url::options();
            let url = options.base_url(Some(&referrer));
            let url = url.parse(specifier)?;
            return Ok(url);
        }
        Ok(Url::from_str(specifier).unwrap())
    }
}

impl ModuleLoader for WebModuleLoader {
    // Support importing URLs because...why not?
    fn load(&self, specifier: &str) -> Result<String, Error> {
        // Create a .cache directory if it does not exist.
        let cache_dir = env::current_dir()?.join(".cache");
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

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
        let source = reqwest::blocking::get(specifier)
            .and_then(|response| response.bytes())
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .with_context(|| format!("Failed to fetch {}", specifier))?;

        fs::write(&module_path, &source)?;

        Ok(source)
    }

    fn resolve(&self, referrer: &str, specifier: &str) -> Result<String, Error> {
        self.resolve_as_url(referrer, specifier)
            .map(|url| url.as_str().to_string())
    }
}
