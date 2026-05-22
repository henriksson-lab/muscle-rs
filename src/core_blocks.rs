// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Identify and emit gap-free rectangular core blocks in an MSA via the greedy rectangle search.
#[track_caller]
pub fn cmd_core_blocks(
    msa_file_name: &str,
    output_file_name: &str,
    min_block_length: uint,
    min_block_seqs: uint,
) -> String {
    let aln = msa_from_fasta_file_l95(msa_file_name);
    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);

    let mut ungapped_mx = vec![vec![false; col_count as usize]; seq_count as usize];
    for seq_idx in 0..seq_count {
        for col_idx in 0..col_count {
            ungapped_mx[seq_idx as usize][col_idx as usize] = !msa_is_gap(&aln, seq_idx, col_idx);
        }
    }

    let blocks = greedy_rects(&ungapped_mx, min_block_length as i32, min_block_seqs as i32);
    let mut out = String::new();
    out.push_str(&format!("core_blocks\t{}\n", blocks.len()));
    for (block_idx, r) in blocks.iter().enumerate() {
        out.push_str(&format!(
            "block\t{}\t{}\t{}\n",
            block_idx, r.width, r.height
        ));
        for seq_idx in r.top..r.top + r.height {
            let label = msa_get_seq_label(&aln, seq_idx as uint);
            let col_to_pos = msa_get_col_to_pos(&aln, seq_idx as uint);
            for col_idx in r.left..r.left + r.width {
                out.push(msa_get_char(&aln, seq_idx as uint, col_idx as uint));
            }
            out.push_str(&format!("\t{}\t{}\n", col_to_pos[r.left as usize], label));
        }
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write core_blocks output");
    }
    out
}
