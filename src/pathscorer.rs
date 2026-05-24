// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub struct PathScorer {
    pub trace: bool,
    pub la: uint,
    pub lb: uint,
} // original: PathScorer (muscle/src/pathscorer.h)

impl Default for PathScorer {
    fn default() -> Self {
        Self {
            trace: false,
            la: uint::MAX,
            lb: uint::MAX,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PathScorerMASMMega {
    pub base: PathScorer,
    pub masm: *const MASM,
    pub mega_profile: *const [Vec<byte>],
} // original: PathScorer_MASM_Mega (muscle/src/pathscorer.h)

impl Default for PathScorerMASMMega {
    fn default() -> Self {
        Self {
            base: PathScorer::default(),
            masm: std::ptr::null(),
            mega_profile: std::ptr::slice_from_raw_parts(std::ptr::null(), 0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PathScorerAABLOSUM62 {
    pub base: PathScorer,
    pub gap_open: f32,
    pub gap_ext: f32,
    pub seq_a: String,
    pub seq_b: String,
} // original: PathScorer_AA_BLOSUM62 (muscle/src/pathscorer.h)

impl Default for PathScorerAABLOSUM62 {
    fn default() -> Self {
        Self {
            base: PathScorer::default(),
            gap_open: f32::MAX,
            gap_ext: f32::MAX,
            seq_a: String::new(),
            seq_b: String::new(),
        }
    }
}

/// Sums per-column scores along `path` starting at `(pos_a, pos_b)`, using `get_score`.
#[track_caller]
pub fn path_scorer_get_local_score<F>(
    la: uint,
    lb: uint,
    mut pos_a: uint,
    mut pos_b: uint,
    path: &str,
    get_score: F,
) -> f32
where
    F: Fn(char, char, uint, uint) -> f32,
{
    let col_count = path.len() as uint;
    assert!(col_count > 0);
    assert_eq!(path.as_bytes()[0], b'M');
    assert_eq!(path.as_bytes()[col_count as usize - 1], b'M');
    assert!(la != uint::MAX && lb != uint::MAX);

    let mut total = 0.0;
    let mut last_state = 'M';
    for state_byte in path.bytes() {
        let state = char::from(state_byte);
        let score = get_score(last_state, state, pos_a, pos_b);
        total += score;
        match state {
            'M' => {
                pos_a += 1;
                pos_b += 1;
            }
            'D' => pos_a += 1,
            'I' => pos_b += 1,
            _ => panic!("invalid path state"),
        }
        last_state = state;
    }

    assert!(pos_a <= la && pos_b <= lb);
    total
}

/// Returns the per-column score for a state transition, dispatching to the appropriate closure.
#[track_caller]
pub fn path_scorer_get_score<FMatch, FMM, FMD, FMI, FDM, FDD, FIM, FII>(
    from_state: char,
    to_state: char,
    pos_a: uint,
    pos_b: uint,
    get_match_score: FMatch,
    get_score_mm: FMM,
    get_score_md: FMD,
    get_score_mi: FMI,
    get_score_dm: FDM,
    get_score_dd: FDD,
    get_score_im: FIM,
    get_score_ii: FII,
) -> f32
where
    FMatch: Fn(uint, uint) -> f32,
    FMM: Fn(uint, uint) -> f32,
    FMD: Fn(uint, uint) -> f32,
    FMI: Fn(uint, uint) -> f32,
    FDM: Fn(uint, uint) -> f32,
    FDD: Fn(uint, uint) -> f32,
    FIM: Fn(uint, uint) -> f32,
    FII: Fn(uint, uint) -> f32,
{
    let mut total = 0.0;
    if to_state == 'M' {
        total += get_match_score(pos_a, pos_b);
    }

    total += match (from_state, to_state) {
        ('M', 'M') => get_score_mm(pos_a, pos_b),
        ('M', 'D') => get_score_md(pos_a, pos_b),
        ('M', 'I') => get_score_mi(pos_a, pos_b),
        ('D', 'M') => get_score_dm(pos_a, pos_b),
        ('D', 'D') => get_score_dd(pos_a, pos_b),
        ('I', 'M') => get_score_im(pos_a, pos_b),
        ('I', 'I') => get_score_ii(pos_a, pos_b),
        _ => panic!("invalid scorer state transition {from_state}{to_state}"),
    };
    total
}

/// Initialises the MASM-vs-mega-profile scorer with the given MASM and profile columns.
#[track_caller]
pub fn path_scorer_masm_mega_init(ps: &mut PathScorerMASMMega, ma: &MASM, pb: &[Vec<byte>]) {
    ps.masm = ma as *const MASM;
    ps.mega_profile = pb as *const [Vec<byte>];
    ps.base.la = ma.col_count;
    ps.base.lb = pb.len() as uint;
}

#[track_caller]
fn path_scorer_masm_mega_get_masm(ps: &PathScorerMASMMega) -> &MASM {
    assert!(!ps.masm.is_null(), "PathScorer_MASM_Mega m_MASM is null");
    // C++ stores a raw MASM pointer. Rust mirrors that lifecycle here; callers
    // must keep the MASM alive and must not mutate it concurrently.
    unsafe { &*ps.masm }
}

#[track_caller]
fn path_scorer_masm_mega_get_profile(ps: &PathScorerMASMMega) -> &[Vec<byte>] {
    assert!(
        !ps.mega_profile.is_null(),
        "PathScorer_MASM_Mega m_MegaProfile is null"
    );
    // C++ stores a pointer to the caller-owned profile vector.
    unsafe { &*ps.mega_profile }
}

/// Returns the M-state match score for `(pos_a, pos_b)` (MASM column vs mega profile position).
#[track_caller]
pub fn path_scorer_masm_mega_get_match_score(
    ps: &PathScorerMASMMega,
    pos_a: uint,
    pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    let mega_profile = path_scorer_masm_mega_get_profile(ps);
    assert!(pos_a < masm.col_count);
    assert!((pos_b as usize) < mega_profile.len());
    let mcol = &masm.cols[pos_a as usize];
    let ppos = &mega_profile[pos_b as usize];
    masm_col_get_match_score_mega_profile_pos(mcol, ppos)
}

/// M->M transition score (zero penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_mm(
    _ps: &PathScorerMASMMega,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    0.0
}

/// M->D transition score (column-specific gap open penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_md(
    ps: &PathScorerMASMMega,
    pos_a: uint,
    _pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    assert!(pos_a < masm.col_count);
    -masm.cols[pos_a as usize].gap_open
}

/// M->I transition score (half the global gap-open penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_mi(
    ps: &PathScorerMASMMega,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    assert!(masm.gap_open != f32::MAX);
    let score = -masm.gap_open / 2.0;
    assert!(score <= 0.0);
    score
}

/// D->M transition score (column-specific gap close penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_dm(
    ps: &PathScorerMASMMega,
    pos_a: uint,
    _pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    assert!(pos_a < masm.col_count);
    let score = -masm.cols[pos_a as usize].gap_close;
    assert!(score <= 0.0);
    score
}

/// D->D transition score (column-specific gap extension penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_dd(
    ps: &PathScorerMASMMega,
    pos_a: uint,
    _pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    assert!(pos_a < masm.col_count);
    let score = -masm.cols[pos_a as usize].gap_ext;
    assert!(score <= 0.0);
    score
}

/// I->M transition score (half the global gap-open penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_im(
    ps: &PathScorerMASMMega,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    assert!(masm.gap_open != f32::MAX);
    let score = -masm.gap_open / 2.0;
    assert!(score <= 0.0);
    score
}

/// I->I transition score (global gap-extension penalty).
#[track_caller]
pub fn path_scorer_masm_mega_get_score_ii(
    ps: &PathScorerMASMMega,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    let masm = path_scorer_masm_mega_get_masm(ps);
    assert!(masm.gap_ext != f32::MAX);
    let score = -masm.gap_ext;
    assert!(score <= 0.0);
    score
}

/// BLOSUM62 substitution score between residues at `pos_a` and `pos_b`.
#[track_caller]
pub fn path_scorer_aa_blosum62_get_match_score(
    ps: &PathScorerAABLOSUM62,
    pos_a: uint,
    pos_b: uint,
) -> f32 {
    assert!((pos_a as usize) < ps.seq_a.len());
    assert!((pos_b as usize) < ps.seq_b.len());
    let a = ps.seq_a.as_bytes()[pos_a as usize];
    let b = ps.seq_b.as_bytes()[pos_b as usize];
    get_blosum_score_chars(a, b)
}

/// M->M transition score (zero penalty).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_mm(
    _ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    0.0
}

/// M->D transition score (constant gap-open).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_md(
    ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    assert!(ps.gap_open < 0.0);
    ps.gap_open
}

/// M->I transition score (constant gap-open).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_mi(
    ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    assert!(ps.gap_open < 0.0);
    ps.gap_open
}

/// D->M transition score (zero penalty).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_dm(
    _ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    0.0
}

/// D->D transition score (constant gap-extension).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_dd(
    ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    assert!(ps.gap_open <= 0.0);
    ps.gap_ext
}

/// I->M transition score (zero penalty).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_im(
    _ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    0.0
}

/// I->I transition score (constant gap-extension).
#[track_caller]
pub fn path_scorer_aa_blosum62_get_score_ii(
    ps: &PathScorerAABLOSUM62,
    _pos_a: uint,
    _pos_b: uint,
) -> f32 {
    assert!(ps.gap_ext <= 0.0);
    ps.gap_ext
}
