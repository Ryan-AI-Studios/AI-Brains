use crate::command_handler::CaptureContext;
use crate::metadata::CaptureMetadata;

pub fn capture_metadata(context: &CaptureContext) -> crate::Result<CaptureMetadata> {
    let git_metadata = if let Some(path) = context.git_working_dir.as_deref() {
        let metadata = ai_brains_git::collect_metadata(path)?;
        if metadata.is_repository() {
            Some(metadata)
        } else {
            None
        }
    } else {
        None
    };

    Ok(CaptureMetadata { git_metadata })
}
