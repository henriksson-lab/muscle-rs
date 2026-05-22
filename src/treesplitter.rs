// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct TreeSplitter {
    pub t: Option<Tree>,
    pub target_size: uint,
    pub split_count: uint,
    pub split_index: uint,
    pub subtree_nodes: Vec<uint>,
} // original: TreeSplitter (muscle/src/treesplitter.h)

/// Split a rooted tree into `split_count` subtrees by repeatedly expanding the largest current subtree.
#[track_caller]
pub fn tree_splitter_run(ts: &mut TreeSplitter, t: &Tree, split_count: uint) {
    ts.t = Some(t.clone());
    let tree = ts.t.as_ref().unwrap();
    let node_count = tree.node_count;
    let root = tree.root_node_index;
    ts.target_size = node_count / split_count;
    ts.split_count = split_count;
    if ts.target_size == 0 {
        ts.target_size = 1;
    }
    ts.subtree_nodes.clear();
    ts.subtree_nodes.push(root);
    let mut terminated_early = false;
    for split_index in 1..ts.split_count {
        ts.split_index = split_index;
        assert!(ts.subtree_nodes.len() as uint == ts.split_index);
        let biggest_node = tree_splitter_get_biggest_node(ts);
        let i = biggest_node as usize;
        let neighbor_count = (tree.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (tree.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if tree.node_count == 1 || neighbor_count == 1 {
            terminated_early = true;
            break;
        }
        let left = tree.neighbor2[biggest_node as usize];
        let right = tree.neighbor3[biggest_node as usize];
        assert!(left != uint::MAX);
        assert!(right != uint::MAX);

        let mut new_subtree_nodes = Vec::new();
        for &node in &ts.subtree_nodes {
            if node == biggest_node {
                new_subtree_nodes.push(left);
                new_subtree_nodes.push(right);
            } else {
                new_subtree_nodes.push(node);
            }
        }
        ts.subtree_nodes = new_subtree_nodes;
        let _ = tree_splitter_log_state(ts);
    }
    if !terminated_early {
        assert!(ts.subtree_nodes.len() as uint == ts.split_count);
    }
}

/// Return indexes of current subtrees ordered from largest to smallest leaf count.
/// Mirrors `TreeSplitter::GetSizeOrder` (treesplitter.cpp:63) which uses the
/// unstable `QuickSortOrderDesc`.
#[track_caller]
pub fn tree_splitter_get_size_order(ts: &TreeSplitter) -> Vec<uint> {
    let tree = ts.t.as_ref().expect("TreeSplitter::GetSizeOrder, no tree");
    quick_sort_order_desc_by(ts.subtree_nodes.len(), |a, b| {
        let size_a = tree_get_subtree_leaf_count(tree, ts.subtree_nodes[a]);
        let size_b = tree_get_subtree_leaf_count(tree, ts.subtree_nodes[b]);
        size_a.cmp(&size_b)
    })
}

/// Render a per-split summary (sizes of each current subtree) for logging.
#[track_caller]
pub fn tree_splitter_log_state(ts: &TreeSplitter) -> String {
    let tree = ts.t.as_ref().expect("TreeSplitter::LogState, no tree");
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!(
        "_______________ Split {} ______________\n",
        ts.split_index
    ));
    out.push_str(" Node   Size  LSize  RSize\n");
    let order = tree_splitter_get_size_order(ts);
    let mut sum_size = 0_u32;
    for i in 0..ts.subtree_nodes.len() {
        let k = order[i] as usize;
        let node = ts.subtree_nodes[k];
        let size = tree_get_subtree_leaf_count(tree, node);
        sum_size += size;
        out.push_str(&format!("{node:5}"));
        out.push_str(&format!("  {size:5}"));
        let ni = node as usize;
        let neighbor_count = (tree.neighbor1[ni] != NULL_NEIGHBOR) as uint
            + (tree.neighbor2[ni] != NULL_NEIGHBOR) as uint
            + (tree.neighbor3[ni] != NULL_NEIGHBOR) as uint;
        if !(tree.node_count == 1 || neighbor_count == 1) {
            let left = tree.neighbor2[ni];
            let right = tree.neighbor2[ni];
            let l_size = tree_get_subtree_leaf_count(tree, left);
            let r_size = tree_get_subtree_leaf_count(tree, right);
            out.push_str(&format!("  {l_size:5}  {r_size:5}"));
        }
        out.push('\n');
    }
    out.push_str(&format!("Total {sum_size}\n"));
    out
}

/// Return the current subtree node with the largest leaf count.
#[track_caller]
pub fn tree_splitter_get_biggest_node(ts: &TreeSplitter) -> uint {
    let tree =
        ts.t.as_ref()
            .expect("TreeSplitter::GetBiggestNode, no tree");
    let mut max_size = 0_u32;
    let mut max_node = uint::MAX;
    for &node in &ts.subtree_nodes {
        let size = tree_get_subtree_leaf_count(tree, node);
        if size > max_size {
            max_node = node;
            max_size = size;
        }
    }
    assert!(max_node != uint::MAX);
    max_node
}

/// Collect the leaf labels for each subtree, ordered by descending size.
#[track_caller]
pub fn tree_splitter_get_labels_vec(ts: &TreeSplitter) -> Vec<Vec<String>> {
    let tree = ts.t.as_ref().expect("TreeSplitter::GetLabelsVec, no tree");
    let split_count = ts.subtree_nodes.len();
    let order = tree_splitter_get_size_order(ts);
    assert!(order.len() == split_count);
    let mut labels_vec = Vec::with_capacity(split_count);
    for i in 0..split_count {
        let k = order[i] as usize;
        let node = ts.subtree_nodes[k];
        labels_vec.push(tree_get_subtree_leaf_labels(tree, node));
    }
    labels_vec
}

/// Write each subtree's leaf labels to `<prefix><i>` files; returns the file names.
#[track_caller]
pub fn tree_splitter_write_labels(ts: &TreeSplitter, file_name_prefix: &str) -> Vec<String> {
    if file_name_prefix.is_empty() {
        return Vec::new();
    }
    let labels_vec = tree_splitter_get_labels_vec(ts);
    let mut file_names = Vec::new();
    for (i, labels) in labels_vec.iter().enumerate() {
        let labels_file_name = format!("{file_name_prefix}{}", i + 1);
        let mut text = String::new();
        for label in labels {
            text.push_str(label);
            text.push('\n');
        }
        std::fs::write(&labels_file_name, text).expect("failed to write split labels");
        file_names.push(labels_file_name);
    }
    file_names
}

/// Build a contracted tree whose leaves correspond to the current split subtrees.
#[track_caller]
pub fn tree_splitter_get_subtree(ts: &TreeSplitter) -> (Tree, Vec<String>) {
    let tree = ts.t.as_ref().expect("TreeSplitter::GetSubtree, no tree");
    let size = ts.subtree_nodes.len();
    let mut split_labels = Vec::new();
    for i in 0..size {
        split_labels.push(format!("split{i}"));
    }
    let subtree = make_subset_nodes(tree, &ts.subtree_nodes, &split_labels);
    (subtree, split_labels)
}

/// CLI entry point: load a tree from `tree_file_name`, split into `n` parts, and write label files.
#[track_caller]
pub fn cmd_split_tree(
    tree_file_name: &str,
    n: uint,
    prefix: &str,
    output_file_name: Option<&str>,
) -> Option<Vec<String>> {
    assert!(n > 1);
    let mut t = Tree::default();
    tree_from_file_l143(&mut t, tree_file_name);
    assert!(t.rooted);

    let mut splitter = TreeSplitter::default();
    tree_splitter_run(&mut splitter, &t, n);
    let files = tree_splitter_write_labels(&splitter, prefix);

    if let Some(output_file_name) = output_file_name {
        let (subtree, _sub_labels) = tree_splitter_get_subtree(&splitter);
        tree_to_file_l13(&subtree, output_file_name);
    }
    Some(files)
}
