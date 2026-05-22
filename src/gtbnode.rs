// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct GTBNode {
    pub builder: Option<GTBuilder>,
    pub seq_indexes: Vec<uint>,
    pub seed_seq_indexes: Vec<uint>,
    pub children: Vec<GTBNode>,
    pub upgma: UPGMA5,
    pub tree: Tree,
} // original: GTBNode (muscle/src/gtbnode.h)

pub(crate) static GTB_NODE_DP_MEMS: std::sync::Mutex<Vec<XDPMem>> =
    std::sync::Mutex::new(Vec::new());

/// Returns the per-thread DP scratch buffer, lazily allocating one slot per requested thread.
#[track_caller]
pub fn get_dp_mem_l18() -> XDPMem {
    let mut mems = GTB_NODE_DP_MEMS.lock().unwrap();
    if mems.is_empty() {
        let n = get_requested_thread_count();
        assert!(n > 0);
        for _i in 0..n {
            mems.push(XDPMem::default());
        }
    }
    let thread_index = get_thread_index();
    assert!(thread_index < mems.len() as uint);
    mems[thread_index as usize].clone()
}

/// Returns the label for the node's underlying sequence at the given index.
#[track_caller]
pub fn gtb_node_get_label(node: &GTBNode, seq_index: uint) -> &str {
    let builder = node
        .builder
        .as_ref()
        .expect("GTBNode::GetLabel null builder");
    let seqs = builder.seqs.as_ref().expect("GTBNode::GetLabel null seqs");
    seqs.seqs[seq_index as usize].label.as_str()
}

/// Returns the byte-encoded sequence held by the node at the given index.
#[track_caller]
pub fn gtb_node_get_byte_seq(node: &GTBNode, seq_index: uint) -> Vec<byte> {
    let builder = node
        .builder
        .as_ref()
        .expect("GTBNode::GetByteSeq null builder");
    let seqs = builder
        .seqs
        .as_ref()
        .expect("GTBNode::GetByteSeq null seqs");
    seqs.seqs[seq_index as usize]
        .char_vec
        .iter()
        .map(|&c| c as byte)
        .collect()
}

/// Viterbi-aligns two of the node's sequences and returns their protein distance.
#[track_caller]
pub fn gtb_node_get_prot_dist<FViterbi, FDist>(
    node: &GTBNode,
    seq_indexi: uint,
    seq_indexj: uint,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> f64
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let seqi = gtb_node_get_byte_seq(node, seq_indexi);
    let li = seqi.len() as uint;
    let _labeli = gtb_node_get_label(node, seq_indexi);

    let seqj = gtb_node_get_byte_seq(node, seq_indexj);
    let lj = seqj.len() as uint;
    let _labelj = gtb_node_get_label(node, seq_indexj);

    let pi = viterbi_fast_mem(&seqi, li, &seqj, lj);
    let (row_x, row_y) = make_aln_rows_l45(&seqi, li, &seqj, lj, &pi);

    let col_count = pi.path.len() as uint;
    assert_eq!(row_x.len() as uint, col_count);
    assert_eq!(row_y.len() as uint, col_count);
    get_prot_dist(&row_x, &row_y, col_count)
}

/// Builds an all-pairs UPGMA tree over the node's sequences (used when seq_count < max_all).
#[track_caller]
pub fn gtb_node_do_all<FDist, FRunUPGMA>(
    node: &mut GTBNode,
    mut get_prot_dist: FDist,
    mut run_upgma: FRunUPGMA,
) where
    FDist: FnMut(&GTBNode, uint, uint) -> f64,
    FRunUPGMA: FnMut(&mut UPGMA5, &mut Tree),
{
    let seq_count = node.seq_indexes.len() as uint;
    assert!(seq_count > 0);
    if seq_count < 3 {
        return;
    }
    node.upgma.dist_mx.resize(seq_count as usize, Vec::new());
    node.upgma.labels.clear();
    for i in 0..seq_count {
        let seq_index = node.seq_indexes[i as usize];
        let label = gtb_node_get_label(node, seq_index).to_string();
        node.upgma.labels.push(label);

        node.upgma.dist_mx[i as usize].resize(seq_count as usize, f32::MAX);
        node.upgma.dist_mx[i as usize][i as usize] = 0.0;
    }

    let pair_count = (seq_count * (seq_count - 1)) / 2;
    let mut i = 1usize;
    let mut j = 0usize;
    for _pair_index in 0..pair_count {
        let seq_indexi = node.seq_indexes[i];
        let seq_indexj = node.seq_indexes[j];
        let d = get_prot_dist(node, seq_indexi, seq_indexj) as f32;
        node.upgma.dist_mx[i][j] = d;
        node.upgma.dist_mx[j][i] = d;

        j += 1;
        assert!(j <= i);
        if j == i {
            i += 1;
            j = 0;
        }
    }
    run_upgma(&mut node.upgma, &mut node.tree);
}

/// Picks seed sequences, builds the seed UPGMA tree, partitions remaining sequences to
/// their nearest seed, and recursively runs `child_run` on each child node.
#[track_caller]
pub fn gtb_node_run<FDist, FRunUPGMA, FChildRun>(
    node: &mut GTBNode,
    mut get_prot_dist: FDist,
    mut run_upgma: FRunUPGMA,
    mut child_run: FChildRun,
) where
    FDist: FnMut(&GTBNode, uint, uint) -> f64,
    FRunUPGMA: FnMut(&mut UPGMA5, &mut Tree),
    FChildRun: FnMut(&mut GTBNode),
{
    assert!(node.builder.is_some());
    assert!(!node.seq_indexes.is_empty());
    let seq_count = node.seq_indexes.len() as uint;
    let builder = node.builder.as_ref().expect("GTBNode::Run null builder");
    if seq_count < builder.max_all {
        gtb_node_do_all(node, get_prot_dist, run_upgma);
        return;
    }
    let n = std::cmp::min(builder.target_seed_count, seq_count);
    let mut seed_set = std::collections::BTreeSet::new();
    for i in 0..n + 4 {
        let _r = randu32() % seq_count;
        let seq_index = node.seq_indexes[i as usize];
        seed_set.insert(seq_index);
        if seed_set.len() as uint == n {
            break;
        }
    }
    let seed_count = seed_set.len() as uint;
    for seq_index in &seed_set {
        node.seed_seq_indexes.push(*seq_index);
    }
    assert_eq!(node.seed_seq_indexes.len() as uint, seed_count);

    node.upgma.dist_mx.resize(seed_count as usize, Vec::new());
    node.upgma.labels.clear();
    for i in 0..seed_count {
        node.upgma.dist_mx[i as usize].resize(seed_count as usize, f32::MAX);
        node.upgma.dist_mx[i as usize][i as usize] = 0.0;
    }

    let pair_count = (seed_count * (seed_count - 1)) / 2;
    let mut i = 1usize;
    let mut j = 0usize;
    for _pair_index in 0..pair_count {
        let seq_indexi = node.seed_seq_indexes[i];
        let seq_indexj = node.seed_seq_indexes[j];
        let d = get_prot_dist(node, seq_indexi, seq_indexj) as f32;
        node.upgma.dist_mx[i][j] = d;
        node.upgma.dist_mx[j][i] = d;

        j += 1;
        assert!(j <= i);
        if j == i {
            i += 1;
            j = 0;
        }
    }

    for i in 0..seed_count {
        let seq_index = node.seed_seq_indexes[i as usize];
        let seq_label = gtb_node_get_label(node, seq_index);
        let label = format!("Seed{i}.{seq_label}");
        node.upgma.labels.push(label);
    }

    run_upgma(&mut node.upgma, &mut node.tree);

    node.children.clear();
    node.children
        .resize(seed_count as usize, GTBNode::default());
    for i in 0..seed_count {
        let seed_seq_index = node.seed_seq_indexes[i as usize];
        node.children[i as usize].builder = node.builder.clone();
        node.children[i as usize].seq_indexes.push(seed_seq_index);
    }

    for i in 0..seq_count {
        let seq_index = node.seq_indexes[i as usize];
        if seed_set.contains(&seq_index) {
            continue;
        }
        let mut min_d = f64::MAX;
        let mut best_seed_index = uint::MAX;
        for seed_index in 0..seed_count {
            let seed_seq_index = node.seed_seq_indexes[seed_index as usize];
            let d = get_prot_dist(node, seq_index, seed_seq_index);
            if d < min_d {
                min_d = d;
                best_seed_index = seed_index;
            }
        }
        assert!((best_seed_index as usize) < node.children.len());
        node.children[best_seed_index as usize]
            .seq_indexes
            .push(seq_index);
    }

    for child_index in 0..seed_count {
        child_run(&mut node.children[child_index as usize]);
    }
}
