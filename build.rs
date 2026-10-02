use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-env-changed=AEROSTAGE_GIT_SHA");

    let sha = std::env::var("AEROSTAGE_GIT_SHA")
        .ok()
        .filter(|sha| !sha.is_empty())
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "--short=12", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
                .filter(|sha| !sha.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_owned());

    println!("cargo:rustc-env=AEROSTAGE_GIT_SHA={sha}");
}
