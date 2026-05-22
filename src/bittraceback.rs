// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Decodes the predecessor edge type ('M'/'D'/'I') from packed traceback bits and current type.
pub fn x_char(bits: byte, c_type: byte) -> byte {
    match c_type {
        b'M' => match bits & BIT_xM {
            BIT_MM => b'M',
            BIT_DM => b'D',
            BIT_IM => b'I',
            _ => panic!("invalid M traceback bits"),
        },
        b'D' => match bits & BIT_xD {
            BIT_MD => b'M',
            BIT_DD => b'D',
            _ => panic!("invalid D traceback bits"),
        },
        b'I' => match bits & BIT_xI {
            BIT_MI => b'M',
            BIT_II => b'I',
            _ => panic!("invalid I traceback bits"),
        },
        _ => panic!("invalid edge type"),
    }
}

/// Reconstructs the M/D/I edit path from a packed traceback matrix produced by Viterbi DP.
pub fn bit_trace_back(
    trace_back: &[Vec<byte>],
    u_length_a: uint,
    u_length_b: uint,
    last_edge: byte,
) -> String {
    let mut path = Vec::new();
    let mut pla = u_length_a as usize;
    let mut plb = u_length_b as usize;
    let mut c_type = last_edge;

    loop {
        path.push(c_type);

        let bits = trace_back[pla][plb];
        let next_edge_type = x_char(bits, c_type);
        match c_type {
            b'M' => {
                if pla == 0 {
                    panic!("BitTraceBack MA=0");
                }
                if plb == 0 {
                    panic!("BitTraceBack MA=0");
                }
                pla -= 1;
                plb -= 1;
            }
            b'D' => {
                if pla == 0 {
                    panic!("BitTraceBack DA=0");
                }
                pla -= 1;
            }
            b'I' => {
                if plb == 0 {
                    panic!("BitTraceBack IB=0");
                }
                plb -= 1;
            }
            _ => panic!("BitTraceBack: Invalid edge {}", c_type as char),
        }

        if pla == 0 && plb == 0 {
            break;
        }
        c_type = next_edge_type;
    }

    path.reverse();
    String::from_utf8(path).expect("traceback path contains only ASCII edge types")
}
