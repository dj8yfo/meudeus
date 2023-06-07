use std::collections::HashMap;
use std::{fs, path::PathBuf};

use kdl::{KdlDocument, KdlNode};
use miette::{Diagnostic, IntoDiagnostic, Report, SourceSpan};
use thiserror::Error;

use crate::impl_try_from_kdl_node_tagged;
use crate::print::format_two_tokens;

use self::color::Color;
pub use self::external_commands::cmd_template::CmdTemplate;
pub use self::external_commands::{ExternalCommands, Open, Preview};
use self::keymap::Keymap;
pub use self::surf_parsing::SurfParsing;

pub mod macros;

pub mod color;
pub mod external_commands;
pub mod keymap;
pub mod surf_parsing;

static PROGRAM_NAME: &str = "mds";
#[derive(Debug)]
pub struct Config {
    pub work_dir: ConfigPath,
    pub surf_parsing: SurfParsing,
    pub external_commands: ExternalCommands,
    pub color: Color,
    pub keymap: Keymap,
}

#[derive(Debug, Clone)]
pub struct ConfigPath(pub PathBuf);

#[derive(Diagnostic, Debug, Error)]
#[error("error associated with a kdl doc/node")]
#[diagnostic()]
pub struct KdlNodeErrorType {
    // Note: label but no source code
    #[label]
    err_span: SourceSpan,

    #[help]
    description: String,
}

impl TryFrom<&KdlNode> for ConfigPath {
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

        Ok(Self(PathBuf::from(string)))
    }
}

impl Config {
    pub fn parse() -> miette::Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(PROGRAM_NAME).into_diagnostic()?;

        let config_path = xdg_dirs.get_config_file("config.kdl");
        eprintln!(
            "{} \n",
            format_two_tokens(
                "expected config path: ",
                config_path.to_str().unwrap_or("bad utf8")
            )
        );

        let config_file = fs::read_to_string(config_path).into_diagnostic()?;

        let doc: KdlDocument = config_file.parse().map_err(|error| Report::new(error))?;
        let world_node: miette::Result<&KdlNode> = doc
            .get("world")
            .ok_or(KdlNodeErrorType {
                err_span: doc.span().clone(),
                description: "couldn't find top-level `world` node in kdl document".to_string(),
            })
            .map_err(|err| Into::<miette::Report>::into(err));
        let world_node = world_node.map_err(|error| error.with_source_code(config_file.clone()))?;

        let result: miette::Result<Self> = world_node.try_into();
        let result = result.map_err(|error| error.with_source_code(config_file))?;
        Ok(result)
    }
}

impl_try_from_kdl_node_tagged!(Config, "world", 
    "surf-parsing" => surf_parsing, 
    "notes-work-dir" => work_dir, 
    "external-commands" => external_commands, 
    "color" => color, 
    "keymap" => keymap);
