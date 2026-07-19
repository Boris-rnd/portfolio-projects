use std::io::{self, IsTerminal};

pub fn format_terminal_output(output: &str) -> String {
    if !io::stdout().is_terminal() {
        return output.to_string();
    }

    let mut formatted = String::new();
    let mut chars = output.chars().peekable();
    let mut in_code = false;
    let mut in_bold = false;
    let mut in_italic = false;

    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                if in_code {
                    formatted.push_str("\x1b[0m");
                    in_code = false;
                } else {
                    formatted.push_str("\x1b[33m");
                    in_code = true;
                }
            }
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    if in_bold {
                        formatted.push_str("\x1b[0m");
                        in_bold = false;
                    } else {
                        formatted.push_str("\x1b[31m");
                        in_bold = true;
                    }
                } else if in_italic {
                    formatted.push_str("\x1b[0m");
                    in_italic = false;
                } else {
                    formatted.push_str("\x1b[32m");
                    in_italic = true;
                }
            }
            _ => formatted.push(ch),
        }
    }

    if in_code || in_bold || in_italic {
        formatted.push_str("\x1b[0m");
    }

    formatted
}
