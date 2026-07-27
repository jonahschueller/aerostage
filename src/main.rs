mod aerospace;

use aerospace::Aerospace;

fn main() {
    Aerospace::ensure_aerospace_installed();
    Aerospace::list_apps()
        .iter()
        .for_each(|app| println!("{:#?}", app));
    Aerospace::list_workspaces()
        .iter()
        .for_each(|workspace| println!("{:#?}", workspace));
    Aerospace::list_monitors()
        .iter()
        .for_each(|monitor| println!("{:#?}", monitor));
    Aerospace::list_windows()
        .iter()
        .for_each(|window| println!("{:#?}", window));
}
