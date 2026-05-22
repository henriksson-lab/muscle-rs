// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Joins two rooted trees under a new root connected by edges of `new_edge_length`.
#[track_caller]
pub fn join_trees(tree1: &Tree, tree2: &Tree, output_tree: &mut Tree, new_edge_length: f32) {
    let node_count1 = tree1.node_count;
    let node_count2 = tree2.node_count;

    let (labels1, parents1, lengths1) = tree_to_vectors(tree1);
    let (labels2, parents2, lengths2) = tree_to_vectors(tree2);

    let root = node_count1 + node_count2;

    let mut labels = Vec::new();
    let mut parents = Vec::new();
    let mut lengths = Vec::new();

    let mut root1_found = false;
    for node1 in 0..node_count1 {
        let label = labels1[node1 as usize].clone();
        let mut length = lengths1[node1 as usize];
        let mut parent = parents1[node1 as usize];
        if parent == uint::MAX {
            assert!(!root1_found);
            root1_found = true;
            parent = root;
            length = new_edge_length;
        }

        labels.push(label);
        parents.push(parent);
        lengths.push(length);
    }
    assert!(root1_found);

    let mut root2_found = false;
    for node2 in 0..node_count2 {
        let label = labels2[node2 as usize].clone();
        let mut length = lengths2[node2 as usize];
        let parent2 = parents2[node2 as usize];
        let parent;
        if parent2 == uint::MAX {
            assert!(!root2_found);
            root2_found = true;
            parent = root;
            length = new_edge_length;
        } else {
            parent = parent2 + node_count1;
        }

        labels.push(label);
        parents.push(parent);
        lengths.push(length);
    }
    assert!(root2_found);

    labels.push("ROOT".to_string());
    parents.push(uint::MAX);
    lengths.push(0.0);

    tree_from_vectors(output_tree, &labels, &parents, &lengths);
}
