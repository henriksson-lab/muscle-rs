// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub static NEWBENCH_UNGAPPED_SEQS: std::sync::Mutex<Vec<String>> =
    std::sync::Mutex::new(Vec::new());

#[derive(Clone, Debug, Default)]
pub struct NewbenchSelectPfamsState {
    pub primary_pf: String,
    pub primary_pfix: uint,
    pub annot_row: String,
    pub unique_pfs: Vec<String>,
    pub pf_to_pfix: std::collections::BTreeMap<String, uint>,
    pub pfix_to_regionixs: Vec<Vec<uint>>,
    pub unique_ups: Vec<String>,
    pub up_to_upix: std::collections::BTreeMap<String, uint>,
    pub upix_to_regionixs: Vec<Vec<uint>>,
    pub up_to_sp: std::collections::BTreeMap<String, String>,
    pub ups: Vec<String>,
    pub sps: Vec<String>,
    pub pfs: Vec<String>,
    pub los: Vec<uint>,
    pub his: Vec<uint>,
    pub ls: Vec<uint>,
    pub aln_regionix_vec: Vec<Vec<uint>>,
    pub aln_ups: Vec<String>,
    pub aln_sps: Vec<String>,
    pub pos_to_col_vec: Vec<Vec<uint>>,
    pub pos_to_pfix_vec: Vec<Vec<uint>>,
    pub col_to_pfix_vec: Vec<Vec<uint>>,
    pub aln_pfixs: Vec<uint>,
    pub aln_pfix_to_char: std::collections::BTreeMap<uint, char>,
    pub dom_str_to_count: std::collections::BTreeMap<String, uint>,
    pub dom_strs: Vec<String>,
    pub dom_coverages: Vec<f64>,
    pub unique_dom_strs: Vec<String>,
    pub unique_dom_str_counts: Vec<uint>,
    pub unique_dom_min_coverages: Vec<f64>,
    pub globalness: f64,
    pub before_primary_pfs: Vec<String>,
    pub after_primary_pfs: Vec<String>,
    pub core_count: uint,
    pub core_primary_count: uint,
    pub core_other_count: uint,
}

pub static NEWBENCH_SELECT_PFAMS_STATE: std::sync::LazyLock<
    std::sync::Mutex<NewbenchSelectPfamsState>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(NewbenchSelectPfamsState::default()));

/// Length of the cached ungapped sequence at `seq_index`.
#[track_caller]
pub fn get_ungapped_seq_length(seq_index: uint) -> uint {
    let seqs = NEWBENCH_UNGAPPED_SEQS.lock().unwrap();
    assert!((seq_index as usize) < seqs.len());
    seqs[seq_index as usize].len() as uint
}

/// Look up or (if `new_ok`) assign a numeric index for PFAM family `pf`.
#[track_caller]
pub fn get_p_fix(pf: &str, new_ok: bool) -> uint {
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    if let Some(pfix) = state.pf_to_pfix.get(pf) {
        return *pfix;
    }
    if !new_ok {
        panic!("GetPFix({pf}) not found");
    }

    let pfix = state.unique_pfs.len() as uint;
    state.unique_pfs.push(pf.to_string());
    state.pf_to_pfix.insert(pf.to_string(), pfix);
    assert_eq!(state.pfix_to_regionixs.len(), pfix as usize);
    state.pfix_to_regionixs.push(Vec::new());
    pfix
}

/// Look up or (if `new_ok`) assign a numeric index for UniProt accession `up`.
#[track_caller]
pub fn get_upix(up: &str, new_ok: bool) -> uint {
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    if let Some(upix) = state.up_to_upix.get(up) {
        return *upix;
    }
    if !new_ok {
        panic!("GetUpix({up}) not found");
    }

    let upix = state.unique_ups.len() as uint;
    assert_eq!(state.upix_to_regionixs.len(), upix as usize);
    state.unique_ups.push(up.to_string());
    state.up_to_upix.insert(up.to_string(), upix);
    state.upix_to_regionixs.push(Vec::new());
    upix
}

/// Parse the PFAM regions TSV file and populate global region/UniProt/family tables.
#[track_caller]
pub fn read_pfam_regions(file_name: &str) {
    let contents = std::fs::read_to_string(file_name).expect("failed to read PFAM regions TSV");
    for line in contents.lines() {
        let fields = split(line, '\t');
        assert_eq!(fields.len(), 6);

        let up = &fields[0];
        let sp = &fields[1];
        let pf = &fields[2];
        let lo = str_to_uint_l1278(&fields[3], false);
        let hi = str_to_uint_l1278(&fields[4], false);
        let l = str_to_uint_l1278(&fields[5], false);
        assert!(lo < hi);
        assert!(hi <= l);

        {
            let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
            if let Some(sp2) = state.up_to_sp.get(up) {
                assert_eq!(sp2, sp);
            } else {
                state.up_to_sp.insert(up.clone(), sp.clone());
            }
        }

        get_p_fix(pf, true);
        let upix = get_upix(up, true);
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        let regionix = state.ups.len() as uint;
        state.ups.push(up.clone());
        state.sps.push(sp.clone());
        state.pfs.push(pf.clone());
        state.los.push(lo);
        state.his.push(hi);
        state.ls.push(l);
        assert!((upix as usize) < state.upix_to_regionixs.len());
        state.upix_to_regionixs[upix as usize].push(regionix);
    }
}

/// True if any sequence in `aln` contains more than one copy of the primary PFAM family.
#[track_caller]
pub fn has_primary_pf_repeat(aln: &MultiSequence) -> bool {
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let seq_count = aln.seqs.len();
    assert_eq!(state.aln_regionix_vec.len(), seq_count);
    for seq_index in 0..seq_count {
        let mut primary_count = 0;
        let regionixs = &state.aln_regionix_vec[seq_index];
        for regionix in regionixs {
            assert!((*regionix as usize) < state.pfs.len());
            let pf = &state.pfs[*regionix as usize];
            if pf == &state.primary_pf {
                primary_count += 1;
            }
        }
        assert!(primary_count > 0);
        if primary_count > 1 {
            return true;
        }
    }
    false
}

/// Mean length of the primary PFAM domain across all sequences of `aln`.
#[track_caller]
pub fn get_mean_primary_domain_length(aln: &MultiSequence) -> f64 {
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let seq_count = aln.seqs.len();
    assert_eq!(state.aln_regionix_vec.len(), seq_count);
    let mut sum_primary_length = 0;
    for seq_index in 0..seq_count {
        let regionixs = &state.aln_regionix_vec[seq_index];
        for regionix in regionixs {
            assert!((*regionix as usize) < state.pfs.len());
            let pf = &state.pfs[*regionix as usize];
            if pf == &state.primary_pf {
                let lo = state.los[*regionix as usize];
                let hi = state.his[*regionix as usize];
                assert!(lo < hi);
                sum_primary_length += hi - lo + 1;
            }
        }
    }
    (sum_primary_length / seq_count as uint) as f64
}

/// Build per-sequence region lists, domain-string statistics, and globalness score for `aln`.
#[track_caller]
pub fn set_aln_regions(aln: &MultiSequence) {
    const MIN_DOM_FRACT: f64 = 0.1;
    const MIN_COVERAGE_GLOBAL: f64 = 0.8;
    let seq_count = aln.seqs.len();
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    state.aln_regionix_vec.clear();
    state.aln_pfixs.clear();
    state.aln_pfix_to_char.clear();
    state.dom_str_to_count.clear();
    state.dom_strs.clear();
    state.dom_coverages.clear();
    state.aln_pfix_to_char.insert(uint::MAX, '.');

    let ungapped_seqs = NEWBENCH_UNGAPPED_SEQS.lock().unwrap();
    for seq_index in 0..seq_count {
        let up = aln.seqs[seq_index].label.clone();
        let upix = *state
            .up_to_upix
            .get(&up)
            .unwrap_or_else(|| panic!("GetUpix({up}) not found"));
        let up2 = &state.unique_ups[upix as usize];
        assert_eq!(up2, &up);

        let _sp = state
            .up_to_sp
            .get(&up)
            .unwrap_or_else(|| panic!("Swissprot label not found for {up}"))
            .clone();

        assert!((upix as usize) < state.upix_to_regionixs.len());
        let regionixs = state.upix_to_regionixs[upix as usize].clone();
        state.aln_regionix_vec.push(regionixs.clone());

        let mut dom_str = String::new();
        let n = regionixs.len();
        let mut sum_dom_lens = 0;
        for regionix in regionixs.iter().take(n) {
            assert!((*regionix as usize) < state.ups.len());
            assert!((*regionix as usize) < state.sps.len());
            assert!((*regionix as usize) < state.pfs.len());
            assert!((*regionix as usize) < state.los.len());
            assert!((*regionix as usize) < state.his.len());
            assert!((*regionix as usize) < state.ls.len());

            let up3 = state.ups[*regionix as usize].clone();
            let pf = state.pfs[*regionix as usize].clone();
            let lo = state.los[*regionix as usize];
            let hi = state.his[*regionix as usize];
            let l = state.ls[*regionix as usize];
            let dom_len = hi - lo + 1;
            if dom_len as f64 / l as f64 >= MIN_DOM_FRACT {
                sum_dom_lens += dom_len;
                if !dom_str.is_empty() {
                    dom_str.push('+');
                }
                dom_str.push_str(&pf);
            }
            let pfix = *state
                .pf_to_pfix
                .get(&pf)
                .unwrap_or_else(|| panic!("GetPFix({pf}) not found"));

            if !state.aln_pfix_to_char.contains_key(&pfix) {
                let i = state.aln_pfixs.len() as uint;
                state.aln_pfixs.push(pfix);
                if pfix == state.primary_pfix {
                    state.aln_pfix_to_char.insert(pfix, '@');
                } else {
                    state
                        .aln_pfix_to_char
                        .insert(pfix, (b'A' + i as u8) as char);
                }
            }

            assert_eq!(up3, up);
        }

        if dom_str.is_empty() {
            dom_str = "-".to_string();
        }
        assert!(seq_index < ungapped_seqs.len());
        let l = ungapped_seqs[seq_index].len() as uint;
        if sum_dom_lens >= l {
            sum_dom_lens = l;
        }
        let dom_coverage = sum_dom_lens as f64 / l as f64;
        state.dom_strs.push(dom_str.clone());
        state.dom_coverages.push(dom_coverage);
        *state.dom_str_to_count.entry(dom_str).or_insert(0) += 1;
    }

    assert_eq!(state.dom_strs.len(), seq_count);
    assert_eq!(state.dom_coverages.len(), seq_count);

    state.dom_str_to_count.clear();
    for dom_str in state.dom_strs.clone() {
        *state.dom_str_to_count.entry(dom_str).or_insert(0) += 1;
    }
    state.unique_dom_strs.clear();
    state.unique_dom_str_counts.clear();
    let dom_counts = state
        .dom_str_to_count
        .iter()
        .map(|(dom_str, count)| (dom_str.clone(), *count))
        .collect::<Vec<_>>();
    for (dom_str, count) in dom_counts {
        state.unique_dom_strs.push(dom_str);
        state.unique_dom_str_counts.push(count);
    }
    let n = state.unique_dom_strs.len();
    // C++ uses unstable QuickSortOrderDesc on UniqueDomStrCounts
    // (cmd_newbench_selectpfams.cpp:341); match it for parity.
    let order = quick_sort_order_desc_by(n, |a, b| {
        state.unique_dom_str_counts[a].cmp(&state.unique_dom_str_counts[b])
    })
    .into_iter()
    .map(|v| v as usize)
    .collect::<Vec<_>>();

    state.unique_dom_min_coverages.clear();
    let mut _global_found = false;
    for k in 0..n {
        let i = order[k];
        let dom_str = &state.unique_dom_strs[i];
        let count = state.unique_dom_str_counts[i];
        let mut min_cvg = f64::MAX;
        for seq_index in 0..seq_count {
            if &state.dom_strs[seq_index] == dom_str {
                let dom_cvg = state.dom_coverages[seq_index];
                min_cvg = min_cvg.min(dom_cvg);
            }
        }
        state.unique_dom_min_coverages.push(min_cvg);
        if k == 0 && n == 1 && min_cvg >= MIN_COVERAGE_GLOBAL {
            _global_found = true;
        }
        let _ = count;
    }

    let top = order[0];
    assert!(top < state.unique_dom_strs.len());
    assert!(top < state.unique_dom_str_counts.len());
    assert!(top < state.unique_dom_min_coverages.len());
    let top_count = state.unique_dom_str_counts[top];
    assert!(top_count <= seq_count as uint);
    let top_fract = top_count as f64 / seq_count as f64;
    let top_min_cvg = state.unique_dom_min_coverages[top];
    state.globalness = top_fract * top_min_cvg;
}

/// Return the PFAM families immediately preceding and following the primary domain in sequence `seq_index`.
#[track_caller]
pub fn get_before_after_p_fs(_aln: &MultiSequence, seq_index: uint) -> (String, String) {
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let mut before = ".".to_string();
    let mut after = ".".to_string();
    assert!((seq_index as usize) < state.aln_regionix_vec.len());
    let regionixs = &state.aln_regionix_vec[seq_index as usize];
    let n = regionixs.len();
    for i in 0..n {
        let regionix = regionixs[i];
        assert!((regionix as usize) < state.pfs.len());
        let region_pf = &state.pfs[regionix as usize];
        if region_pf == &state.primary_pf {
            if i > 0 {
                let before_regionix = regionixs[i - 1];
                assert!((before_regionix as usize) < state.pfs.len());
                before = state.pfs[before_regionix as usize].clone();
            }
            if i + 1 < n {
                let after_regionix = regionixs[i + 1];
                assert!((after_regionix as usize) < state.pfs.len());
                after = state.pfs[after_regionix as usize].clone();
            }
        }
    }
    (before, after)
}

/// Record before/after PFAM families for every sequence and return the agreement score (top-before count * top-after count / N^2).
#[track_caller]
pub fn set_before_after_p_fs(aln: &MultiSequence) -> f64 {
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        state.before_primary_pfs.clear();
        state.after_primary_pfs.clear();
    }
    let seq_count = aln.seqs.len();
    for seq_index in 0..seq_count {
        let (before, after) = get_before_after_p_fs(aln, seq_index as uint);
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        state.before_primary_pfs.push(before);
        state.after_primary_pfs.push(after);
    }

    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let mut before_count_map = std::collections::BTreeMap::<String, uint>::new();
    let mut after_count_map = std::collections::BTreeMap::<String, uint>::new();
    for before in &state.before_primary_pfs {
        *before_count_map.entry(before.clone()).or_insert(0) += 1;
    }
    for after in &state.after_primary_pfs {
        *after_count_map.entry(after.clone()).or_insert(0) += 1;
    }
    let before_top_count = before_count_map.values().copied().max().unwrap_or(0);
    let after_top_count = after_count_map.values().copied().max().unwrap_or(0);
    before_top_count as f64 * after_top_count as f64 / (seq_count * seq_count) as f64
}

/// Replace `.pdb` labels with UniProt accessions and record the resulting (up, sp) pairs.
#[track_caller]
pub fn set_aln_labels(aln: &mut MultiSequence) {
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    state.aln_ups.clear();
    state.aln_sps.clear();
    for seq in &mut aln.seqs {
        let label = seq.label.clone();
        assert!(label.ends_with(".pdb"));
        let up = label[..label.len() - 4].to_string();
        let Some(upix) = state.up_to_upix.get(&up).copied() else {
            continue;
        };
        seq.label = up.clone();
        let up2 = &state.unique_ups[upix as usize];
        assert_eq!(up2, &up);
        let sp = state
            .up_to_sp
            .get(&up)
            .unwrap_or_else(|| panic!("Swissprot label not found for {up}"))
            .clone();
        state.aln_ups.push(up);
        state.aln_sps.push(sp);
    }
}

/// Upper-case core columns (those marked in `annot_row`) and lower-case the rest.
#[track_caller]
pub fn set_aln_case(aln: &mut MultiSequence) {
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let col_count = multi_sequence_get_col_count(aln);
    assert_eq!(state.annot_row.len(), col_count as usize);
    for col_index in 0..col_count as usize {
        let has_gap = aln
            .seqs
            .iter()
            .any(|seq| matches!(seq.char_vec[col_index], '-' | '.'));
        let c = state.annot_row.as_bytes()[col_index] as char;
        let is_core_col = c != '~';
        if is_core_col {
            assert!(!has_gap);
        }
        for seq in &mut aln.seqs {
            let mut c = seq.char_vec[col_index];
            if c.is_ascii_alphabetic() {
                c = if is_core_col {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                };
            }
            seq.char_vec[col_index] = c;
        }
    }
}

/// Cache position-to-column maps for every sequence in `aln`.
#[track_caller]
pub fn set_pos_to_col_vec(aln: &MultiSequence) {
    let seq_count = aln.seqs.len();
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    state.pos_to_col_vec.clear();
    state.pos_to_col_vec.resize(seq_count, Vec::new());
    for seq_index in 0..seq_count {
        let row = sequence_get_seq_as_string(&aln.seqs[seq_index]);
        state.pos_to_col_vec[seq_index] = sequence_get_pos_to_col(&row);
    }
}

/// Map each ungapped sequence position to its PFAM family index; returns the count of overlapping positions.
#[track_caller]
pub fn set_pos_to_p_fix_vec(aln: &MultiSequence) -> uint {
    let seq_count = aln.seqs.len();
    let mut overlap_count = 0;
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    state.pos_to_pfix_vec.clear();
    state.pos_to_pfix_vec.resize(seq_count, Vec::new());
    for seq_index in 0..seq_count {
        assert!(seq_index < state.aln_regionix_vec.len());
        let l = get_ungapped_seq_length(seq_index as uint);
        let mut pos_to_pfix = vec![uint::MAX; l as usize];
        let regionixs = &state.aln_regionix_vec[seq_index];
        for regionix in regionixs {
            assert!((*regionix as usize) < state.ups.len());
            assert!((*regionix as usize) < state.sps.len());
            assert!((*regionix as usize) < state.pfs.len());
            assert!((*regionix as usize) < state.los.len());
            assert!((*regionix as usize) < state.his.len());
            assert!((*regionix as usize) < state.ls.len());
            let pf = &state.pfs[*regionix as usize];
            let lo = state.los[*regionix as usize];
            let hi = state.his[*regionix as usize];
            let l2 = state.ls[*regionix as usize];
            assert_eq!(l2, l);
            let pfix = *state
                .pf_to_pfix
                .get(pf)
                .unwrap_or_else(|| panic!("GetPFix({pf}) not found"));
            assert!(lo > 0);
            assert!(lo < hi);
            assert!(hi <= l);
            for pos in lo..=hi {
                let pfix2 = pos_to_pfix[pos as usize - 1];
                if pfix2 != uint::MAX {
                    overlap_count += 1;
                    if pfix2 != state.primary_pfix {
                        pos_to_pfix[pos as usize - 1] = pfix;
                    }
                } else {
                    pos_to_pfix[pos as usize - 1] = pfix;
                }
            }
        }
        state.pos_to_pfix_vec[seq_index] = pos_to_pfix;
    }
    overlap_count
}

/// Build column-to-PFix maps for every sequence and return the per-sequence printable annotation rows.
#[track_caller]
pub fn set_col_to_p_fix_vec(aln: &MultiSequence) -> Vec<String> {
    let seq_count = aln.seqs.len();
    let col_count = multi_sequence_get_col_count(aln);
    let mut rows = Vec::new();
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    state.col_to_pfix_vec.clear();
    state.col_to_pfix_vec.resize(seq_count, Vec::new());
    for seq_index in 0..seq_count {
        assert!(seq_index < state.aln_regionix_vec.len());
        let l = get_ungapped_seq_length(seq_index as uint);
        let pos_to_pfix = &state.pos_to_pfix_vec[seq_index];
        let pos_to_col = &state.pos_to_col_vec[seq_index];
        assert_eq!(pos_to_pfix.len(), l as usize);
        assert_eq!(pos_to_col.len(), l as usize);
        let mut col_to_pfix = vec![uint::MAX; col_count as usize];
        for pos in 0..l as usize {
            let col = pos_to_col[pos];
            assert!(col < col_count);
            let pfix = pos_to_pfix[pos];
            col_to_pfix[col as usize] = pfix;
        }

        let mut row = String::new();
        for pfix in &col_to_pfix {
            let c = state
                .aln_pfix_to_char
                .get(pfix)
                .unwrap_or_else(|| panic!("PFix char not found: {pfix}"));
            row.push(*c);
        }
        state.col_to_pfix_vec[seq_index] = col_to_pfix;
        rows.push(row);
    }
    rows
}

/// Cache the ungapped (dashes and dots stripped) version of each sequence in `aln`.
#[track_caller]
pub fn set_ungapped_seqs(aln: &MultiSequence) {
    let mut seqs = NEWBENCH_UNGAPPED_SEQS.lock().unwrap();
    seqs.clear();
    for seq in &aln.seqs {
        let mut ungapped = String::new();
        for c in &seq.char_vec {
            if *c != '-' && *c != '.' {
                ungapped.push(*c);
            }
        }
        seqs.push(ungapped);
    }
}

/// Return the PFAM index shared by all sequences at column `col`, or `uint::MAX` if not unanimous.
#[track_caller]
pub fn get_consensus_pf(aln: &MultiSequence, col: uint) -> uint {
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let seq_count = aln.seqs.len();
    let col_count = multi_sequence_get_col_count(aln);
    assert!(col < col_count);
    assert_eq!(state.col_to_pfix_vec.len(), seq_count);
    let mut consensus_pfix = uint::MAX;
    for seq_index in 0..seq_count {
        let col_to_pfix = &state.col_to_pfix_vec[seq_index];
        assert_eq!(col_to_pfix.len(), col_count as usize);
        let pfix = col_to_pfix[col as usize];
        if pfix == uint::MAX {
            return uint::MAX;
        }
        if seq_index == 0 {
            consensus_pfix = pfix;
        } else if pfix != consensus_pfix {
            return uint::MAX;
        }
    }
    consensus_pfix
}

/// Compute per-column annotation row: `^` for primary core, `:` for other core, `~` otherwise.
#[track_caller]
pub fn set_col_annots(aln: &MultiSequence) {
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        state.core_count = 0;
        state.core_primary_count = 0;
        state.core_other_count = 0;
        state.annot_row.clear();
    }
    let col_count = multi_sequence_get_col_count(aln);
    for col in 0..col_count {
        let pfix = get_consensus_pf(aln, col);
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        if pfix == uint::MAX {
            state.annot_row.push('~');
            continue;
        }
        if pfix == state.primary_pfix {
            state.core_primary_count += 1;
            state.annot_row.push('^');
        } else {
            state.core_other_count += 1;
            state.annot_row.push(':');
        }
    }
    let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    state.core_count = state.core_primary_count + state.core_other_count;
}

/// Return the first and last column indices where sequence `seq_index` is annotated with the primary PFAM.
#[track_caller]
pub fn get_primary_col_lo_hi(aln: &MultiSequence, seq_index: uint) -> (uint, uint) {
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    assert!((seq_index as usize) < state.col_to_pfix_vec.len());
    let col_to_pfix = &state.col_to_pfix_vec[seq_index as usize];
    let col_count = multi_sequence_get_col_count(aln);
    assert_eq!(col_to_pfix.len(), col_count as usize);
    let mut primary_col_lo = uint::MAX;
    let mut primary_col_hi = uint::MAX;
    for col in 0..col_count {
        let pfix = col_to_pfix[col as usize];
        if pfix == state.primary_pfix {
            if primary_col_lo == uint::MAX {
                primary_col_lo = col;
            }
            primary_col_hi = col;
        }
    }
    assert_ne!(primary_col_lo, uint::MAX);
    assert_ne!(primary_col_hi, uint::MAX);
    assert!(primary_col_lo < primary_col_hi);
    (primary_col_lo, primary_col_hi)
}

/// Mask non-primary flanks of each sequence with `.` and squeeze the resulting inserts.
#[track_caller]
pub fn trim(aln: &MultiSequence) -> MultiSequence {
    let seq_count = aln.seqs.len();
    let col_count = multi_sequence_get_col_count(aln);
    let mut tmp_aln = aln.clone();
    assert_eq!(tmp_aln.seqs.len(), seq_count);
    assert_eq!(multi_sequence_get_col_count(&tmp_aln), col_count);

    for seq_index in 0..seq_count {
        let (primary_col_lo, primary_col_hi) = get_primary_col_lo_hi(aln, seq_index as uint);
        for col in 0..primary_col_lo as usize {
            tmp_aln.seqs[seq_index].char_vec[col] = '.';
        }
        for col in primary_col_hi as usize + 1..col_count as usize {
            tmp_aln.seqs[seq_index].char_vec[col] = '.';
        }
    }
    squeeze_inserts(&tmp_aln)
}

/// Stub for annotated coverage of a UniProt region set (C++ original returns 0.0).
#[track_caller]
pub fn get_annot_coverage(up_region_ixs: &[uint]) -> f64 {
    let n = up_region_ixs.len();
    assert!(n > 0);
    let mut l = uint::MAX;
    let mut upix = uint::MAX;
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    for (i, regionix) in up_region_ixs.iter().enumerate() {
        assert!((*regionix as usize) < state.los.len());
        assert!((*regionix as usize) < state.his.len());
        assert!((*regionix as usize) < state.ls.len());

        let up = &state.ups[*regionix as usize];
        let upix2 = *state
            .up_to_upix
            .get(up)
            .unwrap_or_else(|| panic!("GetUpix({up}) not found"));
        let _sp = &state.sps[*regionix as usize];
        let _pf = &state.pfs[*regionix as usize];
        let _lo = state.los[*regionix as usize];
        let _hi = state.his[*regionix as usize];
        let l2 = state.ls[*regionix as usize];
        let _dom_len = _hi - _lo + 1;
        if i == 0 {
            l = l2;
            upix = upix2;
        } else {
            assert_eq!(l2, l);
            assert_eq!(upix2, upix);
        }
    }
    0.0
}

/// Decide if UniProt `upix` qualifies (non-overlapping domains, coverage >= 0.9) and, if so, format its annotation row.
#[track_caller]
pub fn select_up(upix: uint) -> Option<String> {
    const MINCVG: f64 = 0.9;
    const MINFRACT: f64 = 0.1;
    let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
    let up = &state.unique_ups[upix as usize];
    assert!((upix as usize) < state.upix_to_regionixs.len());
    let regionixs = &state.upix_to_regionixs[upix as usize];
    let n = regionixs.len();
    let mut pfs = Vec::new();
    let mut los = Vec::new();
    let mut his = Vec::new();
    let mut l = uint::MAX;
    let mut sp = String::new();
    let mut sum_dom_len = 0;
    for i in 0..n {
        let regionix = regionixs[i] as usize;
        let pf = &state.pfs[regionix];
        let up2 = &state.ups[regionix];
        let sp2 = &state.sps[regionix];
        let lo = state.los[regionix];
        let hi = state.his[regionix];
        let l2 = state.ls[regionix];

        if i == 0 {
            l = l2;
        } else {
            assert_eq!(l2, l);
        }
        assert_eq!(up2, up);
        if i == 0 {
            sp = sp2.clone();
        } else {
            assert_eq!(sp2, &sp);
        }

        assert!(lo > 0);
        assert!(lo < hi);
        assert!(hi <= l);
        let dom_len = hi - lo + 1;
        let fract = dom_len as f64 / l as f64;
        if fract >= MINFRACT {
            for j in 0..pfs.len() {
                if get_overlap(los[j], his[j], lo, hi) > 0 {
                    return None;
                }
            }
            sum_dom_len += dom_len;
            pfs.push(pf.clone());
            los.push(lo);
            his.push(hi);
        }
    }
    let coverage = sum_dom_len as f64 / l as f64;
    assert!(coverage <= 1.0);
    if coverage < MINCVG {
        return None;
    }

    // C++ uses unstable QuickSortOrder on Los
    // (cmd_newbench_selectpfams.cpp:969); match it for parity.
    let order = quick_sort_order_by(los.len(), |a, b| los[a].cmp(&los[b]))
        .into_iter()
        .map(|v| v as usize)
        .collect::<Vec<_>>();
    let mut row = String::new();
    row.push_str(up);
    row.push('\t');
    row.push_str(&sp);
    row.push('\t');
    row.push_str(&l.to_string());
    row.push('\t');
    row.push_str(&los.len().to_string());
    for i in order {
        row.push('\t');
        row.push_str(&pfs[i]);
        row.push('\t');
        row.push_str(&los[i].to_string());
        row.push('\t');
        row.push_str(&his[i].to_string());
    }
    row.push('\n');
    Some(row)
}

/// Driver: read PFAM regions TSV, run `select_up` on every UniProt accession, and emit selected rows.
#[track_caller]
pub fn cmd_newbench_selectpfams(
    pfam_regions_tsv_file_name: &str,
    output_file_name: &str,
) -> String {
    read_pfam_regions(pfam_regions_tsv_file_name);
    let up_count = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap().unique_ups.len() as uint;
    let mut out = String::new();
    for upix in 0..up_count {
        if let Some(row) = select_up(upix) {
            out.push_str(&row);
        }
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write selected PFAM rows");
    }
    out
}
