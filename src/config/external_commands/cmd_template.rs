use kdl::KdlNode;

use crate::config::KdlNodeErrorType;

#[derive(Debug, Clone)]
pub struct CmdTemplate {
    pub command: String,
    pub args: Vec<String>,
}
impl TryFrom<&KdlNode> for CmdTemplate {
    type Error = miette::Report;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let entries = value.entries();
        let first = entries.first();
        let Some(first) = first else {
            return Err(KdlNodeErrorType {
                err_span: value.span().clone(),
                description: "node expected to have at least 1 argument".to_string(),
            }).map_err(|err| Into::<miette::Report>::into(err))?;
        };

        let command = first
            .value()
            .as_string()
            .ok_or(KdlNodeErrorType {
                err_span: first.span().clone(),
                description: "all of arguments' values are expected to be of string type"
                    .to_string(),
            })
            .map_err(|err| Into::<miette::Report>::into(err))?
            .to_string();

        let args = &entries[1..];
        let args: Result<Vec<String>, Self::Error> = args
            .into_iter()
            .map(|arg| {
                Ok(arg
                    .value()
                    .as_string()
                    .ok_or(KdlNodeErrorType {
                        err_span: arg.span().clone(),
                        description: "all of arguments' values are expected to be of string type"
                            .to_string(),
                    })
                    .map_err(|err| Into::<miette::Report>::into(err))?
                    .to_string())
            })
            .collect();

        Ok(Self {
            command,
            args: args?,
        })
    }
}

impl CmdTemplate {
    pub fn replace_matching_element(&mut self, placeholder: &str, value: &str) {
        if let Some(index) = self.args.iter().position(|x| x == placeholder) {
            self.args[index] = value.to_string();
        }
    }

    pub fn replace_in_matching_element(&mut self, placeholder: &str, value: &str) {
        let args = self.args.drain(..).collect::<Vec<_>>();
        self.args = args
            .into_iter()
            .map(|element| {
                if element.contains(placeholder) {
                    let new = element.replace(placeholder, value);
                    return new;
                }
                element
            })
            .collect::<Vec<_>>();
    }
}
