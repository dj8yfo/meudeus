use colored::Colorize;

pub fn print_two_tokens(tok_1: &str, tok_2: &str) -> String {
    format!("{} {}", tok_1.cyan(), tok_2.magenta())
}
