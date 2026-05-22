// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Computes Q1, Q2 and pairwise-consistency scores for one sequence pair
/// and appends them to the QScorer3 result lists.
#[track_caller]
pub fn q_scorer3_trans_q_pair(qs3: &mut QScorer3, indexi: uint, indexj: uint) {
    let labeli = &qs3.qs1.labels[indexi as usize];
    let labelj = &qs3.qs1.labels[indexj as usize];

    let ref_seq_indexi = qs3.qs1.ref_seq_indexes[indexi as usize];
    let ref_seq_indexj = qs3.qs1.ref_seq_indexes[indexj as usize];
    assert_eq!(qs3.ref_msa.seqs[ref_seq_indexi as usize].label, *labeli);
    assert_eq!(qs3.ref_msa.seqs[ref_seq_indexj as usize].label, *labelj);

    let indexi2 = qs3.indexes2[indexi as usize];
    let indexj2 = qs3.indexes2[indexj as usize];
    assert_eq!(qs3.qs2.labels[indexi2 as usize], *labeli);
    assert_eq!(qs3.qs2.labels[indexj2 as usize], *labelj);
    assert_eq!(qs3.qs2.ref_seq_indexes[indexi2 as usize], ref_seq_indexi);
    assert_eq!(qs3.qs2.ref_seq_indexes[indexj2 as usize], ref_seq_indexj);

    let test_col_to_posi1 = &qs3.qs1.test_col_to_pos_vec[indexi as usize];
    let test_col_to_posi2 = &qs3.qs2.test_col_to_pos_vec[indexi2 as usize];

    let test_col_to_posj1 = &qs3.qs1.test_col_to_pos_vec[indexj as usize];
    let test_col_to_posj2 = &qs3.qs2.test_col_to_pos_vec[indexj2 as usize];

    let ref_col_to_posi1 = &qs3.qs1.ref_col_to_pos_vec[indexi as usize];
    let ref_col_to_posi2 = &qs3.qs2.ref_col_to_pos_vec[indexi2 as usize];

    let ref_col_to_posj1 = &qs3.qs1.ref_col_to_pos_vec[indexj as usize];
    let ref_col_to_posj2 = &qs3.qs2.ref_col_to_pos_vec[indexj2 as usize];

    let pos_to_test_coli1 = &qs3.qs1.pos_to_test_col_vec[indexi as usize];
    let pos_to_test_coli2 = &qs3.qs2.pos_to_test_col_vec[indexi2 as usize];

    let pos_to_test_colj1 = &qs3.qs1.pos_to_test_col_vec[indexj as usize];
    let pos_to_test_colj2 = &qs3.qs2.pos_to_test_col_vec[indexj2 as usize];

    let ref_col_count = ref_col_to_posi1.len();
    assert_eq!(ref_col_to_posi2.len(), ref_col_count);
    assert_eq!(ref_col_to_posj1.len(), ref_col_count);
    assert_eq!(ref_col_to_posj2.len(), ref_col_count);

    let li = pos_to_test_coli1.len();
    let lj = pos_to_test_colj1.len();

    assert_eq!(pos_to_test_coli2.len(), li);
    assert_eq!(pos_to_test_colj2.len(), lj);

    let ref_cols = &qs3.ref_cols;
    let ref_aligned_col_count = qs3.qs1.ref_aligned_col_count;
    assert_eq!(qs3.qs2.ref_aligned_col_count, ref_aligned_col_count);

    let mut correct_col_count1 = 0;
    let mut correct_col_count2 = 0;
    let mut posis = Vec::new();
    let mut posjs = Vec::new();
    for k in 0..ref_aligned_col_count {
        let ref_col = ref_cols[k as usize];

        let posi = ref_col_to_posi1[ref_col as usize];
        let posi2 = ref_col_to_posi2[ref_col as usize];
        if posi == uint::MAX || posi2 == uint::MAX {
            continue;
        }
        assert_eq!(posi2, posi);

        let test_coli1 = pos_to_test_coli1[posi as usize];
        let test_coli2 = pos_to_test_coli2[posi as usize];

        let posj = ref_col_to_posj1[ref_col as usize];
        let posj2 = qs3.qs2.ref_col_to_pos_vec[indexj2 as usize][ref_col as usize];
        if posj == uint::MAX || posj2 == uint::MAX {
            continue;
        }
        assert_eq!(posj2, posj);
        posis.push(posi);
        posjs.push(posj);

        let test_colj1 = pos_to_test_colj1[posj as usize];
        let test_colj2 = pos_to_test_colj2[posj as usize];

        if test_coli1 == test_colj1 {
            correct_col_count1 += 1;
        }
        if test_coli2 == test_colj2 {
            correct_col_count2 += 1;
        }
    }

    let test_aligned_pos_count = posis.len();
    assert_eq!(posjs.len(), test_aligned_pos_count);

    let mut same_col_count = 0;
    for k in 0..test_aligned_pos_count {
        let posi = posis[k];

        let test_col1 = pos_to_test_coli1[posi as usize];
        let posj1 = test_col_to_posj1[test_col1 as usize];

        let test_col2 = pos_to_test_coli2[posi as usize];
        let posj2 = test_col_to_posj2[test_col2 as usize];

        if posj1 == posj2 {
            same_col_count += 1;
        }
    }

    for k in 0..test_aligned_pos_count {
        let posj = posjs[k];

        let test_col1 = pos_to_test_colj1[posj as usize];
        let posi1 = test_col_to_posi1[test_col1 as usize];

        let test_col2 = pos_to_test_colj2[posj as usize];
        let posi2 = test_col_to_posi2[test_col2 as usize];

        if posi1 == posi2 {
            same_col_count += 1;
        }
    }

    let q1 = correct_col_count1 as f32 / ref_aligned_col_count as f32;
    let q2 = correct_col_count2 as f32 / ref_aligned_col_count as f32;
    let pwc = same_col_count as f32 / (2 * test_aligned_pos_count) as f32;

    qs3.pairs.push((indexi, indexj));
    qs3.pair_index_to_q1.push(q1);
    qs3.pair_index_to_q2.push(q2);
    qs3.pair_index_to_pwc.push(pwc);
}

/// Runs `q_scorer3_trans_q_pair` over all sequence pairs.
#[track_caller]
pub fn q_scorer3_trans_q(qs3: &mut QScorer3) {
    let n = qs3.qs1.labels.len() as uint;

    assert!(!qs3.ref_cols.is_empty());

    qs3.pairs.clear();
    qs3.pair_index_to_q1.clear();
    qs3.pair_index_to_q2.clear();
    qs3.pair_index_to_pwc.clear();
    for indexi in 0..n {
        for indexj in indexi + 1..n {
            q_scorer3_trans_q_pair(qs3, indexi, indexj);
        }
    }
}
