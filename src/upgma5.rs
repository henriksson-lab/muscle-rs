// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct UPGMA5 {
    pub leaf_count: uint,
    pub triangle_size: uint,
    pub internal_node_count: uint,
    pub internal_node_index: uint,
    pub dist: Vec<f32>,
    pub min_dist: Vec<f32>,
    pub nearest_neighbor: Vec<uint>,
    pub node_index: Vec<uint>,
    pub left: Vec<uint>,
    pub right: Vec<uint>,
    pub height: Vec<f32>,
    pub left_length: Vec<f32>,
    pub right_length: Vec<f32>,
    pub labels: Vec<String>,
    pub dist_mx: Vec<Vec<f32>>,
    pub label_to_index: std::collections::BTreeMap<String, uint>,
} // original: UPGMA5 (muscle/src/upgma5.h)

/// Return the arithmetic mean of `x` and `y`.
#[track_caller]
pub fn avg(x: f32, y: f32) -> f32 {
    (x + y) / 2.0
}

fn upgma5_format_g(d: f32, precision: usize) -> String {
    if d == 0.0 {
        return "0".to_string();
    }
    if !d.is_finite() {
        return d.to_string();
    }
    let d64 = f64::from(d);
    let exp = d64.abs().log10().floor() as i32;
    let mut s = if exp < -4 || exp >= precision as i32 {
        let raw = format!("{d64:.prec$e}", prec = precision - 1);
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
        let decimals = (precision as i32 - 1 - exp).max(0) as usize;
        format!("{d64:.decimals$}")
    };
    if !s.contains('e') && !s.contains('E') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Render the current distance matrix, nearest-neighbor table, and join history for logging.
#[track_caller]
pub fn upgma5_log_me(u: &UPGMA5) -> String {
    let triangle_subscript = |index1: uint, index2: uint| -> uint {
        let v = if index1 >= index2 {
            index2 + (index1 * (index1 - 1)) / 2
        } else {
            index1 + (index2 * (index2 - 1)) / 2
        };
        assert!(v < (u.leaf_count * (u.leaf_count - 1)) / 2);
        v
    };
    let format_g2 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 2 {
            let raw = format!("{d64:.1e}");
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
            let decimals = (1 - exp).max(0) as usize;
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

    let mut out = String::new();
    out.push_str("Dist matrix\n");
    out.push_str("     ");
    for i in 0..u.leaf_count as usize {
        if u.node_index[i] == uint::MAX {
            continue;
        }
        out.push_str(&format!("  {:5}", u.node_index[i]));
    }
    out.push('\n');

    for i in 0..u.leaf_count as usize {
        if u.node_index[i] == uint::MAX {
            continue;
        }
        out.push_str(&format!("{:5}  ", u.node_index[i]));
        for j in 0..u.leaf_count as usize {
            if u.node_index[j] == uint::MAX {
                continue;
            }
            if i == j {
                out.push_str("       ");
            } else {
                let v = triangle_subscript(i as uint, j as uint);
                out.push_str(&format!("{:>5}  ", format_g2(u.dist[v as usize])));
            }
        }
        out.push_str(&format!("  {}", u.labels[i]));
        out.push('\n');
    }

    out.push('\n');
    out.push_str("    i   Node   NrNb      Dist\n");
    out.push_str("-----  -----  -----  --------\n");
    for i in 0..u.leaf_count as usize {
        if u.node_index[i] == uint::MAX {
            continue;
        }
        out.push_str(&format!(
            "{:5}  {:5}  {:5}  {:8.3}\n",
            i, u.node_index[i], u.nearest_neighbor[i], u.min_dist[i]
        ));
    }

    out.push('\n');
    out.push_str(" Node      L      R  Height  LLength  RLength\n");
    out.push_str("-----  -----  -----  ------  -------  -------\n");
    for i in 0..=u.internal_node_index as usize {
        out.push_str(&format!(
            "{:5}  {:5}  {:5}  {:>6}  {:>6}  {:>6}\n",
            i,
            u.left[i],
            u.right[i],
            format_g2(u.height[i]),
            format_g2(u.left_length[i]),
            format_g2(u.right_length[i])
        ));
    }
    out
}

/// Dispatch UPGMA5 with the requested linkage name (`min`, `max`, `avg`, `biased`).
#[track_caller]
pub fn upgma5_run_l75(u: &mut UPGMA5, linkage: &str, tree: &mut Tree) {
    match linkage {
        "min" | "max" | "avg" | "biased" => upgma5_run_l87(u, linkage, tree),
        _ => die(&format!("UPGMA5::Run(Linkage={linkage})")),
    }
}

/// Core UPGMA5 join loop: repeatedly merge the closest pair under the selected linkage.
#[track_caller]
pub fn upgma5_run_l87(u: &mut UPGMA5, linkage: &str, tree: &mut Tree) {
    u.leaf_count = u.labels.len() as uint;
    assert_eq!(u.dist_mx.len() as uint, u.leaf_count);
    for i in 0..u.leaf_count as usize {
        assert_eq!(u.dist_mx[i].len() as uint, u.leaf_count);
    }
    assert!(u.leaf_count >= 2);

    let triangle_subscript = |leaf_count: uint, index1: uint, index2: uint| -> uint {
        let v = if index1 >= index2 {
            index2 + (index1 * (index1 - 1)) / 2
        } else {
            index1 + (index2 * (index2 - 1)) / 2
        };
        assert!(v < (leaf_count * (leaf_count - 1)) / 2);
        v
    };

    u.triangle_size = (u.leaf_count * (u.leaf_count - 1)) / 2;
    u.internal_node_count = u.leaf_count - 1;
    u.dist = vec![0.0; u.triangle_size as usize];
    u.node_index = vec![uint::MAX; u.leaf_count as usize];
    u.nearest_neighbor = vec![uint::MAX; u.leaf_count as usize];
    u.min_dist = vec![f32::MAX; u.leaf_count as usize];
    let leaf_ids = (0..u.leaf_count).collect::<Vec<_>>();
    let leaf_names = u.labels.clone();
    u.left = vec![uint::MAX; u.internal_node_count as usize];
    u.right = vec![uint::MAX; u.internal_node_count as usize];
    u.height = vec![f32::MAX; u.internal_node_count as usize];
    u.left_length = vec![f32::MAX; u.internal_node_count as usize];
    u.right_length = vec![f32::MAX; u.internal_node_count as usize];

    for i in 0..u.leaf_count as usize {
        u.min_dist[i] = f32::MAX;
        u.node_index[i] = i as uint;
        u.nearest_neighbor[i] = uint::MAX;
    }

    for i in 1..u.leaf_count {
        let mut base = triangle_subscript(u.leaf_count, i, 0);
        for j in 0..i {
            let mut d = u.dist_mx[i as usize][j as usize];
            if d < 0.0 {
                d = 0.0;
                u.dist_mx[i as usize][j as usize] = 0.0;
                u.dist_mx[j as usize][i as usize] = 0.0;
            }
            u.dist[base as usize] = d;
            base += 1;
            if d < u.min_dist[i as usize] {
                u.min_dist[i as usize] = d;
                u.nearest_neighbor[i as usize] = j;
            }
            if d < u.min_dist[j as usize] {
                u.min_dist[j as usize] = d;
                u.nearest_neighbor[j as usize] = i;
            }
        }
        assert!(base <= u.triangle_size);
    }

    let join_count = u.leaf_count - 1;
    for internal_node_index in 0..join_count {
        u.internal_node_index = internal_node_index;
        let _ = progress_step(internal_node_index, join_count, "UPGMA5");
        let mut lmin = uint::MAX;
        let mut rmin = uint::MAX;
        let mut dt_min_dist = f32::MAX;
        for j in 0..u.leaf_count {
            if u.node_index[j as usize] == uint::MAX {
                continue;
            }
            let d = u.min_dist[j as usize];
            if d < dt_min_dist {
                dt_min_dist = d;
                lmin = j;
                rmin = u.nearest_neighbor[j as usize];
                assert_ne!(rmin, uint::MAX);
                assert_ne!(u.node_index[rmin as usize], uint::MAX);
            }
        }
        assert_ne!(lmin, uint::MAX);
        assert_ne!(rmin, uint::MAX);
        assert_ne!(dt_min_dist, f32::MAX);

        let mut dt_new_min_dist = f32::MAX;
        let mut new_nearest_neighbor = uint::MAX;
        for j in 0..u.leaf_count {
            if j == lmin || j == rmin {
                continue;
            }
            if u.node_index[j as usize] == uint::MAX {
                continue;
            }

            let vl = triangle_subscript(u.leaf_count, lmin, j);
            let vr = triangle_subscript(u.leaf_count, rmin, j);
            let dl = u.dist[vl as usize];
            let dr = u.dist[vr as usize];
            let dt_new_dist = match linkage {
                "avg" => avg(dl, dr),
                "min" => dl.min(dr),
                "max" => dl.max(dr),
                "biased" => 0.1 * avg(dl, dr) + (1.0 - 0.1) * dl.min(dr),
                _ => die(&format!("UPGMA5: Invalid LINKAGE_{linkage}")),
            };

            if u.nearest_neighbor[j as usize] == rmin {
                u.nearest_neighbor[j as usize] = lmin;
            }
            u.dist[vl as usize] = dt_new_dist;
            if dt_new_dist < dt_new_min_dist {
                dt_new_min_dist = dt_new_dist;
                new_nearest_neighbor = j;
            }
        }

        let v = triangle_subscript(u.leaf_count, lmin, rmin);
        let dlr = u.dist[v as usize];
        let height_new = dlr / 2.0;
        let left = u.node_index[lmin as usize];
        let right = u.node_index[rmin as usize];
        let height_left = if left < u.leaf_count {
            0.0
        } else {
            u.height[(left - u.leaf_count) as usize]
        };
        let height_right = if right < u.leaf_count {
            0.0
        } else {
            u.height[(right - u.leaf_count) as usize]
        };

        let ini = internal_node_index as usize;
        u.left[ini] = left;
        u.right[ini] = right;
        u.left_length[ini] = height_new - height_left;
        u.right_length[ini] = height_new - height_right;
        u.height[ini] = height_new;

        u.node_index[lmin as usize] = u.leaf_count + internal_node_index;
        u.nearest_neighbor[lmin as usize] = new_nearest_neighbor;
        u.min_dist[lmin as usize] = dt_new_min_dist;
        u.node_index[rmin as usize] = uint::MAX;
    }

    let root = u.leaf_count - 2;
    tree_create(
        tree,
        u.leaf_count,
        root,
        &u.left,
        &u.right,
        &u.left_length,
        &u.right_length,
        &leaf_ids,
        &leaf_names,
    );

    u.internal_node_index = join_count;
    u.dist.clear();
    u.node_index.clear();
    u.nearest_neighbor.clear();
    u.min_dist.clear();
    u.height.clear();
    u.left.clear();
    u.right.clear();
    u.left_length.clear();
    u.right_length.clear();
}

/// Reset labels, distance matrix, and label index.
#[track_caller]
pub fn upgma5_clear(u: &mut UPGMA5) {
    u.labels.clear();
    u.dist_mx.clear();
    u.label_to_index.clear();
}

/// Initialize the UPGMA state from existing `labels` and a full distance matrix.
#[track_caller]
pub fn upgma5_init(u: &mut UPGMA5, labels: &[String], dist_mx: &[Vec<f32>]) {
    upgma5_clear(u);
    u.dist_mx = dist_mx.to_vec();
    u.labels = labels.to_vec();
    for i in 0..labels.len() {
        let label = &labels[i];
        if u.label_to_index.contains_key(label) {
            die(&format!("UPGMA5::Init(), duplicate label >{label}"));
        }
        u.label_to_index.insert(label.clone(), i as uint);
    }
    u.leaf_count = u.labels.len() as uint;
}

/// Register `label`, assigning it the next free index if unseen.
#[track_caller]
pub fn upgma5_add_label(u: &mut UPGMA5, label: &str) {
    assert!(!label.is_empty());
    if u.label_to_index.contains_key(label) {
        return;
    }
    let index = u.labels.len() as uint;
    u.labels.push(label.to_string());
    u.label_to_index.insert(label.to_string(), index);
}

/// Return the numeric index of a previously added label.
#[track_caller]
pub fn upgma5_get_label_index(u: &UPGMA5, label: &str) -> uint {
    *u.label_to_index
        .get(label)
        .expect("UPGMA5::GetLabelIndex label not found")
}

/// Load a tab-separated `label1\tlabel2\tdist` distance matrix from disk.
#[track_caller]
pub fn upgma5_read_dist_mx(u: &mut UPGMA5, file_name: &str) {
    let _ = progress("Reading dist mx...");
    let text = std::fs::read_to_string(file_name).unwrap();
    u.labels.clear();
    u.label_to_index.clear();
    for line in text.lines() {
        let fields = split(line.trim_end_matches('\r'), '\t');
        assert_eq!(fields.len(), 3);
        upgma5_add_label(u, &fields[0]);
        upgma5_add_label(u, &fields[1]);
    }

    u.leaf_count = u.labels.len() as uint;
    u.dist_mx = vec![vec![f32::MAX; u.leaf_count as usize]; u.leaf_count as usize];

    let mut line_nr = 0;
    for line in text.lines() {
        line_nr += 1;
        let fields = split(line.trim_end_matches('\r'), '\t');
        assert_eq!(fields.len(), 3);
        let index1 = upgma5_get_label_index(u, &fields[0]);
        let index2 = upgma5_get_label_index(u, &fields[1]);
        let dist = str_to_float_l1204(&fields[2], false) as f32;
        if index1 == index2 {
            die(&format!(
                "Line {line_nr} Index1={index1} Index2={index2} Label1='{}' Label2='{}'",
                fields[0], fields[1]
            ));
        }
        u.dist_mx[index1 as usize][index2 as usize] = dist;
        u.dist_mx[index2 as usize][index1 as usize] = dist;
    }
    let _ = progress(" done.\n");
}

/// Load a distance matrix in the reseek header/index/triples format.
#[track_caller]
pub fn upgma5_read_dist_mx2(u: &mut UPGMA5, file_name: &str) {
    let _ = progress("Reading dist mx (reseek format)...");
    let text = std::fs::read_to_string(file_name).unwrap();
    let mut lines = text.lines();

    let hdr = lines.next().expect("UPGMA5::ReadDistMx2 missing header");
    let hdr_fields = split(hdr.trim_end_matches('\r'), '\t');
    assert_eq!(hdr_fields.len(), 2);
    assert_eq!(hdr_fields[0], "distmx");
    u.leaf_count = str_to_uint_l1278(&hdr_fields[1], false);
    assert!(u.leaf_count > 2);

    u.labels.clear();
    u.label_to_index.clear();
    for idx in 0..u.leaf_count {
        let line = lines.next().expect("UPGMA5::ReadDistMx2 missing label");
        let fields = split(line.trim_end_matches('\r'), '\t');
        assert_eq!(fields.len(), 2);
        assert_eq!(str_to_uint_l1278(&fields[0], false), idx);
        upgma5_add_label(u, &fields[1]);
    }
    assert_eq!(u.labels.len(), u.leaf_count as usize);

    u.dist_mx = vec![vec![0.0; u.leaf_count as usize]; u.leaf_count as usize];
    let mut dist_count = 0;
    for line in lines {
        let fields = split(line.trim_end_matches('\r'), '\t');
        assert_eq!(fields.len(), 3);
        let index1 = str_to_uint_l1278(&fields[0], false);
        let index2 = str_to_uint_l1278(&fields[1], false);
        assert!(index1 < u.leaf_count);
        assert!(index2 < u.leaf_count);
        if index1 == index2 {
            continue;
        }
        let dist = str_to_float_l1204(&fields[2], false) as f32;
        u.dist_mx[index1 as usize][index2 as usize] = dist;
        u.dist_mx[index2 as usize][index1 as usize] = dist;
        dist_count += 1;
    }
    let _ = progress_log(&format!("{dist_count} pair-wise distances\n"));
    if dist_count < u.leaf_count {
        die("Distance matrix too sparse");
    }
    let _ = progress(" done.\n");
}

/// Convert an expected-accuracy similarity matrix into a distance matrix in place.
#[track_caller]
pub fn upgma5_fix_ea_dist_mx(u: &mut UPGMA5) {
    for i in 0..u.leaf_count as usize {
        u.dist_mx[i][i] = 0.0;
        for j in 0..i {
            let d = u.dist_mx[i][j];
            assert!((0.0..=1.0).contains(&d));
            let new_dist = 1.0 - d;
            u.dist_mx[i][j] = new_dist;
            u.dist_mx[j][i] = new_dist;
        }
    }
}

/// Linearly rescale the distance matrix to `[0, 10]`, flipping orientation if it is a similarity matrix.
#[track_caller]
pub fn upgma5_scale_dist_mx(u: &mut UPGMA5, input_is_similarity: bool) {
    const SCALE: f32 = 10.0;
    let mut min_dist = u.dist_mx[0][1];
    let mut max_dist = u.dist_mx[0][1];
    for i in 0..u.leaf_count as usize {
        for j in 0..i {
            let d = u.dist_mx[i][j];
            assert_eq!(u.dist_mx[j][i], d);
            min_dist = min_dist.min(d);
            max_dist = max_dist.max(d);
        }
    }
    let _ = progress_log(&format!(
        "Re-scaling, min {}, max {}\n",
        upgma5_format_g(min_dist, 4),
        upgma5_format_g(max_dist, 4)
    ));

    let mut min_new_dist = f32::MAX;
    let mut max_new_dist = f32::MAX;
    for i in 0..u.leaf_count as usize {
        for j in 0..i {
            let d = u.dist_mx[i][j];
            let new_dist = if input_is_similarity {
                SCALE * (max_dist - d) / (max_dist - min_dist)
            } else {
                SCALE * (d - min_dist) / (max_dist - min_dist)
            };
            if min_new_dist == f32::MAX || new_dist < min_new_dist {
                min_new_dist = new_dist;
            }
            if max_new_dist == f32::MAX || new_dist > max_new_dist {
                max_new_dist = new_dist;
            }
            u.dist_mx[i][j] = new_dist;
            u.dist_mx[j][i] = new_dist;
        }
    }
    let _ = progress_log(&format!(
        "Scaled min dist {}, max {}. scale\n",
        upgma5_format_g(min_new_dist, 3),
        upgma5_format_g(max_new_dist, 3)
    ));
}

/// CLI entry: read a distance matrix, optionally rescale, build the UPGMA tree, and save it.
#[track_caller]
pub fn cmd_upgma5<FRunUpgma>(
    input_file_name: &str,
    output_file_name: &str,
    reseek: bool,
    scale_dist: bool,
    ea_dist: bool,
    linkage: Option<&str>,
    mut run_upgma: FRunUpgma,
) -> (UPGMA5, Tree, String)
where
    FRunUpgma: FnMut(&mut UPGMA5, &str) -> Tree,
{
    let mut u = UPGMA5::default();
    if reseek {
        upgma5_read_dist_mx2(&mut u, input_file_name);
        // C++ `U.ScaleDistMx()` uses the default `InputIsSimilarity = true`
        // (upgma5.h:58). The `reseek` path treats input as a similarity score.
        upgma5_scale_dist_mx(&mut u, true);
    } else {
        upgma5_read_dist_mx(&mut u, input_file_name);
        if scale_dist {
            // Same default — input is treated as similarity.
            upgma5_scale_dist_mx(&mut u, true);
        } else if ea_dist {
            upgma5_fix_ea_dist_mx(&mut u);
        }
    }

    let s_link = linkage.unwrap_or("avg");
    match s_link {
        "avg" | "min" | "max" | "biased" => {}
        _ => die(&format!("Invalid -linkage {s_link}")),
    }
    let mut log = format!("UPGMA5({s_link})\n");
    let tree = run_upgma(&mut u, s_link);
    tree_to_file_l13(&tree, output_file_name);
    log.push_str("All done.\n");
    (u, tree, log)
}
