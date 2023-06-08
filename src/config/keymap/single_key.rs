use kdl::KdlNode;
use lazy_static::lazy_static;
use regex::Regex;

use crate::config::KdlNodeErrorType;

lazy_static! {
    static ref KEY_COMBO_REGEX: Regex = Regex::new("^(ctrl-.)$|^(alt-.)$").expect("wrong regex");
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SingleKey {
    pub combo: String,
    pub tui_combo: tuikit::key::Key,
}

impl SingleKey {
    fn from_string_repr(combo: &str) -> tuikit::key::Key {
        if combo.starts_with("ctrl") {
            let char = combo.chars().take(6).last().expect("doesnt match regex");
            tuikit::key::Key::Ctrl(char)
        } else if combo.starts_with("alt") {
            let char = combo.chars().take(5).last().expect("doesnt match regex");
            tuikit::key::Key::Alt(char)
        } else {
            unreachable!("should be unreachable due to `combo` matching KEY_COMBO_REGEX")
        }
    }
}

impl TryFrom<String> for SingleKey {
    type Error = String;
    fn try_from(combo: String) -> Result<Self, Self::Error> {
        if !KEY_COMBO_REGEX.is_match(&combo) {
            return Err(format!(
                "`{}` doesn't match regex {}",
                combo,
                KEY_COMBO_REGEX.as_str()
            ));
        }

        let tui_combo = Self::from_string_repr(&combo);
        if tui_combo == tuikit::key::Key::Ctrl('c') {
            return Err("`ctrl-c` binding is forbidden".to_string());
        }
        Ok(Self { combo, tui_combo })
    }
}
impl TryFrom<&KdlNode> for SingleKey {
    type Error = miette::Report;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let combo = value
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

        combo.try_into().map_err(|err| {
            let err = KdlNodeErrorType {
                err_span: value.span().clone(),
                description: err,
            };

            Into::<miette::Report>::into(err)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SingleKey;

    #[test]
    fn test_correct_parsing() {
        let ctrl_combo = "ctrl-t".to_string();
        let alt_combo = "alt-t".to_string();

        let key_first: SingleKey = ctrl_combo.try_into().expect("no err");
        let key_second: SingleKey = alt_combo.try_into().expect("no err");

        let f_expected = SingleKey {
            combo: "ctrl-t".to_string(),
            tui_combo: tuikit::key::Key::Ctrl('t'),
        };
        let f_expected2 = SingleKey {
            combo: "alt-t".to_string(),
            tui_combo: tuikit::key::Key::Alt('t'),
        };
        assert_eq!(key_first, f_expected);
        assert_eq!(key_second, f_expected2);
    }
}
