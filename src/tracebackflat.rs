// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Traces back a flat B/X/Y-encoded DP matrix and returns the alignment
/// path string.
#[track_caller]
pub fn trace_back_flat(tb: &[i8], lx: uint, ly: uint) -> String {
    assert!(tb.len() >= ((lx + 1) * (ly + 1)) as usize);
    let mut path = Vec::new();
    let mut i = lx as i32;
    let mut j = ly as i32;
    loop {
        if i == 0 && j == 0 {
            break;
        }
        if i < 0 || j < 0 {
            return String::from_utf8(path).unwrap();
        }
        let tb_char = tb[(i as uint * (ly + 1) + j as uint) as usize];
        path.push(tb_char as u8);
        match tb_char {
            x if x == b'B' as i8 => {
                i -= 1;
                j -= 1;
            }
            x if x == b'X' as i8 => i -= 1,
            x if x == b'Y' as i8 => j -= 1,
            _ => panic!("TraceBackFlat invalid byte {}", tb_char),
        }
    }
    path.reverse();
    String::from_utf8(path).unwrap()
}
