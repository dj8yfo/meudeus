use colored::Colorize;

use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use std::process::Command;
use std::{borrow::Cow, path::PathBuf};

use skim::{ItemPreview, PreviewContext, SkimItem};

use crate::database::Database;
use crate::print::print_two_tokens;
use sqlx::Error;
use std::sync::mpsc::{channel, RecvError};

use super::Note;

impl SkimItem for super::Note {
    fn text(&self) -> Cow<str> {
        Cow::Owned(self.name())
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let resources = self.resources().unwrap().clone();
        let name = self.name().clone();

        let (sender, receiver) = channel();
        tokio::runtime::Handle::current().spawn(async move {
            let result_from = resources.db.lock().await.find_links_from(&name).await;
            let result_to = resources.db.lock().await.find_links_to(&name).await;

            sender.send((result_to, result_from)).unwrap()
        });
        let query_result = receiver.recv();
        let links = map_recv_result(query_result);
        let mut string = String::new();
        let title = if self.file_path().is_some() {
            print_two_tokens("it's a note:", &self.name())
        } else {
            print_two_tokens("it's a tag:", &self.name())
        };
        string.push_str(&title);
        string.push_str(&"\n");
        string.push_str(&"\n");
        string.push_str(&links);
        string.push_str(&"\n");
        let body = fetch_content(self.file_path());
        if body.is_some() {
            string.push_str(&body.unwrap());
        }
        ItemPreview::AnsiText(string)
    }
}

type R = Result<Vec<Note>, Error>;

fn map_recv_result(query_result: Result<(R, R), RecvError>) -> String {
    let received = match query_result {
        Ok(received) => received,

        Err(err) => return format!("received err {:?}", err).red().to_string(),
    };

    let links_to = map_db_result(received.0);
    let links_from = map_db_result(received.1);

    let mut string = String::new();
    if !links_to.is_empty() {
        string.push_str("Linked by:\n");
        string.push_str(&links_to);
        string.push_str(&"\n");
    }
    if !links_from.is_empty() {
        string.push_str("Links to:\n");
        string.push_str(&links_from);
    }
    string
}
fn map_db_result(received: R) -> String {
    match received {
        Ok(list) => {
            if !list.is_empty() {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_width(80)
                    .set_header(vec![
                        Cell::new("Name").fg(Color::Blue),
                        Cell::new("Type").fg(Color::Blue),
                    ]);
                list.into_iter().for_each(|note| {
                    let is_tag = note.file_path().is_none();
                    let color = if is_tag {
                        Color::Cyan
                    } else {
                        Color::DarkMagenta
                    };
                    let type_column = if is_tag { "tag" } else { "note" };
                    table.add_row(vec![
                        Cell::new(note.name()),
                        Cell::new(type_column).fg(color),
                    ]);
                });
                format!("{}\n", table)
            } else {
                String::new()
            }
        }
        Err(err) => return format!("db err {:?}", err).red().to_string(),
    }
}

fn fetch_content(file_path: Option<&PathBuf>) -> Option<String> {
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
