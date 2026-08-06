use console::Style;
use std::io::{self, IsTerminal};

const LOGO: &str = r#"▄▄▄ ▄ ▀▄ ▄ ▄▄▄ ▄▄▄ ▀▄ ▄
 █  █ █ ▀█ █ ▀ █▄█ █ ▀█
 ▀  ▀ ▀  ▀ ▀▀▀ ▀ ▀ ▀  ▀"#;

fn logo_style() -> Style {
    Style::new().white()
}

pub fn print() {
    if io::stdout().is_terminal() {
        println!("{}\n", logo_style().apply_to(LOGO));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_is_defined_once_as_three_lines() {
        assert_eq!(LOGO.lines().count(), 3);
        assert_eq!(LOGO.lines().next(), Some("▄▄▄ ▄ ▀▄ ▄ ▄▄▄ ▄▄▄ ▀▄ ▄"));
    }
}
