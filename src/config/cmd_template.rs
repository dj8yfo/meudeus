use anyhow::anyhow;
use kdl::KdlValue;

#[derive(Debug, Clone)]
pub struct CmdTemplate {
    pub command: String,
    pub args: Vec<String>,
}
impl TryFrom<Vec<&KdlValue>> for CmdTemplate {
    type Error = anyhow::Error;

    fn try_from(value: Vec<&KdlValue>) -> Result<Self, Self::Error> {
        let first = value.first();
        if first.is_none() {
            return Err(anyhow!("no command specified"));
        }
        let command = first
            .unwrap()
            .as_string()
            .ok_or(anyhow!("cannot be converted to str"))?
            .to_string();

        let args = &value[1..];
        let args: Result<Vec<String>, Self::Error> = args
            .into_iter()
            .map(|arg| {
                Ok(arg
                    .as_string()
                    .ok_or(anyhow!("cannot be converted to str"))?
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
