mod aerospace;
mod arrangement;
mod capture;

use aerospace::Aerospace;

use crate::capture::capture_arrangement;

fn main() {
    Aerospace::ensure_aerospace_installed();
    // Aerospace::list_apps()
    //     .iter()
    //     .for_each(|app| println!("{:#?}", app));
    // Aerospace::list_workspaces()
    //     .iter()
    //     .for_each(|workspace| println!("{:#?}", workspace));
    // Aerospace::list_monitors()
    //     .iter()
    //     .for_each(|monitor| println!("{:#?}", monitor));
    // Aerospace::list_windows()
    //     .iter()
    //     .for_each(|window| println!("{:#?}", window));

    let aerospace = Aerospace::default();
    let current_arrangement = capture_arrangement(&aerospace, "Current");

    match current_arrangement {
        Ok(arrangement) => {
            println!("Current arrangement:");
            println!("{:#?}", arrangement);

            _ = arrangement.save_to_file("current_arrangement.toml");
        }
        Err(err) => println!("Failed to capture arrangement: {}", err),
    }
}
