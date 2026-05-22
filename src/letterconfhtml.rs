// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns the length of the intersection between two inclusive integer ranges.
pub fn get_overlap(lo1: uint, hi1: uint, lo2: uint, hi2: uint) -> uint {
    let max_lo = lo1.max(lo2);
    let min_hi = hi1.min(hi2);
    if max_lo > min_hi {
        0
    } else {
        min_hi - max_lo + 1
    }
}

/// Groups a confidence-coded sequence into runs of equal confidence using gapped column indices.
#[track_caller]
pub fn get_conf_ranges_gapped(seq: &[byte], l: uint) -> (Vec<byte>, Vec<uint>, Vec<uint>) {
    let mut confs = Vec::new();
    let mut los = Vec::new();
    let mut his = Vec::new();

    let mut curr_conf = seq[0];
    let mut lo = 0_u32;
    for i in 1..=l as usize {
        let conf = if i == l as usize { 0 } else { seq[i] };
        if conf != curr_conf {
            let hi = i as uint - 1;
            let n = his.len();
            if n > 0 {
                assert!(lo == his[n - 1] + 1);
            }

            confs.push(curr_conf);
            los.push(lo);
            his.push(hi);

            lo = i as uint;
            curr_conf = conf;
        }
    }
    (confs, los, his)
}

/// Returns the HTML document head and CSS styles used by the letter-confidence viewer.
#[track_caller]
pub fn html_head() -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n");
    out.push_str("<html lang=\"en\">\n");
    out.push_str("<head>\n");
    out.push_str("    <meta charset=\"utf-8\">\n");
    out.push_str("    <title>Muscle5 alignment</title>\n");
    out.push_str("    <style>\n");
    out.push_str(
        "        .MonoBold {font-family: monospace; font-weight: bold; font-size: 16px;}\n",
    );

    for (i, color) in HEATMAP_COLORS_HTML.iter().enumerate() {
        out.push_str(&format!(
            "        .Style{i} {{background-color: #{color};}}\n"
        ));
    }

    out.push_str("        .StyleG {background-color: #e6e6e6;}\n");
    out.push_str("        .StyleL {font-weight: normal; }\n");
    out.push_str("        }\n");
    out.push_str("    </style>\n");
    out.push_str("</head>\n");
    out.push_str("<body>\n");
    out.push_str("<span class=\"MonoBold\">\n");
    out.push_str("        <br />\n");
    out.push_str("        <br />\n");
    out
}

/// Returns the HTML legend and closing tags for the letter-confidence viewer.
#[track_caller]
pub fn html_foot() -> String {
    let mut out = String::new();
    out.push_str("        <br />\n");
    out.push_str(
        "        <span style=\"font-family:serif; font-weight:normal\">Confidence high</span>\n",
    );

    out.push_str("        ");
    for i in (0..=9).rev() {
        out.push_str(&format!(
            "<span class=\"Style{}\">{}</span>",
            char::from(b'0' + i as byte),
            char::from(b'0' + i as byte)
        ));
    }
    out.push_str("        <span style=\"font-family:serif; font-weight:normal\">low</span>\n");
    out.push('\n');
    out.push_str("        </span>\n");
    out.push_str("    </body>\n");
    out.push_str("</html>\n");
    out
}

/// Writes a heatmap-colored HTML view of the reference MSA annotated with letter confidences.
#[track_caller]
pub fn write_letter_conf_html(file_name: &str, ref_msa: &MultiSequence, conf_aln: &MultiSequence) {
    if file_name.is_empty() {
        return;
    }

    let seq_count = conf_aln.seqs.len();
    let col_count = multi_sequence_get_col_count(conf_aln);
    if ref_msa.seqs.len() != seq_count || multi_sequence_get_col_count(ref_msa) != col_count {
        panic!("-ref has different number of rows or columns");
    }

    let mut labels = Vec::new();
    let mut max_label_length = 0_usize;
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
        max_label_length = max_label_length.max(label.len());
        labels.push(label);
    }

    let mut out = html_head();
    let rowlen = 80_u32;
    let block_count = col_count.div_ceil(rowlen);
    for block_index in 0..block_count {
        let block_lo = block_index * rowlen;
        let block_hi = block_lo + rowlen - 1;
        for seq_index in 0..seq_count {
            let label = &labels[seq_index];
            out.push_str("<span class=\"StyleL\">");
            for _k in label.len()..max_label_length {
                out.push_str("&nbsp;");
            }
            out.push_str(label);
            out.push_str("</span>&nbsp;&nbsp;");
            let conf_seq = sequence_get_seq_as_string(&conf_aln.seqs[seq_index]);
            let (confs, los, his) = get_conf_ranges_gapped(conf_seq.as_bytes(), col_count);

            let range_count = confs.len();
            assert!(his.len() == range_count);
            assert!(los.len() == range_count);
            for i in 0..range_count {
                let conf = confs[i];
                let lo = los[i];
                let hi = his[i];

                let overlap = get_overlap(block_lo, block_hi, lo, hi);
                if overlap == 0 {
                    continue;
                }

                if i > 0 {
                    assert!(lo == his[i - 1] + 1);
                }
                if conf.is_ascii_digit() {
                    out.push_str(&format!("<span class=\"Style{}\">", conf as char));
                } else if conf == b'-' {
                    out.push_str("<span class=\"StyleG\">");
                } else {
                    panic!("Bad conf={}", conf as char);
                }

                for pos in lo..=hi {
                    if pos < block_lo || pos > block_hi {
                        continue;
                    }
                    out.push(ref_msa.seqs[seq_index].char_vec[pos as usize]);
                }

                out.push_str("</span>");
            }
            out.push_str("<br />\n");
        }
        out.push_str("<br />\n");
        out.push_str("<br />\n");
    }
    out.push_str(&html_foot());
    std::fs::write(file_name, out).expect("failed to write letter confidence html");
}

/// Implements `cmd_letterconf_html`: loads conf and ref MSAs and writes the HTML view.
#[track_caller]
pub fn cmd_letterconf_html(letterconf_html: &str, ref_file_name: &str, output_file_name: &str) {
    let mut conf_aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut conf_aln, letterconf_html, false);
    let ref_file_name_check = output_file_name;
    if ref_file_name_check.is_empty() {
        die("Must set -ref");
    }
    let mut ref_msa = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut ref_msa, ref_file_name, false);
    if output_file_name.is_empty() {
        die("Must set -output");
    }
    write_letter_conf_html(output_file_name, &ref_msa, &conf_aln);
}
