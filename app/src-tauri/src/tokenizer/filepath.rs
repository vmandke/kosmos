use std::collections::HashSet;

const SYSTEM_PREFIXES: &[&str] = &["/usr", "/var", "/etc", "/tmp", "/opt", "/bin", "/lib"];
const SKIP_SEGMENTS: &[&str] = &[
    "Users", "home", "src", "lib", "bin", "app",
    "target", "debug", "release", "build", "node_modules",
];

pub fn tokenize_filepath(path: &str) -> HashSet<String> {
    if SYSTEM_PREFIXES.iter().any(|p| path.starts_with(p)) {
        return HashSet::new();
    }

    path.split('/')
        .filter(|s| s.len() > 2)
        .filter(|s| !SKIP_SEGMENTS.contains(s))
        .map(|s| s.split('.').next().unwrap_or(s).to_lowercase())
        .filter(|s| s.len() > 2)
        .collect()
}
