use std::collections::HashSet;
use url::Url;
use super::structural::{is_hash, is_version, is_web_noise};
use super::title::tokenize_title;

pub fn tokenize_url(raw: &str) -> HashSet<String> {
    let Ok(parsed) = Url::parse(raw) else {
        return tokenize_title(raw);
    };

    let mut tokens = HashSet::new();

    // Host: extract meaningful part, skip TLD and "www"
    if let Some(host) = parsed.host_str() {
        let parts: Vec<&str> = host.split('.').collect();
        let meaningful: Vec<&str> = match parts.as_slice() {
            [sub, _, _] if *sub != "www" => vec![sub],
            [domain, _] => vec![domain],
            _ => vec![],
        };
        tokens.extend(
            meaningful
                .into_iter()
                .map(|s| s.to_lowercase())
                .filter(|s| s.len() > 2),
        );
    }

    // Path segments
    for segment in parsed.path_segments().into_iter().flatten() {
        if is_version(segment) || is_hash(segment) || segment.len() <= 1 {
            continue;
        }
        let base = segment.split('.').next().unwrap_or(segment);
        if is_web_noise(base) {
            continue;
        }
        if base.len() > 2 {
            tokens.insert(base.to_lowercase());
        }
    }

    // Query param values (e.g. ?q=rust+async)
    for (_, value) in parsed.query_pairs() {
        tokens.extend(
            value
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| s.len() > 2)
                .map(|s| s.to_lowercase()),
        );
    }

    tokens
}
