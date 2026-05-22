// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Read a tab-separated `<int>\t<label>` file into parallel ID and label vectors.
#[track_caller]
pub fn ints_from_file(file_name: &str) -> (Vec<uint>, Vec<String>) {
    let text = std::fs::read_to_string(file_name)
        .unwrap_or_else(|e| panic!("OpenStdioFile({file_name}) failed: {e}"));
    let mut ints = Vec::new();
    let mut labels = Vec::new();
    for line in text.lines() {
        let fields = split(line, '\t');
        assert!(fields.len() == 2);
        let i = str_to_uint_l1278(&fields[0], false);
        let label = fields[1].clone();
        ints.push(i);
        labels.push(label);
    }
    (ints, labels)
}

/// Build a contracted tree that keeps only the listed `subset_nodes`, relabeled with `subset_labels`.
#[track_caller]
pub fn make_subset_nodes(
    input_tree: &Tree,
    subset_nodes: &[uint],
    subset_labels: &[String],
) -> Tree {
    if !input_tree.rooted {
        panic!("Tree must be rooted");
    }

    let subset_node_count = subset_nodes.len();
    if subset_node_count < 2 {
        panic!("Need at least two nodes");
    }
    assert!(subset_labels.len() == subset_node_count);

    let node_count = input_tree.node_count;

    let mut old_node_to_new_label = std::collections::BTreeMap::<uint, String>::new();
    let mut subset_set = std::collections::BTreeSet::<uint>::new();
    for i in 0..subset_node_count {
        let subset_node = subset_nodes[i];
        subset_set.insert(subset_node);
        old_node_to_new_label.insert(subset_node, subset_labels[i].clone());
    }

    let mut parent_set = std::collections::BTreeSet::<uint>::new();
    for &subset_node in subset_nodes.iter().take(subset_node_count) {
        let path = tree_get_path_to_root(input_tree, subset_node);
        let n = path.len();
        assert!(path[0] == subset_node);
        for &node in path.iter().take(n).skip(1) {
            assert!(node < node_count);
            if subset_set.contains(&node) {
                parent_set.insert(node);
            }
        }
    }

    let parent_count = parent_set.len();
    let mut new_tips = std::collections::BTreeSet::<uint>::new();
    for &node in &subset_set {
        if !parent_set.contains(&node) {
            new_tips.insert(node);
        }
    }
    let new_tip_count = new_tips.len();
    if new_tip_count == 1 {
        panic!("One tip in subset tree");
    }

    assert!(new_tip_count + parent_count == subset_node_count);

    let mut node_to_path_count = vec![0_u32; node_count as usize];
    for &tip in &new_tips {
        let path = tree_get_path_to_root(input_tree, tip);
        let n = path.len();
        assert!(path[0] == tip);
        for &node2 in path.iter().take(n) {
            node_to_path_count[node2 as usize] += 1;
        }
    }

    let mut old_node_to_new_parent = vec![uint::MAX; node_count as usize];
    let mut pending = std::collections::BTreeSet::<uint>::new();
    let mut done = std::collections::BTreeSet::<uint>::new();
    for &tip in &new_tips {
        pending.insert(tip);
    }

    loop {
        if pending.is_empty() {
            break;
        }
        let node = *pending.iter().next().unwrap();
        assert!(!done.contains(&node));
        done.insert(node);
        pending.remove(&node);
        if input_tree.rooted && input_tree.root_node_index == node {
            continue;
        }

        let path = tree_get_path_to_root(input_tree, node);
        assert!(path[0] == node);
        let n = path.len();
        let mut new_parent = uint::MAX;
        for &node2 in path.iter().take(n).skip(1) {
            let left = input_tree.neighbor2[node2 as usize];
            let right = input_tree.neighbor3[node2 as usize];
            assert!(left != uint::MAX);
            assert!(right != uint::MAX);
            let left_path_count = node_to_path_count[left as usize];
            let right_path_count = node_to_path_count[right as usize];
            if left_path_count > 0 && right_path_count > 0 {
                new_parent = node2;
                break;
            }
        }
        assert!(old_node_to_new_parent[node as usize] == uint::MAX);
        old_node_to_new_parent[node as usize] = new_parent;
        if new_parent != uint::MAX && !done.contains(&new_parent) {
            pending.insert(new_parent);
        }
    }

    let mut old_node_to_new_node = vec![uint::MAX; node_count as usize];
    let mut new_node_to_old_node = Vec::new();
    for &old_node in &done {
        assert!((old_node as usize) < old_node_to_new_node.len());
        let new_node = new_node_to_old_node.len() as uint;

        old_node_to_new_node[old_node as usize] = new_node;
        new_node_to_old_node.push(old_node);
    }

    let mut new_parents = Vec::new();
    let mut new_lengths = Vec::new();
    let mut new_labels = Vec::new();
    for &old_node in &done {
        assert!((old_node as usize) < old_node_to_new_node.len());
        assert!((old_node as usize) < old_node_to_new_parent.len());
        let parent = old_node_to_new_parent[old_node as usize];
        let mut new_parent = uint::MAX;
        if parent != uint::MAX {
            new_parent = old_node_to_new_node[parent as usize];
        }

        let label = if let Some(label) = old_node_to_new_label.get(&old_node) {
            label.clone()
        } else {
            input_tree.names[old_node as usize]
                .clone()
                .unwrap_or_default()
        };

        let distance = if parent == uint::MAX {
            0.0
        } else {
            tree_get_distance(input_tree, old_node, parent) as f32
        };

        new_parents.push(new_parent);
        new_labels.push(label);
        new_lengths.push(distance);
    }

    let mut subset_tree = Tree::default();
    tree_from_vectors(&mut subset_tree, &new_labels, &new_parents, &new_lengths);
    subset_tree
}

/// CLI entry: write a subset-nodes tree extracted from `input_file_name` to `output_file_name`.
#[track_caller]
pub fn cmd_tree_subset_nodes(
    input_file_name: &str,
    nodes_file_name: &str,
    output_file_name: &str,
    right: bool,
) {
    let (nodes, new_labels) = ints_from_file(nodes_file_name);

    let mut t = Tree::default();
    tree_from_file_l143(&mut t, input_file_name);

    let subtree = make_subset_nodes(&t, &nodes, &new_labels);
    tree_ladderize(&mut t, right);
    tree_to_file_l13(&subtree, output_file_name);
}
