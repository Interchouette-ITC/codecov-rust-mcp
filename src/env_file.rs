//! Load optional `.env` files into the process environment.

use std::path::{Path, PathBuf};

/// Loads `.env` without overriding variables already set in the process.
///
/// Search order:
/// 1. `dotenvy::dotenv()` (current directory, then parents)
/// 2. Repo root inferred from a checkout binary (`…/target/{debug,release}/…` → `…/.env`)
/// 3. User config: `$XDG_CONFIG_HOME/codecov-mcp/.env` or `$HOME/.config/codecov-mcp/.env`
pub fn load_dotenv() {
    let exe = std::env::current_exe().ok();
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    load_dotenv_from(exe.as_deref(), xdg.as_deref(), home.as_deref());
}

/// Same search order as [`load_dotenv`], with injectable paths for tests.
pub(crate) fn load_dotenv_from(exe: Option<&Path>, xdg: Option<&Path>, home: Option<&Path>) {
    let _ = dotenvy::dotenv();
    if let Some(path) = exe.and_then(repo_env_path_from_exe) {
        let _ = dotenvy::from_path(path);
    }
    if let Some(path) = user_config_env_path_from(xdg, home) {
        let _ = dotenvy::from_path(path);
    }
}

/// Repo `.env` when `exe` lives under `…/target/{debug,release}/`.
#[must_use]
pub(crate) fn repo_env_path_from_exe(exe: &Path) -> Option<PathBuf> {
    let release_or_debug = exe.parent()?;
    let target = release_or_debug.parent()?;
    if target.file_name()?.to_str()? != "target" {
        return None;
    }
    let root = target.parent()?;
    let path = root.join(".env");
    Path::new(&path).is_file().then_some(path)
}

/// User config `.env` from XDG or `$HOME/.config`.
#[must_use]
pub(crate) fn user_config_env_path_from(
    xdg: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    let base = xdg
        .map(Path::to_path_buf)
        .or_else(|| home.map(|h| h.join(".config")))?;
    let path = base.join("codecov-mcp").join(".env");
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("codecov-mcp-env-{tag}-{nanos}"));
        fs::create_dir_all(&root).expect("temp root");
        root
    }

    #[test]
    fn repo_env_path_from_exe_finds_dotenv() {
        let root = temp_root("repo");
        let exe = root.join("target").join("debug").join("codecov-mcp");
        fs::create_dir_all(exe.parent().expect("parent")).expect("dirs");
        let env_path = root.join(".env");
        fs::write(&env_path, "CODECOV_TOKEN=from-repo\n").expect("write env");
        assert_eq!(repo_env_path_from_exe(&exe), Some(env_path));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn repo_env_path_from_exe_rejects_non_target() {
        let root = temp_root("notarget");
        let exe = root.join("bin").join("debug").join("codecov-mcp");
        fs::create_dir_all(exe.parent().expect("parent")).expect("dirs");
        fs::write(root.join(".env"), "x=1\n").expect("write");
        assert!(repo_env_path_from_exe(&exe).is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn repo_env_path_from_exe_missing_file() {
        let root = temp_root("missing");
        let exe = root.join("target").join("release").join("codecov-mcp");
        fs::create_dir_all(exe.parent().expect("parent")).expect("dirs");
        assert!(repo_env_path_from_exe(&exe).is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn user_config_env_path_from_xdg() {
        let root = temp_root("xdg");
        let path = root.join("codecov-mcp").join(".env");
        fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        fs::write(&path, "CODECOV_TOKEN=from-xdg\n").expect("write");
        assert_eq!(
            user_config_env_path_from(Some(root.as_path()), None),
            Some(path)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn user_config_env_path_from_home() {
        let home = temp_root("home");
        let path = home.join(".config").join("codecov-mcp").join(".env");
        fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        fs::write(&path, "CODECOV_TOKEN=from-home\n").expect("write");
        assert_eq!(
            user_config_env_path_from(None, Some(home.as_path())),
            Some(path)
        );
        assert!(user_config_env_path_from(None, None).is_none());
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn load_dotenv_from_loads_repo_and_user_files() {
        let root = temp_root("load");
        let exe = root.join("target").join("debug").join("codecov-mcp");
        fs::create_dir_all(exe.parent().expect("parent")).expect("dirs");
        fs::write(root.join(".env"), "# repo env\n").expect("repo env");

        let xdg = temp_root("load-xdg");
        let user_env = xdg.join("codecov-mcp").join(".env");
        fs::create_dir_all(user_env.parent().expect("parent")).expect("dirs");
        fs::write(&user_env, "# user env\n").expect("user env");

        load_dotenv_from(Some(exe.as_path()), Some(xdg.as_path()), None);
        load_dotenv();

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(xdg);
    }
}
