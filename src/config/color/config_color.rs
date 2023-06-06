use kdl::KdlNode;
use rgb::RGB8;

use crate::config::KdlNodeErrorType;
#[derive(Debug, Clone, Copy)]
pub struct ConfigRGB(pub RGB8);

impl TryFrom<&KdlNode> for ConfigRGB {
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

        let color = serde_json::from_str(&string).map_err(|err| {
            let err = KdlNodeErrorType {
                err_span: value.span().clone(),
                description: format!("RGB8 deserialization from json problem {}", err),
            };
            Into::<miette::Report>::into(err)
        })?;

        Ok(Self(color))
    }
}
