use std::{path::PathBuf, process::Command};

use colored::Colorize;
use duct::cmd;

pub fn fetch_content(file_path: Option<&PathBuf>) -> Option<String> {
    if let Some(file_path) = file_path {
        match Command::new("bat")
            .arg("--color=always")
            .arg(file_path.as_os_str())
            .output()
        {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(string) => Some(string),
                Err(err) => Some(format!("{:?}", err).red().to_string()),
            },
            Err(err) => Some(format!("{:?}", err).red().to_string()),
        }
    } else {
        None
    }
}

pub fn list_dir(file_path: &PathBuf) -> String {
    let args = vec![
        "-l",
        "--all",
        "--color=always",
        "--group-directories-first",
        "--git",
        file_path.to_str().unwrap_or("bad utf path"),
    ];
    match cmd("exa", args).read() {
        Ok(output) => output,
        Err(err) => format!("{:?}", err).red().to_string(),
    }
}
