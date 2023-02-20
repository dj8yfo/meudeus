use std::path::PathBuf;

use colored::Colorize;
use duct::cmd;

use crate::config::cmd_template::CmdTemplate;

pub fn fetch_content(mut file_cmd: CmdTemplate, file_path: Option<&PathBuf>) -> Option<String> {
    if let Some(file_path) = file_path {
        file_cmd.replace_matching_element("$FILE", file_path.to_str().unwrap_or("bad utf path"));
        match cmd(file_cmd.command, file_cmd.args).read() {
            Ok(string) => Some(string),
            Err(err) => Some(format!("{:?}", err).red().to_string()),
        }
    } else {
        None
    }
}

pub fn list_dir(mut dir_cmd: CmdTemplate, dir: &PathBuf) -> String {
    dir_cmd.replace_matching_element("$DIR", dir.to_str().unwrap_or("bad utf path"));
    match cmd(dir_cmd.command, dir_cmd.args).read() {
        Ok(output) => output,
        Err(err) => format!("{:?}", err).red().to_string(),
    }
}
