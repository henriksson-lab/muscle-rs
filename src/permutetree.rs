// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Writes the given strings (one per line) to `file_name`; no-op if the name is empty.
#[track_caller]
pub fn strings_to_file(file_name: &str, v: &[String]) {
    if file_name.is_empty() {
        return;
    }
    let mut data = String::new();
    for s in v {
        data.push_str(s);
        data.push('\n');
    }
    std::fs::write(file_name, data.as_bytes()).unwrap();
}

/// Splits `input_tree` into two subtrees, choosing a split node closest to the requested leaf fraction.
#[track_caller]
pub fn divide_tree_fraction(input_tree: &Tree, fract: f64, tree1: &mut Tree, tree2: &mut Tree) {
    assert!(fract > 0.0 && fract < 1.0);

    let input_leaf_count = (input_tree.node_count + 1) / 2;
    assert!(input_leaf_count >= 3);

    assert!(input_tree.rooted);
    let node_count = input_tree.node_count;
    let mut best_node = uint::MAX;
    let mut best_leaf_count = uint::MAX;
    let mut best_diff = uint::MAX;
    let mut target_leaf_count = (f64::from(input_leaf_count) * fract + 0.5) as uint;
    if target_leaf_count == 0 {
        target_leaf_count = 1;
    }

    for node in 0..node_count {
        let subtree_leaf_count = tree_get_subtree_leaf_count(input_tree, node);
        let diff = if subtree_leaf_count > target_leaf_count {
            subtree_leaf_count - target_leaf_count
        } else {
            target_leaf_count - subtree_leaf_count
        };
        if best_node == uint::MAX || diff < best_diff {
            best_node = node;
            best_diff = diff;
            best_leaf_count = subtree_leaf_count;
        }
    }

    assert!(best_leaf_count != uint::MAX);
    divide_tree(input_tree, best_node, tree1, tree2);
}

/// Splits the input into A/B/C subtrees and produces the three (ABC, ACB, BCA) joined permutations.
#[track_caller]
pub fn permute_tree(
    input_tree: &Tree,
    tree_abc: &mut Tree,
    tree_acb: &mut Tree,
    tree_bca: &mut Tree,
    labels_a: &mut Vec<String>,
    labels_b: &mut Vec<String>,
    labels_c: &mut Vec<String>,
) {
    labels_a.clear();
    labels_b.clear();
    labels_c.clear();

    let input_leaf_count = (input_tree.node_count + 1) / 2;
    if input_leaf_count < 6 {
        tree_copy(tree_abc, input_tree);
        tree_copy(tree_acb, input_tree);
        tree_copy(tree_bca, input_tree);
        return;
    }

    let new_edge_length = 0.1_f32;

    let mut tree_a = Tree::default();
    let mut tree_bc = Tree::default();
    let mut tree_b = Tree::default();
    let mut tree_c = Tree::default();
    divide_tree_fraction(input_tree, 0.33, &mut tree_a, &mut tree_bc);
    divide_tree_fraction(&tree_bc, 0.5, &mut tree_b, &mut tree_c);

    *labels_a = tree_get_leaf_labels(&tree_a);
    *labels_b = tree_get_leaf_labels(&tree_b);
    *labels_c = tree_get_leaf_labels(&tree_c);

    let mut join_ab = Tree::default();
    join_trees(&tree_a, &tree_b, &mut join_ab, new_edge_length);
    join_trees(&join_ab, &tree_c, tree_abc, new_edge_length);

    let mut join_ac = Tree::default();
    join_trees(&tree_a, &tree_c, &mut join_ac, new_edge_length);
    join_trees(&join_ac, &tree_b, tree_acb, new_edge_length);

    let mut join_bc = Tree::default();
    join_trees(&tree_b, &tree_c, &mut join_bc, new_edge_length);
    join_trees(&join_bc, &tree_a, tree_bca, new_edge_length);

    tree_ladderize(tree_abc, true);
    tree_ladderize(tree_acb, true);
    tree_ladderize(tree_bca, true);
}

/// In-place tree permutation: replaces `input_tree` with the requested ABC/ACB/BCA variant.
#[track_caller]
pub fn perm_tree(input_tree: &mut Tree, tp: TREEPERM) {
    if tp == TREEPERM::TP_None {
        return;
    }
    let leaf_count = (input_tree.node_count + 1) / 2;
    if leaf_count < 10 {
        return;
    }

    let mut tree_abc = Tree::default();
    let mut tree_acb = Tree::default();
    let mut tree_bca = Tree::default();
    let mut labels_a = Vec::new();
    let mut labels_b = Vec::new();
    let mut labels_c = Vec::new();
    permute_tree(
        input_tree,
        &mut tree_abc,
        &mut tree_acb,
        &mut tree_bca,
        &mut labels_a,
        &mut labels_b,
        &mut labels_c,
    );
    match tp {
        TREEPERM::TP_ABC => tree_copy(input_tree, &tree_abc),
        TREEPERM::TP_ACB => tree_copy(input_tree, &tree_acb),
        TREEPERM::TP_BCA => tree_copy(input_tree, &tree_bca),
        _ => panic!("PermTree, invalid tree permutation {:?}", tp),
    }
}

/// Implements the `permute_tree` subcommand: reads a tree, permutes it, optionally writes outputs.
#[track_caller]
pub fn cmd_permute_tree(
    input_file_name: &str,
    prefix: Option<&str>,
) -> (Tree, Tree, Tree, Vec<String>, Vec<String>, Vec<String>) {
    let mut input_tree = Tree::default();
    tree_from_file_l143(&mut input_tree, input_file_name);

    let mut tree_abc = Tree::default();
    let mut tree_acb = Tree::default();
    let mut tree_bca = Tree::default();
    let mut labels_a = Vec::new();
    let mut labels_b = Vec::new();
    let mut labels_c = Vec::new();
    permute_tree(
        &input_tree,
        &mut tree_abc,
        &mut tree_acb,
        &mut tree_bca,
        &mut labels_a,
        &mut labels_b,
        &mut labels_c,
    );

    if let Some(prefix) = prefix {
        tree_to_file_l13(&tree_abc, &format!("{prefix}ABC.newick"));
        tree_to_file_l13(&tree_acb, &format!("{prefix}ACB.newick"));
        tree_to_file_l13(&tree_bca, &format!("{prefix}BCA.newick"));
        strings_to_file(&format!("{prefix}labelsA.txt"), &labels_a);
        strings_to_file(&format!("{prefix}labelsB.txt"), &labels_b);
        strings_to_file(&format!("{prefix}labelsC.txt"), &labels_c);
    }

    (tree_abc, tree_acb, tree_bca, labels_a, labels_b, labels_c)
}
