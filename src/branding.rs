use console::Style;
use std::io::{self, IsTerminal};

const LOGO: &str = r#"▄▄▄▄▄▄ ▄▄ ▄▄  ▄▄  ▄▄▄▄  ▄▄▄  ▄▄  ▄▄
  ██   ██ ███▄██ ██▀▀▀ ██▀██ ███▄██
  ██   ██ ██ ▀██ ▀████ ██▀██ ██ ▀██"#;
const TAGLINE: &str = "Plain markdown development journal";

fn logo_style() -> Style {
    Style::new().white()
}

fn tagline_style() -> Style {
    Style::new().white()
}

fn section_style() -> Style {
    Style::new().white().bold()
}

fn heading_style() -> Style {
    Style::new().white()
}

fn path_style() -> Style {
    Style::new().dim()
}

fn terminal_text(text: &str, style: Style) -> String {
    if io::stdout().is_terminal() {
        style.apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn section(text: &str) -> String {
    terminal_text(text, section_style())
}

pub fn heading(text: &str) -> String {
    terminal_text(text, heading_style())
}

pub fn path(text: &str) -> String {
    terminal_text(text, path_style())
}

pub fn print() {
    if io::stdout().is_terminal() {
        println!(
            "{}\n\n{}\n",
            logo_style().apply_to(LOGO),
            tagline_style().apply_to(TAGLINE)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_is_defined_once_as_three_lines() {
        assert_eq!(LOGO.lines().count(), 3);
        assert_eq!(
            LOGO.lines().next(),
            Some("▄▄▄▄▄▄ ▄▄ ▄▄  ▄▄  ▄▄▄▄  ▄▄▄  ▄▄  ▄▄")
        );
        assert_eq!(TAGLINE, "Plain markdown development journal");
    }
}
