// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Return shrub LCAs: non-overlapping subtrees covering the whole tree with at most `n` leaves each.
#[track_caller]
pub fn get_shrubs(t: &Tree, n: uint) -> Vec<uint> {
    let mut shrub_lcas = Vec::new();
    get_shrubs_into(t, n, &mut shrub_lcas);
    shrub_lcas
}

/// Append shrub LCAs to caller-owned storage, matching C++ `GetShrubs`.
#[track_caller]
pub fn get_shrubs_into(t: &Tree, n: uint, shrub_lcas: &mut Vec<uint>) {
    let sizes = tree_get_subtree_sizes(t);

    let mut shrub_leaf_count = 0;
    let node_count = t.node_count;
    let leaf_count = if t.rooted {
        assert!(t.node_count % 2 == 1);
        (t.node_count + 1) / 2
    } else {
        assert!(t.node_count % 2 == 0);
        (t.node_count + 2) / 2
    };

    for node in 0..node_count {
        let size = sizes[node as usize];
        if t.rooted && t.root_node_index == node {
            if size <= n {
                assert!(shrub_lcas.is_empty());
                shrub_lcas.push(node);
                shrub_leaf_count = tree_get_subtree_leaf_count(t, node);
                break;
            }
            continue;
        }

        let parent = t.neighbor1[node as usize];
        let parent_size = sizes[parent as usize];
        if size <= n && parent_size > n {
            shrub_lcas.push(node);
            shrub_leaf_count += tree_get_subtree_leaf_count(t, node);
        }
    }
    assert!(shrub_leaf_count == leaf_count);
}

/// `shrub` command: partition a tree into shrubs of at most `n` leaves and print pruned-tree pairings.
#[track_caller]
pub fn cmd_shrub(input_file_name: &str, n: Option<uint>) -> (Vec<uint>, Tree, String) {
    let mut t = Tree::default();
    tree_from_file_l143(&mut t, input_file_name);
    let mut out = String::new();
    let mut emit = |s: &str| {
        log(s);
        out.push_str(s);
    };
    emit(&tree_log_me(&t));
    assert!(t.rooted);

    let shrub_lcas = get_shrubs(&t, n.unwrap_or(32));
    let shrub_count = shrub_lcas.len() as uint;
    for i in 0..shrub_count {
        let lca = shrub_lcas[i as usize];
        let labels = tree_get_subtree_leaf_labels(&t, lca);
        emit(&format!("[{i:4}] {:3} ", labels.len()));
        for label in &labels {
            emit(&format!(" {label}"));
        }
        emit("\n");
    }

    let mut pt = Tree::default();
    let _shrub_labels = tree_prune_tree(&mut pt, &t, &shrub_lcas, "Shrub_");
    emit(&tree_log_me(&pt));

    let mut node = tree_first_depth_first_node(&pt);
    loop {
        assert!(node < pt.node_count);
        let i = node as usize;
        let neighbor_count = (pt.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (pt.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (pt.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if !(pt.node_count == 1 || neighbor_count == 1) {
            let left = pt.neighbor2[i];
            let right = pt.neighbor3[i];
            let left_labels = tree_get_subtree_leaf_labels(&pt, left);
            let right_labels = tree_get_subtree_leaf_labels(&pt, right);
            emit("[");
            let mut first = true;
            for left_label in &left_labels {
                let l = if let Some(suffix) = left_label.strip_prefix("Shrub_") {
                    str_to_uint_l1278(suffix, false)
                } else {
                    str_to_uint_l1278(left_label, false)
                };
                let lca = shrub_lcas[l as usize];
                let labels = tree_get_subtree_leaf_labels(&t, lca);
                for label in &labels {
                    if first {
                        first = false;
                    } else {
                        emit("+");
                    }
                    emit(label);
                }
            }
            emit("] [");
            for right_label in &right_labels {
                let l = if let Some(suffix) = right_label.strip_prefix("Shrub_") {
                    str_to_uint_l1278(suffix, false)
                } else {
                    str_to_uint_l1278(right_label, false)
                };
                let lca = shrub_lcas[l as usize];
                let labels = tree_get_subtree_leaf_labels(&t, lca);
                for label in &labels {
                    if first {
                        first = false;
                    } else {
                        emit("+");
                    }
                    emit(label);
                }
            }
            emit("]\n");
        }
        node = tree_next_depth_first_node(&pt, node);
        if node == uint::MAX {
            break;
        }
    }
    (shrub_lcas, pt, out)
}
