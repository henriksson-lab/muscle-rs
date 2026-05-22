// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct SimpleCluster {
    pub dist_mx: Vec<Vec<f32>>,
    pub labels: Vec<String>,
    pub linkage: String,
    pub dist_is_similarity: bool,
    pub input_count: uint,
    pub parents: Vec<uint>,
    pub lefts: Vec<uint>,
    pub rights: Vec<uint>,
    pub sizes: Vec<uint>,
    pub lengths: Vec<f32>,
    pub heights: Vec<f32>,
    pub pending: Vec<uint>,
} // original: SimpleCluster (muscle/src/simplecluster.h)

/// Return the (i,j) distance from the cluster matrix, asserting symmetry.
#[track_caller]
pub fn simple_cluster_get_dist(sc: &SimpleCluster, i: uint, j: uint) -> f32 {
    assert!((i as usize) < sc.dist_mx.len());
    assert!((j as usize) < sc.dist_mx[i as usize].len());
    let d = sc.dist_mx[i as usize][j as usize];
    assert!(myfeq(d as f64, sc.dist_mx[j as usize][i as usize] as f64));
    d
}

/// Find the closest pair among pending nodes and return (index1, index2, distance).
#[track_caller]
pub fn simple_cluster_find_closest_pair(sc: &SimpleCluster) -> (uint, uint, f32) {
    let mut index1 = uint::MAX;
    let mut index2 = uint::MAX;
    assert!(!sc.pending.is_empty());
    let mut best_dist = f32::MAX;
    for i_pos in 0..sc.pending.len() {
        let i = sc.pending[i_pos];
        for j_pos in i_pos + 1..sc.pending.len() {
            let j = sc.pending[j_pos];
            let d = simple_cluster_get_dist(sc, i, j);
            if best_dist == f32::MAX {
                index1 = i;
                index2 = j;
                best_dist = d;
            } else {
                let closer = if sc.dist_is_similarity {
                    d > best_dist
                } else {
                    d < best_dist
                };
                if closer {
                    index1 = i;
                    index2 = j;
                    best_dist = d;
                }
            }
        }
    }
    assert!(index1 != uint::MAX && index2 != uint::MAX);
    (index1, index2, best_dist)
}

/// Number of leaves under node `i`.
#[track_caller]
pub fn simple_cluster_get_size(sc: &SimpleCluster, i: uint) -> uint {
    assert!((i as usize) < sc.sizes.len());
    sc.sizes[i as usize]
}

/// Compute the merged-cluster distance from new node (i1,i2) to existing cluster `j` using the chosen linkage.
#[track_caller]
pub fn simple_cluster_calc_new_dist(sc: &SimpleCluster, i1: uint, i2: uint, j: uint) -> f32 {
    assert!((i1 as usize) < sc.dist_mx.len());
    assert!((i2 as usize) < sc.dist_mx.len());
    assert!((j as usize) < sc.dist_mx.len());
    assert!(i1 != i2 && i1 != j && i2 != j);

    let size1 = simple_cluster_get_size(sc, i1);
    let size2 = simple_cluster_get_size(sc, i2);
    assert!(size1 > 0 && size2 > 0);

    let d1 = simple_cluster_get_dist(sc, i1, j);
    let d2 = simple_cluster_get_dist(sc, i2, j);

    if sc.linkage == "avg" {
        (d1 + d2) / 2.0
    } else if sc.linkage == "avgs" {
        (d1 * size1 as f32 + d2 * size2 as f32) / (size1 + size2) as f32
    } else if sc.linkage == "min" {
        d1.min(d2)
    } else if sc.linkage == "max" {
        d1.max(d2)
    } else if sc.linkage == "biased" {
        let b = 0.05_f32;
        b * (d1 + d2) / 2.0 + (1.0 - b) * d1.min(d2)
    } else {
        panic!("SimpleCluster::m_Linkage={}", sc.linkage);
    }
}

/// Perform one agglomerative join: merge the closest pending pair into a new node `JoinIndex`.
#[track_caller]
pub fn simple_cluster_join(sc: &mut SimpleCluster, join_index: uint) {
    assert!(sc.pending.len() > 1);

    let node = sc.input_count + join_index;

    let (index1, index2, _) = simple_cluster_find_closest_pair(sc);

    assert!(sc.pending.contains(&index1));
    assert!(sc.pending.contains(&index2));
    let pending_snapshot = sc.pending.clone();
    for index3 in pending_snapshot {
        if index3 == index1 || index3 == index2 {
            continue;
        }
        let new_dist = simple_cluster_calc_new_dist(sc, index1, index2, index3);
        assert!(sc.dist_mx[node as usize][index3 as usize] == f32::MAX);
        assert!(sc.dist_mx[index3 as usize][node as usize] == f32::MAX);
        sc.dist_mx[node as usize][index3 as usize] = new_dist;
        sc.dist_mx[index3 as usize][node as usize] = new_dist;
    }

    let size1 = simple_cluster_get_size(sc, index1);
    let size2 = simple_cluster_get_size(sc, index2);
    assert!(sc.sizes[node as usize] == 0);
    sc.sizes[node as usize] = size1 + size2;

    sc.parents[index1 as usize] = node;
    sc.parents[index2 as usize] = node;

    sc.lefts[node as usize] = index1;
    sc.rights[node as usize] = index2;

    let d12 = sc.dist_mx[index1 as usize][index2 as usize];
    let height = d12 / 2.0;
    sc.heights[node as usize] = height;

    let height1 = sc.heights[index1 as usize];
    let height2 = sc.heights[index2 as usize];

    sc.lengths[index1 as usize] = height - height1;
    sc.lengths[index2 as usize] = height - height2;

    sc.pending.retain(|x| *x != index1 && *x != index2);
    sc.pending.push(node);
}

/// Run agglomerative clustering on `dist_mx` to build a full binary tree using the named linkage.
#[track_caller]
pub fn simple_cluster_run(
    sc: &mut SimpleCluster,
    dist_mx: &[Vec<f32>],
    labels: &[String],
    linkage: &str,
    dist_is_similarity: bool,
) {
    *sc = SimpleCluster::default();
    sc.dist_mx = dist_mx.to_vec();
    sc.labels = labels.to_vec();
    sc.linkage = linkage.to_string();
    sc.dist_is_similarity = dist_is_similarity;

    sc.input_count = labels.len() as uint;
    let node_count = 2 * sc.input_count - 1;
    let join_count = sc.input_count - 1;

    sc.dist_mx.resize(node_count as usize, Vec::new());
    for node in 0..node_count {
        sc.dist_mx[node as usize].resize(node_count as usize, f32::MAX);
    }

    for join_index in 0..join_count {
        sc.labels.push(format!("Int{join_index}"));
    }

    sc.parents = vec![uint::MAX; node_count as usize];
    sc.lefts = vec![uint::MAX; node_count as usize];
    sc.rights = vec![uint::MAX; node_count as usize];
    sc.sizes = vec![0; node_count as usize];
    sc.lengths = vec![f32::MAX; node_count as usize];
    sc.heights = vec![f32::MAX; node_count as usize];

    for node in 0..sc.input_count {
        sc.pending.push(node);
        sc.sizes[node as usize] = 1;
        sc.heights[node as usize] = 0.0;
    }

    for join_index in 0..join_count {
        simple_cluster_join(sc, join_index);
    }
}

/// Get the label of cluster node `node`.
#[track_caller]
pub fn simple_cluster_get_label(sc: &SimpleCluster, node: uint) -> &str {
    assert!((node as usize) < sc.labels.len());
    &sc.labels[node as usize]
}

/// Render the cluster state (distance matrix, pending list and per-node info) as a log string.
#[track_caller]
pub fn simple_cluster_log_me(sc: &SimpleCluster) -> String {
    let mut s = String::new();
    let format_g3 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d64:.2e}");
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
            format!("{d64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    s.push_str(&format!("{:>8.8}", "DistMx"));
    let node_count = sc.dist_mx.len();
    for node in 0..node_count {
        s.push_str(&format!(
            "  {:>8.8}",
            simple_cluster_get_label(sc, node as uint)
        ));
    }
    s.push('\n');

    for node in 0..node_count {
        let label = simple_cluster_get_label(sc, node as uint);
        s.push_str(&format!("{label:>8.8}"));
        for node2 in 0..node_count {
            if node == node2 {
                s.push_str(&format!("  {:>8.8}", "."));
            } else {
                let d = simple_cluster_get_dist(sc, node as uint, node2 as uint);
                if d == f32::MAX {
                    s.push_str(&format!("  {:>8.8}", "*"));
                } else {
                    s.push_str(&format!("  {:>8}", format_g3(d)));
                }
            }
        }
        s.push('\n');
    }
    s.push_str(&format!("Pending ({}) ", sc.pending.len()));
    for node in &sc.pending {
        s.push_str(&format!(" {node}"));
    }
    s.push('\n');
    s.push_str("  Node    Size  Height  Length  Parent    Left   Right  Label\n");
    for node in 0..node_count {
        let parent = sc.parents[node];
        let left = sc.lefts[node];
        let right = sc.rights[node];
        let height = sc.heights[node];
        let length = sc.lengths[node];
        let label = simple_cluster_get_label(sc, node as uint);
        s.push_str(&format!("{node:6}"));
        s.push_str(&format!("  {:6}", sc.sizes[node]));
        if height == f32::MAX {
            s.push_str(&format!("  {:>6.6}", "*"));
        } else {
            s.push_str(&format!("  {:>6}", format_g3(height)));
        }
        if length == f32::MAX {
            s.push_str(&format!("  {:>6.6}", "*"));
        } else {
            s.push_str(&format!("  {:>6}", format_g3(length)));
        }
        if parent == uint::MAX {
            s.push_str(&format!("  {:>6.6}", "*"));
        } else {
            s.push_str(&format!("  {parent:6}"));
        }
        if left == uint::MAX {
            s.push_str(&format!("  {:>6.6}", "*"));
        } else {
            s.push_str(&format!("  {left:6}"));
        }
        if right == uint::MAX {
            s.push_str(&format!("  {:>6.6}", "*"));
        } else {
            s.push_str(&format!("  {right:6}"));
        }
        s.push_str(&format!("  >{label}\n"));
    }
    s
}

/// Materialize the clustering as a Tree using the labels/parents/lengths vectors.
#[track_caller]
pub fn simple_cluster_get_tree(sc: &SimpleCluster, t: &mut Tree) {
    tree_from_vectors(t, &sc.labels, &sc.parents, &sc.lengths);
}
