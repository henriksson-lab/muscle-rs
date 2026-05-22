// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Kimura distance from fractional identity. Uses the empirical formula below 75%
/// difference, the Dayhoff-PAM table from 75-93%, and a hard cap of 10.0 above 93%.
pub fn get_kimura_dist(fract_id: f32) -> f32 {
    const DAYHOFF_PAMS: [i32; 181] = [
        195, 196, 197, 198, 199, 200, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 209, 210,
        211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 226, 227, 228, 229,
        230, 231, 232, 233, 234, 236, 237, 238, 239, 240, 241, 243, 244, 245, 246, 248, 249, 250,
        252, 253, 254, 255, 257, 258, 260, 261, 262, 264, 265, 267, 268, 270, 271, 273, 274, 276,
        277, 279, 281, 282, 284, 285, 287, 289, 291, 292, 294, 296, 298, 299, 301, 303, 305, 307,
        309, 311, 313, 315, 317, 319, 321, 323, 325, 328, 330, 332, 335, 337, 339, 342, 344, 347,
        349, 352, 354, 357, 360, 362, 365, 368, 371, 374, 377, 380, 383, 386, 389, 393, 396, 399,
        403, 407, 410, 414, 418, 422, 426, 430, 434, 438, 442, 447, 451, 456, 461, 466, 471, 476,
        482, 487, 493, 498, 504, 511, 517, 524, 531, 538, 545, 553, 560, 569, 577, 586, 595, 605,
        615, 626, 637, 649, 661, 675, 688, 703, 719, 736, 754, 775, 796, 819, 845, 874, 907, 945,
        988,
    ];

    let p = 1.0 - fract_id;
    if p < 0.75 {
        return -(1.0 - p - (p * p) / 5.0).ln();
    }
    if p > 0.93 {
        return 10.0;
    }

    assert!((0.75..=1.0).contains(&p));
    let table_index = ((p - 0.75) * 1000.0 + 0.5) as i32;
    if table_index < 0 || table_index as usize >= DAYHOFF_PAMS.len() {
        panic!("Internal error in MSADistKimura::ComputeDist");
    }
    DAYHOFF_PAMS[table_index as usize] as f32 / 100.0
}

/// Fractional identity between two aligned sequences (ignoring gap/gap columns).
#[track_caller]
pub fn get_fract_id(seqi: &Sequence, seqj: &Sequence) -> f32 {
    let col_count = seqi.char_vec.len();
    assert_eq!(seqj.char_vec.len(), col_count);
    let mut n = 0;
    let mut total = 0;
    for col in 0..col_count {
        let ci = seqi.char_vec[col];
        let cj = seqj.char_vec[col];
        if (ci == '-' || ci == '.') && (cj == '-' || cj == '.') {
            continue;
        }
        total += 1;
        if ci.to_ascii_uppercase() == cj.to_ascii_uppercase() {
            n += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        n as f32 / total as f32
    }
}

/// Fractional identity computed by walking a pairwise alignment path (`M`/`D`/`I`).
pub fn get_fract_id_path(seqi: &str, seqj: &str, pi: &PathInfo) -> f32 {
    let path = pi.path.as_bytes();
    let si = seqi.as_bytes();
    let sj = seqj.as_bytes();
    let li = si.len();
    let lj = sj.len();
    let mut posi = 0usize;
    let mut posj = 0usize;
    let mut n = 0;
    let mut total = 0;
    for &edge in path {
        match edge {
            b'M' => {
                let ci = si[posi];
                let cj = sj[posj];
                posi += 1;
                posj += 1;
                if (ci == b'-' || ci == b'.') && (cj == b'-' || cj == b'.') {
                    continue;
                }
                total += 1;
                if ci.to_ascii_uppercase() == cj.to_ascii_uppercase() {
                    n += 1;
                }
            }
            b'D' => {
                posi += 1;
                continue;
            }
            b'I' => {
                posj += 1;
                continue;
            }
            _ => panic!("invalid path char"),
        }
    }
    assert_eq!(posi, li);
    assert_eq!(posj, lj);
    if total == 0 {
        0.0
    } else {
        n as f32 / total as f32
    }
}

/// Computes the pairwise Kimura distance matrix from an already-aligned MSA.
#[track_caller]
pub fn get_kimura_dist_mx(msa: &MultiSequence) -> Vec<Vec<f32>> {
    let seq_count = msa.seqs.len();
    let mut dist_mx = vec![vec![f32::MAX; seq_count]; seq_count];

    for i in 0..seq_count {
        dist_mx[i][i] = 0.0;
        let seqi = &msa.seqs[i];
        for j in 0..i {
            let seqj = &msa.seqs[j];
            let fract_id = get_fract_id(seqi, seqj);
            let d = get_kimura_dist(fract_id);
            dist_mx[i][j] = d;
            dist_mx[j][i] = d;
        }
    }
    dist_mx
}

/// Pairwise Kimura distance matrix using a Viterbi aligner to derive each pair's path.
#[track_caller]
pub fn get_kimura_dist_mx_viterbi<FViterbi>(
    ms: &MultiSequence,
    mut viterbi_fast_mem: FViterbi,
) -> Vec<Vec<f32>>
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
{
    let seq_count = ms.seqs.len();
    let mut dist_mx = vec![vec![f32::MAX; seq_count]; seq_count];

    for i in 0..seq_count {
        dist_mx[i][i] = 0.0;
        let seqi = &ms.seqs[i];
        let seqi_string = sequence_get_seq_as_string(seqi);
        let byte_seqi = seqi_string.as_bytes();
        let li = byte_seqi.len() as uint;
        for j in 0..i {
            let seqj = &ms.seqs[j];
            let seqj_string = sequence_get_seq_as_string(seqj);
            let byte_seqj = seqj_string.as_bytes();
            let lj = byte_seqj.len() as uint;
            let pi = viterbi_fast_mem(byte_seqi, li, byte_seqj, lj);
            let fract_id = get_fract_id_path(&seqi_string, &seqj_string, &pi);
            let d = get_kimura_dist(fract_id);
            dist_mx[i][j] = d;
            dist_mx[j][i] = d;
        }
    }
    dist_mx
}
