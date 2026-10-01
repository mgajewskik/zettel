use std::env;
use std::path::{Component, Path, PathBuf};

use obsidian_core::Vault;

use crate::error::CliError;

/// Resolve vault root: `--vault` wins, else `ZETTEL_VAULT`. Never hardcodes a path.
pub fn resolve_vault_path(cli_vault: Option<PathBuf>) -> Result<PathBuf, CliError> {
    if let Some(path) = cli_vault {
        return Ok(path);
    }
    match env::var_os("ZETTEL_VAULT") {
        Some(val) if !val.is_empty() => Ok(PathBuf::from(val)),
        _ => Err(CliError::Config(
            "vault path not set: pass --vault <path> or set env ZETTEL_VAULT".into(),
        )),
    }
}

/// Open a vault at the resolved path.
pub fn open_vault(cli_vault: Option<PathBuf>) -> Result<Vault, CliError> {
    let path = resolve_vault_path(cli_vault)?;
    Vault::open(&path).map_err(|e| CliError::Vault(e.to_string()))
}

/// Path filter for vault walks: skip `media/` and `canvas/` directory components.
/// (`.obsidian` / `.git` are already skipped by the ignore crate inside core.)
pub fn default_path_filter(path: &Path) -> bool {
    !path.components().any(|c| match c {
        Component::Normal(name) => {
            let name = name.to_string_lossy();
            name.eq_ignore_ascii_case("media") || name.eq_ignore_ascii_case("canvas")
        }
        _ => false,
    })
}

/// Combine default media/canvas filter with optional path-prefix filters.
/// If `prefixes` is empty, only the default filter applies.
/// If non-empty, a path must match the default filter AND be under at least one prefix
/// (or have a relative path starting with / matching any prefix string).
pub fn make_filter<'a>(vault_root: &'a Path, prefixes: &'a [PathBuf]) -> impl Fn(&Path) -> bool + 'a {
    move |path: &Path| {
        if !default_path_filter(path) {
            return false;
        }
        if prefixes.is_empty() {
            return true;
        }
        let rel = path.strip_prefix(vault_root).unwrap_or(path);
        prefixes.iter().any(|prefix| {
            let p = prefix.as_path();
            path.starts_with(p)
                || rel.starts_with(p)
                || path
                    .to_string_lossy()
                    .contains(&*p.to_string_lossy())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize env mutations across tests in this module.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn filter_skips_media_and_canvas() {
        assert!(default_path_filter(Path::new("notes/alpha.md")));
        assert!(!default_path_filter(Path::new("media/ignored.md")));
        assert!(!default_path_filter(Path::new("canvas/ignored.md")));
        assert!(!default_path_filter(Path::new("/vault/Media/x.md")));
    }

    #[test]
    fn resolve_requires_vault_or_env() {
        let _g = ENV_LOCK.lock().unwrap();
        // SAFETY: held under ENV_LOCK; test-only.
        unsafe {
            env::remove_var("ZETTEL_VAULT");
        }
        let err = resolve_vault_path(None).unwrap_err();
        assert!(matches!(err, CliError::Config(_)));
    }

    #[test]
    fn resolve_prefers_cli_over_env() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("ZETTEL_VAULT", "/from-env");
        }
        let path = resolve_vault_path(Some(PathBuf::from("/from-cli"))).unwrap();
        assert_eq!(path, PathBuf::from("/from-cli"));
        unsafe {
            env::remove_var("ZETTEL_VAULT");
        }
    }

    #[test]
    fn resolve_uses_env() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("ZETTEL_VAULT", "/from-env");
        }
        let path = resolve_vault_path(None).unwrap();
        assert_eq!(path, PathBuf::from("/from-env"));
        unsafe {
            env::remove_var("ZETTEL_VAULT");
        }
    }
}
