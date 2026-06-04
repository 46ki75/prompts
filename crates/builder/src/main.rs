//! CLI entry point: `builder [ROOT] [OUT]`.
//!
//! Defaults to scanning the current directory and writing to `dist/`.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let out = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist"));

    match builder::build(&root, &out) {
        Ok(index) => {
            for entry in &index {
                println!("{} -> resources/{}", entry.name, entry.path);
            }
            println!(
                "wrote {} prompt(s) + list.json to {}",
                index.len(),
                out.join("resources").display()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
