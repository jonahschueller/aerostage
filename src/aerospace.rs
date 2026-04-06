use subprocess::{Exec, Redirection};

fn execute_aerospace_command(command: &str) -> Result<String, String> {
    const AEROSPACE_COMMAND: &str = "aerospace";

    let args: Vec<&str> = command.split_whitespace().collect();

    let output = Exec::cmd(AEROSPACE_COMMAND)
        .args(&args)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()
        .expect("Failed to execute aerospace command");

    if output.success() {
        Ok(output.stdout_str().to_string())
    } else {
        Err(output.stderr_str().to_string())
    }
}

enum AerospaceCommand {
    ListApps,
    ListWorkspaces,
    ListMonitors,
    ListWindows,
}

impl AerospaceCommand {
    fn as_str(&self) -> &'static str {
        match self {
            AerospaceCommand::ListApps => "list-apps",
            AerospaceCommand::ListWorkspaces => "list-workspaces",
            AerospaceCommand::ListMonitors => "list-monitors",
            AerospaceCommand::ListWindows => "list-windows",
        }
    }
}

fn query_aerospace(command: AerospaceCommand, args: Option<&str>) -> Result<String, String> {
    let json_query = match args {
        Some(a) => format!("{} --json {}", command.as_str(), a),
        None => format!("{} --json", command.as_str()),
    };
    execute_aerospace_command(&json_query)
}

pub fn ensure_aerospace_installed() {
    if which::which("aerospace").is_err() {
        eprintln!("Error: 'aerospace' command not found. Please install Aerospace CLI tool.");
        std::process::exit(1);
    }
}

pub fn aerospace_list_apps() {
    let result = query_aerospace(AerospaceCommand::ListApps, None);
    match result {
        Ok(output) => println!("{}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

pub fn aerospace_list_workspaces() {
    let result = query_aerospace(AerospaceCommand::ListWorkspaces, Some("--all"));
    match result {
        Ok(output) => println!("{}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

pub fn aerospace_list_monitors() {
    let result = query_aerospace(AerospaceCommand::ListMonitors, None);
    match result {
        Ok(output) => println!("{}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

pub fn aerospace_list_windows() {
    let result = query_aerospace(AerospaceCommand::ListWindows, Some("--all"));
    match result {
        Ok(output) => println!("{}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

