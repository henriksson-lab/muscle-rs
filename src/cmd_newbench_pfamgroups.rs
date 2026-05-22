// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns true if `pf` appears exactly once in the underscore-separated PFam
/// domain string `dom_str`.
pub fn contains_pf_exactly_once(dom_str: &str, pf: &str) -> bool {
    let fields = split(dom_str, '_');
    let mut n = 0;
    for field in fields {
        if field == pf {
            n += 1;
        }
    }
    n == 1
}

/// Returns true if any PFam id inside `dom_str` is already in `done_pfs`.
pub fn dom_str_has_done_pf(dom_str: &str, done_pfs: &[String]) -> bool {
    let pfs = split(dom_str, '_');
    for pf in pfs {
        if done_pfs.iter().any(|done| done == &pf) {
            return true;
        }
    }
    false
}

/// Emits FASTA and TSV records for every fresh sequence belonging to `dom_str`,
/// updating `done_labels` so duplicates from later calls are skipped.
#[track_caller]
pub fn output_dom_str(
    dom_str_to_lines: &std::collections::BTreeMap<String, Vec<String>>,
    up_to_seq_index: &std::collections::BTreeMap<String, uint>,
    seqs: &MultiSequence,
    dom_str: &str,
    done_labels: &mut std::collections::BTreeSet<String>,
) -> (String, String) {
    let lines = dom_str_to_lines
        .get(dom_str)
        .unwrap_or_else(|| panic!("OutputDomStr({dom_str})"));
    let mut fa = String::new();
    let mut tsv = String::new();
    let seq_count_this_dom_str = lines.len();
    for line in lines.iter().take(seq_count_this_dom_str) {
        let mut up = String::new();
        for c in line.chars() {
            if c == '\t' {
                break;
            }
            up.push(c);
        }

        if done_labels.contains(&up) {
            continue;
        }
        done_labels.insert(up.clone());

        let seq_index = *up_to_seq_index
            .get(&up)
            .unwrap_or_else(|| panic!("OutputDomStr label not found {up}"));
        let seq = sequence_get_seq_as_string(&seqs.seqs[seq_index as usize]);
        fa.push_str(&seq_to_fasta_l2561(&seq, &up));
        tsv.push_str(&format!("{dom_str}\t{line}\n"));
    }
    (fa, tsv)
}

/// Groups all domain strings that contain `pf` exactly once and have no
/// already-claimed PFams, emitting a `<pf>_local` bundle if at least 10 sequences.
#[track_caller]
pub fn do_dom_strs(
    pf: &str,
    arg_dom_strs: &std::collections::BTreeSet<String>,
    dom_str_to_lines: &std::collections::BTreeMap<String, Vec<String>>,
    up_to_seq_index: &std::collections::BTreeMap<String, uint>,
    seqs: &MultiSequence,
    done_pfs: &mut std::collections::BTreeSet<String>,
) -> Option<(String, String, String)> {
    if done_pfs.contains(pf) {
        return None;
    }

    let mut dom_strs = Vec::new();
    let done_pf_vec: Vec<String> = done_pfs.iter().cloned().collect();
    for dom_str in arg_dom_strs {
        if dom_str_has_done_pf(dom_str, &done_pf_vec) {
            continue;
        }
        if !contains_pf_exactly_once(dom_str, pf) {
            continue;
        }
        dom_strs.push(dom_str.clone());
    }
    if dom_strs.len() < 2 {
        return None;
    }

    let mut size = 0;
    for dom_str in &dom_strs {
        let lines = dom_str_to_lines
            .get(dom_str)
            .unwrap_or_else(|| panic!("DoDomStrs({pf}) missing {dom_str}"));
        size += lines.len();
    }
    if size < 10 {
        return None;
    }

    let name = format!("{pf}_local");
    let mut done_labels = std::collections::BTreeSet::new();
    let mut fa = String::new();
    let mut tsv = String::new();
    for dom_str in &dom_strs {
        for dom_str2 in &dom_strs {
            let _ = dom_str2;
            let (fa_part, tsv_part) = output_dom_str(
                dom_str_to_lines,
                up_to_seq_index,
                seqs,
                dom_str,
                &mut done_labels,
            );
            fa.push_str(&fa_part);
            tsv.push_str(&tsv_part);
        }
    }

    let mut pf_set = std::collections::BTreeSet::new();
    for dom_str in &dom_strs {
        let pfs = split(dom_str, '_');
        for pf2 in pfs {
            pf_set.insert(pf2);
        }
    }

    for pf2 in pf_set {
        assert!(!done_pfs.contains(&pf2));
        done_pfs.insert(pf2);
    }
    Some((name, fa, tsv))
}

/// Emits FASTA and TSV for a single domain string if its line count meets
/// `min_size`; returns `None` otherwise.
#[track_caller]
pub fn do_dom_str(
    dom_str: &str,
    min_size: uint,
    dom_str_to_lines: &std::collections::BTreeMap<String, Vec<String>>,
    up_to_seq_index: &std::collections::BTreeMap<String, uint>,
    seqs: &MultiSequence,
) -> Option<(String, String)> {
    let lines = dom_str_to_lines
        .get(dom_str)
        .unwrap_or_else(|| panic!("DoDomStr({dom_str})"));
    let seq_count_this_dom_str = lines.len() as uint;

    if seq_count_this_dom_str < min_size {
        return None;
    }

    let mut done_labels = std::collections::BTreeSet::new();
    let (fa, tsv) = output_dom_str(
        dom_str_to_lines,
        up_to_seq_index,
        seqs,
        dom_str,
        &mut done_labels,
    );
    Some((fa, tsv))
}

/// Parses a select-PFams TSV line into its underscore-joined domain string and
/// the list of constituent PFam ids.
pub fn parse_line(line: &str) -> (String, Vec<String>) {
    let fields = split(line, '\t');
    let field_count = fields.len() as uint;
    assert!(field_count >= 7);
    let pf_count = str_to_uint_l1278(&fields[3], false);
    assert_eq!(field_count, 3 * pf_count + 4);

    let mut dom_str = String::new();
    let mut pfs = Vec::new();
    for i in 0..pf_count as usize {
        let pf = &fields[4 + 3 * i];
        assert!(pf.starts_with("PF"));
        pfs.push(pf.clone());
        if i > 0 {
            dom_str.push('_');
        }
        dom_str.push_str(pf);
    }
    (dom_str, pfs)
}

/// Drives PFam-group benchmark construction: reads the select-PFams TSV, emits
/// `<PF>_local` bundles first by shared PFam, then leftover domain strings.
#[track_caller]
pub fn cmd_newbench_pfamgroups(
    select_pfams_file_name: &str,
    input_file_name: &str,
    out_dir_fa: &str,
    out_dir_tsv: &str,
    min_size: uint,
) -> std::collections::BTreeMap<String, (String, String)> {
    let mut seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut seqs, input_file_name, true);
    let seq_count = seqs.seqs.len() as uint;
    let mut up_to_seq_index = std::collections::BTreeMap::<String, uint>::new();
    for i in 0..seq_count {
        let up = seqs.seqs[i as usize].label.clone();
        up_to_seq_index.insert(up, i);
    }

    let mut lines = Vec::new();
    let mut pf_to_dom_strs =
        std::collections::BTreeMap::<String, std::collections::BTreeSet<String>>::new();
    let mut dom_str_to_lines = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut max_dom_strs_per_pf = 0usize;
    for line in std::fs::read_to_string(select_pfams_file_name)
        .expect("failed to read newbench PFAM groups input")
        .lines()
    {
        if line.is_empty() {
            continue;
        }
        let line = line.to_string();
        lines.push(line.clone());
        let (dom_str, pfs) = parse_line(&line);
        for pf in pfs {
            let dom_strs = pf_to_dom_strs.entry(pf).or_default();
            dom_strs.insert(dom_str.clone());
            max_dom_strs_per_pf = max_dom_strs_per_pf.max(dom_strs.len());
        }
        dom_str_to_lines.entry(dom_str).or_default().push(line);
    }

    let mut out_dir_fa_s = out_dir_fa.to_string();
    let mut out_dir_tsv_s = out_dir_tsv.to_string();
    dirize(&mut out_dir_fa_s);
    dirize(&mut out_dir_tsv_s);
    std::fs::create_dir_all(&out_dir_fa_s).expect("failed to create FASTA output dir");
    std::fs::create_dir_all(&out_dir_tsv_s).expect("failed to create TSV output dir");

    let mut done_pfs = std::collections::BTreeSet::<String>::new();
    let mut outputs = std::collections::BTreeMap::<String, (String, String)>::new();
    for dom_strs_per_pf in (2..=max_dom_strs_per_pf).rev() {
        for (pf, dom_strs) in &pf_to_dom_strs {
            if dom_strs.len() != dom_strs_per_pf {
                continue;
            }
            if let Some((name, fa, tsv)) = do_dom_strs(
                pf,
                dom_strs,
                &dom_str_to_lines,
                &up_to_seq_index,
                &seqs,
                &mut done_pfs,
            ) {
                std::fs::write(format!("{out_dir_fa_s}{name}"), &fa)
                    .expect("failed to write PFAM group FASTA");
                std::fs::write(format!("{out_dir_tsv_s}{name}"), &tsv)
                    .expect("failed to write PFAM group TSV");
                outputs.insert(name, (fa, tsv));
            }
        }
    }

    for dom_str in dom_str_to_lines.keys() {
        let pfs = split(dom_str, '_');
        let mut done = false;
        let mut pf_set = std::collections::BTreeSet::<String>::new();
        for pf2 in pfs {
            if done_pfs.contains(&pf2) {
                done = true;
                break;
            }
            pf_set.insert(pf2);
        }
        if done {
            continue;
        }

        if let Some((fa, tsv)) = do_dom_str(
            dom_str,
            min_size,
            &dom_str_to_lines,
            &up_to_seq_index,
            &seqs,
        ) {
            std::fs::write(format!("{out_dir_fa_s}{dom_str}"), &fa)
                .expect("failed to write PFAM domstr FASTA");
            std::fs::write(format!("{out_dir_tsv_s}{dom_str}"), &tsv)
                .expect("failed to write PFAM domstr TSV");
            outputs.insert(dom_str.clone(), (fa, tsv));
        }

        for pf2 in pf_set {
            assert!(!done_pfs.contains(&pf2));
            done_pfs.insert(pf2);
        }
    }

    outputs
}
