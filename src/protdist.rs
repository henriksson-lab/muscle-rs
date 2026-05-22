// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// PHYLIP-style maximum-likelihood protein distance between two aligned byte
/// sequences using the PAM/JTT-style re-prediction loop.
#[track_caller]
pub fn get_prot_dist_l42(q_seq: &[byte], t_seq: &[byte], col_count: uint) -> f64 {
    const ITERS: uint = 20;
    const PROTEPSILON: f64 = 0.00001;

    assert!(q_seq.len() >= col_count as usize);
    assert!(t_seq.len() >= col_count as usize);
    let mut elambdat = 0.0;
    let mut q = 0.0;

    let mut tt = 0.1;
    let mut delta = tt / 2.0;
    let inf = false;
    for _iter in 0..ITERS {
        let mut _lnlike = 0.0;
        let mut slope = 0.0;
        let mut curv = 0.0;
        let mut neginfinity = false;
        let mut overlap = false;
        for col in 0..col_count {
            let char_q = q_seq[col as usize];
            let char_t = t_seq[col as usize];
            let letter_q = match char_q {
                b'A' | b'a' => 0,
                b'C' | b'c' => 1,
                b'D' | b'd' => 2,
                b'E' | b'e' => 3,
                b'F' | b'f' => 4,
                b'G' | b'g' => 5,
                b'H' | b'h' => 6,
                b'I' | b'i' => 7,
                b'K' | b'k' => 8,
                b'L' | b'l' => 9,
                b'M' | b'm' => 10,
                b'N' | b'n' => 11,
                b'P' | b'p' => 12,
                b'Q' | b'q' => 13,
                b'R' | b'r' => 14,
                b'S' | b's' => 15,
                b'T' | b't' => 16,
                b'V' | b'v' => 17,
                b'W' | b'w' => 18,
                b'Y' | b'y' => 19,
                _ => uint::MAX,
            };
            let letter_t = match char_t {
                b'A' | b'a' => 0,
                b'C' | b'c' => 1,
                b'D' | b'd' => 2,
                b'E' | b'e' => 3,
                b'F' | b'f' => 4,
                b'G' | b'g' => 5,
                b'H' | b'h' => 6,
                b'I' | b'i' => 7,
                b'K' | b'k' => 8,
                b'L' | b'l' => 9,
                b'M' | b'm' => 10,
                b'N' | b'n' => 11,
                b'P' | b'p' => 12,
                b'Q' | b'q' => 13,
                b'R' | b'r' => 14,
                b'S' | b's' => 15,
                b'T' | b't' => 16,
                b'V' | b'v' => 17,
                b'W' | b'w' => 18,
                b'Y' | b'y' => 19,
                _ => uint::MAX,
            };
            if letter_q >= 20 || letter_t >= 20 {
                continue;
            }
            overlap = true;
            let mut p = 0.0;
            let mut dp = 0.0;
            let mut d2p = 0.0;
            re_predict(
                letter_q,
                letter_t,
                &mut tt,
                &mut p,
                &mut dp,
                &mut d2p,
                &mut q,
                &mut elambdat,
            );
            if p <= 0.0 {
                neginfinity = true;
            } else {
                _lnlike += p.ln();
                slope += dp / p;
                curv += d2p / p - dp * dp / (p * p);
            }
        }
        if !overlap {
            return -1.0;
        } else if !neginfinity {
            if curv < 0.0 {
                tt -= slope / curv;
                if tt > 10000.0 {
                    return -1.0;
                }
            } else {
                if (slope > 0.0 && delta < 0.0) || (slope < 0.0 && delta > 0.0) {
                    delta /= -2.0;
                }
                tt += delta;
            }
        } else {
            delta /= -2.0;
            tt += delta;
        }
        if tt < PROTEPSILON && !inf {
            tt = PROTEPSILON;
        }
    }
    tt
}

/// String wrapper around `get_prot_dist_l42`. Both sequences must have equal length.
#[track_caller]
pub fn get_prot_dist_l111(q_seq: &str, t_seq: &str) -> f64 {
    let col_count = q_seq.len() as uint;
    assert_eq!(t_seq.len() as uint, col_count);
    get_prot_dist_l42(q_seq.as_bytes(), t_seq.as_bytes(), col_count)
}
