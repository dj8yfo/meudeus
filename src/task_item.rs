use std::{fs, path::PathBuf};

use colored::Colorize;
use regex::Regex;

use crate::{config::SurfParsing, highlight::highlight_code_block, note::Note};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    pub file_name: PathBuf,
    pub title: String,
    pub completed: bool,
    pub nested_level: usize,
    pub checkmark_offsets_in_string: std::ops::Range<usize>,
}

impl From<(PathBuf, regex::Captures<'_>)> for TaskItem {
    fn from(value: (PathBuf, regex::Captures<'_>)) -> Self {
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
        Self {
            file_name: value.0,
            nested_level,
            completed,
            title: title.as_str().to_string(),
            checkmark_offsets_in_string,
        }
    }
}

impl TaskItem {
    fn parse_string(file_name: &PathBuf, input: &str, regex: &Regex) -> Vec<Self> {
        let mut result = vec![];

        for capture in regex.captures_iter(input) {
            result.push((file_name.clone(), capture).into());
        }
        result
    }
    pub fn parse(note: &Note, surf: &SurfParsing) -> std::io::Result<Vec<Self>> {
        if let Some(file_path) = note.file_path() {
            let file_content = fs::read_to_string(file_path)?;
            let result = Self::parse_string(file_path, &file_content, &surf.task_item_regex);

            Ok(result)
        } else {
            return Ok(vec![]);
        }
    }
}

impl TaskItem {
    pub fn skim_display(&self) -> String {
        let symbol = if self.completed {
            "✓".green().to_string()
        } else {
            " ".to_string()
        };
        let input = format!(
            "[{}] {} {}",
            symbol,
            highlight_code_block(&self.title, "markdown"),
            " ".black().to_string()
        );
        input
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use regex::Regex;

    use super::TaskItem;

    static test_str: &str = r#"
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
        let list = TaskItem::parse_string(&PathBuf::from("./tmp.rs"), test_str, &regex);
        assert_eq!(8, list.len());
        assert_eq!(
            &list[4],
            &TaskItem {
                title: "in development <Tue Mar 21 08:20:37 PM EET 2023>".to_string(),
                nested_level: 3,
                completed: true,
                file_name: "./tmp.rs".into(),
                checkmark_offsets_in_string: 362..363,
            }
        );
        for el in list {
            println!("{:#?}", el);
        }
    }
}
