// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Builds the weighted posterior probability matrix used to align two MSAs.
/// `M[i,j]` is the weighted sum of `P(s[i] <-> t[j])` over all pairs `(s, t)`
/// with `s` in MSA1 and `t` in MSA2.
#[track_caller]
pub fn mpc_flat_build_post(mpc: &MPCFlat, msa1: &MultiSequence, msa2: &MultiSequence) -> Vec<f32> {
    let seq_count1 = msa1.seqs.len();
    let seq_count2 = msa2.seqs.len();

    let col_count1 = multi_sequence_get_col_count(msa1);
    let col_count2 = multi_sequence_get_col_count(msa2);

    // Cache per-seq metadata once. The original code recomputed `pos_to_col2`
    // (which calls sequence_get_seq_as_string + a pass over the row) once per
    // outer (seq1, seq2) pair, making it quadratic in seq_count1*seq_count2.
    // For large MSAs this dominates the inner loop.
    let smi_1_cache: Vec<uint> = (0..seq_count1)
        .map(|i| mpc_flat_get_my_input_seq_index(mpc, &msa1.seqs[i].label))
        .collect();
    let smi_2_cache: Vec<uint> = (0..seq_count2)
        .map(|i| mpc_flat_get_my_input_seq_index(mpc, &msa2.seqs[i].label))
        .collect();
    let pos_to_col1_cache: Vec<Vec<uint>> = (0..seq_count1)
        .map(|i| sequence_get_pos_to_col(&sequence_get_seq_as_string(&msa1.seqs[i])))
        .collect();
    let pos_to_col2_cache: Vec<Vec<uint>> = (0..seq_count2)
        .map(|i| sequence_get_pos_to_col(&sequence_get_seq_as_string(&msa2.seqs[i])))
        .collect();

    let mut post = vec![0.0_f32; (col_count1 * col_count2) as usize];
    for seq_index1 in 0..seq_count1 {
        let smi_1 = smi_1_cache[seq_index1];
        assert_ne!(smi_1, uint::MAX);
        let w1 = mpc.weights[seq_index1];

        let pos_to_col1 = &pos_to_col1_cache[seq_index1];
        for seq_index2 in 0..seq_count2 {
            let smi_2 = smi_2_cache[seq_index2];
            assert_ne!(smi_2, uint::MAX);
            assert_ne!(smi_1, smi_2);
            let w2 = mpc.weights[seq_index2];

            let pos_to_col2 = &pos_to_col2_cache[seq_index2];
            if smi_1 < smi_2 {
                let pair_index = mpc_flat_get_pair_index(mpc, smi_1, smi_2);
                let mx = mpc.sparse_posts1[pair_index as usize]
                    .as_ref()
                    .unwrap_or_else(|| panic!("missing sparse post {pair_index}"));
                let lx = mx.lx;
                let ly = mx.ly;
                assert_eq!(pos_to_col1.len(), lx as usize);
                assert_eq!(pos_to_col2.len(), ly as usize);
                for i in 0..lx {
                    let col1 = pos_to_col1[i as usize];
                    let mut offset = my_sparse_mx_get_offset(mx, i);
                    let size = my_sparse_mx_get_size(mx, i);
                    for _k in 0..size {
                        let p = mx.value_vec[offset as usize].0;
                        let j = mx.value_vec[offset as usize].1;
                        offset += 1;
                        let col2 = pos_to_col2[j as usize];
                        post[(col1 * col_count2 + col2) as usize] += w1 * w2 * p;
                    }
                }
            } else {
                let pair_index = mpc_flat_get_pair_index(mpc, smi_2, smi_1);
                let mx = mpc.sparse_posts1[pair_index as usize]
                    .as_ref()
                    .unwrap_or_else(|| panic!("missing sparse post {pair_index}"));
                let lx = mx.lx;
                let ly = mx.ly;
                assert_eq!(pos_to_col2.len(), lx as usize);
                assert_eq!(pos_to_col1.len(), ly as usize);
                for i in 0..lx {
                    let col2 = pos_to_col2[i as usize];
                    let mut offset = my_sparse_mx_get_offset(mx, i);
                    let size = my_sparse_mx_get_size(mx, i);
                    for _k in 0..size {
                        let p = mx.value_vec[offset as usize].0;
                        let j = mx.value_vec[offset as usize].1;
                        offset += 1;
                        let col1 = pos_to_col1[j as usize];
                        post[(col1 * col_count2 + col2) as usize] += w1 * w2 * p;
                    }
                }
            }
        }
    }
    post
}
