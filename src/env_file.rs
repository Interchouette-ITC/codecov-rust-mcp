//! Load optional `.env` files into the process environment.

use std::path::{Path, PathBuf};

/// Loads `.env` without overriding variables already set in the process.
///
/// Search order:
/// 1. `dotenvy::dotenv()` (current directory, then parents)
/// 2. Repo root inferred from the running binary (`…/target/{debug,release}/codecov-mcp` → `…/.env`)
pub fn load_dotenv() {
    let _ = dotenvy::dotenv();
    if let Some(path) = repo_env_path() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_env_path_none_without_target_layout() {
        // Unit test does not require a real exe under target/; just ensures helper is callable.
        let _ = repo_env_path();
    }
}
