mod aerospace;

fn main() {
    aerospace::ensure_aerospace_installed();
    aerospace::aerospace_list_apps();
    aerospace::aerospace_list_workspaces();
    aerospace::aerospace_list_monitors();
    aerospace::aerospace_list_windows();
}
