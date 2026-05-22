// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Find maximal runs of unaligned (`false`) columns in `als`; returns parallel low/high index vectors.
#[track_caller]
pub fn get_insert_lo_his(als: &[bool]) -> (Vec<uint>, Vec<uint>) {
    let mut los = Vec::new();
    let mut his = Vec::new();
    let mut start = uint::MAX;
    let col_count = als.len() as uint;
    for col in 0..col_count {
        if als[col as usize] {
            if start != uint::MAX {
                los.push(start);
                his.push(col - 1);
                start = uint::MAX;
            }
        } else if start == uint::MAX {
            start = col;
        }
    }
    if start != uint::MAX {
        los.push(start);
        his.push(col_count - 1);
    }
    (los, his)
}

/// True iff column `col` contains upper-case (aligned) residues; panics on mixed case.
#[track_caller]
pub fn get_msa_col_is_aligned(aln: &MultiSequence, col: uint) -> bool {
    let seq_count = aln.seqs.len();
    if seq_count == 0 {
        return false;
    }

    let mut upper_count = 0;
    let mut lower_count = 0;
    for seq in &aln.seqs {
        let c = seq.char_vec[col as usize];
        if c == '-' || c == '.' {
        } else if c.is_ascii_uppercase() {
            upper_count += 1;
        } else if c.is_ascii_lowercase() {
            lower_count += 1;
        } else {
            panic!("Unexpected sequence char '{c}'");
        }
    }
    if upper_count > 0 && lower_count > 0 {
        panic!("Mixed-case col");
    }
    upper_count > 0
}

/// Apply `get_msa_col_is_aligned` to every column of `aln`.
#[track_caller]
pub fn get_msa_col_aligned_vec(aln: &MultiSequence) -> Vec<bool> {
    let col_count = multi_sequence_get_col_count(aln);
    let mut aligned_vec = Vec::new();
    for col in 0..col_count {
        let al = get_msa_col_is_aligned(aln, col);
        aligned_vec.push(al);
    }
    aligned_vec
}

/// Collapse unaligned insert columns into compact blocks, centering each per-sequence insert with `.` padding.
#[track_caller]
pub fn squeeze_inserts(aln: &MultiSequence) -> MultiSequence {
    let seq_count = aln.seqs.len();
    let col_count = multi_sequence_get_col_count(aln);

    let mut ungapped_seqs = Vec::new();
    for seq_index in 0..seq_count {
        let mut ungapped_seq = String::new();
        for c in &aln.seqs[seq_index].char_vec {
            if *c != '-' && *c != '.' {
                ungapped_seq.push(*c);
            }
        }
        ungapped_seqs.push(ungapped_seq);
    }

    let als = get_msa_col_aligned_vec(aln);
    let (los, his) = get_insert_lo_his(&als);
    let range_count = los.len();
    if range_count == 0 {
        return aln.clone();
    }

    assert_eq!(his.len(), range_count);
    let mut new_rows = vec![String::new(); seq_count];
    let mut prev_hi = uint::MAX;
    for range_index in 0..range_count {
        let lo = los[range_index];
        let hi = his[range_index];
        assert!(lo <= hi);
        if range_index > 0 {
            assert!(lo > prev_hi);
        }

        let from = if prev_hi == uint::MAX { 0 } else { prev_hi + 1 };
        for col in from..lo {
            for seq_index in 0..seq_count {
                let mut c = aln.seqs[seq_index].char_vec[col as usize];
                if als[col as usize] {
                    c = c.to_ascii_uppercase();
                } else {
                    c = c.to_ascii_lowercase();
                }
                new_rows[seq_index].push(c);
            }
        }

        let mut max_ins_l = 0usize;
        let mut inserts = Vec::new();
        for seq_index in 0..seq_count {
            let mut insert = String::new();
            for col in lo..=hi {
                let c = aln.seqs[seq_index].char_vec[col as usize];
                if c != '-' && c != '.' {
                    insert.push(c.to_ascii_lowercase());
                }
            }
            let l = insert.len();
            if l > max_ins_l {
                max_ins_l = l;
            }
            inserts.push(insert);
        }
        for seq_index in 0..seq_count {
            let insert = &inserts[seq_index];
            let ins_l = insert.len();
            let mut n = 0usize;
            let dots = max_ins_l - ins_l;
            let mut dots1 = dots / 2;
            if from == 0 {
                dots1 = dots;
            } else if range_index + 1 == range_count {
                dots1 = 0;
            }
            while n < dots1 {
                new_rows[seq_index].push('.');
                n += 1;
            }
            new_rows[seq_index].push_str(insert);
            assert!(ins_l <= max_ins_l);
            while n < dots {
                new_rows[seq_index].push('.');
                n += 1;
            }
        }
        for seq_index in 0..seq_count {
            assert_eq!(new_rows[seq_index].len(), new_rows[0].len());
        }

        prev_hi = hi;
    }

    let from = if prev_hi == uint::MAX { 0 } else { prev_hi + 1 };
    for col in from..col_count {
        for seq_index in 0..seq_count {
            new_rows[seq_index].push(aln.seqs[seq_index].char_vec[col as usize]);
        }
    }
    for seq_index in 0..seq_count {
        assert_eq!(new_rows[seq_index].len(), new_rows[0].len());
    }

    let labels = aln
        .seqs
        .iter()
        .map(|seq| seq.label.clone())
        .collect::<Vec<_>>();
    let mut s = MultiSequence::default();
    multi_sequence_from_strings(&mut s, &labels, &new_rows);

    for seq_index in 0..seq_count {
        let mut ungapped_seq = String::new();
        for c in &s.seqs[seq_index].char_vec {
            if *c != '-' && *c != '.' {
                ungapped_seq.push(*c);
            }
        }
        if ungapped_seq != ungapped_seqs[seq_index] {
            panic!("UngappedSeq != UngappedSeqs[SeqIndex]");
        }
    }
    s
}

/// Driver: read a FASTA MSA, squeeze its inserts, and write the compacted MSA back to disk.
#[track_caller]
pub fn cmd_squeeze_inserts(input_file_name: &str, output_file_name: &str) -> MultiSequence {
    let input_msa = msa_from_fasta_file_preserve_case(input_file_name);
    let out_msa = squeeze_inserts(&input_msa);
    msa_to_fasta_file_l103(&out_msa, output_file_name);
    out_msa
}
