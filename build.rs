extern crate clap;
extern crate lalrpop;

use clap::crate_version;
use std::process::Command;

fn main() {
    // Set up LALRPOP.
    let lalrpop_config = lalrpop::Configuration::new()
        .log_verbose()
        .process_current_dir();
    match lalrpop_config {
        Ok(x) => {
            println!("LALRPOP build ran correctly:\n{:?}", x);
        }
        Err(e) => {
            eprintln!("Error while running LALRPOP during build: {:?}", e);
        }
    }

    // Get the current git commit hash.
    // Taken from https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let version_string = format!("{}-{}", crate_version!(), git_hash);
    println!("cargo:rustc-env=VERSION_STR={}", version_string);
}
