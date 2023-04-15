use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub(super) fn highlight_code_block(
    input: &str,
    syntax_desc: &str,
    md_static: MarkdownStatic,
) -> String {
    let (syntax_set, _, theme) = md_static;
    let syntax = syntax_set.find_syntax_by_token(syntax_desc);
    if let Some(syntax) = syntax {
        let mut result = String::new();

        let mut h = HighlightLines::new(syntax, theme);
        for line in LinesWithEndings::from(input) {
            // LinesWithEndings enables use of newlines mode
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, syntax_set).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            result.push_str(&escaped);
        }
        result
    } else {
        input.to_string()
    }
}

pub type MarkdownStatic = (&'static SyntaxSet, &'static SyntaxReference, &'static Theme);

pub(super) fn static_markdown_syntax(loaded_theme: Option<&'static Theme>) -> MarkdownStatic {
    let syntax_set = Box::new(SyntaxSet::load_defaults_newlines());
    let static_synt_set: &'static mut SyntaxSet = Box::leak(syntax_set);
    let syntax_md: &'static SyntaxReference =
        static_synt_set.find_syntax_by_token("markdown").unwrap();

    let theme_set_default = Box::new(ThemeSet::load_defaults());
    let them_set_static: &'static mut ThemeSet = Box::leak(theme_set_default);

    let theme: &'static Theme = if let Some(loaded) = loaded_theme {
        loaded
    } else {
        &them_set_static.themes["base16-eighties.dark"]
    };
    (static_synt_set, syntax_md, theme)
}

pub fn highlight(input: &str, h: &mut HighlightLines, md_static: MarkdownStatic) -> String {
    let mut result = String::new();
    for line in LinesWithEndings::from(input) {
        // LinesWithEndings enables use of newlines mode
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, md_static.0).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        result.push_str(&escaped);
    }
    result
}
