// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns the program banner and built-in help text (printed by `--help`).
#[track_caller]
pub fn help() -> String {
    let raw = include_str!("../muscle/src/help.h");
    let mut text = print_banner();
    for line in raw.lines() {
        let mut chars = line.trim().chars();
        if chars.next() != Some('"') {
            continue;
        }
        let mut escape = false;
        for c in chars {
            if escape {
                match c {
                    'n' => text.push('\n'),
                    'r' => text.push('\r'),
                    't' => text.push('\t'),
                    '"' => text.push('"'),
                    '\\' => text.push('\\'),
                    _ => text.push(c),
                }
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                break;
            } else {
                text.push(c);
            }
        }
    }
    text
}
