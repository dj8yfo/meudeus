use regex::Regex;
use std::fs;

use crate::link::Link;

// let url_regex = Regex::new(r#"^https?://\S+"#).unwrap();

lazy_static! {
    static ref MARKDOWN_LINK: Regex =
        Regex::new(r#"\[(?P<description>[^\]]+)\]\((?P<url>[^\)]+)\)"#).unwrap();
}

impl super::Note {
    pub fn parse(&self) -> std::io::Result<Vec<Link>> {
        if let Some(file_path) = self.file_path() {
            let mut result = vec![];
            let file_content = fs::read_to_string(file_path)?;

            for link in MARKDOWN_LINK.captures_iter(&file_content) {
                result.push(Link::new(
                    link["description"].to_string(),
                    link["url"].to_string(),
                    file_path.clone(),
                    self.name(),
                ));
            }
            Ok(result)
        } else {
            return Ok(vec![]);
        }
    }
}
