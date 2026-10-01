pub mod format;

pub(crate) fn parse_workspace_list(workspaces: &str) -> Vec<&str> {
    workspaces
        .split(',')
        .map(str::trim)
        .filter(|workspace| !workspace.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_list_trims_and_drops_empty_entries() {
        assert_eq!(parse_workspace_list("1, 2, 3"), vec!["1", "2", "3"]);
        assert_eq!(parse_workspace_list("1,,2,"), vec!["1", "2"]);
    }
}
