// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Return subtree root indexes obtained by cutting `tree` at the given height.
#[track_caller]
pub fn cluster_by_height(tree: &mut Tree, d_max_height: f64) -> Vec<uint> {
    if !tree.rooted {
        panic!("ClusterByHeight: requires rooted tree");
    }

    let mut subtrees = Vec::new();
    for node_index in 0..tree.node_count {
        if node_index == tree.root_node_index {
            continue;
        }
        let parent = tree.neighbor1[node_index as usize];
        let height = tree_get_node_height(tree, node_index);
        let parent_height = tree_get_node_height(tree, parent);
        if parent_height > d_max_height && height <= d_max_height {
            subtrees.push(node_index);
        }
    }
    subtrees
}

/// One step of the subfamily-count clustering: split the highest-child subfamily into its children.
#[track_caller]
pub fn cluster_by_subfam_count_iteration(tree: &mut Tree, subfams: &mut [uint], count: uint) {
    let mut highest_height = -1e20;
    let mut parent_subscript = i32::MIN;

    for n in 0..count as usize {
        let node_index = subfams[n];
        let i = node_index as usize;
        let neighbor_count = (tree.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if tree.node_count == 1 || neighbor_count == 1 {
            continue;
        }

        let left = tree.neighbor2[i];
        let height_left = tree_get_node_height(tree, left);
        if height_left > highest_height {
            highest_height = height_left;
            parent_subscript = n as i32;
        }

        let right = tree.neighbor3[i];
        let height_right = tree_get_node_height(tree, right);
        if height_right > highest_height {
            highest_height = height_right;
            parent_subscript = n as i32;
        }
    }

    if parent_subscript == i32::MIN {
        panic!("CBSFCIter: failed to find highest child");
    }

    let node_index = subfams[parent_subscript as usize];
    let left = tree.neighbor2[node_index as usize];
    let right = tree.neighbor3[node_index as usize];
    subfams[parent_subscript as usize] = left;
    subfams[count as usize] = right;
}

/// Divide a tree into `subfam_count` subfamilies by repeatedly splitting at the highest internal node.
#[track_caller]
pub fn cluster_by_subfam_count(tree: &mut Tree, subfam_count: uint) -> Vec<uint> {
    let node_count = tree.node_count;
    let leaf_count = (node_count + 1) / 2;

    if node_count == 0 {
        return Vec::new();
    }

    if subfam_count >= leaf_count {
        return (0..leaf_count).collect();
    }

    let mut subfams = vec![NULL_NEIGHBOR; subfam_count as usize];
    subfams[0] = tree.root_node_index;
    for i in 1..subfam_count {
        cluster_by_subfam_count_iteration(tree, &mut subfams, i);
    }
    subfams
}

/// Recursively collect leaf indexes under `node_index` into `leaves`.
#[track_caller]
pub fn get_leaves_recurse(tree: &Tree, node_index: uint, leaves: &mut Vec<uint>) {
    let i = node_index as usize;
    let neighbor_count = (tree.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (tree.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (tree.neighbor3[i] != NULL_NEIGHBOR) as uint;
    if tree.node_count == 1 || neighbor_count == 1 {
        leaves.push(node_index);
        return;
    }

    let left = tree.neighbor2[i];
    let right = tree.neighbor3[i];
    get_leaves_recurse(tree, left, leaves);
    get_leaves_recurse(tree, right, leaves);
}

/// Return all leaf indexes of the subtree rooted at `node_index`.
#[track_caller]
pub fn get_leaves(tree: &Tree, node_index: uint) -> Vec<uint> {
    let mut leaves = Vec::new();
    get_leaves_recurse(tree, node_index, &mut leaves);
    leaves
}

/// Build `pruned` as the tree whose leaves are the supplied subfamilies of `tree`.
#[track_caller]
pub fn tree_prune_tree(
    pruned: &mut Tree,
    tree: &Tree,
    subfams: &[uint],
    label_prefix: &str,
) -> Vec<String> {
    if !tree.rooted {
        panic!("Tree::PruneTree: requires rooted tree");
    }
    let subfam_count = subfams.len();
    let mut labels = Vec::new();

    *pruned = Tree::default();
    pruned.node_count = 2 * subfam_count as uint - 1;
    tree_init_cache(pruned, pruned.node_count);

    let unpruned_node_count = tree.node_count as usize;
    let pruned_node_count = pruned.node_count as usize;
    let mut unpruned_to_pruned_index = vec![NULL_NEIGHBOR; unpruned_node_count];
    let mut pruned_to_unpruned_index = vec![NULL_NEIGHBOR; pruned_node_count];

    let mut internal_node_index = subfam_count as uint;
    for (subfam_index, subfam) in subfams.iter().enumerate() {
        let mut unpruned_node_index = *subfam;
        unpruned_to_pruned_index[unpruned_node_index as usize] = subfam_index as uint;
        pruned_to_unpruned_index[subfam_index] = unpruned_node_index;
        loop {
            unpruned_node_index = tree.neighbor1[unpruned_node_index as usize];
            if unpruned_node_index == tree.root_node_index {
                break;
            }
            if unpruned_to_pruned_index[unpruned_node_index as usize] != NULL_NEIGHBOR {
                break;
            }

            unpruned_to_pruned_index[unpruned_node_index as usize] = internal_node_index;
            pruned_to_unpruned_index[internal_node_index as usize] = unpruned_node_index;
            internal_node_index += 1;
        }
    }

    let unpruned_root_index = tree.root_node_index;
    unpruned_to_pruned_index[unpruned_root_index as usize] = internal_node_index;
    pruned_to_unpruned_index[internal_node_index as usize] = unpruned_root_index;

    if internal_node_index != pruned.node_count - 1 {
        panic!("Tree::PruneTree, Internal error");
    }

    for subfam_index in 0..subfam_count {
        let label = format!("{label_prefix}{subfam_index}");
        pruned.names[subfam_index] = Some(label.clone());
        labels.push(label);
    }

    for pruned_node_index in subfam_count..pruned_node_count {
        let unpruned_node_index = pruned_to_unpruned_index[pruned_node_index];
        let unpruned_left = tree.neighbor2[unpruned_node_index as usize];
        let unpruned_right = tree.neighbor3[unpruned_node_index as usize];
        let pruned_left = unpruned_to_pruned_index[unpruned_left as usize];
        let pruned_right = unpruned_to_pruned_index[unpruned_right as usize];

        let left_length = tree_get_edge_length(tree, unpruned_node_index, unpruned_left);
        let right_length = tree_get_edge_length(tree, unpruned_node_index, unpruned_right);

        pruned.neighbor2[pruned_node_index] = pruned_left;
        pruned.neighbor3[pruned_node_index] = pruned_right;

        pruned.edge_length1[pruned_left as usize] = left_length;
        pruned.edge_length1[pruned_right as usize] = right_length;

        pruned.neighbor1[pruned_left as usize] = pruned_node_index as uint;
        pruned.neighbor1[pruned_right as usize] = pruned_node_index as uint;

        pruned.has_edge_length1[pruned_left as usize] = true;
        pruned.has_edge_length1[pruned_right as usize] = true;

        pruned.edge_length2[pruned_node_index] = left_length;
        pruned.edge_length3[pruned_node_index] = right_length;

        pruned.has_edge_length2[pruned_node_index] = true;
        pruned.has_edge_length3[pruned_node_index] = true;
    }

    pruned.root_node_index = unpruned_to_pruned_index[unpruned_root_index as usize];
    pruned.rooted = true;
    tree_validate(pruned);
    labels
}

/// Translate a list of leaf node indexes to leaf ids.
#[track_caller]
pub fn leaf_indexes_to_ids(tree: &Tree, leaves: &[uint]) -> Vec<uint> {
    let mut ids = Vec::new();
    for leaf in leaves {
        ids.push(tree_get_leaf_id(tree, *leaf));
    }
    ids
}
