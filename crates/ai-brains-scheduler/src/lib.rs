use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Render error: {0}")]
    RenderError(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;

pub struct TaskScheduler;

impl TaskScheduler {
    /// Renders a Windows schtasks command to run the nightly job.
    /// exe_path: Full path to the ai-brains.exe
    /// task_name: Unique name for the task (e.g. "AI-Brains-Nightly")
    /// start_time: Format "HH:mm" (e.g. "03:00")
    pub fn render_create_command(exe_path: &str, task_name: &str, start_time: &str) -> String {
        // We use single quotes around the path to handle spaces in Windows paths,
        // as per schtasks requirements.
        format!(
            "schtasks /create /tn \"{}\" /tr \"'{}' nightly\" /sc daily /st {} /f",
            task_name, exe_path, start_time
        )
    }

    pub fn render_delete_command(task_name: &str) -> String {
        format!("schtasks /delete /tn \"{}\" /f", task_name)
    }

    /// Renders a schtasks command to run the daemon at every user logon with a
    /// caller-supplied /tr value.
    pub fn render_daemon_logon_command_with_tr(
        task_name: &str,
        delay_seconds: u32,
        task_command: &str,
    ) -> String {
        let mm = delay_seconds / 60;
        let ss = delay_seconds % 60;
        format!(
            "schtasks /create /tn \"{task_name}\" /tr \"{task_command}\" /sc ONLOGON /delay {mm:04}:{ss:02} /f",
        )
    }

    /// Renders a schtasks command to run the daemon at every user logon.
    /// exe_path: Full path to ai-brainsd.exe
    /// task_name: Unique name (e.g. "AI-Brains-Daemon")
    /// delay_seconds: Seconds to wait after logon before starting (30 recommended)
    pub fn render_daemon_logon_command(
        exe_path: &str,
        task_name: &str,
        delay_seconds: u32,
    ) -> String {
        Self::render_daemon_logon_command_with_tr(
            task_name,
            delay_seconds,
            &format!("'{exe_path}'"),
        )
    }
}

pub struct ServiceScheduler;

impl ServiceScheduler {
    const SERVICE_NAME: &'static str = "AI-Brains-Daemon";
    const DISPLAY_NAME: &'static str = "AI-Brains Daemon";
    const DESCRIPTION: &'static str = "Local-first AI coding memory vault — captures conversation history without tool logs or hidden thinking.";

    pub fn render_install_command(exe_path: &str, _env_sidecar_path: &str) -> String {
        format!(
            "sc create \"{name}\" binPath= \"\\\"{exe}\\\" --service\" start= delayed-auto DisplayName= \"{display}\"",
            name = Self::SERVICE_NAME,
            exe = exe_path,
            display = Self::DISPLAY_NAME,
        )
    }

    pub fn render_description_command() -> String {
        format!(
            "sc description \"{name}\" \"{desc}\"",
            name = Self::SERVICE_NAME,
            desc = Self::DESCRIPTION,
        )
    }

    pub fn render_env_sidecar_hint(env_sidecar_path: &str) -> String {
        format!("Write daemon env vars to: {path}", path = env_sidecar_path,)
    }

    pub fn render_start_command() -> String {
        format!("sc start \"{}\"", Self::SERVICE_NAME)
    }

    pub fn render_stop_command() -> String {
        format!("sc stop \"{}\"", Self::SERVICE_NAME)
    }

    pub fn render_uninstall_command() -> String {
        format!("sc delete \"{}\"", Self::SERVICE_NAME)
    }

    pub fn service_name() -> &'static str {
        Self::SERVICE_NAME
    }

    pub fn service_description() -> &'static str {
        Self::DESCRIPTION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_create_command() {
        let cmd = TaskScheduler::render_create_command(
            r"C:\Program Files\AI-Brains\ai-brains.exe",
            "AI-Brains-Nightly",
            "03:00",
        );
        assert_eq!(
            cmd,
            r#"schtasks /create /tn "AI-Brains-Nightly" /tr "'C:\Program Files\AI-Brains\ai-brains.exe' nightly" /sc daily /st 03:00 /f"#
        );
    }

    #[test]
    fn test_render_daemon_logon_command() {
        let cmd = TaskScheduler::render_daemon_logon_command(
            r"C:\Users\RyanB\.cargo\bin\ai-brainsd.exe",
            "AI-Brains-Daemon",
            30,
        );
        // T78: must use the same single-quote convention as
        // render_create_command. The previous escaped-doublequote format
        // produced a literal trailing backslash that schtasks rejected
        // with "Access is denied".
        assert_eq!(
            cmd,
            r#"schtasks /create /tn "AI-Brains-Daemon" /tr "'C:\Users\RyanB\.cargo\bin\ai-brainsd.exe'" /sc ONLOGON /delay 0000:30 /f"#
        );
    }

    #[test]
    fn test_render_daemon_logon_command_with_spaces_in_path() {
        // Regression: schtasks rejects a path with spaces if quoting is
        // malformed. Verify the single-quote convention survives paths that
        // contain spaces.
        let cmd = TaskScheduler::render_daemon_logon_command(
            r"C:\Program Files\AI-Brains\ai-brainsd.exe",
            "AI-Brains-Daemon",
            60,
        );
        assert_eq!(
            cmd,
            r#"schtasks /create /tn "AI-Brains-Daemon" /tr "'C:\Program Files\AI-Brains\ai-brainsd.exe'" /sc ONLOGON /delay 0001:00 /f"#
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__render_install_command__includes_sc_create_and_service_flag() {
        let cmd = ServiceScheduler::render_install_command(
            r"C:\Program Files\AI-Brains\ai-brainsd.exe",
            r"C:\ProgramData\AI-Brains\daemon.env",
        );
        assert!(cmd.starts_with("sc create \"AI-Brains-Daemon\""));
        assert!(cmd.contains("--service"));
        assert!(cmd.contains("start= delayed-auto"));
        assert!(
            cmd.contains(r#"binPath= "\"C:\Program Files\AI-Brains\ai-brainsd.exe\" --service""#),
            "binPath must escape inner quotes: {cmd}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__render_description_command__includes_sc_description() {
        let cmd = ServiceScheduler::render_description_command();
        assert!(cmd.starts_with("sc description \"AI-Brains-Daemon\""));
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__render_uninstall_command__includes_sc_delete() {
        let cmd = ServiceScheduler::render_uninstall_command();
        assert_eq!(cmd, "sc delete \"AI-Brains-Daemon\"");
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__render_start_command__includes_sc_start() {
        let cmd = ServiceScheduler::render_start_command();
        assert_eq!(cmd, "sc start \"AI-Brains-Daemon\"");
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__render_stop_command__includes_sc_stop() {
        let cmd = ServiceScheduler::render_stop_command();
        assert_eq!(cmd, "sc stop \"AI-Brains-Daemon\"");
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__render_env_sidecar_hint__includes_path() {
        let cmd = ServiceScheduler::render_env_sidecar_hint(r"C:\ProgramData\AI-Brains\daemon.env");
        assert!(cmd.contains("C:\\ProgramData\\AI-Brains\\daemon.env"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn service_scheduler__service_name__returns_constant() {
        assert_eq!(ServiceScheduler::service_name(), "AI-Brains-Daemon");
    }
}
