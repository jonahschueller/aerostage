mod aerospace;
mod arrangement;
mod capture;
mod cli;
mod restore;

use std::time::Duration;

use aerospace::Aerospace;

use crate::{arrangement::Arrangement, capture::capture_arrangement};

use clap::{Parser, Subcommand};

fn deprecated_main() {
    use std::time::Instant;
    let now = Instant::now();

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
    // let current_arrangement = capture_arrangement(&aerospace, "Current");
    let current_arrangement = Arrangement::load_from_file(&"current_arrangement.toml");

    match current_arrangement {
        Ok(arrangement) => {
            println!("Current arrangement:");
            println!("{:#?}", arrangement);

            // _ = arrangement.save_to_file("current_arrangement.toml");

            // let seconds = 5;
            // println!(
            //     "Sleeping for {} seconds. Move some windows to see the plan.",
            //     seconds
            // );
            // std::thread::sleep(Duration::from_secs(seconds));

            println!("Restoring arrangement: ");
            _ = restore::restore_arrangement(&aerospace, &arrangement);
        }
        Err(err) => println!("Failed to capture arrangement: {}", err),
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn main() {
    let cli = cli::Cli::parse();

    cli::execute_command(cli.command);
}
