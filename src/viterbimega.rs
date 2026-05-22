// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct ViterbiMegaGapState {
    pub l_open_a: f32,
    pub l_open_b: f32,
    pub l_ext_a: f32,
    pub l_ext_b: f32,
    pub r_open_a: f32,
    pub r_open_b: f32,
    pub r_ext_a: f32,
    pub r_ext_b: f32,
    pub open_a: f32,
    pub open_b: f32,
    pub ext_a: f32,
    pub ext_b: f32,
}

pub static VITERBI_MEGA_GAPS: std::sync::Mutex<ViterbiMegaGapState> =
    std::sync::Mutex::new(ViterbiMegaGapState {
        l_open_a: 0.0,
        l_open_b: 0.0,
        l_ext_a: 0.0,
        l_ext_b: 0.0,
        r_open_a: 0.0,
        r_open_b: 0.0,
        r_ext_a: 0.0,
        r_ext_b: 0.0,
        open_a: 0.0,
        open_b: 0.0,
        ext_a: 0.0,
        ext_b: 0.0,
    });

/// Configure interior and terminal gap penalties for `viterbi_mega`.
#[track_caller]
pub fn set_gaps(int_open: f32, int_ext: f32, term_open: f32, term_ext: f32) {
    let mut gaps = VITERBI_MEGA_GAPS.lock().unwrap();
    gaps.l_open_a = term_open;
    gaps.l_open_b = term_open;
    gaps.l_ext_a = term_ext;
    gaps.l_ext_b = term_ext;

    gaps.r_open_a = term_open;
    gaps.r_open_b = term_open;
    gaps.r_ext_a = term_ext;
    gaps.r_ext_b = term_ext;

    gaps.open_a = int_open;
    gaps.open_b = int_open;
    gaps.ext_a = int_ext;
    gaps.ext_b = int_ext;
}

/// Viterbi alignment of two Mega structure profiles, writing trace bits into `mem`.
#[track_caller]
pub fn viterbi_mega(
    mem: &mut XDPMem,
    prof_a: &[Vec<byte>],
    prof_b: &[Vec<byte>],
    pi: &mut PathInfo,
) -> f32 {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;

    let la = prof_a.len() as uint;
    let lb = prof_b.len() as uint;
    if la * lb > 100 * 1000 * 1000 {
        die(&format!("ViterbiMega, seqs too long LA={la}, LB={lb}"));
    }

    let gaps = *VITERBI_MEGA_GAPS.lock().unwrap();
    mem.max_la = la;
    mem.max_lb = lb;
    mem.buffer1 = vec![MINUS_INFINITY; lb as usize + 1];
    mem.buffer2 = vec![MINUS_INFINITY; lb as usize + 1];
    mem.tb_bit = vec![vec![0; lb as usize + 1]; la as usize + 1];
    mem.tb_bit_row_count = la + 1;
    mem.tb_bit_col_count = lb + 1;
    mem.tb_bit_allocated_row_count = la + 1;
    mem.tb_bit_allocated_col_count = lb + 1;
    path_info_alloc2(pi, la, lb);

    let mut open_a = gaps.l_open_a;
    let mut ext_a = gaps.l_ext_a;
    let mut m0 = 0.0_f32;
    for i in 0..la {
        let mut open_b = gaps.l_open_b;
        let mut ext_b = gaps.l_ext_b;
        let mut i0 = MINUS_INFINITY;

        for j in 0..lb {
            let mut trace_bits = 0;
            let saved_m0 = m0;

            let mut xm = m0;
            if mem.buffer2[j as usize] > xm {
                xm = mem.buffer2[j as usize];
                trace_bits = TRACEBITS_DM;
            }
            if i0 > xm {
                xm = i0;
                trace_bits = TRACEBITS_IM;
            }
            m0 = mem.buffer1[j as usize];
            let match_score = mega_get_match_score_log_odds(prof_a, i, prof_b, j);
            mem.buffer1[j as usize] = xm + match_score;

            let md = saved_m0 + open_b;
            mem.buffer2[j as usize] += ext_b;
            if md >= mem.buffer2[j as usize] {
                mem.buffer2[j as usize] = md;
                trace_bits |= TRACEBITS_MD;
            }

            let mi = saved_m0 + open_a;
            i0 += ext_a;
            if mi >= i0 {
                i0 = mi;
                trace_bits |= TRACEBITS_MI;
            }

            open_b = gaps.open_b;
            ext_b = gaps.ext_b;
            mem.tb_bit[i as usize][j as usize] = trace_bits;
        }

        mem.tb_bit[i as usize][lb as usize] = 0;
        let md = m0 + gaps.r_open_b;
        mem.buffer2[lb as usize] += gaps.r_ext_b;
        if md >= mem.buffer2[lb as usize] {
            mem.buffer2[lb as usize] = md;
            mem.tb_bit[i as usize][lb as usize] = TRACEBITS_MD;
        }

        m0 = MINUS_INFINITY;
        open_a = gaps.open_a;
        ext_a = gaps.ext_a;
    }

    let mut i1 = MINUS_INFINITY;
    for j in 1..lb {
        mem.tb_bit[la as usize][j as usize] = 0;
        let mi = mem.buffer1[j as usize - 1] + gaps.r_open_a;
        i1 += gaps.r_ext_a;
        if mi > i1 {
            i1 = mi;
            mem.tb_bit[la as usize][j as usize] = TRACEBITS_MI;
        }
    }

    let final_m = mem.buffer1[lb as usize - 1];
    let final_d = mem.buffer2[lb as usize];
    let final_i = i1;
    let mut score = final_m;
    let mut state = b'M';
    if final_d > score {
        score = final_d;
        state = b'D';
    }
    if final_i > score {
        score = final_i;
        state = b'I';
    }

    trace_back_bit_mem(mem, la, lb, state, pi);
    score
}

/// Pair-align the two profiles in the global Mega state and optionally write the alignment.
#[track_caller]
pub fn align_mega2(
    output_file_name: &str,
    gap_open: Option<f32>,
    gap_ext: Option<f32>,
    term_gap_open: Option<f32>,
    term_gap_ext: Option<f32>,
) -> (f32, PathInfo, Option<String>) {
    {
        let mut gaps = VITERBI_MEGA_GAPS.lock().unwrap();
        let int_open = -gap_open.unwrap_or(0.85);
        let int_ext = -gap_ext.unwrap_or(0.10);
        let term_open = -term_gap_open.unwrap_or(0.0);
        let term_ext = -term_gap_ext.unwrap_or(0.10);
        gaps.l_open_a = term_open;
        gaps.l_open_b = term_open;
        gaps.l_ext_a = term_ext;
        gaps.l_ext_b = term_ext;
        gaps.r_open_a = term_open;
        gaps.r_open_b = term_open;
        gaps.r_ext_a = term_ext;
        gaps.r_ext_b = term_ext;
        gaps.open_a = int_open;
        gaps.open_b = int_open;
        gaps.ext_a = int_ext;
        gaps.ext_b = int_ext;
    }

    let (prof_a, prof_b, seq_a, seq_b, label_a, label_b) = {
        let mega = MEGA_STATE.lock().unwrap();
        let profile_count = mega.profiles.len() as uint;
        if profile_count != 2 {
            die(&format!(
                "AlignMega2(): {profile_count} structures found, 2 required"
            ));
        }
        (
            mega.profiles[0].clone(),
            mega.profiles[1].clone(),
            mega.seqs[0].clone(),
            mega.seqs[1].clone(),
            mega.labels[0].clone(),
            mega.labels[1].clone(),
        )
    };

    let mut mem = XDPMem::default();
    let mut pi = PathInfo::default();
    let score = viterbi_mega(&mut mem, &prof_a, &prof_b, &mut pi);
    if output_file_name.is_empty() {
        return (score, pi, None);
    }

    let mut out = String::new();
    let mut pos_a = 0_usize;
    let mut pos_b = 0_usize;
    out.push_str(&format!(">{label_a}\n"));
    for c in pi.path.bytes() {
        if c == b'M' || c == b'D' {
            out.push(seq_a.as_bytes()[pos_a] as char);
            pos_a += 1;
        } else {
            out.push('-');
        }
    }
    out.push('\n');
    out.push_str(&format!(">{label_b}\n"));
    for c in pi.path.bytes() {
        if c == b'M' || c == b'I' {
            out.push(seq_b.as_bytes()[pos_b] as char);
            pos_b += 1;
        } else {
            out.push('-');
        }
    }
    out.push('\n');
    std::fs::write(output_file_name, &out).expect("failed to write AlignMega2 output");
    (score, pi, Some(out))
}

/// CLI entry: load Mega input, pair-align the two profiles, and write the alignment.
#[track_caller]
pub fn cmd_mega2(
    input_file_name: &str,
    output_file_name: &str,
    gap_open: Option<f32>,
    gap_ext: Option<f32>,
    term_gap_open: Option<f32>,
    term_gap_ext: Option<f32>,
) -> (f32, PathInfo, Option<String>) {
    mega_from_file(input_file_name);
    align_mega2(
        output_file_name,
        gap_open,
        gap_ext,
        term_gap_open,
        term_gap_ext,
    )
}
