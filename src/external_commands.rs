use std::path::PathBuf;

use colored::Colorize;
use duct::cmd;

use crate::config::CmdTemplate;

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

pub fn fetch_content_range(
    mut file_line_cmd: CmdTemplate,
    file_path: Option<&PathBuf>,
    line: u64,
) -> Option<String> {
    if let Some(file_path) = file_path {
        let first = if line - 20 > line {
            1
        } else {
            std::cmp::max(1, line - 20)
        };
        let last = line + 5;

        file_line_cmd
            .replace_in_matching_element("$FILE", file_path.to_str().unwrap_or("bad utf path"));
        file_line_cmd.replace_in_matching_element("$FIRST", &format!("{}", first));
        file_line_cmd.replace_in_matching_element("$LAST", &format!("{}", last));
        file_line_cmd.replace_in_matching_element("$LINE", &format!("{}", line));
        match cmd(file_line_cmd.command, file_line_cmd.args).read() {
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
