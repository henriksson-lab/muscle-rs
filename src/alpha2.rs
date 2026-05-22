// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Sets the global alphabet to nucleotide or amino, idempotent when already configured.
#[track_caller]
pub fn set_alpha_lc(is_nucleo: bool) {
    let alpha = ALPHA_STATE.lock().unwrap().alpha;
    if is_nucleo {
        if alpha == ALPHA::ALPHA_Nucleo {
            return;
        }
        assert_eq!(alpha, ALPHA::ALPHA_Undefined);
        set_alpha_l209(ALPHA::ALPHA_Nucleo);
    } else {
        if alpha == ALPHA::ALPHA_Amino {
            return;
        }
        assert_eq!(alpha, ALPHA::ALPHA_Undefined);
        set_alpha_l209(ALPHA::ALPHA_Amino);
    }
}

/// Sets the global alphabet from an `ALPHA` enum value.
#[track_caller]
pub fn set_alpha_l35(alpha: ALPHA) {
    match alpha {
        ALPHA::ALPHA_Amino => set_alpha_lc(false),
        ALPHA::ALPHA_Nucleo => set_alpha_lc(true),
        _ => panic!("Invalid Alpha={alpha:?}"),
    }
}

/// Convenience wrapper for `set_alpha_l35` selecting nucleotide vs. amino by boolean.
#[track_caller]
pub fn set_alphab(is_nucleo: bool) {
    set_alpha_l35(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
}
