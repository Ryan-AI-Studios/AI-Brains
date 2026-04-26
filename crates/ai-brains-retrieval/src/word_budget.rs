pub fn trim_to_word_budget(input: &str, max_words: usize) -> String {
    input
        .split_whitespace()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn word_count(input: &str) -> usize {
    input.split_whitespace().count()
}
