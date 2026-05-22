// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const NULL_NEIGHBOR: uint = uint::MAX;

pub const MISSING_LENGTH: f64 = f64::MAX;

#[derive(Clone, Debug, Default)]
pub struct Clust; // original: Clust (muscle/src/tree.h)

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NEWICKTOKENTYPE {
    #[default]
    NTT_Unknown,
    NTT_Lparen,
    NTT_Rparen,
    NTT_Colon,
    NTT_Comma,
    NTT_Semicolon,
    NTT_String,
    NTT_SingleQuotedString,
    NTT_DoubleQuotedString,
    NTT_Comment,
} // original: NEWICK_TOKEN_TYPE (muscle/src/tree.h)

#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub node_count: uint,
    pub cache_count: uint,
    pub neighbor1: Vec<uint>,
    pub neighbor2: Vec<uint>,
    pub neighbor3: Vec<uint>,
    pub edge_length1: Vec<f64>,
    pub edge_length2: Vec<f64>,
    pub edge_length3: Vec<f64>,
    pub height: Vec<f64>,
    pub has_edge_length1: Vec<bool>,
    pub has_edge_length2: Vec<bool>,
    pub has_edge_length3: Vec<bool>,
    pub has_height: Vec<bool>,
    pub ids: Vec<uint>,
    pub names: Vec<Option<String>>,
    pub rooted: bool,
    pub root_node_index: uint,
} // original: Tree (muscle/src/tree.h)

#[derive(Clone, Debug, Default)]
pub struct PhyEnumEdgeState {
    pub init: bool,
    pub node_index1: uint,
    pub node_index2: uint,
} // original: PhyEnumEdgeState (muscle/src/tree.h)

/// Allocates the per-node neighbour and edge-length arrays with the given
/// capacity.
pub fn tree_init_cache(t: &mut Tree, cache_count: uint) {
    t.cache_count = cache_count;
    let n = cache_count as usize;
    t.neighbor1 = vec![NULL_NEIGHBOR; n];
    t.neighbor2 = vec![NULL_NEIGHBOR; n];
    t.neighbor3 = vec![NULL_NEIGHBOR; n];
    t.ids = vec![uint::MAX; n];
    t.edge_length1 = vec![f64::MAX; n];
    t.edge_length2 = vec![f64::MAX; n];
    t.edge_length3 = vec![f64::MAX; n];
    t.height = vec![f64::MAX; n];
    t.has_edge_length1 = vec![false; n];
    t.has_edge_length2 = vec![false; n];
    t.has_edge_length3 = vec![false; n];
    t.has_height = vec![false; n];
    t.names = vec![None; n];
}

/// Panics unless `node_index1` and `node_index2` are connected.
pub fn tree_assert_are_neighbors(t: &Tree, node_index1: uint, node_index2: uint) {
    if node_index1 >= t.node_count || node_index2 >= t.node_count {
        panic!(
            "AssertAreNeighbors({node_index1},{node_index2}), are {} nodes",
            t.node_count
        );
    }

    let i1 = node_index1 as usize;
    let i2 = node_index2 as usize;
    if t.neighbor1[i1] != node_index2
        && t.neighbor2[i1] != node_index2
        && t.neighbor3[i1] != node_index2
    {
        panic!("AssertAreNeighbors({node_index1},{node_index2}) failed");
    }
    if t.neighbor1[i2] != node_index1
        && t.neighbor2[i2] != node_index1
        && t.neighbor3[i2] != node_index1
    {
        panic!("AssertAreNeighbors({node_index1},{node_index2}) failed");
    }

    let has12 = tree_has_edge_length(t, node_index1, node_index2);
    let has21 = tree_has_edge_length(t, node_index2, node_index1);
    if has12 != has21 {
        panic!("Tree::AssertAreNeighbors, HasEdgeLength not symmetric");
    }
    if has12 {
        let d12 = tree_get_edge_length(t, node_index1, node_index2);
        let d21 = tree_get_edge_length(t, node_index2, node_index1);
        if d12 != d21 {
            let format_g3 = |d: f64| -> String {
                if d == 0.0 {
                    return "0".to_string();
                }
                if !d.is_finite() {
                    return d.to_string();
                }
                let exp = d.abs().log10().floor() as i32;
                let mut s = if exp < -4 || exp >= 3 {
                    let raw = format!("{d:.2e}");
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
                    let decimals = (2 - exp).max(0) as usize;
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
            panic!(
                "Tree::AssertAreNeighbors, Edge length disagrees {node_index1}-{node_index2}={}, {node_index2}-{node_index1}={}",
                format_g3(d12),
                format_g3(d21)
            );
        }
    }
}

/// Sanity-checks the neighbour/edge invariants for a single tree node.
pub fn tree_validate_node(t: &Tree, node_index: uint) {
    if node_index >= t.node_count {
        panic!("ValidateNode({node_index}), {} nodes", t.node_count);
    }

    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    if neighbor_count == 2 {
        if !t.rooted {
            panic!("Tree::ValidateNode: Node {node_index} has two neighbors, tree is not rooted");
        }
        if node_index != t.root_node_index {
            panic!(
                "Tree::ValidateNode: Node {node_index} has two neighbors, but not root node={}",
                t.root_node_index
            );
        }
    }

    let n1 = t.neighbor1[i];
    let n2 = t.neighbor2[i];
    let n3 = t.neighbor3[i];
    if n2 == NULL_NEIGHBOR && n3 != NULL_NEIGHBOR {
        panic!("Tree::ValidateNode, n2=null, n3!=null");
    }
    if n3 == NULL_NEIGHBOR && n2 != NULL_NEIGHBOR {
        panic!("Tree::ValidateNode, n3=null, n2!=null");
    }

    if n1 != NULL_NEIGHBOR {
        tree_assert_are_neighbors(t, node_index, n1);
    }
    if n2 != NULL_NEIGHBOR {
        tree_assert_are_neighbors(t, node_index, n2);
    }
    if n3 != NULL_NEIGHBOR {
        tree_assert_are_neighbors(t, node_index, n3);
    }

    if n1 != NULL_NEIGHBOR && (n1 == n2 || n1 == n3) {
        panic!("Tree::ValidateNode, duplicate neighbors in node {node_index}");
    }
    if n2 != NULL_NEIGHBOR && (n2 == n1 || n2 == n3) {
        panic!("Tree::ValidateNode, duplicate neighbors in node {node_index}");
    }
    if n3 != NULL_NEIGHBOR && (n3 == n1 || n3 == n2) {
        panic!("Tree::ValidateNode, duplicate neighbors in node {node_index}");
    }

    if t.rooted {
        let parent = t.neighbor1[i];
        if parent == NULL_NEIGHBOR {
            if node_index != t.root_node_index {
                panic!("Tree::ValiateNode({node_index}), no parent");
            }
        } else if t.neighbor2[parent as usize] != node_index
            && t.neighbor3[parent as usize] != node_index
        {
            panic!("Tree::ValidateNode({node_index}), parent / child mismatch");
        }
    }
}

/// Validates every node in the tree.
pub fn tree_validate(t: &Tree) {
    for node_index in 0..t.node_count {
        tree_validate_node(t, node_index);
    }
}

/// Returns the sibling of `node` in a rooted tree, or `UINT_MAX` if none.
pub fn tree_get_sibling(t: &Tree, node: uint) -> uint {
    if node == uint::MAX {
        return uint::MAX;
    }
    assert!(t.rooted && node < t.node_count);
    let parent = t.neighbor1[node as usize];
    if parent == uint::MAX {
        return uint::MAX;
    }
    assert!(t.rooted && parent < t.node_count);
    let parent_left = t.neighbor2[parent as usize];
    let parent_right = t.neighbor3[parent as usize];
    assert!(parent_left != uint::MAX);
    assert!(parent_right != uint::MAX);
    if parent_left == node {
        return parent_right;
    }
    if parent_right == node {
        return parent_left;
    }
    panic!("Tree::GetSibling: node is not a child of its parent");
}

/// Returns true if there is an edge directly connecting the two nodes.
pub fn tree_is_edge(t: &Tree, node_index1: uint, node_index2: uint) -> bool {
    assert!(node_index1 < t.node_count && node_index2 < t.node_count);
    let i = node_index1 as usize;
    t.neighbor1[i] == node_index2 || t.neighbor2[i] == node_index2 || t.neighbor3[i] == node_index2
}

/// Returns the length of the edge connecting the two nodes.
pub fn tree_get_edge_length(t: &Tree, node_index1: uint, node_index2: uint) -> f64 {
    assert!(node_index1 < t.node_count && node_index2 < t.node_count);
    if !tree_has_edge_length(t, node_index1, node_index2) {
        panic!("Missing edge length in tree {node_index1}-{node_index2}");
    }

    let i = node_index1 as usize;
    if t.neighbor1[i] == node_index2 {
        t.edge_length1[i]
    } else if t.neighbor2[i] == node_index2 {
        t.edge_length2[i]
    } else {
        assert!(t.neighbor3[i] == node_index2);
        t.edge_length3[i]
    }
}

/// Doubles the per-node array capacity when adding nodes overflows it.
pub fn tree_expand_cache(t: &mut Tree) {
    let new_cache_count = t.cache_count + 100;
    let new_len = new_cache_count as usize;
    t.neighbor1.resize(new_len, NULL_NEIGHBOR);
    t.neighbor2.resize(new_len, NULL_NEIGHBOR);
    t.neighbor3.resize(new_len, NULL_NEIGHBOR);
    t.ids.resize(new_len, uint::MAX);
    t.edge_length1.resize(new_len, f64::MAX);
    t.edge_length2.resize(new_len, f64::MAX);
    t.edge_length3.resize(new_len, f64::MAX);
    t.height.resize(new_len, f64::MAX);
    t.has_edge_length1.resize(new_len, false);
    t.has_edge_length2.resize(new_len, false);
    t.has_edge_length3.resize(new_len, false);
    t.has_height.resize(new_len, false);
    t.names.resize(new_len, None);
    t.cache_count = new_cache_count;
}

/// Resets `t` to a minimal rooted tree consisting of just a root node.
pub fn tree_create_rooted(t: &mut Tree) {
    *t = Tree::default();
    tree_expand_cache(t);
    t.node_count = 1;
    t.neighbor1[0] = NULL_NEIGHBOR;
    t.neighbor2[0] = NULL_NEIGHBOR;
    t.neighbor3[0] = NULL_NEIGHBOR;
    t.has_edge_length1[0] = false;
    t.has_edge_length2[0] = false;
    t.has_edge_length3[0] = false;
    t.has_height[0] = false;
    t.root_node_index = 0;
    t.rooted = true;
}

/// Resets `t` to a minimal unrooted tree of two leaves joined by an edge
/// of the given length.
pub fn tree_create_unrooted(t: &mut Tree, edge_length: f64) {
    *t = Tree::default();
    tree_expand_cache(t);
    t.node_count = 2;
    t.neighbor1[0] = 1;
    t.neighbor2[0] = NULL_NEIGHBOR;
    t.neighbor3[0] = NULL_NEIGHBOR;
    t.neighbor1[1] = 0;
    t.neighbor2[1] = NULL_NEIGHBOR;
    t.neighbor3[1] = NULL_NEIGHBOR;
    t.edge_length1[0] = edge_length;
    t.edge_length1[1] = edge_length;
    t.has_edge_length1[0] = true;
    t.has_edge_length1[1] = true;
    t.rooted = false;
}

/// Sets the human-readable name of a leaf node.
pub fn tree_set_leaf_name(t: &mut Tree, node_index: uint, name: &str) {
    assert!(node_index < t.node_count);
    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    assert!(t.node_count == 1 || neighbor_count == 1);
    t.names[i] = Some(name.to_string());
}

/// Sets the integer id of a leaf node.
pub fn tree_set_leaf_id(t: &mut Tree, node_index: uint, id: uint) {
    assert!(node_index < t.node_count);
    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    assert!(t.node_count == 1 || neighbor_count == 1);
    t.ids[i] = id;
}

/// Returns the name of a leaf node, or `None` if it has none.
pub fn tree_get_leaf_name(t: &Tree, node_index: uint) -> Option<&str> {
    assert!(node_index < t.node_count);
    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    assert!(t.node_count == 1 || neighbor_count == 1);
    t.names[i].as_deref()
}

/// Returns the integer id assigned to a leaf node.
pub fn tree_get_leaf_id(t: &Tree, node_index: uint) -> uint {
    assert!(node_index < t.node_count);
    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    assert!(t.node_count == 1 || neighbor_count == 1);
    t.ids[i]
}

#[track_caller]
/// Splits an existing leaf into an internal node with two new leaf
/// children and returns the index of the first new leaf.
pub fn tree_append_branch(t: &mut Tree, existing_leaf_index: uint) -> uint {
    if t.node_count == 0 {
        panic!("Tree::AppendBranch: tree has not been created");
    }
    assert!(existing_leaf_index < t.node_count);
    let existing = existing_leaf_index as usize;
    let neighbor_count = (t.neighbor1[existing] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[existing] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[existing] != NULL_NEIGHBOR) as uint;
    if !(t.node_count == 1 || neighbor_count == 1) {
        panic!("AppendBranch({existing_leaf_index}): not leaf");
    }

    if t.node_count >= t.cache_count - 2 {
        tree_expand_cache(t);
    }

    let new_leaf1 = t.node_count;
    let new_leaf2 = t.node_count + 1;
    t.node_count += 2;

    assert!(t.neighbor2[existing] == NULL_NEIGHBOR);
    assert!(t.neighbor3[existing] == NULL_NEIGHBOR);

    t.neighbor2[existing] = new_leaf1;
    t.neighbor3[existing] = new_leaf2;

    let l1 = new_leaf1 as usize;
    let l2 = new_leaf2 as usize;
    t.neighbor1[l1] = existing_leaf_index;
    t.neighbor1[l2] = existing_leaf_index;
    t.neighbor2[l1] = NULL_NEIGHBOR;
    t.neighbor2[l2] = NULL_NEIGHBOR;
    t.neighbor3[l1] = NULL_NEIGHBOR;
    t.neighbor3[l2] = NULL_NEIGHBOR;

    t.edge_length2[existing] = 0.0;
    t.edge_length3[existing] = 0.0;
    t.edge_length1[l1] = 0.0;
    t.edge_length2[l1] = 0.0;
    t.edge_length3[l1] = 0.0;
    t.edge_length1[l2] = 0.0;
    t.edge_length2[l2] = 0.0;
    t.edge_length3[l2] = 0.0;

    t.has_edge_length1[l1] = false;
    t.has_edge_length2[l1] = false;
    t.has_edge_length3[l1] = false;
    t.has_edge_length1[l2] = false;
    t.has_edge_length2[l2] = false;
    t.has_edge_length3[l2] = false;
    t.has_height[l1] = false;
    t.has_height[l2] = false;
    t.ids[l1] = uint::MAX;
    t.ids[l2] = uint::MAX;
    new_leaf1
}

#[track_caller]
/// Returns a debug dump of the tree structure for diagnostics.
pub fn tree_log_me(t: &Tree) -> String {
    let is_leaf = |node_index: usize| -> bool {
        let neighbor_count = (t.neighbor1[node_index] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[node_index] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[node_index] != NULL_NEIGHBOR) as uint;
        t.node_count == 1 || neighbor_count == 1
    };
    let mut out = String::new();
    out.push_str(&format!("Tree::LogMe {} nodes, ", t.node_count));
    if t.rooted {
        out.push_str("rooted.\n");
        out.push('\n');
        out.push_str("Index  Parnt  LengthP  Left   LengthL  Right  LengthR     Id  Name\n");
        out.push_str("-----  -----  -------  ----   -------  -----  -------  -----  ----\n");
    } else {
        out.push_str("unrooted.\n");
        out.push('\n');
        out.push_str("Index  Nbr_1  Length1  Nbr_2  Length2  Nbr_3  Length3     Id  Name\n");
        out.push_str("-----  -----  -------  -----  -------  -----  -------  -----  ----\n");
    }

    for node_index in 0..t.node_count as usize {
        out.push_str(&format!("{node_index:5}  "));
        let ns = [
            t.neighbor1[node_index],
            t.neighbor2[node_index],
            t.neighbor3[node_index],
        ];
        let lengths = [
            t.edge_length1[node_index],
            t.edge_length2[node_index],
            t.edge_length3[node_index],
        ];
        let has_lengths = [
            t.has_edge_length1[node_index],
            t.has_edge_length2[node_index],
            t.has_edge_length3[node_index],
        ];
        for n in 0..3 {
            if ns[n] != NULL_NEIGHBOR {
                out.push_str(&format!("{:5}  ", ns[n]));
                if has_lengths[n] {
                    out.push_str(&format!("{:7.4}  ", lengths[n]));
                } else {
                    out.push_str("      *  ");
                }
            } else {
                out.push_str("                ");
            }
        }

        if is_leaf(node_index) {
            let id = t.ids[node_index];
            if id == uint::MAX {
                out.push_str("    *");
            } else {
                out.push_str(&format!("{id:5}"));
            }
        } else {
            out.push_str("     ");
        }

        if t.rooted && node_index as uint == t.root_node_index {
            out.push_str("  [ROOT] ");
        }
        if let Some(name) = &t.names[node_index] {
            out.push_str("  ");
            out.push_str(name);
        }
        out.push('\n');
    }
    out
}

/// Sets the length of the edge connecting the two nodes (both
/// directions).
pub fn tree_set_edge_length(t: &mut Tree, node_index1: uint, node_index2: uint, length: f64) {
    assert!(node_index1 < t.node_count && node_index2 < t.node_count);
    assert!(tree_is_edge(t, node_index1, node_index2));

    let i1 = node_index1 as usize;
    if t.neighbor1[i1] == node_index2 {
        t.edge_length1[i1] = length;
        t.has_edge_length1[i1] = true;
    } else if t.neighbor2[i1] == node_index2 {
        t.edge_length2[i1] = length;
        t.has_edge_length2[i1] = true;
    } else {
        assert!(t.neighbor3[i1] == node_index2);
        t.edge_length3[i1] = length;
        t.has_edge_length3[i1] = true;
    }

    let i2 = node_index2 as usize;
    if t.neighbor1[i2] == node_index1 {
        t.edge_length1[i2] = length;
        t.has_edge_length1[i2] = true;
    } else if t.neighbor2[i2] == node_index1 {
        t.edge_length2[i2] = length;
        t.has_edge_length2[i2] = true;
    } else {
        assert!(t.neighbor3[i2] == node_index1);
        t.edge_length3[i2] = length;
        t.has_edge_length3[i2] = true;
    }
}

#[track_caller]
/// Removes a root node introduced by file parsing, returning the new
/// pseudo-root index.
pub fn tree_unroot_from_file(t: &mut Tree) -> uint {
    if !t.rooted {
        panic!("Tree::Unroot, not rooted");
    }
    assert!(t.root_node_index == 0);
    assert!(t.neighbor1[0] == NULL_NEIGHBOR);
    if t.node_count >= t.cache_count {
        tree_expand_cache(t);
    }

    let third_node = t.node_count;
    t.node_count += 1;
    let third = third_node as usize;

    t.neighbor1[0] = third_node;
    t.neighbor1[third] = 0;
    t.neighbor2[third] = NULL_NEIGHBOR;
    t.neighbor3[third] = NULL_NEIGHBOR;
    t.edge_length1[0] = 0.0;
    t.edge_length1[third] = 0.0;
    t.has_edge_length1[third] = true;
    t.rooted = false;
    third_node
}

/// Returns the first neighbour of `node_index` that is not
/// `neighbor_index`.
pub fn tree_get_first_neighbor(t: &Tree, node_index: uint, neighbor_index: uint) -> uint {
    assert!(node_index < t.node_count);
    assert!(neighbor_index < t.node_count);
    assert!(tree_is_edge(t, node_index, neighbor_index));

    for n in 0..3 {
        let neighbor = tree_get_neighbor(t, node_index, n);
        if neighbor != NULL_NEIGHBOR && neighbor_index != neighbor {
            return neighbor;
        }
    }
    NULL_NEIGHBOR
}

/// Returns the second neighbour of `node_index` that is not
/// `neighbor_index`.
pub fn tree_get_second_neighbor(t: &Tree, node_index: uint, neighbor_index: uint) -> uint {
    assert!(node_index < t.node_count);
    assert!(neighbor_index < t.node_count);
    assert!(tree_is_edge(t, node_index, neighbor_index));

    let mut found_one = false;
    for n in 0..3 {
        let neighbor = tree_get_neighbor(t, node_index, n);
        if neighbor != NULL_NEIGHBOR && neighbor_index != neighbor {
            if found_one {
                return neighbor;
            }
            found_one = true;
        }
    }
    NULL_NEIGHBOR
}

/// Counts leaves and accumulates distance from `node_index1` going away
/// from `node_index2`.
pub fn tree_get_leaf_count_unrooted(t: &Tree, node_index1: uint, node_index2: uint) -> (uint, f64) {
    assert!(!t.rooted);

    let i2 = node_index2 as usize;
    let neighbor_count = (t.neighbor1[i2] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i2] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i2] != NULL_NEIGHBOR) as uint;
    if t.node_count == 1 || neighbor_count == 1 {
        return (1, tree_get_edge_length(t, node_index1, node_index2));
    }

    let left = tree_get_first_neighbor(t, node_index2, node_index1);
    let right = tree_get_second_neighbor(t, node_index2, node_index1);
    let (left_count, left_distance) = tree_get_leaf_count_unrooted(t, node_index2, left);
    let (right_count, right_distance) = tree_get_leaf_count_unrooted(t, node_index2, right);
    (left_count + right_count, left_distance + right_distance)
}

/// Returns true if the edge between the two nodes has a defined length.
pub fn tree_has_edge_length(t: &Tree, node_index1: uint, node_index2: uint) -> bool {
    assert!(node_index1 < t.node_count);
    assert!(node_index2 < t.node_count);
    assert!(tree_is_edge(t, node_index1, node_index2));

    let i = node_index1 as usize;
    if t.neighbor1[i] == node_index2 {
        t.has_edge_length1[i]
    } else if t.neighbor2[i] == node_index2 {
        t.has_edge_length2[i]
    } else {
        assert!(t.neighbor3[i] == node_index2);
        t.has_edge_length3[i]
    }
}

/// Reorders neighbour slots so that `parent_node_index` becomes the
/// canonical parent of `node_index`.
pub fn tree_orient_parent(t: &mut Tree, node_index: uint, parent_node_index: uint) {
    if node_index == NULL_NEIGHBOR {
        return;
    }

    let i = node_index as usize;
    if t.neighbor1[i] == parent_node_index {
    } else if t.neighbor2[i] == parent_node_index {
        let edge_length2 = t.edge_length2[i];
        t.neighbor2[i] = t.neighbor1[i];
        t.edge_length2[i] = t.edge_length1[i];
        t.neighbor1[i] = parent_node_index;
        t.edge_length1[i] = edge_length2;
    } else {
        assert!(t.neighbor3[i] == parent_node_index);
        let edge_length3 = t.edge_length3[i];
        t.neighbor3[i] = t.neighbor1[i];
        t.edge_length3[i] = t.edge_length1[i];
        t.neighbor1[i] = parent_node_index;
        t.edge_length1[i] = edge_length3;
    }

    let child1 = t.neighbor2[i];
    let child2 = t.neighbor3[i];
    tree_orient_parent(t, child1, node_index);
    tree_orient_parent(t, child2, node_index);
}

/// Returns the first node in left-first depth-first traversal order.
pub fn tree_first_depth_first_node(t: &Tree) -> uint {
    assert!(t.rooted);
    let mut node_index = t.root_node_index;
    loop {
        let i = node_index as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            return node_index;
        }
        assert!(t.rooted && node_index < t.node_count);
        node_index = t.neighbor2[i];
    }
}

/// Returns the first node in right-first depth-first traversal order.
pub fn tree_first_depth_first_node_r(t: &Tree) -> uint {
    assert!(t.rooted);
    let mut node_index = t.root_node_index;
    loop {
        let i = node_index as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            return node_index;
        }
        assert!(t.rooted && node_index < t.node_count);
        node_index = t.neighbor3[i];
    }
}

/// Returns the next node after `node_index` in left-first DFS order.
pub fn tree_next_depth_first_node(t: &Tree, node_index: uint) -> uint {
    assert!(t.rooted);
    assert!(node_index < t.node_count);

    if t.rooted && t.root_node_index == node_index {
        return NULL_NEIGHBOR;
    }

    let parent = t.neighbor1[node_index as usize];
    assert!(t.rooted && parent < t.node_count);
    if t.neighbor3[parent as usize] == node_index {
        return parent;
    }

    let mut node_index = t.neighbor3[parent as usize];
    loop {
        let i = node_index as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            return node_index;
        }
        node_index = t.neighbor2[i];
    }
}

/// Returns the next node after `node_index` in right-first DFS order.
pub fn tree_next_depth_first_node_r(t: &Tree, node_index: uint) -> uint {
    assert!(t.rooted);
    assert!(node_index < t.node_count);

    if t.rooted && t.root_node_index == node_index {
        return NULL_NEIGHBOR;
    }

    let parent = t.neighbor1[node_index as usize];
    assert!(t.rooted && parent < t.node_count);
    if t.neighbor2[parent as usize] == node_index {
        return parent;
    }

    let mut node_index = t.neighbor2[parent as usize];
    loop {
        let i = node_index as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            return node_index;
        }
        node_index = t.neighbor3[i];
    }
}

/// Returns the lexicographically greatest string in `v`.
pub fn get_max_string(v: &[String]) -> String {
    assert!(!v.is_empty());
    let mut max_str = v[0].clone();
    for s in &v[1..] {
        if s > &max_str {
            max_str = s.clone();
        }
    }
    max_str
}

/// Returns true if `labels1`'s maximum string sorts before `labels2`'s.
pub fn compare_labels(labels1: &[String], labels2: &[String]) -> bool {
    let max1 = get_max_string(labels1);
    let max2 = get_max_string(labels2);
    max1 > max2
}

/// Sorts each internal node's children so that subtrees with larger
/// labels appear on the requested side.
pub fn tree_ladderize(t: &mut Tree, more_right: bool) -> uint {
    let node_count = t.node_count;
    let mut rotated_count = 0;
    for node in 0..node_count {
        let i = node as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            continue;
        }

        let left = t.neighbor2[i];
        let right = t.neighbor3[i];
        let n_left = tree_get_subtree_leaf_count(t, left);
        let n_right = tree_get_subtree_leaf_count(t, right);

        let mut do_rotate = if more_right {
            n_right < n_left
        } else {
            n_left < n_right
        };
        if n_left == n_right {
            let left_labels = tree_get_subtree_leaf_labels(t, left);
            let right_labels = tree_get_subtree_leaf_labels(t, right);
            do_rotate = compare_labels(&left_labels, &right_labels);
        }
        if do_rotate {
            rotated_count += 1;
            let left = t.neighbor2[i];
            let right = t.neighbor3[i];
            t.neighbor2[i] = right;
            t.neighbor3[i] = left;
        }
    }
    rotated_count
}

/// Converts a rooted tree to unrooted by removing the root node.
pub fn tree_unroot_by_deleting_root(t: &mut Tree) {
    assert!(t.rooted);
    assert!(t.node_count >= 3);

    let root = t.root_node_index;
    let r = root as usize;
    let left = t.neighbor2[r];
    let right = t.neighbor3[r];

    t.neighbor1[left as usize] = right;
    t.neighbor1[right as usize] = left;

    let has_edge_length =
        tree_has_edge_length(t, root, left) && tree_has_edge_length(t, root, right);
    if has_edge_length {
        let edge_length =
            tree_get_edge_length(t, root, left) + tree_get_edge_length(t, root, right);
        t.edge_length1[left as usize] = edge_length;
        t.edge_length1[right as usize] = edge_length;
    }

    t.neighbor1.remove(r);
    t.neighbor2.remove(r);
    t.neighbor3.remove(r);
    t.edge_length1.remove(r);
    t.edge_length2.remove(r);
    t.edge_length3.remove(r);
    t.has_edge_length1.remove(r);
    t.has_edge_length2.remove(r);
    t.has_edge_length3.remove(r);
    t.names.remove(r);

    t.node_count -= 1;
    t.rooted = false;

    for node_index in 0..t.node_count as usize {
        if t.neighbor1[node_index] != NULL_NEIGHBOR && t.neighbor1[node_index] > root {
            t.neighbor1[node_index] -= 1;
        }
        if t.neighbor2[node_index] != NULL_NEIGHBOR && t.neighbor2[node_index] > root {
            t.neighbor2[node_index] -= 1;
        }
        if t.neighbor3[node_index] != NULL_NEIGHBOR && t.neighbor3[node_index] > root {
            t.neighbor3[node_index] -= 1;
        }
    }
}

/// Returns the parent node index of a leaf.
pub fn tree_get_leaf_parent(t: &Tree, node_index: uint) -> uint {
    assert!(node_index < t.node_count);
    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    assert!(t.node_count == 1 || neighbor_count == 1);

    if t.rooted {
        return t.neighbor1[i];
    }
    if t.neighbor1[i] != NULL_NEIGHBOR {
        return t.neighbor1[i];
    }
    if t.neighbor2[i] != NULL_NEIGHBOR {
        return t.neighbor2[i];
    }
    t.neighbor3[i]
}

/// Returns the lowest common ancestor of two nodes.
pub fn tree_get_lca(t: &Tree, node1: uint, node2: uint) -> uint {
    let path1 = tree_get_path_to_root(t, node1);
    let path2 = tree_get_path_to_root(t, node2);
    for anc_node1 in &path1 {
        for anc_node2 in &path2 {
            if anc_node2 == anc_node1 {
                return *anc_node1;
            }
        }
    }
    panic!("Tree::GetLCA: no common ancestor");
}

/// Returns the path of node indices from `node` up to the root.
pub fn tree_get_path_to_root(t: &Tree, mut node: uint) -> Vec<uint> {
    if !t.rooted {
        panic!("GetPathToRoot(), not rooted");
    }
    let node_count = t.node_count;
    let mut path = Vec::new();
    loop {
        assert!(node < node_count);
        path.push(node);
        assert!(path.len() <= node_count as usize);
        if t.rooted && t.root_node_index == node {
            return path;
        }
        node = t.neighbor1[node as usize];
    }
}

/// Sums edge lengths along the path from `node` up to ancestor
/// `anc_node`.
pub fn tree_get_distance(t: &Tree, node: uint, anc_node: uint) -> f64 {
    if node == t.root_node_index && anc_node == uint::MAX {
        return 0.0;
    }

    let path = tree_get_path_to_root(t, node);
    let mut distance = 0.0;
    assert!(path[0] == node);
    for path_node in path {
        if path_node == anc_node {
            return distance;
        }
        if t.has_edge_length1[path_node as usize] {
            distance += t.edge_length1[path_node as usize];
        }
    }
    panic!("GetDistance, not ancestor");
}

/// Returns the labels of all leaves rooted at `node`.
pub fn tree_get_subtree_leaf_labels(t: &Tree, node: uint) -> Vec<String> {
    let leaves = tree_get_subtree_leaf_nodes(t, node);
    let mut labels = Vec::new();
    for leaf_node in leaves {
        labels.push(t.names[leaf_node as usize].clone().unwrap_or_default());
    }
    labels
}

/// Returns the node indices of all leaves rooted at `node`.
pub fn tree_get_subtree_leaf_nodes(t: &Tree, node: uint) -> Vec<uint> {
    let mut leaf_nodes = Vec::new();
    tree_append_leaves(t, node, &mut leaf_nodes);
    leaf_nodes
}

/// Returns the number of leaves in the subtree rooted at `node`.
pub fn tree_get_subtree_leaf_count(t: &Tree, node: uint) -> uint {
    if node == uint::MAX {
        return 0;
    }
    let leaves = tree_get_subtree_leaf_nodes(t, node);
    leaves.len() as uint
}

/// Returns the labels of every leaf in the tree.
pub fn tree_get_leaf_labels(t: &Tree) -> Vec<String> {
    let mut labels = Vec::new();
    for node in 0..t.node_count {
        let i = node as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            labels.push(t.names[i].clone().unwrap_or_default());
        }
    }
    labels
}

/// Appends the node indices of all leaves below `node` to `leaves`.
pub fn tree_append_leaves(t: &Tree, node: uint, leaves: &mut Vec<uint>) {
    let node_count = t.node_count;
    let leaf_count = if t.rooted {
        assert!(t.node_count % 2 == 1);
        (t.node_count + 1) / 2
    } else {
        assert!(t.node_count % 2 == 0);
        (t.node_count + 2) / 2
    };
    assert!(node < node_count);
    assert!((leaves.len() as uint) < leaf_count);

    let i = node as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    if t.node_count == 1 || neighbor_count == 1 {
        leaves.push(node);
    } else {
        let edge2 = t.neighbor2[i];
        let edge3 = t.neighbor3[i];
        tree_append_leaves(t, edge2, leaves);
        tree_append_leaves(t, edge3, leaves);
    }
}

/// Lazily computes (and caches) the height of `node_index` from its
/// deepest descendant leaf.
pub fn tree_get_node_height(t: &mut Tree, node_index: uint) -> f64 {
    if !t.rooted {
        panic!("Tree::GetNodeHeight: undefined unless rooted tree");
    }

    let i = node_index as usize;
    let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
        + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
    if t.node_count == 1 || neighbor_count == 1 {
        return 0.0;
    }

    if t.has_height[i] {
        return t.height[i];
    }

    let left = t.neighbor2[i];
    let right = t.neighbor3[i];
    let mut left_length = tree_get_edge_length(t, node_index, left);
    let mut right_length = tree_get_edge_length(t, node_index, right);
    if left_length < 0.0 {
        left_length = 0.0;
    }
    if right_length < 0.0 {
        right_length = 0.0;
    }

    let left_height = left_length + tree_get_node_height(t, left);
    let right_height = right_length + tree_get_node_height(t, right);
    let height = (left_height + right_height) / 2.0;
    t.has_height[i] = true;
    t.height[i] = height;
    height
}

/// Returns 0/1/2 identifying which neighbour slot connects `node_index`
/// to `neighbor_index`.
pub fn tree_get_neighbor_subscript(t: &Tree, node_index: uint, neighbor_index: uint) -> uint {
    assert!(node_index < t.node_count);
    assert!(neighbor_index < t.node_count);
    let i = node_index as usize;
    if neighbor_index == t.neighbor1[i] {
        return 0;
    }
    if neighbor_index == t.neighbor2[i] {
        return 1;
    }
    if neighbor_index == t.neighbor3[i] {
        return 2;
    }
    NULL_NEIGHBOR
}

/// Returns the neighbour stored in slot `neighbor_subscript` of
/// `node_index`.
pub fn tree_get_neighbor(t: &Tree, node_index: uint, neighbor_subscript: uint) -> uint {
    let i = node_index as usize;
    match neighbor_subscript {
        0 => t.neighbor1[i],
        1 => t.neighbor2[i],
        2 => t.neighbor3[i],
        _ => panic!("Tree::GetNeighbor, sub={neighbor_subscript}"),
    }
}

/// Maps a leaf ordinal to the corresponding node index.
pub fn tree_leaf_index_to_node_index(t: &Tree, leaf_index: uint) -> uint {
    let mut leaf_count = 0;
    for node_index in 0..t.node_count {
        let i = node_index as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            if leaf_count == leaf_index {
                return node_index;
            }
            leaf_count += 1;
        }
    }
    panic!("LeafIndexToNodeIndex: out of range");
}

/// Returns the node index whose label equals `label`, or `UINT_MAX`.
pub fn tree_get_node_index_l1199(t: &Tree, label: &str) -> uint {
    tree_get_node_index_l1204(t, label)
}

/// Returns the node index whose name equals `name`, or `UINT_MAX`.
pub fn tree_get_node_index_l1204(t: &Tree, name: &str) -> uint {
    for node_index in 0..t.node_count {
        if let Some(leaf_name) = &t.names[node_index as usize] {
            if leaf_name == name {
                return node_index;
            }
        }
    }
    panic!("Tree::GetLeafNodeIndex, name not found");
}

/// Deep-copies `src` into `dst`.
pub fn tree_copy(dst: &mut Tree, src: &Tree) {
    let node_count = src.node_count as usize;
    tree_init_cache(dst, src.node_count);
    dst.node_count = src.node_count;
    dst.neighbor1[..node_count].copy_from_slice(&src.neighbor1[..node_count]);
    dst.neighbor2[..node_count].copy_from_slice(&src.neighbor2[..node_count]);
    dst.neighbor3[..node_count].copy_from_slice(&src.neighbor3[..node_count]);
    dst.ids[..node_count].copy_from_slice(&src.ids[..node_count]);
    dst.edge_length1[..node_count].copy_from_slice(&src.edge_length1[..node_count]);
    dst.edge_length2[..node_count].copy_from_slice(&src.edge_length2[..node_count]);
    dst.edge_length3[..node_count].copy_from_slice(&src.edge_length3[..node_count]);
    dst.height[..node_count].copy_from_slice(&src.height[..node_count]);
    dst.has_edge_length1[..node_count].copy_from_slice(&src.has_edge_length1[..node_count]);
    dst.has_edge_length2[..node_count].copy_from_slice(&src.has_edge_length2[..node_count]);
    dst.has_edge_length3[..node_count].copy_from_slice(&src.has_edge_length3[..node_count]);
    dst.has_height[..node_count].copy_from_slice(&src.has_height[..node_count]);
    dst.root_node_index = src.root_node_index;
    dst.rooted = src.rooted;
    dst.names.clear();
    dst.names.resize(node_count, None);
    for i in 0..node_count {
        dst.names[i] = src.names[i].clone();
    }
}

/// Serializes the tree to parallel (labels, parents, lengths) vectors.
pub fn tree_to_vectors(t: &Tree) -> (Vec<String>, Vec<uint>, Vec<f32>) {
    assert!(t.rooted);

    let mut labels = Vec::new();
    let mut parents = Vec::new();
    let mut lengths = Vec::new();
    for node in 0..t.node_count {
        let label = t.names[node as usize].clone().unwrap_or_default();
        let parent = t.neighbor1[node as usize];
        let mut length = 0.0;
        if parent != uint::MAX {
            length = tree_get_edge_length(t, node, parent) as f32;
        }
        labels.push(label);
        parents.push(parent);
        lengths.push(length);
    }
    (labels, parents, lengths)
}

/// Reconstructs the tree from the parallel vectors produced by
/// `tree_to_vectors`.
pub fn tree_from_vectors(t: &mut Tree, labels: &[String], parents: &[uint], lengths: &[f32]) {
    let node_count = labels.len();
    assert!(parents.len() == node_count);
    assert!(lengths.len() == node_count);

    let mut lefts = vec![uint::MAX; node_count];
    let mut rights = vec![uint::MAX; node_count];
    let mut root = uint::MAX;
    for node in 0..node_count {
        let parent = parents[node];
        if parent == uint::MAX {
            assert!(root == uint::MAX);
            root = node as uint;
            continue;
        }
        assert!((parent as usize) < node_count);
        let p = parent as usize;
        if lefts[p] == uint::MAX {
            lefts[p] = node as uint;
        } else if rights[p] == uint::MAX {
            rights[p] = node as uint;
        } else {
            panic!("Tree::FromVectors(), invalid vector topology");
        }
    }
    assert!(root != uint::MAX);

    let mut leaf_nodes = Vec::new();
    let mut int_nodes = Vec::new();
    let mut node_to_leaf_index = vec![uint::MAX; node_count];
    let mut node_to_int_index = vec![uint::MAX; node_count];
    for node in 0..node_count {
        if lefts[node] == uint::MAX {
            assert!(rights[node] == uint::MAX);
            let leaf_index = leaf_nodes.len() as uint;
            node_to_leaf_index[node] = leaf_index;
            leaf_nodes.push(node as uint);
        } else {
            assert!(rights[node] != uint::MAX);
            let int_index = int_nodes.len() as uint;
            node_to_int_index[node] = int_index;
            int_nodes.push(node as uint);
        }
    }

    let leaf_count = leaf_nodes.len();
    let int_count = int_nodes.len();
    assert!(leaf_count == (node_count + 1) / 2);
    assert!(int_count == leaf_count - 1);

    t.node_count = node_count as uint;
    t.cache_count = node_count as uint;
    t.neighbor1 = vec![NULL_NEIGHBOR; node_count];
    t.neighbor2 = vec![NULL_NEIGHBOR; node_count];
    t.neighbor3 = vec![NULL_NEIGHBOR; node_count];
    t.edge_length1 = vec![0.0; node_count];
    t.edge_length2 = vec![0.0; node_count];
    t.edge_length3 = vec![0.0; node_count];
    t.height = vec![0.0; node_count];
    t.has_edge_length1 = vec![false; node_count];
    t.has_edge_length2 = vec![false; node_count];
    t.has_edge_length3 = vec![false; node_count];
    t.has_height = vec![false; node_count];
    t.ids = vec![uint::MAX; node_count];
    t.names = vec![None; node_count];
    t.rooted = false;
    t.root_node_index = uint::MAX;

    for (i, node) in leaf_nodes.iter().enumerate() {
        let node = *node as usize;
        let label = &labels[node];
        assert!(!label.is_empty());
        t.ids[i] = i as uint;
        t.names[i] = Some(label.clone());
    }

    for (i, node) in int_nodes.iter().enumerate() {
        let node = *node as usize;
        let new_node = leaf_count + i;
        let left = lefts[node] as usize;
        let right = rights[node] as usize;

        let left_int_index = node_to_int_index[left];
        let right_int_index = node_to_int_index[right];
        let left_leaf_index = node_to_leaf_index[left];
        let right_leaf_index = node_to_leaf_index[right];

        let new_left_node = if left_int_index == uint::MAX {
            assert!((left_leaf_index as usize) < leaf_count);
            left_leaf_index
        } else {
            assert!((left_int_index as usize) < int_count);
            leaf_count as uint + left_int_index
        };
        let new_right_node = if right_int_index == uint::MAX {
            assert!((right_leaf_index as usize) < leaf_count);
            right_leaf_index
        } else {
            assert!((right_int_index as usize) < int_count);
            leaf_count as uint + right_int_index
        };

        t.neighbor2[new_node] = new_left_node;
        t.neighbor3[new_node] = new_right_node;
        t.neighbor1[new_left_node as usize] = new_node as uint;
        t.neighbor1[new_right_node as usize] = new_node as uint;
        t.has_edge_length1[new_left_node as usize] = false;
        t.has_edge_length1[new_right_node as usize] = false;

        let left_length = lengths[left];
        let right_length = lengths[right];
        t.has_edge_length2[new_node] = false;
        t.has_edge_length3[new_node] = false;
        if (left_length as f64) != MISSING_LENGTH {
            t.has_edge_length2[new_node] = true;
            t.edge_length2[new_node] = left_length as f64;
            t.has_edge_length1[new_left_node as usize] = true;
            t.edge_length1[new_left_node as usize] = left_length as f64;
        }
        if (right_length as f64) != MISSING_LENGTH {
            t.has_edge_length3[new_node] = true;
            t.edge_length3[new_node] = right_length as f64;
            t.has_edge_length1[new_right_node as usize] = true;
            t.edge_length1[new_right_node as usize] = right_length as f64;
        }
    }

    let new_root_int_index = node_to_int_index[root as usize];
    assert!(new_root_int_index != uint::MAX);
    t.rooted = true;
    t.root_node_index = leaf_count as uint + new_root_int_index;
}

/// Builds a rooted tree from MUSCLE's UPGMA-style `Left`, `Right`,
/// `LeftLength`, `RightLength` arrays.
pub fn tree_create(
    t: &mut Tree,
    leaf_count: uint,
    root: uint,
    left: &[uint],
    right: &[uint],
    left_length: &[f32],
    right_length: &[f32],
    leaf_ids: &[uint],
    leaf_names: &[String],
) {
    *t = Tree::default();
    t.node_count = 2 * leaf_count - 1;
    tree_init_cache(t, t.node_count);

    for node_index in 0..leaf_count as usize {
        t.ids[node_index] = leaf_ids[node_index];
        t.names[node_index] = Some(leaf_names[node_index].clone());
    }

    for node_index in leaf_count..t.node_count {
        let i = node_index as usize;
        let v = (node_index - leaf_count) as usize;
        let left_node = left[v];
        let right_node = right[v];
        let left_len = left_length[v] as f64;
        let right_len = right_length[v] as f64;

        t.neighbor2[i] = left_node;
        t.neighbor3[i] = right_node;
        t.has_edge_length2[i] = true;
        t.has_edge_length3[i] = true;
        t.edge_length2[i] = left_len;
        t.edge_length3[i] = right_len;

        t.neighbor1[left_node as usize] = node_index;
        t.neighbor1[right_node as usize] = node_index;
        t.edge_length1[left_node as usize] = left_len;
        t.edge_length1[right_node as usize] = right_len;
        t.has_edge_length1[left_node as usize] = true;
        t.has_edge_length1[right_node as usize] = true;
    }

    t.rooted = true;
    t.root_node_index = root + leaf_count;
}

/// Returns a vector indexed by node giving the size of the subtree
/// rooted there.
pub fn tree_get_subtree_sizes(t: &Tree) -> Vec<uint> {
    assert!(t.rooted);
    let node_count = t.node_count;
    let mut sizes = vec![uint::MAX; node_count as usize];
    let mut node = tree_first_depth_first_node(t);
    loop {
        assert!(node < node_count);
        let i = node as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if t.node_count == 1 || neighbor_count == 1 {
            sizes[i] = 1;
        } else {
            let left = t.neighbor2[i];
            let right = t.neighbor3[i];
            assert!(left < node_count);
            assert!(right < node_count);
            let size_left = sizes[left as usize];
            let size_right = sizes[right as usize];
            assert!(size_left != uint::MAX && size_left > 0);
            assert!(size_right != uint::MAX && size_right > 0);
            assert!(sizes[i] == uint::MAX);
            sizes[i] = size_left + size_right;
        }
        node = tree_next_depth_first_node(t, node);
        if node == uint::MAX {
            break;
        }
    }
    sizes
}
