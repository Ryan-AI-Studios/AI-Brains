//! Temporary environment variable mutation with automatic restore on drop.
//!
//! Intended for tests and integration tests that must set process env.
//! Prefer CLI flags / `assert_cmd` env over process env when possible.
//! Under nextest, process isolation is the primary isolation story; use
//! `#[serial(env)]` only when multiple tests in one binary mutate overlapping keys.

use std::env;
use std::ffi::{OsStr, OsString};

/// RAII guard that sets (or removes) an environment variable and restores the
/// previous value when dropped.
///
/// # Example
///
/// ```no_run
/// use ai_brains_core::temp_env::TempEnv;
///
/// let _guard = TempEnv::set("AI_BRAINS_CTX_SIZE", "1600");
/// // env restored when `_guard` drops
/// ```
pub struct TempEnv {
    key: OsString,
    previous: Option<OsString>,
}

impl TempEnv {
    /// Set `key` to `value`, saving the prior value for restore on drop.
    pub fn set(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        let key_os = key.as_ref().to_os_string();
        let previous = env::var_os(&key_os);
        // SAFETY: TempEnv is a test/process helper. Callers must ensure no
        // concurrent readers of this key without serialization (nextest
        // process isolation or #[serial(env)]).
        unsafe {
            env::set_var(&key_os, value.as_ref());
        }
        Self {
            key: key_os,
            previous,
        }
    }

    /// Remove `key` from the environment, saving the prior value for restore.
    pub fn remove(key: impl AsRef<OsStr>) -> Self {
        let key_os = key.as_ref().to_os_string();
        let previous = env::var_os(&key_os);
        // SAFETY: see TempEnv::set.
        unsafe {
            env::remove_var(&key_os);
        }
        Self {
            key: key_os,
            previous,
        }
    }
}

impl Drop for TempEnv {
    fn drop(&mut self) {
        match &self.previous {
            Some(val) => {
                // SAFETY: restoring the value we saved in set/remove.
                unsafe {
                    env::set_var(&self.key, val);
                }
            }
            None => {
                // SAFETY: removing a key that was previously unset.
                unsafe {
                    env::remove_var(&self.key);
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::TempEnv;
    use std::env;

    #[test]
    fn temp_env_set__restores_previous_on_drop() {
        let key = "AI_BRAINS_TEMP_ENV_TEST_RESTORE";
        // SAFETY: single-threaded unit test isolation for this key.
        unsafe {
            env::remove_var(key);
        }
        {
            let _g = TempEnv::set(key, "first");
            assert_eq!(env::var(key).ok().as_deref(), Some("first"));
            {
                let _inner = TempEnv::set(key, "second");
                assert_eq!(env::var(key).ok().as_deref(), Some("second"));
            }
            assert_eq!(env::var(key).ok().as_deref(), Some("first"));
        }
        assert!(env::var_os(key).is_none());
    }

    #[test]
    fn temp_env_remove__restores_previous_on_drop() {
        let key = "AI_BRAINS_TEMP_ENV_TEST_REMOVE";
        let _setup = TempEnv::set(key, "present");
        {
            let _g = TempEnv::remove(key);
            assert!(env::var_os(key).is_none());
        }
        assert_eq!(env::var(key).ok().as_deref(), Some("present"));
    }
}
