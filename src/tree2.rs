// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Iterator step over all edges of the tree in DFS order; returns false
/// when exhausted.
pub fn phy_enum_edges(tree: &Tree, es: &mut PhyEnumEdgeState) -> bool {
    let node1;
    if !es.init {
        if tree.node_count <= 1 {
            es.node_index1 = NULL_NEIGHBOR;
            es.node_index2 = NULL_NEIGHBOR;
            return false;
        }
        node1 = tree_first_depth_first_node(tree);
        es.init = true;
    } else {
        let mut next = tree_next_depth_first_node(tree, es.node_index1);
        if next == NULL_NEIGHBOR {
            return false;
        }
        if tree.rooted && tree.root_node_index == next {
            next = tree_next_depth_first_node(tree, next);
            if next == NULL_NEIGHBOR {
                return false;
            }
        }
        node1 = next;
    }
    let node2 = tree.neighbor1[node1 as usize];
    es.node_index1 = node1;
    es.node_index2 = node2;
    true
}

/// Right-first variant of `phy_enum_edges`.
pub fn phy_enum_edges_r(tree: &Tree, es: &mut PhyEnumEdgeState) -> bool {
    let node1;
    if !es.init {
        if tree.node_count <= 1 {
            es.node_index1 = NULL_NEIGHBOR;
            es.node_index2 = NULL_NEIGHBOR;
            return false;
        }
        node1 = tree_first_depth_first_node_r(tree);
        es.init = true;
    } else {
        let mut next = tree_next_depth_first_node_r(tree, es.node_index1);
        if next == NULL_NEIGHBOR {
            return false;
        }
        if tree.rooted && tree.root_node_index == next {
            next = tree_next_depth_first_node(tree, next);
            if next == NULL_NEIGHBOR {
                return false;
            }
        }
        node1 = next;
    }
    let node2 = tree.neighbor1[node1 as usize];
    es.node_index1 = node1;
    es.node_index2 = node2;
    true
}

/// Recursively collects leaves in the subtree rooted at `node_index1`
/// when walking away from `node_index2`.
pub fn get_leaves_subtree(
    tree: &Tree,
    node_index1: uint,
    node_index2: uint,
    leaves: &mut Vec<uint>,
) {
    let i = node_index1 as usize;
    let neighbor_count = (tree.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (tree.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (tree.neighbor3[i] != NULL_NEIGHBOR) as uint;
    if tree.node_count == 1 || neighbor_count == 1 {
        leaves.push(node_index1);
        return;
    }

    let left = tree_get_first_neighbor(tree, node_index1, node_index2);
    let right = tree_get_second_neighbor(tree, node_index1, node_index2);
    if left != NULL_NEIGHBOR {
        get_leaves_subtree(tree, left, node_index1, leaves);
    }
    if right != NULL_NEIGHBOR {
        get_leaves_subtree(tree, right, node_index1, leaves);
    }
}

/// Returns the leaves on `node_index1`'s side of the edge to
/// `node_index2`.
pub fn phy_get_leaves(tree: &Tree, node_index1: uint, node_index2: uint) -> Vec<uint> {
    let mut leaves = Vec::new();
    get_leaves_subtree(tree, node_index1, node_index2, &mut leaves);
    leaves
}

/// Iterator step over all bipartitions induced by tree edges; returns
/// `(side1, side2)` leaf lists or `None` when exhausted.
pub fn phy_enum_bi_parts(tree: &Tree, es: &mut PhyEnumEdgeState) -> Option<(Vec<uint>, Vec<uint>)> {
    let mut ok = phy_enum_edges(tree, es);
    if !ok {
        return None;
    }

    if tree.rooted
        && tree.root_node_index == es.node_index2
        && tree.neighbor3[es.node_index2 as usize] == es.node_index1
    {
        ok = phy_enum_edges(tree, es);
        if !ok {
            return None;
        }
    }

    let leaves1 = phy_get_leaves(tree, es.node_index1, es.node_index2);
    let leaves2 = phy_get_leaves(tree, es.node_index2, es.node_index1);
    let leaf_count = if tree.rooted {
        assert!(tree.node_count % 2 == 1);
        (tree.node_count + 1) / 2
    } else {
        assert!(tree.node_count % 2 == 0);
        (tree.node_count + 2) / 2
    };
    if leaves1.len() as uint + leaves2.len() as uint != leaf_count {
        panic!(
            "PhyEnumBiParts {} + {} != {}",
            leaves1.len(),
            leaves2.len(),
            leaf_count
        );
    }
    Some((leaves1, leaves2))
}

/// Diagnostic dump of all bipartitions for a tree.
#[track_caller]
pub fn test_bi_part(tree: &Tree) -> String {
    let mut out = tree_log_me(tree);
    let mut es = PhyEnumEdgeState {
        init: false,
        node_index1: NULL_NEIGHBOR,
        node_index2: NULL_NEIGHBOR,
    };
    loop {
        let bipart = phy_enum_bi_parts(tree, &mut es);
        let ok = bipart.is_some();
        out.push_str(&format!(
            "PEBP={} ES.Init={} ES.ni1={} ES.ni2={}\n",
            ok as uint, es.init as uint, es.node_index1, es.node_index2
        ));
        let Some((leaves1, leaves2)) = bipart else {
            break;
        };
        out.push('\n');
        out.push_str("Part1: ");
        for leaf in &leaves1 {
            let name = tree_get_leaf_name(tree, *leaf).unwrap_or("");
            out.push_str(&format!(" {}({})", leaf, name));
        }
        out.push('\n');
        out.push_str("Part2: ");
        for leaf in &leaves2 {
            let name = tree_get_leaf_name(tree, *leaf).unwrap_or("");
            out.push_str(&format!(" {}({})", leaf, name));
        }
        out.push('\n');
    }
    out
}

/// Collects leaves below `node_index` while pruning the subtree rooted
/// at `exclude`.
pub fn get_leaves_subtree_excluding(
    tree: &Tree,
    node_index: uint,
    exclude: uint,
    leaves: &mut Vec<uint>,
) {
    if node_index == exclude {
        return;
    }

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
    if left != NULL_NEIGHBOR {
        get_leaves_subtree_excluding(tree, left, exclude, leaves);
    }
    if right != NULL_NEIGHBOR {
        get_leaves_subtree_excluding(tree, right, exclude, leaves);
    }
}

/// Returns leaves descended from `node_index` excluding the subtree
/// rooted at `exclude`.
pub fn get_leaves_excluding(tree: &Tree, node_index: uint, exclude: uint) -> Vec<uint> {
    let mut leaves = Vec::new();
    get_leaves_subtree_excluding(tree, node_index, exclude, &mut leaves);
    leaves
}

/// Returns internal node indices sorted by ascending node height.
pub fn get_internal_nodes_in_height_order(tree: &mut Tree) -> Vec<uint> {
    let node_count = tree.node_count;
    if node_count < 3 {
        panic!("GetInternalNodesInHeightOrder: {node_count} nodes, none are internal");
    }
    let internal_node_count = (node_count - 1) / 2;
    let mut node_indexes = Vec::new();
    let mut heights = Vec::new();
    for node_index in 0..node_count {
        let i = node_index as usize;
        let neighbor_count = (tree.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if tree.node_count == 1 || neighbor_count == 1 {
            continue;
        }
        node_indexes.push(node_index);
        heights.push(tree_get_node_height(tree, node_index));
    }
    if node_indexes.len() as uint != internal_node_count {
        panic!("Internal error: GetInternalNodesInHeightOrder");
    }

    let mut done = false;
    while !done {
        done = true;
        for i in 0..(internal_node_count as usize - 1) {
            if heights[i] > heights[i + 1] {
                heights.swap(i, i + 1);
                node_indexes.swap(i, i + 1);
                done = false;
            }
        }
    }
    node_indexes
}

/// Raises any edge length below `min_edge_length` to that floor.
pub fn apply_min_edge_length(tree: &mut Tree, min_edge_length: f64) {
    let node_count = tree.node_count;
    for node_index in 0..node_count {
        let i = node_index as usize;
        let neighbor_count = (tree.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor3[i] != NULL_NEIGHBOR) as uint;
        for n in 0..neighbor_count {
            let neighbor_node_index = tree_get_neighbor(tree, node_index, n);
            if !tree_has_edge_length(tree, node_index, neighbor_node_index) {
                continue;
            }
            if tree_get_edge_length(tree, node_index, neighbor_node_index) < min_edge_length {
                tree_set_edge_length(tree, node_index, neighbor_node_index, min_edge_length);
            }
        }
    }
}
