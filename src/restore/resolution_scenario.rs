use super::resolution::WindowResolution;
use super::types::ResolvedWindowMatch;
use crate::aerospace::AerospaceWindow;
use crate::stage::{Stage, StageWindow, StageWorkspace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum App {
    Safari,
    Slack,
    Code,
    Terminal,
    Custom {
        name: &'static str,
        bundle_id: &'static str,
    },
}

impl App {
    pub const fn custom(name: &'static str, bundle_id: &'static str) -> Self {
        Self::Custom { name, bundle_id }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Safari => "Safari",
            Self::Slack => "Slack",
            Self::Code => "Code",
            Self::Terminal => "Terminal",
            Self::Custom { name, .. } => name,
        }
    }

    pub fn bundle_id(self) -> &'static str {
        match self {
            Self::Safari => "com.apple.Safari",
            Self::Slack => "com.tinyspeck.slackmacgap",
            Self::Code => "com.microsoft.VSCode",
            Self::Terminal => "com.apple.Terminal",
            Self::Custom { bundle_id, .. } => bundle_id,
        }
    }

    fn stage_window(self, title: Option<&str>) -> StageWindow {
        StageWindow {
            app: Some(self.name().to_string()),
            bundle_id: Some(self.bundle_id().to_string()),
            title: title.map(str::to_string),
        }
    }
}

struct LiveWindow {
    handle: String,
    window: AerospaceWindow,
}

pub struct Scenario {
    next_id: u32,
    default_workspace: Option<String>,
    lives: Vec<LiveWindow>,
    workspaces: Vec<StageWorkspace>,
}

pub struct LiveWindowBuilder {
    scenario: Scenario,
    handle: String,
    app: App,
    title: String,
}

pub struct WorkspaceBuilder {
    name: String,
    windows: Vec<StageWindow>,
}

#[derive(Debug)]
struct PendingClaim {
    workspace: String,
    app: Option<String>,
    title: Option<String>,
}

pub struct ScenarioResult {
    lives: Vec<LiveWindow>,
    stage: Stage,
    resolved: Vec<ResolvedWindowMatch>,
    pending: Vec<PendingClaim>,
    unresolved_ids: Vec<u32>,
}

impl Scenario {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            default_workspace: None,
            lives: Vec::new(),
            workspaces: Vec::new(),
        }
    }

    pub fn default_workspace(mut self, workspace: &str) -> Self {
        self.default_workspace = Some(workspace.to_string());
        self
    }

    pub fn live(self, handle: &str, app: App, title: &str) -> LiveWindowBuilder {
        if self.lives.iter().any(|live| live.handle == handle) {
            panic!("duplicate live window handle `{handle}`");
        }

        LiveWindowBuilder {
            scenario: self,
            handle: handle.to_string(),
            app,
            title: title.to_string(),
        }
    }

    pub fn workspace(
        mut self,
        name: &str,
        build: impl FnOnce(WorkspaceBuilder) -> WorkspaceBuilder,
    ) -> Self {
        self.workspaces
            .push(build(WorkspaceBuilder::new(name)).finish());
        self
    }

    pub fn resolve(self) -> ScenarioResult {
        let stage = Stage {
            name: None,
            description: None,
            workspaces: self.workspaces,
            default_workspace: self.default_workspace,
        };
        let windows: Vec<AerospaceWindow> =
            self.lives.iter().map(|live| live.window.clone()).collect();
        let (resolved, pending, unresolved_ids) = {
            let resolution = WindowResolution::resolve(&stage, &windows);
            let pending = resolution
                .pending_targets
                .iter()
                .map(|target| PendingClaim {
                    workspace: target.target_workspace.name.clone(),
                    app: target.target_window.app.clone(),
                    title: target.target_window.title.clone(),
                })
                .collect();
            let unresolved_ids = resolution
                .unresolved_windows
                .iter()
                .map(|window| window.window_id)
                .collect();
            (resolution.resolved_windows, pending, unresolved_ids)
        };

        ScenarioResult {
            lives: self.lives,
            stage,
            resolved,
            pending,
            unresolved_ids,
        }
    }
}

impl LiveWindowBuilder {
    pub fn on(self, workspace: &str) -> Scenario {
        let mut scenario = self.scenario;
        let window_id = scenario.next_id;
        scenario.next_id += 1;
        scenario.lives.push(LiveWindow {
            handle: self.handle,
            window: AerospaceWindow::dummy()
                .with_window_id(window_id)
                .with_app_name(self.app.name())
                .with_bundle_id(self.app.bundle_id())
                .with_window_title(&self.title)
                .with_workspace(workspace),
        });
        scenario
    }
}

impl WorkspaceBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            windows: Vec::new(),
        }
    }

    pub fn wants(mut self, app: App, title: &str) -> Self {
        self.windows.push(app.stage_window(Some(title)));
        self
    }

    pub fn wants_app(mut self, app: App) -> Self {
        self.windows.push(app.stage_window(None));
        self
    }

    pub fn wants_title(mut self, title: &str) -> Self {
        self.windows.push(StageWindow {
            app: None,
            bundle_id: None,
            title: Some(title.to_string()),
        });
        self
    }

    fn finish(self) -> StageWorkspace {
        StageWorkspace {
            name: self.name,
            windows: self.windows,
            layout: None,
        }
    }
}

impl ScenarioResult {
    fn handle_for(&self, window_id: u32) -> &str {
        self.lives
            .iter()
            .find(|live| live.window.window_id == window_id)
            .map(|live| live.handle.as_str())
            .unwrap_or("<unknown>")
    }

    fn live(&self, handle: &str) -> &LiveWindow {
        self.lives
            .iter()
            .find(|live| live.handle == handle)
            .unwrap_or_else(|| panic!("unknown live window handle `{handle}`\n\n{}", self.dump()))
    }

    fn resolved_for(&self, handle: &str) -> Option<&ResolvedWindowMatch> {
        let window_id = self.live(handle).window.window_id;
        self.resolved
            .iter()
            .find(|matched| matched.window_id == window_id)
    }

    fn is_fallback(&self, matched: &ResolvedWindowMatch) -> bool {
        let Some(default) = self.stage.default_workspace.as_deref() else {
            return false;
        };
        matched.target_workspace == default
            && !self
                .stage
                .workspaces
                .iter()
                .any(|workspace| workspace.name == matched.target_workspace)
    }

    fn dump(&self) -> String {
        let mut out = String::from("live:\n");
        for live in &self.lives {
            out.push_str(&format!(
                "  {:<8} #{:<3} {:<10} {:<24} now on {}\n",
                live.handle,
                live.window.window_id,
                live.window.app_name,
                format!("\"{}\"", live.window.window_title),
                live.window.workspace
            ));
        }

        out.push_str("stage:\n");
        for workspace in &self.stage.workspaces {
            let claims: Vec<String> = workspace
                .windows
                .iter()
                .map(|window| {
                    let app = window.app.as_deref().unwrap_or("*");
                    match window.title.as_deref() {
                        Some(title) => format!("{app} \"{title}\""),
                        None => app.to_string(),
                    }
                })
                .collect();
            out.push_str(&format!("  {}: {}\n", workspace.name, claims.join(", ")));
        }
        match &self.stage.default_workspace {
            Some(default) => out.push_str(&format!("  default: {default}\n")),
            None => out.push_str("  default: (none)\n"),
        }

        out.push_str("result:\n");
        let matched: Vec<_> = self
            .resolved
            .iter()
            .filter(|matched| !self.is_fallback(matched))
            .collect();
        if matched.is_empty() {
            out.push_str("  matched:  (none)\n");
        } else {
            for matched in matched {
                out.push_str(&format!(
                    "  matched:  {} -> {}\n",
                    self.handle_for(matched.window_id),
                    matched.target_workspace
                ));
            }
        }

        if self.pending.is_empty() {
            out.push_str("  pending:  (none)\n");
        } else {
            for pending in &self.pending {
                let app = pending.app.as_deref().unwrap_or("*");
                let title = pending
                    .title
                    .as_deref()
                    .map(|title| format!(" \"{title}\""))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  pending:  {app}{title} on {}\n",
                    pending.workspace
                ));
            }
        }

        let leftover: Vec<_> = self
            .unresolved_ids
            .iter()
            .map(|id| self.handle_for(*id))
            .collect();
        if leftover.is_empty() {
            out.push_str("  leftover: (none)\n");
        } else {
            out.push_str(&format!("  leftover: {}\n", leftover.join(", ")));
        }

        let fallback: Vec<_> = self
            .resolved
            .iter()
            .filter(|matched| self.is_fallback(matched))
            .collect();
        if fallback.is_empty() {
            out.push_str("  fallback: (none)\n");
        } else {
            for matched in fallback {
                out.push_str(&format!(
                    "  fallback: {} -> {}\n",
                    self.handle_for(matched.window_id),
                    matched.target_workspace
                ));
            }
        }

        out
    }

    fn fail(&self, message: String) -> ! {
        panic!("{message}\n\n{}", self.dump());
    }

    pub fn matched(&self, handle: &str, workspace: &str) -> &Self {
        match self.resolved_for(handle) {
            Some(matched) if matched.target_workspace == workspace => self,
            Some(matched) => self.fail(format!(
                "expected `{handle}` to match workspace `{workspace}`, got `{}`",
                matched.target_workspace
            )),
            None => self.fail(format!(
                "expected `{handle}` to match workspace `{workspace}`, but it was not resolved"
            )),
        }
    }

    pub fn fallback(&self, handle: &str, workspace: &str) -> &Self {
        match self.resolved_for(handle) {
            Some(matched)
                if matched.target_workspace == workspace && self.is_fallback(matched) =>
            {
                self
            }
            Some(matched) => self.fail(format!(
                "expected `{handle}` to fall back to `{workspace}`, got workspace `{}` (fallback={})",
                matched.target_workspace,
                self.is_fallback(matched)
            )),
            None => self.fail(format!(
                "expected `{handle}` to fall back to `{workspace}`, but it was not resolved"
            )),
        }
    }

    pub fn pending_title(&self, title: &str) -> &Self {
        if self
            .pending
            .iter()
            .any(|pending| pending.title.as_deref() == Some(title))
        {
            self
        } else {
            self.fail(format!(
                "expected a pending stage claim with title `{title}`"
            ))
        }
    }

    pub fn pending_app(&self, app: App) -> &Self {
        if self
            .pending
            .iter()
            .any(|pending| pending.app.as_deref() == Some(app.name()))
        {
            self
        } else {
            self.fail(format!(
                "expected a pending stage claim for app `{}`",
                app.name()
            ))
        }
    }

    pub fn unresolved(&self, handle: &str) -> &Self {
        let window_id = self.live(handle).window.window_id;
        if self.unresolved_ids.contains(&window_id) {
            self
        } else {
            self.fail(format!("expected `{handle}` to remain unresolved"))
        }
    }

    pub fn pending_empty(&self) -> &Self {
        if self.pending.is_empty() {
            self
        } else {
            self.fail("expected no pending stage claims".to_string())
        }
    }

    pub fn unresolved_empty(&self) -> &Self {
        if self.unresolved_ids.is_empty() {
            self
        } else {
            self.fail("expected no unresolved live windows".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTES: App = App::custom("Notes", "com.apple.Notes");
    const MAIL: App = App::custom("Mail", "com.apple.mail");

    #[test]
    fn exact_unique_title_wins_across_workspaces() {
        Scenario::new()
            .live("inbox", App::Safari, "Inbox")
            .on("2")
            .live("other", App::Safari, "Other")
            .on("3")
            .workspace("1", |ws| ws.wants(App::Safari, "Inbox"))
            .resolve()
            .matched("inbox", "1")
            .unresolved("other")
            .pending_empty();
    }

    #[test]
    fn consuming_an_exact_title_lets_unique_identity_bind_the_rest() {
        Scenario::new()
            .live("inbox", App::Safari, "Inbox")
            .on("8")
            .live("extra", App::Safari, "Random")
            .on("9")
            .workspace("1", |ws| ws.wants(App::Safari, "Inbox"))
            .workspace("2", |ws| ws.wants(App::Safari, "Something else"))
            .resolve()
            .matched("inbox", "1")
            .matched("extra", "2")
            .pending_empty()
            .unresolved_empty();
    }

    #[test]
    fn exact_title_wins_when_a_similar_title_is_also_present() {
        Scenario::new()
            .live("exact", NOTES, "Project Ideas 2026")
            .on("3")
            .live("similar", NOTES, "Project Ideas 2025")
            .on("4")
            .workspace("work", |ws| ws.wants(NOTES, "Project Ideas 2026"))
            .resolve()
            .matched("exact", "work")
            .unresolved("similar")
            .pending_empty();
    }

    #[test]
    fn similar_titles_above_threshold_do_not_bind_without_an_exact_match() {
        Scenario::new()
            .live("a", NOTES, "Project Ideas 2025")
            .on("3")
            .live("b", NOTES, "Project Ideas 2024")
            .on("4")
            .workspace("work", |ws| ws.wants(NOTES, "Project Ideas 2026"))
            .resolve()
            .pending_title("Project Ideas 2026")
            .unresolved("a")
            .unresolved("b");
    }

    #[test]
    fn unique_app_race_goes_to_the_first_stage_workspace() {
        Scenario::new()
            .live("safari", App::Safari, "Inbox")
            .on("9")
            .workspace("1", |ws| ws.wants_app(App::Safari))
            .workspace("2", |ws| ws.wants_app(App::Safari))
            .resolve()
            .matched("safari", "1")
            .pending_app(App::Safari)
            .unresolved_empty();
    }

    #[test]
    fn unique_app_already_on_target_workspace_binds_without_a_title() {
        Scenario::new()
            .live("here", App::Slack, "Team")
            .on("work")
            .live("elsewhere", App::Slack, "DMs")
            .on("personal")
            .workspace("work", |ws| ws.wants_app(App::Slack))
            .resolve()
            .matched("here", "work")
            .unresolved("elsewhere")
            .pending_empty();
    }

    #[test]
    fn fallback_moves_windows_the_stage_did_not_claim() {
        Scenario::new()
            .default_workspace("9")
            .live("inbox", App::Safari, "Inbox")
            .on("2")
            .live("team", App::Slack, "Team")
            .on("3")
            .workspace("1", |ws| ws.wants(App::Safari, "Inbox"))
            .resolve()
            .matched("inbox", "1")
            .fallback("team", "9")
            .pending_empty()
            .unresolved_empty();
    }

    #[test]
    fn fallback_leaves_windows_that_unmatched_stage_entries_may_own() {
        Scenario::new()
            .default_workspace("9")
            .live("tab_a", App::Safari, "Tab A")
            .on("2")
            .live("tab_b", App::Safari, "Tab B")
            .on("3")
            .live("team", App::Slack, "Team")
            .on("4")
            .workspace("1", |ws| ws.wants(App::Safari, "Missing Tab"))
            .resolve()
            .pending_title("Missing Tab")
            .unresolved("tab_a")
            .unresolved("tab_b")
            .fallback("team", "9");
    }

    #[test]
    fn extras_stay_unresolved_when_there_is_no_default_workspace() {
        Scenario::new()
            .live("inbox", App::Safari, "Inbox")
            .on("2")
            .live("team", App::Slack, "Team")
            .on("3")
            .live("code", App::Code, "main.rs")
            .on("4")
            .live("term", App::Terminal, "zsh")
            .on("5")
            .workspace("1", |ws| ws.wants(App::Safari, "Inbox"))
            .resolve()
            .matched("inbox", "1")
            .unresolved("team")
            .unresolved("code")
            .unresolved("term")
            .pending_empty();
    }

    #[test]
    fn title_only_stage_window_can_bind_any_app() {
        Scenario::new()
            .live("mail", MAIL, "Inbox")
            .on("2")
            .live("safari", App::Safari, "Other")
            .on("3")
            .workspace("1", |ws| ws.wants_title("Inbox"))
            .resolve()
            .matched("mail", "1")
            .unresolved("safari")
            .pending_empty();
    }
}
