use regex::Regex;

pub fn normalize_title(title: &str, app_name: &str) -> String {
    let mut title = title.to_string();

    title = title
        .replace(format!("— {}", app_name).as_str(), "")
        .replace(format!("-{}", app_name).as_str(), "");

    let re = Regex::new(r"\(\d+\)").unwrap();
    title = re.replace_all(&title, "").to_string();

    title.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_removes_dash_and_en_dash_app_name() {
        // Simple dash
        assert_eq!(
            normalize_title("My Document -Notion", "Notion"),
            "My Document"
        );
        // En dash
        assert_eq!(
            normalize_title("My Document — Notion", "Notion"),
            "My Document"
        );
        // No match: app name in the middle should not be trimmed
        assert_eq!(
            normalize_title("Notion is great!", "Notion"),
            "Notion is great!"
        );
        // No dash with app name at end: doesn't match, but should return original
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
        // with en dash
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
        // "notion" != "Notion"
        assert_eq!(
            normalize_title("My Document -notion", "Notion"),
            "My Document -notion"
        );
    }
}
