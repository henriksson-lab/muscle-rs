// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Traces back the bit-packed DP matrix in `mem` from `(la, lb, state)`
/// and writes the M/D/I path into `pi`.
#[track_caller]
pub fn trace_back_bit_mem(mem: &XDPMem, la: uint, lb: uint, state: byte, pi: &mut PathInfo) {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;

    path_info_alloc2(pi, la, lb);
    path_info_set_empty(pi);

    let mut i = la;
    let mut j = lb;
    let mut state = state;
    loop {
        if i == 0 && j == 0 {
            break;
        }

        path_info_append_char(pi, state);

        match state {
            b'M' => {
                assert!(i > 0 && j > 0);
                let t = mem.tb_bit[(i - 1) as usize][(j - 1) as usize];
                if t & TRACEBITS_DM != 0 {
                    state = b'D';
                } else if t & TRACEBITS_IM != 0 {
                    state = b'I';
                } else {
                    state = b'M';
                }
                i -= 1;
                j -= 1;
            }
            b'D' => {
                assert!(i > 0);
                let t = mem.tb_bit[(i - 1) as usize][j as usize];
                if t & TRACEBITS_MD != 0 {
                    state = b'M';
                } else {
                    state = b'D';
                }
                i -= 1;
            }
            b'I' => {
                assert!(j > 0);
                let t = mem.tb_bit[i as usize][(j - 1) as usize];
                if t & TRACEBITS_MI != 0 {
                    state = b'M';
                } else {
                    state = b'I';
                }
                j -= 1;
            }
            _ => panic!("TraceBackBit, invalid state {}", state as char),
        }
    }
    path_info_reverse(pi);
}
