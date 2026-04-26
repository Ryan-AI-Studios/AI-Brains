use crate::finding::Finding;
use crate::pattern::detect_patterns;

pub fn scan_text(input: &str) -> Vec<Finding> {
    detect_patterns(input)
}
