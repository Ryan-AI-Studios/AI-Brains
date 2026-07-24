#![cfg(windows)]
#![allow(clippy::disallowed_methods)]

use std::io;

use windows::{
    Win32::Security::{
        Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorA, PSECURITY_DESCRIPTOR,
        SECURITY_ATTRIBUTES,
    },
    core::PCSTR,
};

const SDDL: &str = "D:(A;;GA;;;WD)";

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
            "ConvertStringSecurityDescriptorToSecurityDescriptorW failed: {:?}",
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
    fn build_pipe_security_attributes__dacl_present_grants_everyone() {
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
            "DACL must be present (explicit Everyone grant)"
        );
        assert!(!dacl_ptr.is_null(), "DACL pointer must not be null");
    }
}
