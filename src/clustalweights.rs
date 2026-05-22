// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct ClustalWeights {
    pub node_to_subtree_size: Vec<uint>,
    pub node_to_strength: Vec<f32>,
} // original: ClustalWeights (muscle/src/clustalweights.h)

/// Computes Clustal-style sequence weights from a guide tree: per-edge strength
/// is `length / subtree_leaf_count` (with a 0.05 floor), weights sum to 1.
#[track_caller]
pub fn clustal_weights_run(cw: &mut ClustalWeights, ms: &MultiSequence, t: &Tree) -> Vec<f32> {
    let root = t.root_node_index;
    let node_count = t.node_count;
    let seq_count = ms.seqs.len() as uint;
    let mut weights = vec![f32::MAX; seq_count as usize];

    cw.node_to_subtree_size.clear();
    cw.node_to_subtree_size.resize(node_count as usize, 0);

    let root_subtree_size = clustal_weights_set_subtree_size(cw, t, root);
    let leaf_count = (0..node_count)
        .filter(|node| {
            let i = *node as usize;
            t.node_count == 1
                || (t.neighbor2[i] == NULL_NEIGHBOR && t.neighbor3[i] == NULL_NEIGHBOR)
        })
        .count() as uint;
    assert_eq!(root_subtree_size, leaf_count);

    cw.node_to_strength.clear();
    cw.node_to_strength.reserve(node_count as usize);
    for node in 0..node_count {
        if node == t.root_node_index {
            cw.node_to_strength.push(0.0);
            continue;
        }
        let parent = t.neighbor1[node as usize];
        let mut length = tree_get_edge_length(t, node, parent) as f32;
        if length < 0.05 {
            length = 0.05;
        }
        let leaf_count = cw.node_to_subtree_size[node as usize];
        let strength = length / leaf_count as f32;
        cw.node_to_strength.push(strength);
    }

    let mut sum_weights = 0.0_f32;
    for node in 0..node_count {
        let i = node as usize;
        if !(t.node_count == 1
            || (t.neighbor2[i] == NULL_NEIGHBOR && t.neighbor3[i] == NULL_NEIGHBOR))
        {
            continue;
        }
        let seq_index = tree_get_leaf_id(t, node);
        assert!((seq_index as usize) < weights.len() && weights[seq_index as usize] == f32::MAX);
        let path = tree_get_path_to_root(t, node);
        let mut weight = 0.0_f32;
        for node2 in path {
            weight += cw.node_to_strength[node2 as usize];
        }
        weights[seq_index as usize] = weight;
        sum_weights += weight;
    }

    for seq_index in 0..seq_count {
        weights[seq_index as usize] /= sum_weights;
    }
    weights
}

/// Recursively counts the leaves under `node` and caches the result in
/// `cw.node_to_subtree_size`.
#[track_caller]
pub fn clustal_weights_set_subtree_size(cw: &mut ClustalWeights, t: &Tree, node: uint) -> uint {
    let i = node as usize;
    if t.node_count == 1 || (t.neighbor2[i] == NULL_NEIGHBOR && t.neighbor3[i] == NULL_NEIGHBOR) {
        cw.node_to_subtree_size[i] = 1;
        return 1;
    }
    let left = t.neighbor2[i];
    let right = t.neighbor3[i];
    let size = clustal_weights_set_subtree_size(cw, t, left)
        + clustal_weights_set_subtree_size(cw, t, right);
    cw.node_to_subtree_size[i] = size;
    size
}
