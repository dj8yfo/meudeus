use colored::Colorize;

pub fn format_two_tokens(tok_1: &str, tok_2: &str) -> String {
    format!(
        "{} {}",
        tok_1.truecolor(0, 255, 255),
        tok_2.truecolor(255, 0, 255)
    )
}
