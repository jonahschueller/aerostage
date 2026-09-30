use std::sync::LazyLock;

use regex::Regex;

static COUNT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(\d+\)").unwrap());

pub fn normalize_title(title: &str, app_name: &str) -> String {
    let mut title = title.to_string();

    if !app_name.is_empty() {
        title = title
            .replace(format!("— {app_name}").as_str(), "")
            .replace(format!("-{app_name}").as_str(), "");
    }

    title = COUNT_RE.replace_all(&title, "").into_owned();
    title = title.trim_start_matches(['●', '•']).trim().to_string();

    title
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_removes_dash_and_en_dash_app_name() {
        assert_eq!(
            normalize_title("My Document -Notion", "Notion"),
            "My Document"
        );
        assert_eq!(
            normalize_title("My Document — Notion", "Notion"),
            "My Document"
        );
        assert_eq!(
            normalize_title("Notion is great!", "Notion"),
            "Notion is great!"
        );
        assert_eq!(
            normalize_title("My Document Notion", "Notion"),
            "My Document Notion"
        );
    }

    #[test]
    fn it_removes_parenthesis_counts() {
        assert_eq!(
            normalize_title("Meeting Notes (5)", "Notion"),
            "Meeting Notes"
        );
        assert_eq!(
            normalize_title("Task List (123) -Notion", "Notion"),
            "Task List"
        );
        assert_eq!(normalize_title("Inbox (42) — Notion", "Notion"), "Inbox");
    }

    #[test]
    fn it_handles_leading_and_trailing_spaces() {
        assert_eq!(
            normalize_title("  Hello World -Notion  ", "Notion"),
            "Hello World"
        );
        assert_eq!(normalize_title("  Hello (3) -Notion  ", "Notion"), "Hello");
        assert_eq!(normalize_title("  Meeting (4)  ", "Notion"), "Meeting");
    }

    #[test]
    fn it_handles_titles_without_app_name_or_patterns() {
        assert_eq!(normalize_title("Plain Title", "Notion"), "Plain Title");
        assert_eq!(normalize_title("", "Notion"), "");
    }

    #[test]
    fn it_is_case_sensitive() {
        assert_eq!(
            normalize_title("My Document -notion", "Notion"),
            "My Document -notion"
        );
    }

    #[test]
    fn it_strips_dirty_file_markers() {
        assert_eq!(normalize_title("● main.rs", "Code"), "main.rs");
        assert_eq!(normalize_title("• main.rs — Code", "Code"), "main.rs");
    }
}
