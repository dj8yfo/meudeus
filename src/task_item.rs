use std::{fs, path::PathBuf};

use colored::Colorize;
use regex::Regex;
use syntect::easy::HighlightLines;

use crate::{
    config::SurfParsing,
    highlight::{highlight, MarkdownStatic},
    note::Note,
};
mod skim_item;

pub use skim_item::TaskTreeWrapper;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    pub file_name: PathBuf,
    pub title: String,
    pub title_markdown: String,
    pub completed: bool,
    pub nested_level: usize,
    pub checkmark_offsets_in_string: std::ops::Range<usize>,
    pub self_index: usize,
    pub next_index: Option<usize>,
}

impl TaskItem {
    fn parse_capture(
        value: (PathBuf, regex::Captures<'_>, usize),
        highlighter: &mut HighlightLines,
        md_static: MarkdownStatic,
    ) -> Self {
        let title = value.1.name("task_text").unwrap();
        let checkmark = value.1.name("checkmark").unwrap();
        let completed = if checkmark.as_str() == "x" {
            true
        } else {
            false
        };
        let whitespace = value.1.name("whitespace").unwrap().as_str();
        let nested_level = whitespace.len() / 2;
        let checkmark_offsets_in_string = checkmark.start()..checkmark.end();

        let title_markdown = format!(
            "{} {}",
            highlight(title.as_str(), highlighter, md_static),
            " ".truecolor(0, 0, 0).to_string()
        );
        Self {
            file_name: value.0,
            nested_level,
            completed,
            title: title.as_str().to_string(),
            checkmark_offsets_in_string,
            self_index: value.2,
            next_index: None,
            title_markdown,
        }
    }
    fn parse_string(
        file_name: &PathBuf,
        input: &str,
        regex: &Regex,
        highlighter: &mut HighlightLines,
        md_static: MarkdownStatic,
    ) -> Vec<Self> {
        let mut result = vec![];

        for (index, capture) in regex.captures_iter(input).enumerate() {
            result.push(Self::parse_capture(
                (file_name.clone(), capture, index),
                highlighter,
                md_static,
            ));
        }
        result
    }
    pub fn parse(
        note: &Note,
        surf: &SurfParsing,
        highlighter: &mut HighlightLines,
        md_static: MarkdownStatic,
    ) -> std::io::Result<Vec<Self>> {
        if let Some(file_path) = note.file_path() {
            let file_content = fs::read_to_string(file_path)?;
            let result = Self::parse_string(
                file_path,
                &file_content,
                &surf.task_item_regex,
                highlighter,
                md_static,
            );

            Ok(result)
        } else {
            return Ok(vec![]);
        }
    }
}

impl TaskItem {
    pub fn skim_display(&self, indented: bool) -> String {
        let indent = if indented {
            let mut string = String::new();
            for _i in 0..self.nested_level {
                string.push_str("  ");
            }
            string
        } else {
            "".to_string()
        };
        let symbol = if self.completed {
            "✓".truecolor(0, 255, 0).to_string()
        } else {
            " ".to_string()
        };
        let input = format!("{}[{}] {}", indent, symbol, self.title_markdown,);
        input
    }

    pub fn skim_display_mono(&self, indented: bool) -> String {
        let indent = if indented {
            let mut string = String::new();
            for _i in 0..self.nested_level {
                string.push_str("  ");
            }
            string
        } else {
            "".to_string()
        };
        let symbol = if self.completed {
            "✓".to_string()
        } else {
            " ".to_string()
        };
        let input = format!("{}[{}] {} {}", indent, symbol, self.title, " ".to_string());
        input
    }

    pub fn toggle(mut self) -> std::io::Result<()> {
        let prev = format!("{}", self.skim_display(false));
        self.completed = !self.completed;
        let next = format!("{}", self.skim_display(false));
        println!("{} -> {}", prev, next);

        let mut file_content = fs::read_to_string(&self.file_name)?;
        let target = if self.completed { "x" } else { " " };
        file_content.replace_range(self.checkmark_offsets_in_string, target);
        fs::write(&self.file_name, file_content)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use regex::Regex;
    use syntect::easy::HighlightLines;

    use crate::highlight::static_markdown_syntax;

    use super::TaskItem;

    static TEST_STR: &str = r#"
- [x] move `construct_term_tree` to a separate module on note.rs  <Tue Mar 21 08:18:59 PM EET 2023>
- [ ] create `TaskItem` struct  <Tue Mar 21 08:18:59 PM EET 2023>
  - [x] test parsing of this snippet <Tue Mar 21 09:47:55 PM EET 2023>
    - [x] it's a very very meta test, depicting what has actually benn happening <Tue Mar 21 08:20:16 PM EET 2023>
      - [x] in development <Tue Mar 21 08:20:37 PM EET 2023>
  - [ ] implement `skim_display` for it  <Tue Mar 21 08:19:22 PM EET 2023>
- [ ] allow starring of subtasks as in [mdt](https://github.com/basilioss/mdt) <Tue Mar 21 08:25:06 PM EET 2023>
  - [ ] command is called `mds chm`, short for `checkmark` <Tue Mar 21 08:25:45 PM EET 2023>        
    "#;

    #[test]
    fn test_tasks_items_parsing() {
        let regex =
            Regex::new(r#"(?P<whitespace>(  )*)- \[(?P<checkmark>[x ])\]\s+(?P<task_text>.+)"#)
                .unwrap();

        let md_static = static_markdown_syntax(None);
        let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
        let list = TaskItem::parse_string(
            &PathBuf::from("./tmp.rs"),
            TEST_STR,
            &regex,
            &mut highlighter,
            md_static,
        );
        assert_eq!(8, list.len());
        assert_eq!(
            &list[4],
            &TaskItem {
                title: "in development <Tue Mar 21 08:20:37 PM EET 2023>".to_string(),
                title_markdown: "\u{1b}[48;2;45;45;45m\u{1b}[38;2;211;208;200min development \u{1b}[48;2;45;45;45m\u{1b}[38;2;211;208;200m<\u{1b}[48;2;45;45;45m\u{1b}[38;2;211;208;200mTue Mar 21 08:20:37 PM EET 2023>  ".to_string(),

                nested_level: 3,
                completed: true,
                file_name: "./tmp.rs".into(),
                checkmark_offsets_in_string: 362..363,
                self_index: 4,
                next_index: None,
            }
        );
        for el in list {
            println!("{:#?}", el);
        }
    }
}
