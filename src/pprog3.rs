// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct PProg3 {
    pub ap: Option<M3AlnParams>,
    pub input_seqs: Option<MultiSequence>,
    pub input_seq_weights: Vec<f32>,
    pub guide_tree: Option<Tree>,
    pub node_to_profile: Vec<Option<Profile3>>,
    pub node_to_sum_input_weights: Vec<f32>,
    pub node_to_path: Vec<String>,
    pub msa: MultiSequence,
    pub cm: CacheMem3,
} // original: PProg3 (muscle/src/pprog3.h)

/// Builds profiles from the input sequences and the guide tree using `nw_small3` and profile-profile alignment.
#[track_caller]
pub fn p_prog3_run<FNwSmall3, FAlignTwoProfsGivenPath>(
    pp: &mut PProg3,
    input_seqs: &MultiSequence,
    input_seq_weights: &[f32],
    guide_tree: &Tree,
    mut nw_small3: FNwSmall3,
    mut align_two_profs_given_path: FAlignTwoProfsGivenPath,
) where
    FNwSmall3: FnMut(&mut CacheMem3, &Profile3, &Profile3) -> String,
    FAlignTwoProfsGivenPath:
        FnMut(&Profile3, f32, &Profile3, f32, &[[f32; 20]; 20], f32, &str) -> Profile3,
{
    let ap = pp.ap.as_ref().expect("PProg3::Run no alignment params");
    let subst_mx_letter = ap.subst_mx_letter;
    let gap_open = ap.gap_open;
    pp.input_seqs = Some(input_seqs.clone());
    pp.input_seq_weights = input_seq_weights.to_vec();
    pp.guide_tree = Some(guide_tree.clone());
    assert!(guide_tree.rooted);
    let seq_count = input_seqs.seqs.len() as uint;
    let leaf_count = (guide_tree.node_count + 1) / 2;
    assert_eq!(leaf_count, seq_count);
    assert_eq!(input_seq_weights.len() as uint, seq_count);

    let node_count = guide_tree.node_count;
    pp.node_to_profile = vec![None; node_count as usize];
    pp.node_to_sum_input_weights = vec![f32::MAX; node_count as usize];
    pp.node_to_path = vec![String::new(); node_count as usize];

    let mut weights1 = vec![f32::MAX; 1];
    let mut node = tree_first_depth_first_node(guide_tree);
    loop {
        pp.node_to_profile[node as usize] = None;
        let i = node as usize;
        let is_leaf = guide_tree.node_count == 1
            || (guide_tree.neighbor2[i] == NULL_NEIGHBOR
                && guide_tree.neighbor3[i] == NULL_NEIGHBOR);
        if is_leaf {
            let seq_index = tree_get_leaf_id(guide_tree, node);
            let seq = input_seqs.seqs[seq_index as usize].clone();
            let mut msa1 = MultiSequence::default();
            msa1.seqs.push(seq);
            msa1.owners.push(false);
            let weight = input_seq_weights[seq_index as usize];
            pp.node_to_sum_input_weights[node as usize] = weight;

            weights1[0] = 1.0;
            let mut prof = Profile3::default();
            profile3_from_msa(&mut prof, &msa1, &subst_mx_letter, gap_open, &weights1);
            pp.node_to_profile[node as usize] = Some(prof);
        } else {
            let left = guide_tree.neighbor2[i];
            let right = guide_tree.neighbor3[i];
            assert!(left < node_count && right < node_count);

            let sum_input_weights_left = pp.node_to_sum_input_weights[left as usize];
            let sum_input_weights_right = pp.node_to_sum_input_weights[right as usize];
            assert!(sum_input_weights_left != f32::MAX);
            assert!(sum_input_weights_right != f32::MAX);
            pp.node_to_sum_input_weights[node as usize] =
                sum_input_weights_left + sum_input_weights_right;

            let prof_left = pp.node_to_profile[left as usize]
                .take()
                .expect("PProg3::Run missing left profile");
            let prof_right = pp.node_to_profile[right as usize]
                .take()
                .expect("PProg3::Run missing right profile");

            let path = nw_small3(&mut pp.cm, &prof_left, &prof_right);
            pp.node_to_path[node as usize] = path.clone();

            if node != guide_tree.root_node_index {
                let prof = align_two_profs_given_path(
                    &prof_left,
                    sum_input_weights_left,
                    &prof_right,
                    sum_input_weights_right,
                    &subst_mx_letter,
                    gap_open,
                    &path,
                );
                pp.node_to_profile[node as usize] = Some(prof);
            }
        }

        node = tree_next_depth_first_node(guide_tree, node);
        if node == uint::MAX {
            break;
        }
    }
    p_prog3_build_msa(pp);
}

/// Materialises the final MSA by collecting each leaf's aligned sequence.
#[track_caller]
pub fn p_prog3_build_msa(pp: &mut PProg3) {
    let guide_tree = pp
        .guide_tree
        .as_ref()
        .expect("PProg3::BuildMSA no guide tree");
    let node_count = guide_tree.node_count;
    let leaf_count = ((guide_tree.node_count + 1) / 2) as usize;
    multi_sequence_clear(&mut pp.msa);

    for node in 0..node_count {
        let i = node as usize;
        let is_leaf = guide_tree.node_count == 1
            || (guide_tree.neighbor2[i] == NULL_NEIGHBOR
                && guide_tree.neighbor3[i] == NULL_NEIGHBOR);
        if !is_leaf {
            continue;
        }
        let seq = p_prog3_get_aligned_seq(pp, node);
        pp.msa.seqs.push(seq);
        pp.msa.owners.push(true);
    }
    assert_eq!(pp.msa.seqs.len(), leaf_count);
}

/// Walks the path from `leaf_node` to the root, inserting gaps along each parent alignment.
#[track_caller]
pub fn p_prog3_get_aligned_seq(pp: &PProg3, leaf_node: uint) -> Sequence {
    let guide_tree = pp
        .guide_tree
        .as_ref()
        .expect("PProg3::GetAlignedSeq no guide tree");
    let leaf_i = leaf_node as usize;
    assert!(
        guide_tree.node_count == 1
            || (guide_tree.neighbor2[leaf_i] == NULL_NEIGHBOR
                && guide_tree.neighbor3[leaf_i] == NULL_NEIGHBOR)
    );
    let input_seq_index = tree_get_leaf_id(guide_tree, leaf_node);
    let input_seqs = pp
        .input_seqs
        .as_ref()
        .expect("PProg3::GetAlignedSeq no input seqs");
    let mut seq = input_seqs.seqs[input_seq_index as usize].clone();

    let path_to_root = tree_get_path_to_root(guide_tree, leaf_node);
    let n = path_to_root.len();
    for i in 0..n.saturating_sub(1) {
        let path_node = path_to_root[i];
        let parent = guide_tree.neighbor1[path_node as usize];
        let parent_left = guide_tree.neighbor2[parent as usize];
        let parent_right = guide_tree.neighbor3[parent as usize];
        let dor_i = if parent_right == path_node {
            'I'
        } else if parent_left == path_node {
            'D'
        } else {
            panic!("PProg3::GetAlignedSeq invalid parent/child relation");
        };
        assert!((parent as usize) < pp.node_to_path.len());
        let alignment_path = &pp.node_to_path[parent as usize];
        seq = sequence_add_gaps_path(&seq, alignment_path, dor_i);
    }
    seq
}
