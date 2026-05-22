// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Builds the LA x LB BLOSUM62 score matrix between two Sequence objects.
#[track_caller]
pub fn make_blosum62_s_mx_l30(a: &Sequence, b: &Sequence) -> Mx {
    let la = a.char_vec.len();
    let lb = b.char_vec.len();
    let mut mx_s = Mx {
        name: "BlosumS".to_string(),
        row_count: la as uint,
        col_count: lb as uint,
        data: vec![vec![0.0; lb]; la],
    };
    for i in 0..la {
        let ai = match a.char_vec[i].to_ascii_uppercase() {
            'A' => 0,
            'C' => 1,
            'D' => 2,
            'E' => 3,
            'F' => 4,
            'G' => 5,
            'H' => 6,
            'I' => 7,
            'K' => 8,
            'L' => 9,
            'M' => 10,
            'N' => 11,
            'P' => 12,
            'Q' => 13,
            'R' => 14,
            'S' => 15,
            'T' => 16,
            'V' => 17,
            'W' => 18,
            'Y' => 19,
            _ => uint::MAX,
        };
        for j in 0..lb {
            let bi = match b.char_vec[j].to_ascii_uppercase() {
                'A' => 0,
                'C' => 1,
                'D' => 2,
                'E' => 3,
                'F' => 4,
                'G' => 5,
                'H' => 6,
                'I' => 7,
                'K' => 8,
                'L' => 9,
                'M' => 10,
                'N' => 11,
                'P' => 12,
                'Q' => 13,
                'R' => 14,
                'S' => 15,
                'T' => 16,
                'V' => 17,
                'W' => 18,
                'Y' => 19,
                _ => uint::MAX,
            };
            if ai < 20 && bi < 20 {
                mx_s.data[i][j] = BLOSUM62_SIJ[ai as usize][bi as usize];
            } else {
                mx_s.data[i][j] = 0.0;
            }
        }
    }
    mx_s
}

/// Builds the LA x LB BLOSUM62 score matrix between two strings (string variant).
#[track_caller]
pub fn make_blosum62_s_mx_l54(a: &str, b: &str) -> Mx {
    let la = a.len();
    let lb = b.len();
    let mut mx_s = Mx {
        name: "BlosumS".to_string(),
        row_count: la as uint,
        col_count: lb as uint,
        data: vec![vec![0.0; lb]; la],
    };
    for (i, ca) in a.bytes().enumerate() {
        for (j, cb) in b.bytes().enumerate() {
            mx_s.data[i][j] = get_blosum_score_chars(ca, cb);
        }
    }
    mx_s
}

/// Returns the BLOSUM62 log-odds matrix as a 20x20 vector-of-vectors.
#[track_caller]
pub fn get_blosum62_log_odds_letter_mx() -> Vec<Vec<f32>> {
    let mut log_odds_mx = Vec::new();
    log_odds_mx.resize(20, Vec::new());
    for i in 0..20 {
        log_odds_mx[i].resize(20, 0.0);
    }
    for i in 0..20 {
        for j in 0..20 {
            log_odds_mx[i][j] = BLOSUM62_SIJ[i][j];
        }
    }
    log_odds_mx
}
