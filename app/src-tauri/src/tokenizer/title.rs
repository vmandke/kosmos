use std::collections::HashSet;

const APP_SUFFIXES: &[&str] = &[
    " - Visual Studio Code",
    " - Sublime Text",
    " — Google Chrome",
    " - Google Chrome",
    " - PyCharm",
    " - PyCharm Community Edition",
    " - Discord",
];

pub fn tokenize_title(title: &str) -> HashSet<String> {
    let mut clean = title.to_string();
    for suffix in APP_SUFFIXES {
        if let Some(stripped) = clean.strip_suffix(suffix) {
            clean = stripped.to_string();
            break;
        }
    }

    clean
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .filter(|s| !s.chars().all(|c| c.is_numeric()))
        .map(|s| s.to_lowercase())
        .collect()
}
