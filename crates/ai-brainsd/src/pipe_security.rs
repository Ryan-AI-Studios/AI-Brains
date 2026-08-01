#![cfg(windows)]
#![allow(clippy::disallowed_methods)]

//! Named-pipe security descriptor for `ai-brainsd`.
//!
//! # Design (T144 + T184 F-1)
//!
//! The daemon may run as **LocalSystem** (Session 0 Windows service). CLI clients
//! run in **Session 1+** as interactive users and must open the pipe. Owner-only
//! (`OW`) is insufficient when SYSTEM creates the pipe.
//!
//! SDDL grants:
//! - **SY** (Local System) — service host
//! - **BA** (Built-in Administrators) — elevated operators
//! - **IU** (Interactive) — interactive logon sessions (Session 1+)
//!
//! This replaces the prior **WD** (Everyone / World) grant, which contradicted
//! T144 non-goals (multi-user World was listed as a security risk) and
//! OPERATIONS prose (“interactive user”).
//!
//! **Residual (R-MULTI / R-PIPE-IU):** On a multi-user machine, *any* interactive
//! logon can open the pipe. Pipe messages still have no bearer (contrast HTTP).
//! Single-owner desktops are the primary model; multi-user hosts accept residual
//! risk documented in SECURITY-LIMITS.

use std::io;

use windows::{
    Win32::Security::{
        Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorA, PSECURITY_DESCRIPTOR,
        SECURITY_ATTRIBUTES,
    },
    core::PCSTR,
};

/// SYSTEM + Administrators + Interactive (not World/Everyone).
/// See module docs and T184 finding F-1.
const SDDL: &str = "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)";

pub fn build_pipe_security_attributes() -> io::Result<SECURITY_ATTRIBUTES> {
    let mut psd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR::default();

    let result = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorA(
            PCSTR(SDDL.as_ptr()),
            1,
            &mut psd,
            None,
        )
    };

    if result.is_err() {
        return Err(io::Error::other(format!(
            "ConvertStringSecurityDescriptorToSecurityDescriptorA failed: {:?}",
            result
        )));
    }

    if psd.0.is_null() {
        return Err(io::Error::other("SDDL conversion returned null SD"));
    }

    let valid = unsafe { windows::Win32::Security::IsValidSecurityDescriptor(psd) };
    if !valid.as_bool() {
        return Err(io::Error::other("IsValidSecurityDescriptor returned false"));
    }

    Ok(SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: psd.0,
        bInheritHandle: false.into(),
    })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn build_pipe_security_attributes__returns_valid_sa_with_nonnull_sd() {
        let sa = build_pipe_security_attributes().expect("should build security attributes");
        assert_eq!(
            sa.nLength,
            std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32
        );
        assert!(!sa.lpSecurityDescriptor.is_null());

        let psd = PSECURITY_DESCRIPTOR(sa.lpSecurityDescriptor);
        let valid = unsafe { windows::Win32::Security::IsValidSecurityDescriptor(psd) };
        assert!(valid.as_bool(), "SD must be valid");
    }

    #[test]
    fn build_pipe_security_attributes__dacl_present_not_world() {
        let sa = build_pipe_security_attributes().expect("should build security attributes");
        let psd = PSECURITY_DESCRIPTOR(sa.lpSecurityDescriptor);

        let mut dacl_present = windows::core::BOOL::default();
        let mut dacl_defaulted = windows::core::BOOL::default();
        let mut dacl_ptr: *mut windows::Win32::Security::ACL = std::ptr::null_mut();

        let result = unsafe {
            windows::Win32::Security::GetSecurityDescriptorDacl(
                psd,
                &mut dacl_present,
                &mut dacl_ptr,
                &mut dacl_defaulted,
            )
        };
        assert!(result.is_ok());
        assert!(
            dacl_present.as_bool(),
            "DACL must be present (explicit SY+BA+IU grant)"
        );
        assert!(!dacl_ptr.is_null(), "DACL pointer must not be null");

        // Normative SDDL must not grant World/Everyone (WD).
        assert!(
            !SDDL.contains(";;;WD)"),
            "pipe SDDL must not grant Everyone/World (WD); got {SDDL}"
        );
        assert!(
            SDDL.contains(";;;IU)"),
            "pipe SDDL must grant Interactive (IU); got {SDDL}"
        );
        assert!(
            SDDL.contains(";;;SY)"),
            "pipe SDDL must grant SYSTEM (SY); got {SDDL}"
        );
        assert!(
            SDDL.contains(";;;BA)"),
            "pipe SDDL must grant Administrators (BA); got {SDDL}"
        );
    }

    #[test]
    fn pipe_sddl__excludes_world_and_includes_interactive() {
        assert_eq!(SDDL, "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)");
        assert!(!SDDL.contains("WD"));
    }
}
