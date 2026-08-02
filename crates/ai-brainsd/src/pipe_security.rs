#![cfg(windows)]
#![allow(clippy::disallowed_methods)]

//! Named-pipe security descriptor for `ai-brainsd`.
//!
//! # Design (T144 + T184 F-1 + T195 F3/F4)
//!
//! The daemon may run as **LocalSystem** (Session 0 Windows service). CLI clients
//! run in **Session 1+** as interactive users and must open the pipe. Owner-only
//! (`OW`) is insufficient when SYSTEM creates the pipe.
//!
//! Default SDDL (`AI_BRAINS_PIPE_ACL=interactive` or unset) grants:
//! - **SY** (Local System) — service host
//! - **BA** (Built-in Administrators) — elevated operators
//! - **IU** (Interactive) — interactive logon sessions (Session 1+)
//!
//! Opt-in `AI_BRAINS_PIPE_ACL=service-only` drops **IU** (SY+BA only). Interactive
//! CLI then cannot open a SYSTEM service pipe (expects NotRunning); use elevated
//! BA, `sc query`, or interactive daemon + HTTP+bearer — see OPERATIONS.
//!
//! This replaces the prior **WD** (Everyone / World) grant, which contradicted
//! T144 non-goals (multi-user World was listed as a security risk) and
//! OPERATIONS prose (“interactive user”).
//!
//! **Residual (R-PIPE-IU / R-MULTI):** Default still allows any interactive logon
//! to open the pipe. Pipe messages still have no bearer (contrast HTTP).
//! Single-owner desktops are the primary model (ADR-0022).

use std::io;

use ai_brains_daemon_api::{PipeAclMode, pipe_acl_mode_from_env, sddl_for_pipe_acl_mode};
use windows::{
    Win32::Security::{
        Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorA, PSECURITY_DESCRIPTOR,
        SECURITY_ATTRIBUTES,
    },
    core::PCSTR,
};

/// Build pipe `SECURITY_ATTRIBUTES` from `AI_BRAINS_PIPE_ACL` (default interactive).
///
/// Unknown env values fail closed (do not apply an unknown DACL).
pub fn build_pipe_security_attributes() -> io::Result<SECURITY_ATTRIBUTES> {
    let mode = pipe_acl_mode_from_env().map_err(|e| io::Error::other(e.to_string()))?;
    build_pipe_security_attributes_for_mode(mode)
}

/// Build pipe SA for an explicit ACL mode (unit-tested without env).
pub fn build_pipe_security_attributes_for_mode(
    mode: PipeAclMode,
) -> io::Result<SECURITY_ATTRIBUTES> {
    let sddl = sddl_for_pipe_acl_mode(mode);
    security_attributes_from_sddl(sddl)
}

fn security_attributes_from_sddl(sddl: &str) -> io::Result<SECURITY_ATTRIBUTES> {
    let mut psd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR::default();

    // SDDL is ASCII; append NUL for PCSTR.
    let mut sddl_c = sddl.as_bytes().to_vec();
    sddl_c.push(0);

    let result = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorA(
            PCSTR(sddl_c.as_ptr()),
            1,
            &mut psd,
            None,
        )
    };

    if result.is_err() {
        return Err(io::Error::other(format!(
            "ConvertStringSecurityDescriptorToSecurityDescriptorA failed for {sddl}: {:?}",
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
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_daemon_api::{
        PIPE_SDDL_INTERACTIVE, PIPE_SDDL_SERVICE_ONLY, PipeAclMode, sddl_for_pipe_acl_mode,
    };

    fn assert_dacl_present(sa: &SECURITY_ATTRIBUTES) {
        let psd = PSECURITY_DESCRIPTOR(sa.lpSecurityDescriptor);
        let valid = unsafe { windows::Win32::Security::IsValidSecurityDescriptor(psd) };
        assert!(valid.as_bool(), "SD must be valid");

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
        assert!(dacl_present.as_bool(), "DACL must be present");
        assert!(!dacl_ptr.is_null(), "DACL pointer must not be null");
    }

    #[test]
    fn build_pipe_security_attributes__default_interactive__valid_sa_with_iu() {
        let _clear = TempEnv::remove("AI_BRAINS_PIPE_ACL");
        let sa = build_pipe_security_attributes().expect("should build security attributes");
        assert_eq!(
            sa.nLength,
            std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32
        );
        assert!(!sa.lpSecurityDescriptor.is_null());
        assert_dacl_present(&sa);

        let sddl = sddl_for_pipe_acl_mode(PipeAclMode::Interactive);
        assert_eq!(sddl, PIPE_SDDL_INTERACTIVE);
        assert!(sddl.contains(";;;IU)"), "default must grant IU");
        assert!(!sddl.contains(";;;WD)"), "must not grant World");
        assert!(!sddl.contains("WD"));
    }

    #[test]
    fn build_pipe_security_attributes_for_mode__service_only__valid_sa_without_iu() {
        let sa = build_pipe_security_attributes_for_mode(PipeAclMode::ServiceOnly)
            .expect("service-only SA");
        assert_dacl_present(&sa);

        let sddl = sddl_for_pipe_acl_mode(PipeAclMode::ServiceOnly);
        assert_eq!(sddl, PIPE_SDDL_SERVICE_ONLY);
        assert!(
            !sddl.contains(";;;IU)"),
            "service-only must not grant IU; got {sddl}"
        );
        assert!(sddl.contains(";;;SY)"));
        assert!(sddl.contains(";;;BA)"));
        assert!(!sddl.contains("WD"));
    }

    #[test]
    fn build_pipe_security_attributes_for_mode__interactive__valid_sa_with_iu() {
        let sa = build_pipe_security_attributes_for_mode(PipeAclMode::Interactive)
            .expect("interactive SA");
        assert_dacl_present(&sa);
        let sddl = sddl_for_pipe_acl_mode(PipeAclMode::Interactive);
        assert!(sddl.contains(";;;IU)"));
        assert!(!sddl.contains("WD"));
    }

    #[test]
    fn build_pipe_security_attributes__unknown_env__fail_closed() {
        let _g = TempEnv::set("AI_BRAINS_PIPE_ACL", "world");
        let err = build_pipe_security_attributes().expect_err("unknown mode must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("service-only") || msg.contains("PIPE_ACL") || msg.contains("unknown"),
            "actionable error: {msg}"
        );
    }

    #[test]
    fn pipe_sddl__service_only_and_interactive__no_world() {
        assert_eq!(
            PIPE_SDDL_INTERACTIVE,
            "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)"
        );
        assert_eq!(PIPE_SDDL_SERVICE_ONLY, "D:(A;;GA;;;SY)(A;;GA;;;BA)");
        assert!(!PIPE_SDDL_INTERACTIVE.contains("WD"));
        assert!(!PIPE_SDDL_SERVICE_ONLY.contains("WD"));
    }
}
