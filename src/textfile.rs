// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TEXTFILEPOS {
    pub offset: uint,
    pub line_nr: uint,
    pub col_nr: uint,
} // original: TEXTFILEPOS (muscle/src/textfile.h)

/// Constructs a `TextFile` from a path; opens for reading unless `write`.
#[track_caller]
pub fn text_file_text_file_l5(file_name: &str, write: bool) -> TextFile {
    if write {
        let mut file = TextFile::default();
        text_file_init(&mut file, &[], file_name);
        return file;
    }
    let data = std::fs::read(file_name)
        .unwrap_or_else(|err| panic!("Cannot open '{}' errno={}", file_name, err));
    let mut file = TextFile::default();
    text_file_init(&mut file, &data, file_name);
    file
}

/// Constructs a `TextFile` from a `String` filename overload.
#[track_caller]
pub fn text_file_text_file_l27(file_name: &str, write: bool) -> TextFile {
    if write {
        let mut file = TextFile::default();
        text_file_init(&mut file, &[], file_name);
        return file;
    }
    let data = std::fs::read(file_name)
        .unwrap_or_else(|err| panic!("Cannot open '{}' errno={}", file_name, err));
    let mut file = TextFile::default();
    text_file_init(&mut file, &data, file_name);
    file
}

/// Initialises a `TextFile`'s buffer, name and parser cursor.
#[track_caller]
pub fn text_file_init(file: &mut TextFile, data: &[u8], name: &str) {
    file.data = data.to_vec();
    file.pos = 0;
    file.name = name.to_string();
    file.line_nr = 1;
    file.col_nr = 0;
    file.last_char_was_eol = true;
    file.pushed_back = None;
}

/// Constructs a `TextFile` backed by an in-memory byte buffer.
#[track_caller]
pub fn text_file_text_file_l63(data: &[u8], file_name: &str) -> TextFile {
    let mut file = TextFile::default();
    text_file_init(&mut file, data, file_name);
    file
}

/// Releases the buffer and resets the cursor; mirrors the C++ destructor.
#[track_caller]
pub fn text_file_destructor_text_file(file: &mut TextFile) {
    file.data.clear();
    file.pos = 0;
    file.name.clear();
    file.pushed_back = None;
}

/// Reads the next line up to `bytes` chars, returning `None` at EOF.
#[track_caller]
pub fn text_file_get_line(file: &mut TextFile, bytes: uint) -> Option<String> {
    if bytes == 0 {
        panic!("TextFile::GetLine, buffer zero size");
    }
    let mut line = Vec::new();
    loop {
        let c = text_file_get_char(file)?;
        if c == b'\r' {
            continue;
        }
        if c == b'\n' {
            return Some(String::from_utf8_lossy(&line).into_owned());
        }
        if line.len() < bytes as usize - 1 {
            line.push(c);
        } else {
            panic!(
                "TextFile::GetLine: input buffer too small, line {}",
                file.line_nr
            );
        }
    }
}

/// Reads a line and trims whitespace (not yet implemented).
#[track_caller]
pub fn text_file_get_trim_line(_file: &mut TextFile, _bytes: uint) -> Option<String> {
    panic!("GetTrimLine");
}

/// Resets the read cursor to the beginning of the buffer.
#[track_caller]
pub fn text_file_rewind(file: &mut TextFile) {
    file.pos = 0;
    file.line_nr = 1;
    file.last_char_was_eol = true;
    file.pushed_back = None;
}

/// Appends a single character to the file, updating line/column tracking.
#[track_caller]
pub fn text_file_put_char(file: &mut TextFile, c: u8) {
    file.data.push(c);
    if c == b'\n' {
        file.line_nr += 1;
        file.col_nr = 1;
        file.last_char_was_eol = true;
    } else {
        file.col_nr += 1;
        file.last_char_was_eol = false;
    }
}

/// Appends the bytes of `line` to the file's buffer.
#[track_caller]
pub fn text_file_put_string(file: &mut TextFile, line: &str) {
    file.data.extend_from_slice(line.as_bytes());
}

/// Formatted-write wrapper that simply forwards to `put_string`.
#[track_caller]
pub fn text_file_put_format(file: &mut TextFile, s: &str) {
    text_file_put_string(file, s);
}

/// Like `text_file_get_line`, but panics at EOF.
#[track_caller]
pub fn text_file_get_line_x(file: &mut TextFile, bytes: uint) -> String {
    if bytes == 0 {
        panic!("GetLineX");
    }
    text_file_get_line(file, bytes).unwrap_or_else(|| panic!("end-of-file in GetLineX"))
}

/// Reads the next whitespace-separated token, treating chars in
/// `char_tokens` as single-char tokens.
#[track_caller]
pub fn text_file_get_token(file: &mut TextFile, bytes: uint, char_tokens: &str) -> Option<String> {
    let mut c;
    loop {
        c = text_file_get_char(file)?;
        if !(c as char).is_ascii_whitespace() {
            break;
        }
    }
    if char_tokens.as_bytes().contains(&c) {
        if bytes < 2 {
            panic!(
                "TextFile::GetToken: input buffer too small, line {}",
                file.line_nr
            );
        }
        return Some((c as char).to_string());
    }

    let mut token = Vec::new();
    loop {
        if token.len() < bytes as usize - 1 {
            token.push(c);
        } else {
            panic!(
                "TextFile::GetToken: input buffer too small, line {}",
                file.line_nr
            );
        }
        match text_file_get_char(file) {
            None => return Some(String::from_utf8_lossy(&token).into_owned()),
            Some(next) => {
                c = next;
                if char_tokens.as_bytes().contains(&c) {
                    file.pushed_back = Some(c);
                    return Some(String::from_utf8_lossy(&token).into_owned());
                }
                if (c as char).is_ascii_whitespace() {
                    return Some(String::from_utf8_lossy(&token).into_owned());
                }
            }
        }
    }
}

/// Like `text_file_get_token`, but panics at EOF.
#[track_caller]
pub fn text_file_get_token_x(file: &mut TextFile, bytes: uint, char_tokens: &str) -> String {
    text_file_get_token(file, bytes, char_tokens)
        .unwrap_or_else(|| panic!("End-of-file in GetTokenX"))
}

/// Skips whitespace up to and including the next newline.
#[track_caller]
pub fn text_file_skip(file: &mut TextFile) {
    loop {
        let Some(c) = text_file_get_char(file) else {
            return;
        };
        if c == b'\n' {
            return;
        }
        assert!((c as char).is_ascii_whitespace());
    }
}

/// Returns the current parser position (offset, line, column).
#[track_caller]
pub fn text_file_get_pos_l234(file: &TextFile) -> TEXTFILEPOS {
    TEXTFILEPOS {
        offset: file.pos as uint,
        line_nr: file.line_nr,
        col_nr: file.col_nr,
    }
}

/// Restores the parser to a previously captured position.
#[track_caller]
pub fn text_file_set_pos_l247(file: &mut TextFile, pos: TEXTFILEPOS) {
    file.pos = pos.offset as usize;
    file.line_nr = pos.line_nr;
    file.col_nr = pos.col_nr;
    file.pushed_back = None;
}

/// Returns the current parser position; const overload mirroring C++.
#[track_caller]
pub fn text_file_get_pos_l258(file: &TextFile) -> TEXTFILEPOS {
    TEXTFILEPOS {
        offset: file.pos as uint,
        line_nr: file.line_nr,
        col_nr: file.col_nr,
    }
}

/// Restores the parser position; const overload mirroring C++.
#[track_caller]
pub fn text_file_set_pos_l267(file: &mut TextFile, pos: TEXTFILEPOS) {
    file.pos = pos.offset as usize;
    file.line_nr = pos.line_nr;
    file.col_nr = pos.col_nr;
    file.pushed_back = None;
}

/// Reads the next byte, honouring the one-byte pushback buffer.
#[track_caller]
pub fn text_file_get_char(file: &mut TextFile) -> Option<u8> {
    if let Some(c) = file.pushed_back.take() {
        return Some(c);
    }
    if file.pos >= file.data.len() {
        if !file.last_char_was_eol && file.line_nr > 0 {
            file.last_char_was_eol = true;
            return Some(b'\n');
        }
        return None;
    }
    let c = file.data[file.pos];
    file.pos += 1;
    if c == b'\n' {
        file.last_char_was_eol = true;
        file.line_nr += 1;
        file.col_nr = 1;
    } else {
        file.last_char_was_eol = false;
        file.col_nr += 1;
    }
    Some(c)
}

/// Like `text_file_get_char`, but panics at EOF.
#[track_caller]
pub fn text_file_get_char_x(file: &mut TextFile) -> u8 {
    text_file_get_char(file).unwrap_or_else(|| panic!("End-of-file in GetCharX"))
}

/// Returns the next non-whitespace byte; panics at EOF.
#[track_caller]
pub fn text_file_get_nonblank_char(file: &mut TextFile) -> u8 {
    loop {
        let c = text_file_get_char(file).unwrap_or_else(|| panic!("End-of-file in GetCharX"));
        if !(c as char).is_ascii_whitespace() {
            return c;
        }
    }
}

/// Advances past the remainder of the current line.
#[track_caller]
pub fn text_file_skip_line(file: &mut TextFile) {
    if file.last_char_was_eol {
        return;
    }
    loop {
        let c = text_file_get_char(file).unwrap_or_else(|| panic!("End-of-file in SkipLine"));
        if c == b'\n' {
            break;
        }
    }
}

/// Skips whitespace; panics if EOF is reached.
#[track_caller]
pub fn text_file_skip_white(file: &mut TextFile) {
    if text_file_skip_white_x(file) {
        panic!("End-of-file skipping white space");
    }
}

/// Skips whitespace, returning `true` if EOF was reached before any
/// non-whitespace byte.
#[track_caller]
pub fn text_file_skip_white_x(file: &mut TextFile) -> bool {
    loop {
        let Some(c) = text_file_get_char(file) else {
            return true;
        };
        if !(c as char).is_ascii_whitespace() {
            file.pushed_back = Some(c);
            break;
        }
    }
    false
}
