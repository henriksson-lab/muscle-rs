// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Asserts that the pair of node-index vectors describes a valid join order.
#[track_caller]
pub fn validate_join_order(indexes1: &[uint], indexes2: &[uint]) {
    let join_count = indexes1.len() as uint;
    assert!(indexes2.len() as uint == join_count);

    let leaf_count = join_count + 1;
    let node_count = 2 * leaf_count - 1;

    let mut pending = std::collections::BTreeSet::new();
    for leaf_index in 0..leaf_count {
        pending.insert(leaf_index);
    }

    let mut used = vec![false; node_count as usize];
    for join_index in 0..join_count {
        let index1 = indexes1[join_index as usize];
        let index2 = indexes2[join_index as usize];
        assert!(index1 != index2);
        assert!(index1 < node_count);
        assert!(index2 < node_count);

        assert!(!used[index1 as usize]);
        assert!(!used[index2 as usize]);
        assert!(pending.contains(&index1));
        assert!(pending.contains(&index2));

        let join_node_index = leaf_count + join_index;
        used[index1 as usize] = true;
        used[index2 as usize] = true;

        pending.remove(&index1);
        pending.remove(&index2);
        pending.insert(join_node_index);
    }
    assert!(pending.len() == 1);
    let mut used_count = 0_u32;
    let mut not_used_count = 0_u32;
    for node_index in 0..node_count {
        if used[node_index as usize] {
            used_count += 1;
        } else {
            not_used_count += 1;
        }
    }
    assert!(used_count == node_count - 1);
    assert!(not_used_count == 1);
}

/// Returns the leaf label whose entry in the map equals `index`.
#[track_caller]
pub fn get_label(label_to_index: &std::collections::HashMap<String, uint>, index: uint) -> String {
    for (label, label_index) in label_to_index {
        if *label_index == index {
            return label.clone();
        }
    }
    panic!("GetLabel({index}) failed");
}

/// Formats the join order as a human-readable table for logging.
#[track_caller]
pub fn log_guide_tree_join_order(
    guide_tree: &Tree,
    label_to_index: &std::collections::HashMap<String, uint>,
    indexes1: &[uint],
    indexes2: &[uint],
) -> String {
    assert!(guide_tree.rooted);
    let node_count = guide_tree.node_count;
    let leaf_count = (node_count + 1) / 2;
    let join_count = leaf_count - 1;
    assert!(indexes1.len() as uint == join_count);
    assert!(indexes2.len() as uint == join_count);

    let mut s = String::new();
    s.push_str("  Join  Index1  Index2\n");
    for join_index in 0..join_count {
        let index1 = indexes1[join_index as usize];
        let index2 = indexes2[join_index as usize];
        s.push_str(&format!("{join_index:6}"));
        s.push_str(&format!("  {index1:6}"));
        s.push_str(&format!("  {index2:6}"));

        s.push_str("  ");
        if index1 < leaf_count {
            s.push_str(&format!(" '{}'", get_label(label_to_index, index1)));
        } else {
            s.push_str(&format!(" Join{}", index1 - leaf_count));
        }
        s.push_str(" +");
        if index2 < leaf_count {
            s.push_str(&format!(" '{}'", get_label(label_to_index, index2)));
        } else {
            s.push_str(&format!(" Join{}", index2 - leaf_count));
        }

        s.push('\n');
    }
    s
}

/// Derives the depth-first node-pair join order from a rooted guide tree.
#[track_caller]
pub fn get_guide_tree_join_order(
    guide_tree: &Tree,
    label_to_index: &std::collections::HashMap<String, uint>,
) -> (Vec<uint>, Vec<uint>) {
    assert!(guide_tree.rooted);

    let mut indexes1 = Vec::new();
    let mut indexes2 = Vec::new();

    let node_count = guide_tree.node_count;
    let leaf_count = (node_count + 1) / 2;
    let mut index_used = vec![false; leaf_count as usize];

    let join_count = leaf_count - 1;
    let mut join_index = leaf_count;
    let mut stack = Vec::new();
    let mut node = tree_first_depth_first_node(guide_tree);
    while node != uint::MAX {
        let neighbor_count = (guide_tree.neighbor1[node as usize] != NULL_NEIGHBOR) as uint
            + (guide_tree.neighbor2[node as usize] != NULL_NEIGHBOR) as uint
            + (guide_tree.neighbor3[node as usize] != NULL_NEIGHBOR) as uint;
        if guide_tree.node_count == 1 || neighbor_count == 1 {
            let label = tree_get_leaf_name(guide_tree, node)
                .unwrap_or_else(|| panic!("Guide tree leaf {node} has no label"));
            let index = *label_to_index
                .get(label)
                .unwrap_or_else(|| panic!("Label not found >{label}"));
            assert!(index < leaf_count);
            assert!(!index_used[index as usize]);
            stack.push(index);
            index_used[index as usize] = true;
        } else {
            assert!(stack.len() >= 2);
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();

            indexes1.push(right);
            indexes2.push(left);

            stack.push(join_index);
            join_index += 1;
        }
        node = tree_next_depth_first_node(guide_tree, node);
    }
    assert!(indexes1.len() as uint == join_count);
    assert!(indexes2.len() as uint == join_count);
    (indexes1, indexes2)
}

/// Rebuilds a guide tree from a join-order pair of index vectors.
#[track_caller]
pub fn make_guide_tree_from_join_order(
    indexes1: &[uint],
    indexes2: &[uint],
    label_to_index: &std::collections::HashMap<String, uint>,
    guide_tree: &mut Tree,
) {
    let join_count = indexes1.len() as uint;
    assert!(indexes2.len() as uint == join_count);
    assert!(join_count > 0);
    let leaf_count = join_count + 1;
    let node_count = leaf_count + join_count;

    let mut leaf_labels = Vec::new();
    for leaf_index in 0..leaf_count {
        leaf_labels.push(get_label(label_to_index, leaf_index));
    }

    let mut lefts = Vec::new();
    let mut rights = Vec::new();
    for join_index in 0..join_count {
        let index1 = indexes1[join_index as usize];
        let index2 = indexes2[join_index as usize];
        lefts.push(index1);
        rights.push(index2);
    }

    let leaf_ids = vec![1; leaf_count as usize];
    let lengths = vec![1.0_f32; node_count as usize];

    tree_create(
        guide_tree,
        leaf_count,
        join_count - 1,
        &lefts,
        &rights,
        &lengths,
        &lengths,
        &leaf_ids,
        &leaf_labels,
    );
}

/// Implements the `guide_tree_join_order` command: reads a tree file and writes the join order.
#[track_caller]
pub fn cmd_guide_tree_join_order(tree_file_name: &str, output_file_name: Option<&str>) -> String {
    let mut guide_tree = Tree::default();
    tree_from_file_l143(&mut guide_tree, tree_file_name);

    let mut label_to_index = std::collections::HashMap::new();
    let node_count = guide_tree.node_count;
    let leaf_count = (node_count + 1) / 2;
    let join_count = leaf_count - 1;

    let mut leaf_index = 0_u32;
    for node in 0..node_count {
        let neighbor_count = (guide_tree.neighbor1[node as usize] != NULL_NEIGHBOR) as uint
            + (guide_tree.neighbor2[node as usize] != NULL_NEIGHBOR) as uint
            + (guide_tree.neighbor3[node as usize] != NULL_NEIGHBOR) as uint;
        if guide_tree.node_count == 1 || neighbor_count == 1 {
            let label = tree_get_leaf_name(&guide_tree, node)
                .unwrap_or_else(|| panic!("Guide tree leaf {node} has no label"))
                .to_string();
            assert!(!label_to_index.contains_key(&label));
            label_to_index.insert(label, leaf_index);
            leaf_index += 1;
        }
    }

    let (indexes1, indexes2) = get_guide_tree_join_order(&guide_tree, &label_to_index);
    let log = log_guide_tree_join_order(&guide_tree, &label_to_index, &indexes1, &indexes2);
    validate_join_order(&indexes1, &indexes2);

    let mut output = String::new();
    if output_file_name.is_some() {
        assert!(guide_tree.rooted);
        assert!(indexes1.len() as uint == join_count);
        assert!(indexes2.len() as uint == join_count);

        for join_index in 0..join_count {
            let index1 = indexes1[join_index as usize];
            let index2 = indexes2[join_index as usize];
            output.push_str(&format!("{join_index}"));
            output.push_str(&format!("\t{index1}"));
            output.push_str(&format!("\t{index2}"));

            if index1 < leaf_count {
                output.push_str(&format!("\tleaf\t{}", get_label(&label_to_index, index1)));
            } else {
                output.push_str(&format!("\tjoin\t{}", index1 - leaf_count));
            }

            if index2 < leaf_count {
                output.push_str(&format!("\tleaf\t{}", get_label(&label_to_index, index2)));
            } else {
                output.push_str(&format!("\tjoin\t{}", index2 - leaf_count));
            }

            output.push('\n');
        }
    }

    if let Some(file_name) = output_file_name {
        std::fs::write(file_name, output.as_bytes()).unwrap();
    }

    log
}
