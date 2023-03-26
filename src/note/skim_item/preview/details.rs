use colored::Colorize;

use crate::note::Note;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};

use sqlx::Error;
use std::sync::mpsc::{channel, RecvError};

use crate::external_commands::fetch_content;
use crate::print::format_two_tokens;

type R = Result<Vec<Note>, Error>;

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
                        Color::DarkYellow
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
fn map_recv_result(query_result: Result<R, RecvError>, tag: String) -> String {
    let received = match query_result {
        Ok(received) => received,

        Err(err) => return format!("received err {:?}", err).red().to_string(),
    };

    let links_to = map_db_result(received);

    let mut string = String::new();
    if !links_to.is_empty() {
        string.push_str(&tag);
        string.push_str(&"\n");
        string.push_str(&links_to);
        string.push_str(&"\n");
    }
    string
}
impl Note {
    pub fn details(&self) -> String {
        let (sender_1, receiver_1) = channel();
        let other_me = self.clone();
        tokio::runtime::Handle::current().spawn(async move {
            let result_from = other_me.fetch_forward_links().await.unwrap();

            sender_1.send(result_from).unwrap()
        });
        let (sender_2, receiver_2) = channel();
        let despicable_me = self.clone();
        tokio::runtime::Handle::current().spawn(async move {
            let result_to = despicable_me.fetch_backlinks().await.unwrap();

            sender_2.send(result_to).unwrap()
        });
        let result_from = receiver_1.recv();
        let result_to = receiver_2.recv();
        let links_to = map_recv_result(result_from, "Links to:".to_string());
        let linked_by = map_recv_result(result_to, "Linked by:".to_string());
        let mut string = String::new();
        let title = if self.file_path().is_some() {
            format_two_tokens("it's a note:", &self.name())
        } else {
            format_two_tokens("it's a tag:", &self.name())
        };
        string.push_str(&title);
        string.push_str(&"\n\n");
        string.push_str(&linked_by);
        string.push_str(&links_to);
        if let Some(resources) = self.resources() {
            let body = fetch_content(
                resources.external_commands.preview.file_cmd.clone(),
                self.file_path(),
            );
            if body.is_some() {
                string.push_str(&body.unwrap());
            }
        }
        string
    }
}
