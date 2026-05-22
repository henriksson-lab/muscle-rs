// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub static FASTA_UPPER: std::sync::Mutex<bool> = std::sync::Mutex::new(true);

pub static FASTA_ALLOW_DIGITS: std::sync::Mutex<bool> = std::sync::Mutex::new(true);

/// Reads the next FASTA record from `file`, returning `(label, sequence)` or `None` at EOF.
#[track_caller]
pub fn get_fasta_seq(file: &mut TextFile, delete_gaps: bool) -> Option<(String, String)> {
    loop {
        let c = text_file_get_char(file)?;
        if c != b'>' {
            panic!("Invalid file format, expected '>' to start FASTA label");
        }

        let mut label = Vec::new();
        loop {
            let Some(c) = text_file_get_char(file) else {
                panic!("End-of-file or input error in FASTA label");
            };
            if c == b'\n' || c == b'\r' {
                break;
            }
            label.push(c);
        }

        let mut seq = Vec::new();
        let mut previous_char = b'\n';
        loop {
            let Some(mut c) = text_file_get_char(file) else {
                break;
            };
            if c == b'>' {
                if previous_char == b'\n' || previous_char == b'\r' {
                    file.pushed_back = Some(c);
                    break;
                }
                panic!("Unexpected '>' in FASTA sequence data");
            } else if (c as char).is_ascii_whitespace() {
            } else if c == b'-' || c == b'.' {
                if !delete_gaps {
                    seq.push(c);
                }
            } else if (c as char).is_ascii_alphabetic() {
                if *FASTA_UPPER.lock().unwrap() {
                    c = c.to_ascii_uppercase();
                }
                seq.push(c);
            } else if *FASTA_ALLOW_DIGITS.lock().unwrap() && (c as char).is_ascii_digit() {
                seq.push(c);
            } else if (c as char).is_ascii_graphic() || c == b' ' {
                previous_char = c;
                continue;
            } else {
                previous_char = c;
                continue;
            }
            previous_char = c;
        }

        if seq.is_empty() {
            continue;
        }
        return Some((
            String::from_utf8_lossy(&label).into_owned(),
            String::from_utf8_lossy(&seq).into_owned(),
        ));
    }
}
