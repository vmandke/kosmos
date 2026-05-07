pub fn is_version(s: &str) -> bool {
    let s = s.trim_start_matches(|c| c == 'v' || c == 'V');
    !s.is_empty() && s.chars().all(|c: char| c.is_numeric() || c == '.')
}

pub fn is_hash(s: &str) -> bool {
    s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn is_web_noise(s: &str) -> bool {
    matches!(
        s,
        "index" | "home" | "page" | "view" | "edit"
            | "new" | "show" | "list" | "feed" | "main"
            | "blob" | "tree" | "refs" | "src" | "raw"
            | "wiki" | "issues" | "pull" | "commit"
    )
}
