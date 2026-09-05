//! Test-only process-env helpers (excluded from coverage totals).

#![cfg(test)]

use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

pub fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn restore_env(key: &str, prev: Option<String>) {
    if let Some(value) = prev {
        std::env::set_var(key, value);
    } else {
        std::env::remove_var(key);
    }
}

#[test]
fn restore_env_both_branches() {
    let _guard = env_lock();
    let key = "CODECOV_MCP_RESTORE_TEST_ENV";
    restore_env(key, Some("kept".into()));
    assert_eq!(std::env::var(key).as_deref(), Ok("kept"));
    restore_env(key, None);
    assert!(std::env::var(key).is_err());
}
