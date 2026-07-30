//! WebView2 Evergreen Runtime pre-launch diagnostic (S21).
//!
//! On Windows, detect a missing WebView2 runtime via registry and show a clear
//! dialog with Bootstrapper guidance, then exit cleanly (code 1) — no panic.

/// Bootstrapper download guidance shown when WebView2 is missing.
pub const WEBVIEW2_BOOTSTRAPPER_URL: &str =
    "https://developer.microsoft.com/en-us/microsoft-edge/webview2/";

/// Exit code used when WebView2 is required but missing.
pub const EXIT_WEBVIEW2_MISSING: i32 = 1;

/// Evergreen WebView2 Runtime client GUID used by Microsoft Edge Update.
const WEBVIEW2_CLIENT_GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

/// Result of a WebView2 availability check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebView2Status {
    /// Runtime appears installed (or non-Windows host).
    Available,
    /// Windows host without a detectable Evergreen Runtime install.
    Missing,
}

/// Pure: whether a registry `pv` value indicates an installed Evergreen Runtime.
///
/// Empty and `0.0.0.0` are treated as not installed (Microsoft placeholder values).
pub fn pv_indicates_installed(pv: &str) -> bool {
    !pv.is_empty() && pv != "0.0.0.0"
}

/// Pure: dialog / console body shown when WebView2 is missing (includes Bootstrapper URL).
pub fn webview2_missing_message() -> String {
    format!(
        "Microsoft Edge WebView2 Runtime was not found on this system.\n\n\
         AI-Brains Desktop needs WebView2 to display its UI.\n\n\
         WebView2 is preinstalled on Windows 10 (1803+) and Windows 11 for most SKUs.\n\
         On stripped or older systems, install the Evergreen Bootstrapper from:\n\
         {WEBVIEW2_BOOTSTRAPPER_URL}\n\n\
         The application will now exit."
    )
}

/// Detect WebView2 Evergreen Runtime presence.
///
/// Non-Windows targets always report [`WebView2Status::Available`] (optional
/// platforms; Windows is the primary target).
#[cfg(not(windows))]
pub fn detect_webview2() -> WebView2Status {
    WebView2Status::Available
}

#[cfg(windows)]
pub fn detect_webview2() -> WebView2Status {
    if webview2_registry_has_version() {
        WebView2Status::Available
    } else {
        WebView2Status::Missing
    }
}

/// If WebView2 is missing on Windows, show a blocking dialog and exit with code 1.
///
/// Safe to call before Tauri starts. Does not panic.
///
/// Process-exit path is intentionally not unit-tested (would terminate the test
/// process). Coverage for the Missing branch is the pure message builder plus
/// production wiring: `detect_webview2() == Missing` → `show_missing_dialog()` →
/// `std::process::exit(EXIT_WEBVIEW2_MISSING)`.
pub fn ensure_webview2_or_exit() {
    if detect_webview2() == WebView2Status::Missing {
        show_missing_dialog();
        std::process::exit(EXIT_WEBVIEW2_MISSING);
    }
}

#[cfg(windows)]
fn webview2_registry_has_version() -> bool {
    // Common install locations for Evergreen Runtime (HKLM + HKCU, 64- and 32-bit views).
    const ROOTS: &[(windows::Win32::System::Registry::HKEY, &str)] = &[
        (
            windows::Win32::System::Registry::HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients",
        ),
        (
            windows::Win32::System::Registry::HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\EdgeUpdate\Clients",
        ),
        (
            windows::Win32::System::Registry::HKEY_CURRENT_USER,
            r"Software\Microsoft\EdgeUpdate\Clients",
        ),
    ];

    for &(root, base) in ROOTS {
        let subkey = format!(r"{base}\{WEBVIEW2_CLIENT_GUID}");
        if read_pv(root, &subkey).is_some_and(|pv| pv_indicates_installed(&pv)) {
            return true;
        }
    }
    false
}

#[cfg(windows)]
fn read_pv(root: windows::Win32::System::Registry::HKEY, subkey: &str) -> Option<String> {
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        HKEY, KEY_READ, REG_SZ, REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    };
    use windows::core::PCWSTR;

    let wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
    let mut hkey = HKEY::default();

    // SAFETY: subkey is a valid NUL-terminated UTF-16 path; hkey is written only on success.
    let open_status =
        unsafe { RegOpenKeyExW(root, PCWSTR(wide.as_ptr()), Some(0), KEY_READ, &mut hkey) };
    if open_status != ERROR_SUCCESS {
        return None;
    }

    let name: Vec<u16> = "pv".encode_utf16().chain(std::iter::once(0)).collect();
    let mut kind = REG_VALUE_TYPE::default();
    let mut data_size: u32 = 0;

    // SAFETY: Query size only; NULL data buffer is allowed when size is requested.
    let size_status = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            None,
            Some(&mut data_size),
        )
    };
    if size_status != ERROR_SUCCESS || data_size == 0 {
        // SAFETY: hkey was opened successfully above.
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        return None;
    }

    let mut buf = vec![0u8; data_size as usize];
    // SAFETY: buf length matches data_size reported by the previous query.
    let read_status = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            Some(buf.as_mut_ptr()),
            Some(&mut data_size),
        )
    };

    // SAFETY: always close the key we opened.
    unsafe {
        let _ = RegCloseKey(hkey);
    }

    if read_status != ERROR_SUCCESS || kind != REG_SZ {
        return None;
    }

    // REG_SZ is UTF-16LE; trim trailing NULs.
    let u16_len = (data_size as usize) / 2;
    if u16_len == 0 {
        return None;
    }
    let mut words = Vec::with_capacity(u16_len);
    for chunk in buf.chunks_exact(2).take(u16_len) {
        words.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    while words.last().copied() == Some(0) {
        words.pop();
    }
    String::from_utf16(&words).ok()
}

#[cfg(windows)]
fn show_missing_dialog() {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    use windows::core::PCWSTR;

    let title: Vec<u16> = "AI-Brains — WebView2 required"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let body = webview2_missing_message();
    let body_wide: Vec<u16> = body.encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: title/body are valid NUL-terminated UTF-16; HWND null is allowed for no owner.
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(body_wide.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
fn show_missing_dialog() {
    eprintln!("{}", webview2_missing_message());
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn bootstrapper_url__is_https_microsoft() {
        assert!(WEBVIEW2_BOOTSTRAPPER_URL.starts_with("https://"));
        assert!(WEBVIEW2_BOOTSTRAPPER_URL.contains("webview2"));
    }

    #[test]
    fn webview2_missing_message__contains_bootstrapper_url() {
        let msg = webview2_missing_message();
        assert!(
            msg.contains(WEBVIEW2_BOOTSTRAPPER_URL),
            "missing-dialog body must include Bootstrapper URL; got: {msg}"
        );
        assert!(
            msg.contains("WebView2"),
            "missing-dialog body must mention WebView2; got: {msg}"
        );
        assert!(
            msg.contains("https://developer.microsoft.com"),
            "missing-dialog body must point at Microsoft docs; got: {msg}"
        );
    }

    #[test]
    fn pv_indicates_installed__empty__false() {
        assert!(!pv_indicates_installed(""));
    }

    #[test]
    fn pv_indicates_installed__placeholder_zero__false() {
        assert!(!pv_indicates_installed("0.0.0.0"));
    }

    #[test]
    fn pv_indicates_installed__real_version__true() {
        assert!(pv_indicates_installed("1.2.3.4"));
        assert!(pv_indicates_installed("144.0.3485.54"));
    }

    #[cfg(not(windows))]
    #[test]
    fn detect_webview2__non_windows__available() {
        assert_eq!(detect_webview2(), WebView2Status::Available);
    }

    /// Proves the Windows registry path does not panic and returns a known variant.
    /// Does not assert Available vs Missing (host-dependent); both are valid.
    #[cfg(windows)]
    #[test]
    fn detect_webview2__windows__returns_known_variant() {
        let status = detect_webview2();
        // Visible under `cargo test -- --nocapture` for smoke evidence hosts.
        eprintln!("detect_webview2() = {status:?}");
        assert!(
            matches!(status, WebView2Status::Available | WebView2Status::Missing),
            "detect_webview2 must return Available or Missing; got: {status:?}"
        );
    }
}
