// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Groups a confidence-coded sequence into runs of equal confidence using ungapped positions.
pub fn get_conf_ranges_ungapped(seq: &[byte], l: uint) -> (Vec<byte>, Vec<uint>, Vec<uint>) {
    let mut confs = Vec::<byte>::new();
    let mut los = Vec::<uint>::new();
    let mut his = Vec::<uint>::new();

    let mut curr_conf = seq[0];
    let mut lo = 0;
    let mut pos: uint = 0;
    if seq[0] != b'-' && seq[0] != b'.' {
        pos += 1;
    }
    for i in 1..=l as usize {
        let conf = if i == l as usize { 0 } else { seq[i] };
        if conf != curr_conf {
            let hi = pos.wrapping_sub(1);
            let n = his.len();
            if n > 0 {
                assert_eq!(lo, his[n - 1].wrapping_add(1));
            }

            confs.push(curr_conf);
            los.push(lo);
            his.push(hi);

            lo = pos;
            curr_conf = conf;
        }
        if conf != b'-' && conf != b'.' {
            pos += 1;
        }
    }

    (confs, los, his)
}

/// Writes per-letter confidence annotations as a JalView feature file.
#[track_caller]
pub fn write_letter_conf_jal_view(
    file_name: &str,
    ref_msa: &MultiSequence,
    conf_aln: &MultiSequence,
) {
    if file_name.is_empty() {
        return;
    }

    const HEATMAP_COLORS_JALVIEW: [&str; 10] = [
        "A00000", "902020", "803030", "703030", "603030", "404040", "407040", "308030", "009000",
        "00A000",
    ];

    let seq_count = conf_aln.seqs.len();
    let col_count = multi_sequence_get_col_count(conf_aln);
    if ref_msa.seqs.len() != seq_count || multi_sequence_get_col_count(ref_msa) != col_count {
        panic!("-ref has different number of rows or columns");
    }

    let mut labels = Vec::new();
    for seq_index in 0..seq_count {
        let label = conf_aln.seqs[seq_index].label.clone();
        let ref_label = ref_msa.seqs[seq_index].label.clone();
        if label != ref_label {
            panic!(
                "-ref labels do not match, seq {} input={} ref={}",
                seq_index + 1,
                label,
                ref_label
            );
        }
        labels.push(label);
    }

    let mut out = String::new();
    for (i, color) in HEATMAP_COLORS_JALVIEW.iter().enumerate() {
        out.push_str(&format!("LC{i}\t{color}\n"));
    }

    out.push_str("STARTGROUP\tMuscle5_LetterConfs\n");
    for seq_index in 0..seq_count {
        let label = &labels[seq_index];
        let seq = sequence_get_seq_as_string(&conf_aln.seqs[seq_index]);

        let (confs, los, his) = get_conf_ranges_ungapped(seq.as_bytes(), col_count);

        let range_count = confs.len();
        assert_eq!(his.len(), range_count);
        assert_eq!(los.len(), range_count);
        for i in 0..range_count {
            let conf = confs[i] as char;
            let lo = los[i];
            let hi = his[i];
            if conf == '-' {
                continue;
            }
            out.push_str(&format!(
                "-\t{label}\t{seq_index}\t{}\t{}\tLC{conf}\n",
                lo.wrapping_add(1),
                hi.wrapping_add(1)
            ));
        }
    }
    out.push_str("ENDGROUP\tMuscle5_LetterConfs\n");
    std::fs::write(file_name, out).expect("failed to write letter confidence JalView");
}
