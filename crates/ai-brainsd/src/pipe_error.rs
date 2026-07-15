use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeErrorKind {
    AccessDenied,
    PipeBusy,
    Other,
}

pub fn classify_pipe_error(err: &io::Error) -> PipeErrorKind {
    if let Some(raw) = err.raw_os_error() {
        const ERROR_ACCESS_DENIED: i32 = 5;
        const ERROR_PIPE_BUSY: i32 = 231;
        const ERROR_SHARING_VIOLATION: i32 = 32;
        const ERROR_PIPE_NOT_CONNECTED: i32 = 229;

        return match raw {
            ERROR_ACCESS_DENIED => PipeErrorKind::AccessDenied,
            ERROR_PIPE_BUSY | ERROR_SHARING_VIOLATION => PipeErrorKind::PipeBusy,
            ERROR_PIPE_NOT_CONNECTED => PipeErrorKind::PipeBusy,
            _ => PipeErrorKind::Other,
        };
    }
    match err.kind() {
        io::ErrorKind::PermissionDenied => PipeErrorKind::AccessDenied,
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => PipeErrorKind::PipeBusy,
        _ => PipeErrorKind::Other,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn classify_pipe_error__access_denied_raw_os_5__returns_access_denied() {
        let err = io::Error::from_raw_os_error(5);
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::AccessDenied);
    }

    #[test]
    fn classify_pipe_error__pipe_busy_raw_os_231__returns_pipe_busy() {
        let err = io::Error::from_raw_os_error(231);
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::PipeBusy);
    }

    #[test]
    fn classify_pipe_error__sharing_violation_raw_os_32__returns_pipe_busy() {
        let err = io::Error::from_raw_os_error(32);
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::PipeBusy);
    }

    #[test]
    fn classify_pipe_error__other_raw_os__returns_other() {
        let err = io::Error::from_raw_os_error(1234);
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::Other);
    }

    #[test]
    fn classify_pipe_error__permission_denied_kind__returns_access_denied() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::AccessDenied);
    }

    #[test]
    fn classify_pipe_error__would_block_kind__returns_pipe_busy() {
        let err = io::Error::new(io::ErrorKind::WouldBlock, "busy");
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::PipeBusy);
    }

    #[test]
    fn classify_pipe_error__other_kind__returns_other() {
        let err = io::Error::new(io::ErrorKind::NotFound, "missing");
        assert_eq!(classify_pipe_error(&err), PipeErrorKind::Other);
    }
}
