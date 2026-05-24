// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Build a full pairwise distance matrix and label list from an aligned MSA.
#[track_caller]
pub fn make_dist_mx<FDist>(
    aln: &MultiSequence,
    mut get_prot_dist: FDist,
) -> (Vec<Vec<f32>>, Vec<String>)
where
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let seq_count = aln.seqs.len();
    let col_count = multi_sequence_get_col_count(aln);
    let mut dist_mx = vec![vec![f32::MAX; seq_count]; seq_count];
    let mut labels = Vec::with_capacity(seq_count);

    for i in 0..seq_count {
        dist_mx[i][i] = 0.0;
        labels.push(aln.seqs[i].label.clone());
    }

    let pair_count = (seq_count * (seq_count - 1)) / 2;
    let mut seq_indexi = uint::MAX;
    let mut seq_indexj = uint::MAX;
    let mut pair_counter: uint = 0;
    for _pair_index in 0..pair_count {
        if seq_indexi == uint::MAX {
            seq_indexi = 1;
            seq_indexj = 0;
        } else {
            seq_indexj += 1;
            if seq_indexj == seq_indexi {
                seq_indexi += 1;
                seq_indexj = 0;
            }
        }

        let _ = progress_step(pair_counter, pair_count as uint, "Protdists");
        pair_counter += 1;

        let seqi = sequence_get_seq_as_string(&aln.seqs[seq_indexi as usize]);
        let seqj = sequence_get_seq_as_string(&aln.seqs[seq_indexj as usize]);
        let dij = get_prot_dist(&seqi, &seqj, col_count) as f32;
        dist_mx[seq_indexi as usize][seq_indexj as usize] = dij;
        dist_mx[seq_indexj as usize][seq_indexi as usize] = dij;
    }

    (dist_mx, labels)
}

/// CLI entry: compute pairwise distances over an MSA, then run UPGMA5 and save the tree.
#[track_caller]
pub fn cmd_upgma5_msa<FDist, FRunUpgma>(
    input_file_name: &str,
    output_file_name: &str,
    linkage: Option<&str>,
    mut get_prot_dist: FDist,
    mut run_upgma: FRunUpgma,
) -> (UPGMA5, Tree, String)
where
    FDist: FnMut(&str, &str, uint) -> f64,
    FRunUpgma: FnMut(&mut UPGMA5, &str) -> Tree,
{
    let mut aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut aln, input_file_name, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&aln);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let (dist_mx, labels) = make_dist_mx(&aln, |seqi, seqj, col_count| {
        get_prot_dist(seqi, seqj, col_count)
    });
    let mut u = UPGMA5::default();
    upgma5_init(&mut u, &labels, &dist_mx);

    let s_link = linkage.unwrap_or("avg");
    match s_link {
        "avg" | "min" | "max" | "biased" => {}
        _ => die(&format!("Invalid -linkage {s_link}")),
    }
    let mut log = format!("UPGMA5({s_link})\n");
    let _ = progress_log(&log);
    let tree = run_upgma(&mut u, s_link);
    tree_to_file_l13(&tree, output_file_name);
    log.push_str("All done.\n");
    let _ = progress_log("All done.\n");
    (u, tree, log)
}
