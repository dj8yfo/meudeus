use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub(super) fn highlight_code_block(input: &str, syntax_desc: &str) -> String {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_token(syntax_desc);
    if let Some(syntax) = syntax {
        let mut result = String::new();

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-eighties.dark"]);
        for line in LinesWithEndings::from(input) {
            // LinesWithEndings enables use of newlines mode
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            result.push_str(&escaped);
        }
        result
    } else {
        input.to_string()
    }
}

pub type MarkdownStatic = (&'static SyntaxSet, &'static SyntaxReference, &'static Theme);

pub(super) fn static_markdown_syntax() -> MarkdownStatic {
    let ps = Box::new(SyntaxSet::load_defaults_newlines());
    let static_ps: &'static mut SyntaxSet = Box::leak(ps);
    let syntax: &'static SyntaxReference = static_ps.find_syntax_by_token("markdown").unwrap();

    let ts = Box::new(ThemeSet::load_defaults());
    let static_ts: &'static mut ThemeSet = Box::leak(ts);

    let theme: &'static Theme = &static_ts.themes["base16-eighties.dark"];
    (static_ps, syntax, theme)
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
