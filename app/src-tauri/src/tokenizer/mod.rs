mod filepath;
mod structural;
mod title;
mod url;

use std::collections::HashSet;

pub use filepath::tokenize_filepath;
pub use title::tokenize_title;
pub use url::tokenize_url;

use crate::capture::Capture;

pub fn tokenize(capture: &Capture) -> HashSet<String> {
    match capture.source.as_str() {
        "chrome" | "safari" => {
            if let Some(u) = &capture.url {
                tokenize_url(u)
            } else {
                tokenize_title(&capture.title)
            }
        }
        "vscode" | "sublime" | "pycharm" => tokenize_filepath(&capture.doc_identity),
        _ => tokenize_title(&capture.title),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::Capture;

    fn cap(source: &str, url: Option<&str>, title: &str, doc_identity: &str) -> Capture {
        Capture {
            source: source.to_string(),
            url: url.map(|s| s.to_string()),
            title: title.to_string(),
            doc_identity: doc_identity.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_chrome_github() {
        let c = cap(
            "chrome",
            Some("https://github.com/vmandke/kosmos/blob/main/src/episode.rs"),
            "",
            "",
        );
        let t = tokenize(&c);
        assert!(t.contains("vmandke"), "missing vmandke: {t:?}");
        assert!(t.contains("kosmos"), "missing kosmos: {t:?}");
        assert!(t.contains("episode"), "missing episode: {t:?}");
        assert!(!t.contains("blob"), "blob should be filtered: {t:?}");
        assert!(!t.contains("main"), "main should be filtered: {t:?}");
    }

    #[test]
    fn test_chrome_arxiv() {
        let c = cap("chrome", Some("https://arxiv.org/abs/2501.13956"), "", "");
        let t = tokenize(&c);
        assert!(t.contains("arxiv"), "missing arxiv: {t:?}");
        assert!(!t.contains("org"), "org should not be present: {t:?}");
    }

    #[test]
    fn test_chrome_gmail() {
        let c = cap(
            "chrome",
            Some("https://mail.google.com/mail/u/0/#inbox"),
            "",
            "",
        );
        let t = tokenize(&c);
        assert!(t.contains("mail"), "missing mail: {t:?}");
        assert!(!t.contains("google"), "google should be filtered: {t:?}");
    }

    #[test]
    fn test_vscode_filepath() {
        let c = cap(
            "vscode",
            None,
            "episode.rs - kosmos - Visual Studio Code",
            "/Users/vmandke/kosmos/src/episode.rs",
        );
        let t = tokenize(&c);
        assert!(t.contains("vmandke"), "missing vmandke: {t:?}");
        assert!(t.contains("kosmos"), "missing kosmos: {t:?}");
        assert!(t.contains("episode"), "missing episode: {t:?}");
        assert!(!t.contains("src"), "src should be filtered: {t:?}");
        assert!(!t.contains("Users"), "Users should be filtered: {t:?}");
    }

    #[test]
    fn test_version_skipped_in_url() {
        let c = cap("chrome", Some("https://api.example.com/v1/users"), "", "");
        let t = tokenize(&c);
        assert!(!t.contains("v1"), "v1 should be filtered: {t:?}");
        assert!(t.contains("users"), "missing users: {t:?}");
    }

    #[test]
    fn test_query_params_extracted() {
        let c = cap(
            "chrome",
            Some("https://google.com/search?q=rust+async+tokio"),
            "",
            "",
        );
        let t = tokenize(&c);
        assert!(t.contains("rust"), "missing rust: {t:?}");
        assert!(t.contains("async"), "missing async: {t:?}");
        assert!(t.contains("tokio"), "missing tokio: {t:?}");
    }

    #[test]
    fn test_title_strips_app_suffix() {
        let _c = cap(
            "vscode",
            None,
            "episode.rs - kosmos - Visual Studio Code",
            "/tmp/x",
        );
        // vscode routes through filepath tokenizer, but let's test title directly
        let t = tokenize_title("episode.rs - kosmos - Visual Studio Code");
        assert!(t.contains("kosmos"), "missing kosmos: {t:?}");
        assert!(t.contains("episode"), "missing episode: {t:?}");
        assert!(!t.contains("Visual"), "Visual should be stripped: {t:?}");
        assert!(!t.contains("Code"), "Code should be stripped: {t:?}");
    }
}
