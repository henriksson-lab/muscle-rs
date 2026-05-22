// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Split a rooted tree at `node` into the rooted subtree under that node and the complementary supertree.
#[track_caller]
pub fn divide_tree(input_tree: &Tree, node: uint, subtree: &mut Tree, supertree: &mut Tree) {
    assert!(input_tree.rooted);
    let input_node_count = input_tree.node_count;
    let input_leaf_count = (input_tree.node_count + 1) / 2;
    assert!(node < input_node_count);
    assert!(node != input_tree.root_node_index);

    let subtree_leaf_nodes = tree_get_subtree_leaf_nodes(input_tree, node);
    let n = subtree_leaf_nodes.len();
    assert!(n > 0);

    let mut subtree_set = std::collections::BTreeSet::new();
    let mut subtree_labels = Vec::new();
    for node2 in &subtree_leaf_nodes {
        let label = input_tree.names[*node2 as usize]
            .clone()
            .unwrap_or_default();
        subtree_set.insert(*node2);
        subtree_labels.push(label);
    }

    let mut supertree_leaf_nodes = Vec::new();
    let mut supertree_labels = Vec::new();
    for node2 in 0..input_node_count {
        let neighbor_count = (input_tree.neighbor1[node2 as usize] != NULL_NEIGHBOR) as uint
            + (input_tree.neighbor2[node2 as usize] != NULL_NEIGHBOR) as uint
            + (input_tree.neighbor3[node2 as usize] != NULL_NEIGHBOR) as uint;
        if !(input_tree.node_count == 1 || neighbor_count == 1) {
            continue;
        }
        if !subtree_set.contains(&node2) {
            let label = input_tree.names[node2 as usize].clone().unwrap_or_default();
            supertree_leaf_nodes.push(node2);
            supertree_labels.push(label);
        }
    }

    let subtree_leaf_count = subtree_leaf_nodes.len() as uint;
    let supertree_leaf_count = supertree_leaf_nodes.len() as uint;
    assert!(subtree_leaf_count > 0);
    assert!(supertree_leaf_count > 0);
    assert!(subtree_leaf_count + supertree_leaf_count == input_leaf_count);

    *subtree = make_subset_nodes(input_tree, &subtree_leaf_nodes, &subtree_labels);
    *supertree = make_subset_nodes(input_tree, &supertree_leaf_nodes, &supertree_labels);
}

/// Driver: split an input tree at the LCA of two leaves and write subtree/supertree files.
#[track_caller]
pub fn cmd_divide_tree(
    input_file_name: &str,
    label1: &str,
    label2: &str,
    subtree_out: &str,
    supertree_out: &str,
) -> (Tree, Tree) {
    let mut input_tree = Tree::default();
    tree_from_file_l143(&mut input_tree, input_file_name);

    let node1 = tree_get_node_index_l1199(&input_tree, label1);
    let node2 = tree_get_node_index_l1199(&input_tree, label2);
    let divide_node = tree_get_lca(&input_tree, node1, node2);

    let mut subtree = Tree::default();
    let mut supertree = Tree::default();
    divide_tree(&input_tree, divide_node, &mut subtree, &mut supertree);

    tree_to_file_l13(&subtree, subtree_out);
    tree_to_file_l13(&supertree, supertree_out);
    (subtree, supertree)
}
