//! Windows UAC elevation helpers for SYSTEM scheduling / service install.

/// Result of ensuring an elevated token is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevationOutcome {
    /// Current process already has an elevated token — continue in-process.
    AlreadyElevated,
    /// A new elevated process was started, waited on, and finished with this exit code.
    /// Caller should return without re-doing work (the elevated child did the work).
    Relaunched { exit_code: u32 },
}

/// True when the process token is elevated (Administrators full token, not UAC filtered).
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        is_elevated_windows()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// If not elevated, re-launch this process with the same argv via UAC (`runas`) and wait.
///
/// On success with relaunch, returns [`ElevationOutcome::Relaunched`] so the caller
/// exits without double-executing the command. Dry-run paths should not call this.
pub fn ensure_elevated_or_relaunch() -> Result<ElevationOutcome, Box<dyn std::error::Error>> {
    if is_elevated() {
        return Ok(ElevationOutcome::AlreadyElevated);
    }

    #[cfg(windows)]
    {
        println!(
            "Administrator elevation is required. Showing UAC prompt (approve to continue)..."
        );
        let code = relaunch_elevated_and_wait()?;
        Ok(ElevationOutcome::Relaunched { exit_code: code })
    }
    #[cfg(not(windows))]
    {
        Err("Elevation is only supported on Windows".into())
    }
}

/// Quote a single Windows command-line argument (spaces / quotes).
pub fn quote_windows_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    let needs_quotes = arg.chars().any(|c| c.is_whitespace() || c == '"');
    if !needs_quotes {
        return arg.to_string();
    }
    let mut out = String::from("\"");
    for c in arg.chars() {
        if c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Build the parameter string for ShellExecute from argv (skipping argv[0]).
pub fn build_relaunch_params(args: impl IntoIterator<Item = String>) -> String {
    args.into_iter()
        .map(|a| quote_windows_arg(&a))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(windows)]
fn is_elevated_windows() -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);

        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

#[cfg(windows)]
fn relaunch_elevated_and_wait() -> Result<u32, Box<dyn std::error::Error>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().map_err(|e| format!("current_exe failed: {e}"))?;
    let params = build_relaunch_params(std::env::args().skip(1));

    let mut exe_wide: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut verb_wide: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();
    let mut params_wide: Vec<u16> = params.encode_utf16().chain(std::iter::once(0)).collect();

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: windows::Win32::Foundation::HWND::default(),
        lpVerb: PCWSTR(verb_wide.as_mut_ptr()),
        lpFile: PCWSTR(exe_wide.as_mut_ptr()),
        lpParameters: PCWSTR(params_wide.as_mut_ptr()),
        lpDirectory: PCWSTR::null(),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok.is_err() {
        let err = std::io::Error::last_os_error();
        // ERROR_CANCELLED = 1223 when user declines UAC
        if err.raw_os_error() == Some(1223) {
            return Err(
                "UAC elevation was cancelled. Approve the prompt, or re-run from an Administrator shell."
                    .into(),
            );
        }
        return Err(format!(
            "Failed to relaunch elevated (ShellExecuteExW): {err}. \
             Re-run from an Administrator shell if UAC is unavailable."
        )
        .into());
    }

    let process = info.hProcess;
    if process.is_invalid() {
        // Launched but no process handle — treat as fire-and-forget success.
        return Ok(0);
    }

    let wait = unsafe { WaitForSingleObject(process, INFINITE) };
    if wait != WAIT_OBJECT_0 {
        let _ = unsafe { CloseHandle(process) };
        return Err("WaitForSingleObject on elevated process failed".into());
    }

    let mut exit_code: u32 = 1;
    let got = unsafe { GetExitCodeProcess(process, &mut exit_code) };
    let _ = unsafe { CloseHandle(process) };
    if got.is_err() {
        return Err("GetExitCodeProcess on elevated process failed".into());
    }
    Ok(exit_code)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn quote_windows_arg__no_spaces__unchanged() {
        assert_eq!(quote_windows_arg("--run-as-system"), "--run-as-system");
    }

    #[test]
    fn quote_windows_arg__with_spaces__quoted() {
        assert_eq!(quote_windows_arg("C:\\Program Files\\x"), "\"C:\\Program Files\\x\"");
    }

    #[test]
    fn build_relaunch_params__joins_quoted_args() {
        let params = build_relaunch_params([
            "nightly".to_string(),
            "--schedule".to_string(),
            "--run-as-system".to_string(),
            "--start-time".to_string(),
            "03:00".to_string(),
        ]);
        assert_eq!(
            params,
            "nightly --schedule --run-as-system --start-time 03:00"
        );
    }

    #[test]
    fn build_relaunch_params__path_with_space__quoted() {
        let params = build_relaunch_params([
            "--vault-path".to_string(),
            r"C:\Users\Test User\vault.db".to_string(),
        ]);
        assert!(params.contains("\"C:\\Users\\Test User\\vault.db\""));
    }
}
