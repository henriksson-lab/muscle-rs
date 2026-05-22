// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct PProgMega; // original: PProg_mega (muscle/src/pprog_mega.h)

/// Mega forward DP wrapper: looks up the two mega profiles by GSI and writes the flat result.
#[track_caller]
pub fn p_prog_mega_calc_fwd_flat_p_prog(
    gsi1: uint,
    l1: uint,
    gsi2: uint,
    l2: uint,
    flat: &mut [f32],
) {
    let profile1 = mega_get_profile_by_gsi(gsi1);
    let profile2 = mega_get_profile_by_gsi(gsi2);
    assert_eq!(profile1.len(), l1 as usize);
    assert_eq!(profile2.len(), l2 as usize);
    mega_calc_fwd_flat_mega(&profile1, &profile2, flat);
}

/// Mega backward DP wrapper: looks up the two mega profiles by GSI and writes the flat result.
#[track_caller]
pub fn p_prog_mega_calc_bwd_flat_p_prog(
    gsi1: uint,
    l1: uint,
    gsi2: uint,
    l2: uint,
    flat: &mut [f32],
) {
    let profile1 = mega_get_profile_by_gsi(gsi1);
    let profile2 = mega_get_profile_by_gsi(gsi2);
    assert_eq!(profile1.len(), l1 as usize);
    assert_eq!(profile2.len(), l2 as usize);
    mega_calc_bwd_flat_mega(&profile1, &profile2, flat);
}
