//! Load optional `.env` files into the process environment.

use std::path::{Path, PathBuf};

/// Loads `.env` without overriding variables already set in the process.
///
/// Search order:
/// 1. `dotenvy::dotenv()` (current directory, then parents)
/// 2. Repo root inferred from a checkout binary (`…/target/{debug,release}/…` → `…/.env`)
/// 3. User config: `$XDG_CONFIG_HOME/codecov-mcp/.env` or `$HOME/.config/codecov-mcp/.env`
pub fn load_dotenv() {
    let _ = dotenvy::dotenv();
    if let Some(path) = repo_env_path() {
        let _ = dotenvy::from_path(path);
    }
    if let Some(path) = user_config_env_path() {
        let _ = dotenvy::from_path(path);
    }
}

fn repo_env_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let release_or_debug = exe.parent()?;
    let target = release_or_debug.parent()?;
    if target.file_name()?.to_str()? != "target" {
        return None;
    }
    let root = target.parent()?;
    let path = root.join(".env");
    Path::new(&path).is_file().then_some(path)
}

fn user_config_env_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    let path = base.join("codecov-mcp").join(".env");
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_env_path_helper_is_callable() {
        let _ = repo_env_path();
        let _ = user_config_env_path();
    }
}
