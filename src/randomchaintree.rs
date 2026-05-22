// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Build a random caterpillar/chain guide tree over the given labels.
#[track_caller]
pub fn make_random_chain_tree(labels: &[String], t: &mut Tree) {
    let seq_count = labels.len();
    let mut seq_indexes: Vec<uint> = (0..seq_count as uint).collect();
    shuffle(&mut seq_indexes);

    let mut parents = vec![uint::MAX; 2 * seq_count - 1];
    let mut lengths = Vec::new();
    let mut node_labels = Vec::new();
    for seq_index in &seq_indexes {
        node_labels.push(labels[*seq_index as usize].clone());
        lengths.push(1.0);
    }

    for i in 0..seq_count.saturating_sub(1) {
        if i == 0 {
            let left = seq_indexes[0];
            let right = seq_indexes[1];
            parents[left as usize] = seq_count as uint;
            parents[right as usize] = seq_count as uint;
        } else {
            let left = seq_count as uint + i as uint - 1;
            let right = seq_indexes[i + 1];
            parents[left as usize] = seq_count as uint + i as uint;
            parents[right as usize] = seq_count as uint + i as uint;
        }
        node_labels.push(String::new());
        lengths.push(1.0);
    }

    tree_from_vectors(t, &node_labels, &parents, &lengths);
}

/// Populate `mpc.guide_tree` with a random chain tree over the MPCFlat labels.
#[track_caller]
pub fn mpc_flat_calc_guide_tree_random_chain(mpc: &mut MPCFlat) {
    make_random_chain_tree(&mpc.labels, &mut mpc.guide_tree);
}

/// `labels2randomchaintree` subcommand: build a random chain tree from a
/// labels file and write it to Newick.
#[track_caller]
pub fn cmd_labels2randomchaintree(labels_file_name: &str, newick_file_name: &str) -> Tree {
    let labels = read_strings_from_file(labels_file_name);
    let mut t = Tree::default();
    make_random_chain_tree(&labels, &mut t);
    tree_to_file_l13(&t, newick_file_name);
    t
}
