use std::fs;

use crate::{
    config::{ExternalCommands, SurfParsing},
    link::Link,
};

impl super::Note {
    pub fn parse(
        &self,
        surf: &SurfParsing,
        external_commands: &ExternalCommands,
    ) -> std::io::Result<Vec<Link>> {
        if let Some(file_path) = self.file_path() {
            let mut result = vec![];
            let file_content = fs::read_to_string(file_path)?;

            for link in surf
                .markdown_reference_link_regex
                .captures_iter(&file_content)
            {
                result.push(Link::new(
                    link["description"].to_string(),
                    link["url"].to_string(),
                    file_path.clone(),
                    self.name(),
                    &surf.url_regex,
                    external_commands,
                ));
            }
            Ok(result)
        } else {
            return Ok(vec![]);
        }
    }
}
