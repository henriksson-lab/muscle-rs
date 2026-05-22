// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct MPCFlatMega; // original: MPCFlat_mega (muscle/src/mpcflat_mega.h)

/// MPCFlat forward DP wrapper that fetches Mega profiles by global sequence index.
#[track_caller]
pub fn mpc_flat_mega_calc_fwd_flat_mpc_flat(
    gsix: uint,
    lx: uint,
    gsiy: uint,
    ly: uint,
    flat: &mut [f32],
) {
    let profile_x = mega_get_profile_by_gsi(gsix);
    let profile_y = mega_get_profile_by_gsi(gsiy);
    assert_eq!(profile_x.len(), lx as usize);
    assert_eq!(profile_y.len(), ly as usize);
    mega_calc_fwd_flat_mega(&profile_x, &profile_y, flat);
}

/// MPCFlat backward DP wrapper that fetches Mega profiles by global sequence index.
#[track_caller]
pub fn mpc_flat_mega_calc_bwd_flat_mpc_flat(
    gsix: uint,
    lx: uint,
    gsiy: uint,
    ly: uint,
    flat: &mut [f32],
) {
    let profile_x = mega_get_profile_by_gsi(gsix);
    let profile_y = mega_get_profile_by_gsi(gsiy);
    assert_eq!(profile_x.len(), lx as usize);
    assert_eq!(profile_y.len(), ly as usize);
    mega_calc_bwd_flat_mega(&profile_x, &profile_y, flat);
}
