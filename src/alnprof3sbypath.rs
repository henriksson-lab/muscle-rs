// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Sanity-check helper that materialises a start-dimer ProfPos3 (LL=1, others 0).
#[track_caller]
pub fn init_pp_start() -> bool {
    let mut pp_start = ProfPos3::default();
    prof_pos3_set_start_dimers(&mut pp_start);
    assert_eq!(pp_start.ll, 1.0);
    assert_eq!(pp_start.lg, 0.0);
    assert_eq!(pp_start.gl, 0.0);
    assert_eq!(pp_start.gg, 0.0);
    true
}

/// Combines two LL/LG/GL/GG dimer profiles for an `MM` edge (match in both profiles).
#[track_caller]
pub fn set_dimers_mm(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wa * ppa.ll + wb * ppb.ll;
    ppab.lg = wa * ppa.lg + wb * ppb.lg;
    ppab.gl = wa * ppa.gl + wb * ppb.gl;
    ppab.gg = wa * ppa.gg + wb * ppb.gg;
}

/// Combines dimer profiles for an `MD` edge: match on A, deletion (gap) on B.
#[track_caller]
pub fn set_dimers_md(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wa * ppa.ll;
    ppab.lg = wa * ppa.lg + wb * (ppb.ll + ppb.gl);
    ppab.gl = wa * ppa.gl;
    ppab.gg = wa * ppa.gg + wb * (ppb.lg + ppb.gg);
}

/// Combines dimer profiles for a `DD` edge: consecutive deletions on B side.
#[track_caller]
pub fn set_dimers_dd(ppa: &ProfPos3, wa: f32, _ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wa * ppa.ll;
    ppab.lg = wa * ppa.lg;
    ppab.gl = wa * ppa.gl;
    ppab.gg = wa * ppa.gg + wb;
}

/// Combines dimer profiles for an `MI` edge: previous match on A, insertion on B.
#[track_caller]
pub fn set_dimers_mi(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wb * ppb.ll;
    ppab.lg = wb * ppb.lg + wa * (ppa.ll + ppa.gl);
    ppab.gl = wb * ppb.gl;
    ppab.gg = wb * ppb.gg + wa * (ppa.lg + ppa.gg);
}

/// Combines dimer profiles for a `DM` edge: previous deletion, current match.
#[track_caller]
pub fn set_dimers_dm(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wa * ppa.ll;
    ppab.lg = wa * ppa.lg;
    ppab.gl = wa * ppa.gl + wb * (ppb.ll + ppb.gl);
    ppab.gg = wa * ppa.gg + wb * (ppb.lg + ppb.gg);
}

/// Combines dimer profiles for an `IM` edge: previous insertion, current match.
#[track_caller]
pub fn set_dimers_im(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wb * ppb.ll;
    ppab.lg = wb * ppb.lg;
    ppab.gl = wb * ppb.gl + wa * (ppa.ll + ppa.gl);
    ppab.gg = wb * ppb.gg + wa * (ppa.lg + ppa.gg);
}

/// Combines dimer profiles for an `ID` edge: previous insertion, current deletion.
#[track_caller]
pub fn set_dimers_id(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = 0.0;
    ppab.lg = wb * ppb.gl + wb * ppb.ll;
    ppab.gl = wa * ppa.gl + wa * ppa.ll;
    ppab.gg = wa * (ppa.lg + ppa.gg) + wb * (ppb.lg + ppb.gg);
}

/// Combines dimer profiles for a `DI` edge: previous deletion, current insertion.
#[track_caller]
pub fn set_dimers_di(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = 0.0;
    ppab.lg = wa * ppa.gl + wa * ppa.ll;
    ppab.gl = wb * ppb.gl + wb * ppb.ll;
    ppab.gg = wa * (ppa.lg + ppa.gg) + wb * (ppb.lg + ppb.gg);
}

/// Combines dimer profiles for an `II` edge: consecutive insertions on B.
#[track_caller]
pub fn set_dimers_ii(_ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    ppab.ll = wb * ppb.ll;
    ppab.lg = wb * ppb.lg;
    ppab.gl = wb * ppb.gl;
    ppab.gg = wb * ppb.gg + wa;
}

/// Sets the merged ProfPos3 letter frequencies from a single profile (weighted copy).
#[track_caller]
pub fn set_freqs1(pp: &ProfPos3, w: f32, ppab: &mut ProfPos3) {
    let alpha_size = ALPHA_STATE.lock().unwrap().alpha_size as usize;
    for i in 0..alpha_size {
        ppab.freqs[i] = w * pp.freqs[i];
    }
}

/// Sets the merged ProfPos3 letter frequencies as the weighted sum of two profile positions.
#[track_caller]
pub fn set_freqs2(ppa: &ProfPos3, wa: f32, ppb: &ProfPos3, wb: f32, ppab: &mut ProfPos3) {
    let alpha_size = ALPHA_STATE.lock().unwrap().alpha_size as usize;
    for i in 0..alpha_size {
        ppab.freqs[i] = wa * ppa.freqs[i] + wb * ppb.freqs[i];
    }
}

/// Merges two Profile3 profiles into one along a fixed `M/D/I` edit path, computing each merged
/// position's frequencies, dimer counts and scores.
#[track_caller]
pub fn align_two_profs_given_path(
    prof_a: &Profile3,
    weight_a: f32,
    prof_b: &Profile3,
    weight_b: f32,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
    path: &str,
) -> Profile3 {
    assert!(weight_a > 0.0 && weight_b > 0.0);
    let wa = weight_a / (weight_a + weight_b);
    let wb = weight_b / (weight_a + weight_b);
    assert!(myfeq((wa + wb) as f64, 1.0));

    let edge_count = path.len();
    let mut pp_start = ProfPos3::default();
    prof_pos3_set_start_dimers(&mut pp_start);
    let mut prof_ab = Profile3::default();
    prof_ab.pps.reserve(edge_count);
    let mut prev_type = b'M';
    let mut pos_a = 0usize;
    let mut pos_b = 0usize;
    let mut ppa_prev = pp_start.clone();
    let mut ppb_prev = pp_start.clone();

    for c_type in path.bytes() {
        let mut ppab = ProfPos3 {
            all_gaps: false,
            ..ProfPos3::default()
        };
        match c_type {
            b'M' => {
                let ppa = &prof_a.pps[pos_a];
                let ppb = &prof_b.pps[pos_b];
                set_freqs2(ppa, wa, ppb, wb, &mut ppab);
                match prev_type {
                    b'M' => set_dimers_mm(ppa, wa, ppb, wb, &mut ppab),
                    b'D' => set_dimers_dm(ppa, wa, ppb, wb, &mut ppab),
                    b'I' => set_dimers_im(ppa, wa, ppb, wb, &mut ppab),
                    _ => panic!("invalid previous path edge"),
                }
                ppa_prev = ppa.clone();
                ppb_prev = ppb.clone();
                pos_a += 1;
                pos_b += 1;
            }
            b'D' => {
                let ppa = &prof_a.pps[pos_a];
                set_freqs1(ppa, wa, &mut ppab);
                match prev_type {
                    b'M' => set_dimers_md(ppa, wa, &ppb_prev, wb, &mut ppab),
                    b'D' => set_dimers_dd(ppa, wa, &ppb_prev, wb, &mut ppab),
                    b'I' => set_dimers_id(ppa, wa, &ppb_prev, wb, &mut ppab),
                    _ => panic!("invalid previous path edge"),
                }
                ppa_prev = ppa.clone();
                pos_a += 1;
            }
            b'I' => {
                let ppb = &prof_b.pps[pos_b];
                set_freqs1(ppb, wb, &mut ppab);
                match prev_type {
                    b'M' => set_dimers_mi(&ppa_prev, wa, ppb, wb, &mut ppab),
                    b'D' => set_dimers_di(&ppa_prev, wa, ppb, wb, &mut ppab),
                    b'I' => set_dimers_ii(&ppa_prev, wa, ppb, wb, &mut ppab),
                    _ => panic!("invalid previous path edge"),
                }
                ppb_prev = ppb.clone();
                pos_b += 1;
            }
            _ => panic!("invalid path edge"),
        }
        prof_pos3_set_occ(&mut ppab);
        prof_ab.pps.push(ppab);
        prev_type = c_type;
    }
    assert_eq!(prof_ab.pps.len(), edge_count);
    profile3_set_scores(&mut prof_ab, subst_mx_letter, gap_open);
    profile3_validate(&prof_ab);
    prof_ab
}
