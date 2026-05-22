// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Return the index of any internal (non-leaf) node, or `NULL_NEIGHBOR` if none.
#[track_caller]
pub fn tree_get_any_non_leaf_node(t: &Tree) -> uint {
    for node_index in 0..t.node_count {
        let i = node_index as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if !(t.node_count == 1 || neighbor_count == 1) {
            return node_index;
        }
    }
    NULL_NEIGHBOR
}

/// Write tree `t` to `file_name` in Newick format.
#[track_caller]
pub fn tree_to_file_l13(t: &Tree, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    let mut file = TextFile::default();
    tree_to_file_l22(t, &mut file);
    std::fs::write(file_name, &file.data).expect("failed to write tree file");
}

/// Serialize tree `t` to `file` in Newick format (rooted or unrooted).
#[track_caller]
pub fn tree_to_file_l22(t: &Tree, file: &mut TextFile) {
    if t.rooted {
        tree_to_file_node_rooted(t, file, t.root_node_index);
        text_file_put_string(file, ";\n");
        return;
    }

    let node_index = tree_get_any_non_leaf_node(t);
    text_file_put_string(file, "(\n");
    tree_to_file_node_unrooted(t, file, t.neighbor1[node_index as usize], node_index);
    text_file_put_string(file, ",\n");
    tree_to_file_node_unrooted(t, file, t.neighbor2[node_index as usize], node_index);
    text_file_put_string(file, ",\n");
    tree_to_file_node_unrooted(t, file, t.neighbor3[node_index as usize], node_index);
    text_file_put_string(file, ");\n");
}

/// Recursively emit Newick syntax for an unrooted-tree node, treating `parent` as the incoming edge.
#[track_caller]
pub fn tree_to_file_node_unrooted(t: &Tree, file: &mut TextFile, node_index: uint, parent: uint) {
    assert!(!t.rooted);
    let format_g = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 6 {
            let raw = format!("{d:.5e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (5 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };

    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    let group = !(t.node_count == 1 || neighbor_count == 1);
    if group {
        text_file_put_string(file, "(\n");
    }

    if t.node_count == 1 || neighbor_count == 1 {
        text_file_put_string(file, t.names[i].as_deref().unwrap_or(""));
    } else {
        tree_to_file_node_unrooted(
            t,
            file,
            tree_get_first_neighbor(t, node_index, parent),
            node_index,
        );
        text_file_put_string(file, ",\n");
        tree_to_file_node_unrooted(
            t,
            file,
            tree_get_second_neighbor(t, node_index, parent),
            node_index,
        );
    }

    if group {
        text_file_put_string(file, ")");
    }

    if tree_has_edge_length(t, node_index, parent) {
        text_file_put_string(
            file,
            &format!(":{}", format_g(tree_get_edge_length(t, node_index, parent))),
        );
    }
    text_file_put_string(file, "\n");
}

/// Recursively emit Newick syntax for a rooted-tree node.
#[track_caller]
pub fn tree_to_file_node_rooted(t: &Tree, file: &mut TextFile, node_index: uint) {
    assert!(t.rooted);
    let format_g = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 6 {
            let raw = format!("{d:.5e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (5 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };

    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    let is_leaf = t.node_count == 1 || neighbor_count == 1;
    let is_root = node_index == t.root_node_index;
    let group = !is_leaf || is_root;
    if group {
        text_file_put_string(file, "(\n");
    }

    if is_leaf {
        text_file_put_string(file, t.names[i].as_deref().unwrap_or(""));
    } else {
        tree_to_file_node_rooted(t, file, t.neighbor2[i]);
        text_file_put_string(file, ",\n");
        tree_to_file_node_rooted(t, file, t.neighbor3[i]);
    }

    if group {
        text_file_put_string(file, ")");
    }

    if !is_root {
        let parent = t.neighbor1[i];
        if tree_has_edge_length(t, node_index, parent) {
            text_file_put_string(
                file,
                &format!(":{}", format_g(tree_get_edge_length(t, node_index, parent))),
            );
        }
    }
    text_file_put_string(file, "\n");
}
