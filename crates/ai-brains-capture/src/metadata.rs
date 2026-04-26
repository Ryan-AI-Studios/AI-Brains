use ai_brains_git::GitMetadata;

#[derive(Debug, Clone, Default)]
pub struct CaptureMetadata {
    pub git_metadata: Option<GitMetadata>,
}
