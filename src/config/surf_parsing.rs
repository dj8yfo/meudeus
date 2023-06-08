use kdl::KdlNode;
use regex::Regex;
use std::collections::HashMap;

use crate::impl_try_from_kdl_node_tagged;

use super::KdlNodeErrorType;
#[derive(Debug, Clone)]
pub struct SurfParsing {
    pub url_regex: ConfigRegex,
    pub markdown_reference_link_regex: ConfigRegex,
    pub task_item_regex: ConfigRegex,
    pub has_line_regex: ConfigRegex,
}
impl_try_from_kdl_node_tagged!(SurfParsing, "world.surf-parsing", 
    "markdown-reference-link-regex" => markdown_reference_link_regex,
    "url-regex" => url_regex,
    "file-dest-has-line-regex" => has_line_regex,
    "task-item-regex" => task_item_regex);

#[derive(Debug, Clone)]
pub struct ConfigRegex(pub Regex);

impl TryFrom<&KdlNode> for ConfigRegex {
    type Error = miette::Report;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let string = value
            .get(0)
            .ok_or(KdlNodeErrorType {
                err_span: value.span().clone(),
                description: "node's first argument not found".to_string(),
            })
            .map_err(|err| Into::<miette::Report>::into(err))?
            .value()
            .as_string()
            .ok_or(KdlNodeErrorType {
                err_span: value.span().clone(),
                description: "argument's value is expected to be of string type".to_string(),
            })
            .map_err(|err| Into::<miette::Report>::into(err))?
            .to_string();

        let regex = Regex::new(&string).map_err(|err| {
            let err = KdlNodeErrorType {
                err_span: value.span().clone(),
                description: format!("{}", err),
            };

            Into::<miette::Report>::into(err)
        })?;

        Ok(Self(regex))
    }
}
