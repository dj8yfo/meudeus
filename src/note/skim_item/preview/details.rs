use colored::Colorize;

use crate::config::color::ColorScheme;
use crate::database::SqliteAsyncHandle;
use crate::highlight::MarkdownStatic;
use crate::note::Note;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};

use sqlx::Error;

use crate::external_commands::fetch_content;
use crate::print::format_two_tokens;

fn map_db_result(received: Result<Vec<Note>, Error>) -> String {
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
        Err(err) => format!("db err {:?}", err).truecolor(255, 0, 0).to_string(),
    }
}

fn map_result(query_result: Result<Vec<Note>, Error>, tag: String) -> String {
    let links_to = map_db_result(query_result);

    let mut string = String::new();
    if !links_to.is_empty() {
        string.push_str(&tag);
        string.push('\n');
        string.push_str(&links_to);
        string.push('\n');
    }
    string
}

impl Note {
    pub async fn details(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> String {
        let result_from = self
            .fetch_forward_links(db, md_static, color_scheme, straight)
            .await;
        let result_to = self
            .fetch_backlinks(db, md_static, color_scheme, straight)
            .await;
        let links_to = map_result(result_from, "Links to:".to_string());
        let linked_by = map_result(result_to, "Linked by:".to_string());
        let mut string = String::new();
        let title = if self.file_path().is_some() {
            format_two_tokens("it's a note:", &self.name())
        } else {
            format_two_tokens("it's a tag:", &self.name())
        };
        string.push_str(&title);
        string.push_str("\n\n");
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
