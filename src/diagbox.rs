// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagBox {
    pub la: uint,
    pub lb: uint,
    pub dlo: uint,
    pub dhi: uint,
    pub dlo_mini: uint,
    pub dlo_minj: uint,
    pub dlo_maxi: uint,
    pub dlo_maxj: uint,
    pub dhi_mini: uint,
    pub dhi_minj: uint,
    pub dhi_maxi: uint,
    pub dhi_maxj: uint,
} // original: DiagBox (muscle/src/diagbox.h)

/// Min/max (i, j) cell indices that lie on diagonal `d` of an LA*LB DP matrix.
#[track_caller]
pub fn get_diag_range(la: uint, lb: uint, d: uint) -> (uint, uint, uint, uint) {
    let maxi = std::cmp::min(la + lb - 1 - d, la - 1);
    let maxj = std::cmp::min(lb - 1, d - 1);
    if d >= la {
        (0, d - la, maxi, maxj)
    } else {
        (la - d, 0, maxi, maxj)
    }
}

/// Build the diagonal rectangle (DiagBox) spanning diagonals `diag_lo`..=`diag_hi` of an LA*LB DP matrix.
#[track_caller]
pub fn get_diag_box(la: uint, lb: uint, diag_lo: uint, diag_hi: uint) -> DiagBox {
    assert!(diag_lo <= diag_hi);
    assert!(diag_lo >= 1);
    assert!(diag_hi <= la + lb - 1);
    let (dlo_mini, dlo_minj, dlo_maxi, dlo_maxj) = get_diag_range(la, lb, diag_lo);
    let (dhi_mini, dhi_minj, dhi_maxi, dhi_maxj) = get_diag_range(la, lb, diag_hi);
    DiagBox {
        la,
        lb,
        dlo: diag_lo,
        dhi: diag_hi,
        dlo_mini,
        dlo_minj,
        dlo_maxi,
        dlo_maxj,
        dhi_mini,
        dhi_minj,
        dhi_maxi,
        dhi_maxj,
    }
}

/// Return the minimum and maximum diagonal indices visited by the alignment `path`.
#[track_caller]
pub fn get_diag_lo_hi(la: uint, _lb: uint, path: &str) -> (uint, uint) {
    let mut dlo = uint::MAX;
    let mut dhi = uint::MAX;
    let mut i = 0;
    let mut j = 0;
    for c in path.bytes() {
        if c == b'M' {
            let d = la - i + j;
            if dlo == uint::MAX {
                dlo = d;
                dhi = d;
            } else {
                if d < dlo {
                    dlo = d;
                }
                if d > dhi {
                    dhi = d;
                }
            }
        }
        if c == b'M' || c == b'D' {
            i += 1;
        }
        if c == b'M' || c == b'I' {
            j += 1;
        }
    }
    (dlo, dhi)
}

/// Test helper: builds a DiagBox and validates its computed bounds match `get_diag_range`.
#[track_caller]
pub fn test2_l88(la: uint, lb: uint, diag_lo: uint, diag_hi: uint) -> DiagBox {
    let box_ = get_diag_box(la, lb, diag_lo, diag_hi);
    assert!(box_.dlo <= box_.dhi);
    assert!(box_.dlo >= 1);
    assert!(box_.dhi <= box_.la + box_.lb - 1);
    let (dlo_mini, dlo_minj, dlo_maxi, dlo_maxj) = get_diag_range(la, lb, diag_lo);
    let (dhi_mini, dhi_minj, dhi_maxi, dhi_maxj) = get_diag_range(la, lb, diag_hi);
    assert_eq!(
        (box_.dlo_mini, box_.dlo_minj, box_.dlo_maxi, box_.dlo_maxj),
        (dlo_mini, dlo_minj, dlo_maxi, dlo_maxj)
    );
    assert_eq!(
        (box_.dhi_mini, box_.dhi_minj, box_.dhi_maxi, box_.dhi_maxj),
        (dhi_mini, dhi_minj, dhi_maxi, dhi_maxj)
    );
    box_
}

/// Test helper: asserts `get_diag_range` for `(la, lb, d)` matches expected `(i, j, I, J)`.
#[track_caller]
pub fn test1_l96(la: uint, lb: uint, d: uint, i: uint, j: uint, i_upper: uint, j_upper: uint) {
    let (mini, minj, maxi, maxj) = get_diag_range(la, lb, d);
    assert_eq!(mini, i);
    assert_eq!(maxi, i_upper);
    assert_eq!(minj, j);
    assert_eq!(maxj, j_upper);
}

/// Self-test routine exercising `get_diag_range` and `get_diag_box` over a grid of (la, lb, d) values.
#[track_caller]
pub fn test_diag_box() -> String {
    test1_l96(5, 3, 1, 4, 0, 4, 0);
    test1_l96(5, 3, 2, 3, 0, 4, 1);
    test1_l96(5, 3, 3, 2, 0, 4, 2);
    test1_l96(5, 3, 4, 1, 0, 3, 2);
    test1_l96(5, 3, 5, 0, 0, 2, 2);
    test1_l96(5, 3, 6, 0, 1, 1, 2);
    test1_l96(5, 3, 7, 0, 2, 0, 2);

    test1_l96(3, 5, 1, 2, 0, 2, 0);
    test1_l96(3, 5, 2, 1, 0, 2, 1);
    test1_l96(3, 5, 3, 0, 0, 2, 2);
    test1_l96(3, 5, 4, 0, 1, 2, 3);
    test1_l96(3, 5, 5, 0, 2, 2, 4);
    test1_l96(3, 5, 6, 0, 3, 1, 4);
    test1_l96(3, 5, 7, 0, 4, 0, 4);

    test1_l96(5, 5, 1, 4, 0, 4, 0);
    test1_l96(5, 5, 2, 3, 0, 4, 1);
    test1_l96(5, 5, 3, 2, 0, 4, 2);
    test1_l96(5, 5, 4, 1, 0, 4, 3);
    test1_l96(5, 5, 5, 0, 0, 4, 4);
    test1_l96(5, 5, 6, 0, 1, 3, 4);
    test1_l96(5, 5, 7, 0, 2, 2, 4);
    test1_l96(5, 5, 8, 0, 3, 1, 4);
    test1_l96(5, 5, 9, 0, 4, 0, 4);

    for la in 2..=5 {
        for lb in 2..=5 {
            for dlo in 1..=la + lb - 1 {
                for dhi in dlo..=la + lb - 1 {
                    test2_l88(la, lb, dlo, dhi);
                }
            }
        }
    }

    "\nALL OK\n".to_string()
}
