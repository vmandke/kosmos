/// Normalize text for fingerprinting. Original content is preserved separately.
/// Collapses all whitespace (including intra-line runs) so "hello  world" and
/// "hello world" produce the same fingerprint.
pub fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}
