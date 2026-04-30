use muscle_rs::*;
use std::sync::Mutex;

static RNG_TEST_LOCK: Mutex<()> = Mutex::new(());
static GLOBAL_STATE_TEST_LOCK: Mutex<()> = Mutex::new(());

fn original_muscle_bin() -> Option<String> {
    if let Ok(path) = std::env::var("MUSCLE_CPP_BIN") {
        if std::path::Path::new(&path).is_file() {
            return Some(path);
        }
    }
    let default_path = "/data/henriksson/github/claude/oldmuscle/muscle/bin/muscle";
    if std::path::Path::new(default_path).is_file() {
        Some(default_path.to_string())
    } else {
        None
    }
}

fn normalize_command_text(s: &str) -> String {
    let s = s.trim_end();
    if s.is_empty() {
        String::new()
    } else {
        format!("{s}\n")
    }
}

fn original_stream_body(output: &[u8]) -> String {
    let stdout = String::from_utf8(output.to_vec()).unwrap();
    let mut seen_cmd = false;
    let mut body = String::new();
    for line in stdout.lines() {
        if seen_cmd {
            body.push_str(line);
            body.push('\n');
        } else if line.starts_with('[') && line.ends_with(']') {
            seen_cmd = true;
        }
    }
    normalize_command_text(&body)
}

fn original_command_body(output: &std::process::Output) -> String {
    let stdout_body = original_stream_body(&output.stdout);
    if !stdout_body.is_empty() {
        return stdout_body;
    }
    original_stream_body(&output.stderr)
}

#[test]
fn flat_buffer_sizes_match_cpp_formulas() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    assert_eq!(get_fb_size(2, 3), 60);
    assert_eq!(get_post_size(2, 3), 6);
    assert_eq!(get_dp_rows_size(2, 3), 8);
    assert_eq!(get_tb_size(2, 3), 12);
    assert_eq!(alloc_fb(2, 3).len(), 60);
    assert_eq!(alloc_post(2, 3).len(), 6);
    assert_eq!(alloc_dp_rows(2, 3).len(), 8);
    assert_eq!(alloc_tb(2, 3).len(), 12);

    let post = vec![0.0, 0.02, 0.5, 0.009, 0.01, 0.3];
    let mut sparse = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse, &post, 2, 3);
    assert_eq!(sparse.lx, 2);
    assert_eq!(sparse.ly, 3);
    assert_eq!(sparse.vec_size, 4);
    assert_eq!(&sparse.offsets[..=2], &[0, 2, 4]);
    assert_eq!(my_sparse_mx_get_offset(&sparse, 1), 2);
    assert_eq!(my_sparse_mx_get_size(&sparse, 0), 2);
    assert_eq!(my_sparse_mx_get_prob(&sparse, 0, 0), 0.0);
    assert_eq!(my_sparse_mx_get_prob(&sparse, 0, 1), 0.02);
    assert_eq!(my_sparse_mx_get_prob(&sparse, 1, 1), 0.01);
    assert_eq!(my_sparse_mx_get_max_prob_row(&sparse, 0), 0.5);
    assert_eq!(
        my_sparse_mx_to_post(&sparse),
        vec![0.0, 0.02, 0.5, 0.0, 0.01, 0.3]
    );
    assert_eq!(
        my_sparse_mx_get_col_to_row_lo_hi(&sparse),
        (vec![uint::MAX, 0, 0], vec![uint::MAX, 1, 1])
    );
    let mut sparse_dense_rows = alloc_dp_rows(2, 3);
    let sparse_dense_score =
        calc_aln_score_flat(&my_sparse_mx_to_post(&sparse), 2, 3, &mut sparse_dense_rows);
    assert!((calc_aln_score_sparse(&sparse) - sparse_dense_score).abs() < 1e-6);
    assert_eq!(
        my_sparse_mx_log_stats(&sparse, "P"),
        "MySparseMx(P) LX=2, LY=3 VecSize=4\n"
    );
    let sparse_log = my_sparse_mx_log_me(&sparse);
    assert!(sparse_log.contains("LX=2, LY=3"));
    assert!(sparse_log.contains("    0"));
    assert!(sparse_log.contains("."));
    let mut sparse_log_probe = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse_log_probe, &[0.0, 0.02, 1234.0], 1, 3);
    assert!(my_sparse_mx_log_me(&sparse_log_probe).contains("      0.02  1.23e+03"));

    let update_post = vec![0.0, 0.2, 0.6, 0.0, 0.4, 0.8];
    let mut updated = MySparseMx::default();
    my_sparse_mx_update_from_post(&mut updated, &sparse, &update_post, 2);
    assert_eq!(updated.vec_size, sparse.vec_size);
    assert_eq!(
        my_sparse_mx_to_post(&updated),
        vec![0.0, 0.1, 0.3, 0.0, 0.2, 0.4]
    );
    my_sparse_mx_alloc_lx(&mut updated, 200);
    assert!(updated.max_lx >= 328);
    my_sparse_mx_alloc_vec(&mut updated, 300);
    assert!(updated.max_vec_size >= 556);

    let mut xz = MySparseMx::default();
    my_sparse_mx_from_post(&mut xz, &[0.5, 0.2, 0.0, 0.3], 2, 2);
    let mut zy = MySparseMx::default();
    my_sparse_mx_from_post(&mut zy, &[0.1, 0.0, 0.4, 0.0, 0.6, 0.8], 2, 3);
    let mut zx = MySparseMx::default();
    my_sparse_mx_from_post(&mut zx, &[0.5, 0.0, 0.2, 0.3], 2, 2);
    let mut yz = MySparseMx::default();
    my_sparse_mx_from_post(&mut yz, &[0.1, 0.0, 0.0, 0.6, 0.4, 0.8], 3, 2);
    let expected_relaxed = vec![0.1, 0.24, 0.72, 0.0, 0.36, 0.48];
    let mut post_xz_zy = vec![0.0; 6];
    relax_flat_xz_zy(&xz, &zy, 2.0, &mut post_xz_zy);
    for (got, want) in post_xz_zy.iter().zip(expected_relaxed.iter()) {
        assert!((got - want).abs() < 1e-6);
    }
    let mut post_zx_zy = vec![0.0; 6];
    relax_flat_zx_zy(&zx, &zy, 2.0, &mut post_zx_zy);
    for (got, want) in post_zx_zy.iter().zip(expected_relaxed.iter()) {
        assert!((got - want).abs() < 1e-6);
    }
    let mut post_xz_yz = vec![0.0; 6];
    relax_flat_xz_yz(&xz, &yz, 2.0, &mut post_xz_yz);
    for (got, want) in post_xz_yz.iter().zip(expected_relaxed.iter()) {
        assert!((got - want).abs() < 1e-6);
    }

    let mut cons_ms = MultiSequence::default();
    multi_sequence_from_strings(
        &mut cons_ms,
        &["x".to_string(), "y".to_string(), "z".to_string()],
        &["AA".to_string(), "CC".to_string(), "GG".to_string()],
    );
    let mut cons_mpc = MPCFlat {
        weights: vec![10.0, 20.0, 30.0],
        ..MPCFlat::default()
    };
    mpc_flat_init_seqs(&mut cons_mpc, &cons_ms);
    mpc_flat_init_pairs(&mut cons_mpc);
    mpc_flat_alloc_pair_count(&mut cons_mpc, 3);
    let mut xy = MySparseMx::default();
    my_sparse_mx_from_post(&mut xy, &[0.2, 0.0, 0.0, 0.4], 2, 2);
    let mut xz2 = MySparseMx::default();
    my_sparse_mx_from_post(&mut xz2, &[0.5, 0.0, 0.0, 0.6], 2, 2);
    let mut yz2 = MySparseMx::default();
    my_sparse_mx_from_post(&mut yz2, &[0.7, 0.0, 0.0, 0.8], 2, 2);
    cons_mpc.sparse_posts1[0] = Some(xy);
    cons_mpc.sparse_posts1[1] = Some(xz2);
    cons_mpc.sparse_posts1[2] = Some(yz2);
    mpc_flat_cons_pair(&mut cons_mpc, 0);
    let cons_updated = cons_mpc.sparse_posts2[0].as_ref().unwrap();
    let cons_post = my_sparse_mx_to_post(cons_updated);
    assert!((cons_post[0] - 0.25).abs() < 1e-6);
    assert_eq!(cons_post[1], 0.0);
    assert_eq!(cons_post[2], 0.0);
    assert!((cons_post[3] - (1.28 / 3.0)).abs() < 1e-6);

    let mut prof_msa1 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut prof_msa1,
        &["p1a".to_string(), "p1b".to_string()],
        &["A-C".to_string(), "AGC".to_string()],
    );
    let mut prof_msa2 = MultiSequence::default();
    multi_sequence_from_strings(&mut prof_msa2, &["p2a".to_string()], &["T-T".to_string()]);
    let mut prof_mpc = MPCFlat::default();
    let mut prof_pairs = Vec::new();
    let prof_out = prof_align(
        &mut prof_mpc,
        &prof_msa1,
        &prof_msa2,
        |mpc, pair_index| {
            prof_pairs.push(mpc_flat_get_pair(mpc, pair_index));
        },
        |mpc, msa1, msa2| {
            assert_eq!(mpc.labels, vec!["p1a", "p1b", "p2a"]);
            assert_eq!(
                sequence_get_seq_as_string(&get_global_input_seq_by_label("p1a")),
                "AC"
            );
            assert_eq!(
                sequence_get_seq_as_string(&get_global_input_seq_by_label("p2a")),
                "TT"
            );
            assert_eq!(multi_sequence_get_col_count(msa1), 3);
            assert_eq!(multi_sequence_get_col_count(msa2), 3);
            let mut out = MultiSequence::default();
            multi_sequence_from_strings(
                &mut out,
                &["p1a".to_string(), "p1b".to_string(), "p2a".to_string()],
                &[
                    "A-C---".to_string(),
                    "AGC---".to_string(),
                    "---T-T".to_string(),
                ],
            );
            out
        },
    );
    assert_eq!(prof_pairs, vec![(0, 2), (1, 2)]);
    assert_eq!(prof_mpc.pairs, vec![(0, 1), (0, 2), (1, 2)]);
    assert_eq!(prof_out.seqs.len(), 3);
    assert_eq!(sequence_get_seq_as_string(&prof_out.seqs[0]), "A-C---");
    assert_eq!(sequence_get_seq_as_string(&prof_out.seqs[2]), "---T-T");

    let prof_dir = std::env::temp_dir().join(format!("muscle_rs_profalign_{}", std::process::id()));
    std::fs::create_dir_all(&prof_dir).unwrap();
    let prof1_file = prof_dir.join("p1.fa");
    let prof2_file = prof_dir.join("p2.fa");
    let prof_out_file = prof_dir.join("out.fa");
    std::fs::write(&prof1_file, b">q1\nEF-IL\n>q2\nPQRS-\n").unwrap();
    std::fs::write(&prof2_file, b">t1\nWY-VV\n").unwrap();
    let mut cmd_prof_pairs = Vec::new();
    let (_hp, cmd_prof_msa) = cmd_profalign(
        prof1_file.to_str().unwrap(),
        prof2_file.to_str().unwrap(),
        prof_out_file.to_str().unwrap(),
        None,
        |_hp| {},
        |mpc, pair_index| {
            cmd_prof_pairs.push(mpc_flat_get_pair(mpc, pair_index));
        },
        |_mpc, msa1, msa2| {
            let mut out = MultiSequence::default();
            multi_sequence_from_strings(
                &mut out,
                &["q1".to_string(), "q2".to_string(), "t1".to_string()],
                &[
                    format!("{}-----", sequence_get_seq_as_string(&msa1.seqs[0])),
                    format!("{}-----", sequence_get_seq_as_string(&msa1.seqs[1])),
                    format!("-----{}", sequence_get_seq_as_string(&msa2.seqs[0])),
                ],
            );
            out
        },
    );
    assert_eq!(cmd_prof_pairs, vec![(0, 2), (1, 2)]);
    assert_eq!(cmd_prof_msa.seqs.len(), 3);
    assert_eq!(
        std::fs::read_to_string(&prof_out_file).unwrap(),
        ">q1\nEF-IL-----\n>q2\nPQRS------\n>t1\n-----WY-VV\n"
    );

    let mut profseq_profile = MultiSequence::default();
    multi_sequence_from_strings(
        &mut profseq_profile,
        &["ps1".to_string(), "ps2".to_string()],
        &["AA".to_string(), "CC".to_string()],
    );
    let mut profseq_query = Sequence::default();
    sequence_from_string(&mut profseq_query, "query", "GG");
    let mut profseq_mpc = MPCFlat::default();
    let mut profseq_pairs = Vec::new();
    let profseq_path = prof_seq(
        &mut profseq_mpc,
        &profseq_profile,
        &profseq_query,
        |mpc, pair_index| {
            profseq_pairs.push(mpc_flat_get_pair(mpc, pair_index));
            my_sparse_mx_from_post(
                mpc_flat_get_sparse_post(mpc, pair_index),
                &[1.0, 0.0, 0.0, 1.0],
                2,
                2,
            );
        },
    );
    assert_eq!(profseq_pairs, vec![(0, 2), (1, 2)]);
    assert_eq!(profseq_path, "BB");

    let profseq_profile_file = prof_dir.join("profseq_profile.fa");
    let profseq_query_file = prof_dir.join("profseq_query.fa");
    std::fs::write(&profseq_profile_file, b">ps1\nAA\n>ps2\nCC\n").unwrap();
    std::fs::write(&profseq_query_file, b">q1\nGG\n>q2\nTT\n").unwrap();
    let mut cmd_profseq_pairs = Vec::new();
    let (_hp, cmd_profseq_paths) = cmd_profseq(
        profseq_profile_file.to_str().unwrap(),
        profseq_query_file.to_str().unwrap(),
        None,
        |_hp| {},
        |mpc, pair_index| {
            cmd_profseq_pairs.push(mpc_flat_get_pair(mpc, pair_index));
            my_sparse_mx_from_post(
                mpc_flat_get_sparse_post(mpc, pair_index),
                &[1.0, 0.0, 0.0, 1.0],
                2,
                2,
            );
        },
    );
    assert_eq!(cmd_profseq_pairs, vec![(0, 2), (1, 2), (0, 2), (1, 2)]);
    assert_eq!(cmd_profseq_paths, vec!["BB".to_string(), "BB".to_string()]);

    let profprof_a = prof_dir.join("profprof_a.fa");
    let profprof_b = prof_dir.join("profprof_b.fa");
    let profprof_out = prof_dir.join("profprof_out.fa");
    let profprof_p1 = prof_dir.join("profprof_p1.tsv");
    let profprof_p2 = prof_dir.join("profprof_p2.tsv");
    let profprof_p3 = prof_dir.join("profprof_p3.tsv");
    let profprof_p4 = prof_dir.join("profprof_p4.tsv");
    std::fs::write(&profprof_a, b">pa1\nAA\n>pa2\nCC\n").unwrap();
    std::fs::write(&profprof_b, b">pb1\nGG\n>pb2\nTT\n").unwrap();
    let profprof_ap = M3AlnParams {
        subst_mx_letter: [[0.0; 20]; 20],
        gap_open: -1.0,
        ready: true,
        ..M3AlnParams::default()
    };
    let (profprof_msa, _prof1, _prof2, prof12_msa, _prof12_path, diff_count, profprof_log) =
        cmd_profprof3(
            profprof_a.to_str().unwrap(),
            profprof_b.to_str().unwrap(),
            profprof_out.to_str().unwrap(),
            profprof_p1.to_str().unwrap(),
            profprof_p2.to_str().unwrap(),
            profprof_p3.to_str().unwrap(),
            profprof_p4.to_str().unwrap(),
            &profprof_ap,
            |_cm, prof1, prof2| {
                assert_eq!(prof1.pps.len(), 2);
                assert_eq!(prof2.pps.len(), 2);
                (4.0, "BB".to_string())
            },
            |_prof1, _w1, _prof2, _w2, subst, gap_open, path| {
                assert_eq!(path, "BB");
                let mut msa = MultiSequence::default();
                multi_sequence_from_strings(
                    &mut msa,
                    &[
                        "pa1".to_string(),
                        "pa2".to_string(),
                        "pb1".to_string(),
                        "pb2".to_string(),
                    ],
                    &[
                        "AA".to_string(),
                        "CC".to_string(),
                        "GG".to_string(),
                        "TT".to_string(),
                    ],
                );
                let mut prof = Profile3::default();
                profile3_from_msa(&mut prof, &msa, subst, gap_open, &[0.25; 4]);
                prof
            },
        );
    assert_eq!(diff_count, 0);
    assert_eq!(profprof_msa.seqs.len(), 4);
    assert_eq!(prof12_msa.pps.len(), 2);
    assert!(profprof_log.contains("Score=4"));
    assert_eq!(
        std::fs::read_to_string(&profprof_out).unwrap(),
        ">pa1\nAA\n>pa2\nCC\n>pb1\nGG\n>pb2\nTT\n"
    );
    assert!(
        std::fs::read_to_string(&profprof_p1)
            .unwrap()
            .contains("0\t1")
    );
    assert!(
        std::fs::read_to_string(&profprof_p3)
            .unwrap()
            .contains("0\t1")
    );
    std::fs::remove_dir_all(&prof_dir).unwrap();

    let mut dp_rows = alloc_dp_rows(2, 3);
    assert!((calc_aln_score_flat(&post, 2, 3, &mut dp_rows) - 0.5).abs() < 1e-6);
    let mut dp_rows2 = alloc_dp_rows(2, 3);
    let mut tb = alloc_tb(2, 3);
    let (score, path) = calc_aln_flat(&post, 2, 3, &mut dp_rows2, &mut tb);
    assert!((score - 0.5).abs() < 1e-6);
    assert_eq!(path, "YYBX");
    assert_eq!(trace_back_flat(&tb, 2, 3), "YYBX");

    let mut flat_fwd = alloc_fb(1, 1);
    let mut flat_bwd = alloc_fb(1, 1);
    flat_fwd.fill(LOG_ZERO);
    flat_bwd.fill(LOG_ZERO);
    let terminal_base = HMMSTATE_COUNT as usize * (1 * (1 + 1) + 1);
    flat_fwd[terminal_base] = 0.5_f32.ln();
    flat_bwd[terminal_base] = 0.2_f32.ln();
    flat_fwd[terminal_base + 1] = 0.25_f32.ln();
    flat_bwd[terminal_base + 1] = 0.4_f32.ln();
    let total = calc_total_prob_flat(&flat_fwd, &flat_bwd, 1, 1);
    assert!((total - 0.2_f32.ln()).abs() < 1e-3);
    let mut dense_post = alloc_post(1, 1);
    calc_post_flat(&flat_fwd, &flat_bwd, 1, 1, &mut dense_post);
    assert!((dense_post[0] - 0.5).abs() < 1e-3);

    {
        let mut start = PAIR_HMM_START_SCORE.lock().unwrap();
        *start = [
            0.6_f32.ln(),
            0.2_f32.ln(),
            0.2_f32.ln(),
            0.1_f32.ln(),
            0.1_f32.ln(),
        ];
        let mut trans = PAIR_HMM_TRANS_SCORE.lock().unwrap();
        *trans = [[LOG_ZERO; 5]; 5];
        trans[HMMSTATE_M as usize][HMMSTATE_M as usize] = 0.5_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_IX as usize] = 0.1_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_IY as usize] = 0.1_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_JX as usize] = 0.1_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_JY as usize] = 0.1_f32.ln();
        trans[HMMSTATE_IX as usize][HMMSTATE_IX as usize] = 0.3_f32.ln();
        trans[HMMSTATE_IY as usize][HMMSTATE_IY as usize] = 0.3_f32.ln();
        trans[HMMSTATE_JX as usize][HMMSTATE_JX as usize] = 0.4_f32.ln();
        trans[HMMSTATE_JY as usize][HMMSTATE_JY as usize] = 0.4_f32.ln();
        trans[HMMSTATE_IX as usize][HMMSTATE_M as usize] = 0.7_f32.ln();
        trans[HMMSTATE_IY as usize][HMMSTATE_M as usize] = 0.7_f32.ln();
        trans[HMMSTATE_JX as usize][HMMSTATE_M as usize] = 0.6_f32.ln();
        trans[HMMSTATE_JY as usize][HMMSTATE_M as usize] = 0.6_f32.ln();
        let mut ins = PAIR_HMM_INS_SCORE.lock().unwrap();
        *ins = [0.05_f32.ln(); 256];
        ins[b'A' as usize] = 0.25_f32.ln();
        ins[b'C' as usize] = 0.20_f32.ln();
        let mut mat = PAIR_HMM_MATCH_SCORE.lock().unwrap();
        *mat = [[0.001_f32.ln(); 256]; 256];
        mat[b'A' as usize][b'C' as usize] = 0.30_f32.ln();
        mat[b'C' as usize][b'A' as usize] = 0.30_f32.ln();
    }
    MEGA_STATE.lock().unwrap().loaded = false;
    let mut fwd = alloc_fb(1, 1);
    let mut bwd = alloc_fb(1, 1);
    calc_fwd_flat_l12(b"A", 1, b"C", 1, &mut fwd);
    calc_bwd_flat_l10(b"A", 1, b"C", 1, &mut bwd);
    let mut fwd_wrapped = alloc_fb(1, 1);
    let mut bwd_wrapped = alloc_fb(1, 1);
    calc_fwd_flat_l31("A", "C", &mut fwd_wrapped);
    calc_bwd_flat_l22("A", "C", &mut bwd_wrapped);
    assert_eq!(fwd_wrapped, fwd);
    assert_eq!(bwd_wrapped, bwd);
    let m11 = HMMSTATE_COUNT as usize * (1 * (1 + 1) + 1) + HMMSTATE_M as usize;
    assert!((fwd[m11] - (0.6_f32 * 0.30_f32).ln()).abs() < 1e-6);
    assert!((bwd[m11] - 0.6_f32.ln()).abs() < 1e-6);
    let ix10 = HMMSTATE_COUNT as usize * (1 * (1 + 1)) + HMMSTATE_IX as usize;
    assert!((fwd[ix10] - (0.2_f32 * 0.25_f32).ln()).abs() < 1e-6);
    let iy01 = HMMSTATE_COUNT as usize + HMMSTATE_IY as usize;
    assert!((fwd[iy01] - (0.2_f32 * 0.20_f32).ln()).abs() < 1e-6);

    let init = vec![0.5, 0.2, 0.2, 0.05, 0.05];
    let gap_open = vec![0.10, 0.11, 0.03, 0.04];
    let gap_extend = vec![0.25, 0.26, 0.70, 0.71];
    let emit_single = vec![0.01; 256];
    let emit_pairs = vec![vec![0.001; 256]; 256];
    pair_hmm_create(&init, &gap_open, &gap_extend, &emit_pairs, &emit_single);
    {
        let trans_mat = PAIR_HMM_TRANS_MAT.lock().unwrap();
        assert!((trans_mat[0][0] - 0.72).abs() < 1e-6);
        assert_eq!(trans_mat[1][2], 0.0);
        assert!((trans_mat[3][0] - 0.30).abs() < 1e-6);
    }
    {
        let start = PAIR_HMM_START_SCORE.lock().unwrap();
        assert!((start[HMMSTATE_M as usize] - 0.5_f32.ln()).abs() < 1e-6);
        let trans = PAIR_HMM_TRANS_SCORE.lock().unwrap();
        assert!((trans[HMMSTATE_M as usize][HMMSTATE_IX as usize] - 0.10_f32.ln()).abs() < 1e-6);
        let ins = PAIR_HMM_INS_SCORE.lock().unwrap();
        assert!((ins[b'A' as usize] - 0.01_f32.ln()).abs() < 1e-6);
        let mat = PAIR_HMM_MATCH_SCORE.lock().unwrap();
        assert!((mat[b'A' as usize][b'C' as usize] - 0.001_f32.ln()).abs() < 1e-6);
    }
    MEGA_STATE.lock().unwrap().loaded = false;
    let mut post_ms = MultiSequence::default();
    multi_sequence_from_strings(
        &mut post_ms,
        &["cp0".to_string(), "cp1".to_string()],
        &["A".to_string(), "C".to_string()],
    );
    set_global_input_ms(&post_ms);
    let calc_post_vec = calc_post("cp0", "cp1");
    let mut direct_fwd = alloc_fb(1, 1);
    let mut direct_bwd = alloc_fb(1, 1);
    calc_fwd_flat_l12(b"A", 1, b"C", 1, &mut direct_fwd);
    calc_bwd_flat_l10(b"A", 1, b"C", 1, &mut direct_bwd);
    let mut direct_post = alloc_post(1, 1);
    calc_post_flat(&direct_fwd, &direct_bwd, 1, 1, &mut direct_post);
    assert_eq!(calc_post_vec.len(), 1);
    assert!((calc_post_vec[0] - direct_post[0]).abs() < 1e-6);
    let mut sparse_post = MySparseMx::default();
    let (ea_sparse, path_sparse) =
        align_pair_flat_sparse_post("cp0", "cp1", Some(&mut sparse_post));
    let (score_direct, path_direct) = calc_aln_flat(
        &direct_post,
        1,
        1,
        &mut alloc_dp_rows(1, 1),
        &mut alloc_tb(1, 1),
    );
    assert!((ea_sparse - score_direct).abs() < 1e-6);
    assert_eq!(path_sparse, path_direct);
    assert_eq!(my_sparse_mx_to_post(&sparse_post), direct_post);
    let (ea_plain, path_plain) = align_pair_flat("cp0", "cp1");
    assert!((ea_plain - ea_sparse).abs() < 1e-6);
    assert_eq!(path_plain, path_sparse);
    let mut mpc = MPCFlat::default();
    mpc_flat_init_seqs(&mut mpc, &post_ms);
    mpc_flat_init_pairs(&mut mpc);
    mpc_flat_init_dist_mx(&mut mpc);
    mpc_flat_alloc_pair_count(&mut mpc, 1);
    let mut mpc_fwd = alloc_fb(1, 1);
    let mut mpc_bwd = alloc_fb(1, 1);
    mpc_flat_calc_fwd_flat_mpc_flat(&mpc, 0, 1, 1, 1, &mut mpc_fwd);
    mpc_flat_calc_bwd_flat_mpc_flat(&mpc, 0, 1, 1, 1, &mut mpc_bwd);
    assert_eq!(mpc_fwd, direct_fwd);
    assert_eq!(mpc_bwd, direct_bwd);
    mpc_flat_calc_posterior(&mut mpc, 0);
    assert_eq!(
        my_sparse_mx_to_post(mpc.sparse_posts1[0].as_ref().unwrap()),
        direct_post
    );
    mpc.weights = vec![1.0, 1.0];
    let msa_left = msa_from_seq_range(&post_ms, 0, 1);
    let msa_right = msa_from_seq_range(&post_ms, 1, 1);
    assert_eq!(
        mpc_flat_build_post(&mpc, &msa_left, &msa_right),
        direct_post
    );
    assert_eq!(
        mpc_flat_build_post(&mpc, &msa_right, &msa_left),
        direct_post
    );
    let mut pprog_sparse_posts = Vec::new();
    let avg_ea = p_prog_get_post_pairs_aligned_flat(
        "long-progress-string-that-is-truncated",
        &msa_left,
        &msa_right,
        &[0],
        &[0],
        &mut pprog_sparse_posts,
    );
    assert_eq!(pprog_sparse_posts.len(), 1);
    assert_eq!(my_sparse_mx_to_post(&pprog_sparse_posts[0]), direct_post);
    assert!((avg_ea - score_direct).abs() < 1e-6);
    let mut pprog_path = String::new();
    let pprog_avg = p_prog_align_ms_as_flat(
        "flat-progress",
        &msa_left,
        &msa_right,
        uint::MAX,
        &mut pprog_path,
    );
    assert!((pprog_avg - score_direct).abs() < 1e-6);
    assert_eq!(pprog_path, path_direct);
    let mut pprog3_path = String::new();
    let pprog3_avg = p_prog_align_ms_as_flat3(
        "flat3-progress",
        &msa_left,
        &msa_right,
        &[],
        0,
        1,
        uint::MAX,
        &mut pprog3_path,
    );
    assert!((pprog3_avg - score_direct).abs() < 1e-6);
    assert_eq!(pprog3_path, path_direct);
    let mut mega_left = MultiSequence::default();
    let mut mega_right = MultiSequence::default();
    multi_sequence_from_strings(&mut mega_left, &["mega_l".to_string()], &["A-".to_string()]);
    multi_sequence_from_strings(
        &mut mega_right,
        &["mega_r".to_string()],
        &["-C".to_string()],
    );
    let mut mega_sparse_posts = Vec::new();
    let mega_avg = get_post_pairs_aligned_flat_mega(
        "mega-progress-string-that-is-truncated",
        &mega_left,
        &mega_right,
        &[0],
        &[0],
        &mut mega_sparse_posts,
    );
    assert_eq!(mega_sparse_posts.len(), 1);
    assert_eq!(my_sparse_mx_to_post(&mega_sparse_posts[0]), direct_post);
    assert!((mega_avg - score_direct).abs() < 1e-6);
    let mut mega_path = String::new();
    let mega_align_avg = align_ms_as_flat_mega(
        "mega-align-progress",
        &mega_left,
        &mega_right,
        uint::MAX,
        &mut mega_path,
    );
    assert!((mega_align_avg - score_direct).abs() < 1e-6);
    assert_eq!(mega_path, "YBX");
    let mut flat3_msa1 = MultiSequence::default();
    let mut flat3_msa2 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut flat3_msa1,
        &["f1a".to_string(), "f1b".to_string()],
        &["A-C".to_string(), "GG-".to_string()],
    );
    multi_sequence_from_strings(
        &mut flat3_msa2,
        &["f2a".to_string(), "f2b".to_string()],
        &["TT-".to_string(), "C-A".to_string()],
    );
    let mut sparse_a = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse_a, &[0.1, 0.2, 0.3, 0.4], 2, 2);
    let mut sparse_b = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse_b, &[0.5, 0.6, 0.7, 0.8], 2, 2);
    let mut flat3 = vec![9.0; 9];
    calc_posterior_flat3(
        &flat3_msa1,
        &flat3_msa2,
        &[0, 1],
        &[0, 1],
        &[sparse_a, sparse_b],
        &mut flat3,
    );
    assert_eq!(flat3, vec![0.6, 0.2, 0.6, 0.7, 0.0, 0.8, 0.3, 0.4, 0.0]);
    assert_eq!(
        mpc.sparse_posts1[0].as_ref().unwrap().x,
        Some(b"A".to_vec())
    );
    assert_eq!(
        mpc.sparse_posts1[0].as_ref().unwrap().y,
        Some(b"C".to_vec())
    );
    assert!((mpc.dist_mx[0][1] - score_direct).abs() < 1e-6);
    assert_eq!(mpc.dist_mx[1][0], mpc.dist_mx[0][1]);
    mpc.sparse_posts1[0] = None;
    mpc.dist_mx[0][1] = f32::MAX;
    mpc.dist_mx[1][0] = f32::MAX;
    mpc_flat_calc_posteriors(&mut mpc);
    assert_eq!(
        my_sparse_mx_to_post(mpc.sparse_posts1[0].as_ref().unwrap()),
        direct_post
    );
    assert!((mpc.dist_mx[0][1] - score_direct).abs() < 1e-6);
}

#[test]
fn unsigned_string_helpers_match_cpp_behavior() {
    assert!(is_uint_str("0"));
    assert!(is_uint_str("12345"));
    assert!(!is_uint_str(""));
    assert!(!is_uint_str(" 1"));
    assert!(!is_uint_str("1x"));
    assert_eq!(str_to_uint_l1278("42", false), 42);
    assert_eq!(str_to_uint_l1278("*", true), uint::MAX);
    assert_eq!(str_to_uint_l1313("17", false), 17);
    assert_eq!(str_to_uint64_l1294("4294967296"), 4_294_967_296);
    assert_eq!(str_to_uint64_l1308("99"), 99);
}

#[test]
fn formatting_helpers_follow_muscle_units() {
    assert_eq!(secs_to_hhmmss(59), "00:59");
    assert_eq!(secs_to_hhmmss(60), "01:00");
    assert_eq!(secs_to_hhmmss(3661), "01:01:01");
    assert_eq!(secs_to_str(0.000000123), "1.2e-07s");
    assert_eq!(secs_to_str(0.0005), "0.50ms");
    assert_eq!(secs_to_str(0.5), "0.500s");
    assert_eq!(secs_to_str(5.0), "5.00s");
    assert_eq!(secs_to_str(15.0), "15.0s");
    assert_eq!(secs_to_str(60.0), "01:00");
    assert_eq!(mem_bytes_to_str(9999.0), "9999.0b");
    assert_eq!(mem_bytes_to_str(10_000.0), "10.0kb");
    assert_eq!(mem_bytes_to_str(2_500_000.0), "2.5Mb");
    assert_eq!(mem_bytes_to_str(25_000_000.0), "25Mb");
    assert_eq!(mem_bytes_to_str(1_500_000_000.0), "1.5Gb");
    assert_eq!(pct_str(0.0, 0.0), "100%");
    assert_eq!(pct_str(1.0, 0.0), "inf%");
    assert_eq!(pct_str(1.0, 200.0), "  0.5%");
    assert_eq!(pct_str(1.0, 3.0), " 33.3%");
    assert_eq!(default_pcb(), "Processing");
    assert_eq!(int_to_str(9999), "9999");
    assert_eq!(int_to_str(10_000), "10.0k");
    assert_eq!(int_to_str(1_500_000), "1.5M");
    assert_eq!(int_to_str(123_456_789_012), "1.23e+11");
    assert_eq!(int64_to_str(10_000_000), "10M");
    assert_eq!(int64_to_str(123_456_789_012), "1.23e+11");
    assert_eq!(int_to_str2(9998), "9998");
    assert_eq!(int_to_str2(10_000), "10000 (10.0k)");
    assert_eq!(pct_to_str(0.0), "0%");
    assert_eq!(pct_to_str(0.00001234), "1.23e-05%");
    assert_eq!(pct_to_str(0.01234), "0.0123%");
    assert_eq!(pct_to_str(12.345), "12.35%");
    assert_eq!(float_to_str_l1385(0.00001234), "1.23e-05");
    assert_eq!(float_to_str_l1385(0.001234), "0.00123");
    assert_eq!(float_to_str_l1385(1.01), "1");
    assert_eq!(float_to_str_l1385(1.06), "1.1");
    assert_eq!(float_to_str_l1385(1.234e15), "1.23e+15");
    assert_eq!(float_to_str_l1417(12_345), "12.3k");
    assert_eq!(int_float_to_str(0.00001234), "1.23e-05");
    assert_eq!(int_float_to_str(12_345.0), "12.3k");
    assert_eq!(int_float_to_str(1.234e12), "1.23e+12");
    assert_eq!(myvstrprintf("abc"), "abc");
    assert_eq!(pf("file text"), "file text");
    let mut s = String::new();
    ps(&mut s, "alpha");
    assert_eq!(s, "alpha");
    psa(&mut s, "beta");
    assert_eq!(s, "alphabeta");
    psasc(&mut s, "gamma");
    assert_eq!(s, "alphabeta;gamma;");
    psasc(&mut s, "delta;");
    assert_eq!(s, "alphabeta;gamma;delta;");
    assert_eq!(warning("careful"), "\nWARNING: careful\n\n");
    assert_eq!(get_thread_str().capacity(), 64001);
    assert!(get_progress_prefix_str().ends_with(' '));
    assert_eq!(get_progress_prefix_c_str(), get_progress_prefix_str());
    assert!(progress_prefix(false));
    assert_eq!(progress("work\n"), "work\n");
    assert!(!progress_prefix(true));
    assert!(progress("work\n").ends_with("work\n"));
    assert_eq!(progress_log("log text"), "log text");
    assert_eq!(progress_log_prefix("log text"), "log text\n");
    assert_eq!(pr("printed"), "printed");
    let tmp =
        std::env::temp_dir().join(format!("muscle_rs_stdio_state_{}.txt", std::process::id()));
    std::fs::write(&tmp, b"abcdef").unwrap();
    let mut file = open_stdio_file(tmp.to_str().unwrap());
    set_stdio_file_pos64(&mut file, 3);
    let state_log = log_stdio_file_state(&mut file);
    assert!(state_log.contains("fileno"));
    assert!(state_log.contains("ftell      3"));
    assert!(state_log.contains("Not found in FileToFileName"));
    drop(file);
    std::fs::remove_file(&tmp).unwrap();
    let p = myalloc(3, 4);
    assert_eq!(p.len(), 12);
    myfree(Some(p));
    myfree(None);
    let (tracked, loc) = myalloc_track("/tmp/source.cpp", 17, 2, 5);
    assert_eq!(tracked.len(), 10);
    assert_eq!(loc, "source.cpp:17");
    let alloc_log = log_allocs_l132();
    assert!(alloc_log.contains("source.cpp:17"));
    assert!(alloc_log.contains("TOTAL LEAK 10.0b"));
    myfree_track(Some(tracked), &loc);
    assert!(log_allocs_l132().contains("TOTAL LEAK 0.0b"));
    assert_eq!(log_allocs_l171(), "");
}

#[test]
fn log_probability_helpers_match_reference_identities() {
    assert!((mylog2(8.0) - 3.0).abs() < 1e-12);
    assert!((mylog10(1000.0) - 3.0).abs() < 1e-12);
    assert_eq!(log1pexp(-100.0), 0.0);
    let a = -2.0_f32;
    let b = -3.0_f32;
    let expected = (a.exp() + b.exp()).ln();
    assert!((sum_log_prob(a, b) - expected).abs() < 1e-6);
    assert_eq!(log_add_hack(LOG_ZERO, -4.0), -4.0);
    assert!(post_eq(0.25, 0.255));
    assert!(!post_eq(0.25, 0.40));
    assert!(post_eq(0.0, 100.0));
    assert_eq!(
        log_tom_post(&[0.0, 0.1, 1234.0, 0.00125], 1, 1),
        "\nTomPost LX=1 LY=1\n                  0           1\n[  0]             0         0.1\n[  1]      1.23e+03     0.00125\n"
    );
    assert_eq!(
        log_flat_post(&[1234.0, 0.00125], 1, 2),
        "\nMyPost LX=1 LY=2\n                  0           1\n[  0]      1.23e+03     0.00125\n"
    );
    let lx = 2;
    let ly = 3;
    let mut flat = vec![0.0_f32; HMMSTATE_COUNT as usize * (lx + 1) * (ly + 1)];
    for (i, value) in flat.iter_mut().enumerate() {
        *value = i as f32;
    }
    let mxs = cvt_flat(&flat, lx as uint, ly as uint);
    assert_eq!(mxs[HMMSTATE_M as usize][0][0], 0.0);
    assert_eq!(mxs[HMMSTATE_IX as usize][0][0], 1.0);
    assert_eq!(mxs[HMMSTATE_JY as usize][2][3], 59.0);
    let mut tom = vec![0.0_f32; (lx + 1) * (ly + 1)];
    let my = vec![0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.6];
    for i in 0..lx {
        for j in 0..ly {
            tom[(ly + 1) * (i + 1) + j + 1] = my[ly * i + j];
        }
    }
    cmp_post(&tom, &my, lx as uint, ly as uint);
    let cmp_post_panic = std::panic::catch_unwind(|| {
        let mut bad_tom = vec![0.0_f32; 4];
        bad_tom[3] = 1.23456;
        cmp_post(&bad_tom, &[123456.0], 1, 1);
    });
    let cmp_post_panic = cmp_post_panic.unwrap_err();
    let cmp_post_panic = cmp_post_panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| cmp_post_panic.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(cmp_post_panic, "CmpPost i=0 j=0 Tom=1.2346 My=1.2346e+05");
}

#[test]
fn path_and_string_helpers_match_cpp_behavior() {
    assert_eq!(base_name("/tmp/foo.txt"), "foo.txt");
    assert_eq!(base_name(r"C:\tmp\foo.txt"), "foo.txt");
    assert_eq!(base_name("foo.txt"), "foo.txt");
    assert!(starts_with_l1955("abcdef", "abc"));
    assert!(starts_with_l1977("abcdef", "abc"));
    assert!(starts_with_l1982("abcdef", "abc"));
    assert!(!starts_with_l1955("ab", "abc"));
    assert!(ends_with("abcdef", "def"));
    assert!(!ends_with("ab", "abc"));
    assert_eq!(split("a b\tc", '\0'), vec!["a", "b", "c"]);
    assert_eq!(split("a,,b,", ','), vec!["a", "", "b"]);
    let mut s = "abcdef".to_string();
    reverse(&mut s);
    assert_eq!(s, "fedcba");
    assert_eq!(to_upper("aBc1-"), "ABC1-");
    assert_eq!(to_lower("aBc1-"), "abc1-");
    let mut s = " \t abc def \n".to_string();
    strip_white_space(&mut s);
    assert_eq!(s, "abc def");
    let mut all_space = " \t\n".to_string();
    strip_white_space(&mut all_space);
    assert_eq!(all_space, " \t\n");
    let mut r = "abcXYZdef".to_string();
    assert!(replace(&mut r, "XYZ", "123"));
    assert_eq!(r, "abc");
    assert!(!replace(&mut r, "XYZ", "123"));
    assert_eq!(get_acc_from_label("abc_123 rest"), "abc_123");
    assert_eq!(get_acc_from_label("abc-123"), "abc");
    {
        let mut seqs = NEWBENCH_UNGAPPED_SEQS.lock().unwrap();
        seqs.clear();
        seqs.push("ACDE".to_string());
        seqs.push("FG".to_string());
    }
    assert_eq!(get_ungapped_seq_length(0), 4);
    assert_eq!(get_ungapped_seq_length(1), 2);
    NEWBENCH_UNGAPPED_SEQS.lock().unwrap().clear();
    assert_eq!(get_base_name("/tmp/foo.fa"), "foo");
    assert_eq!(get_base_name(r"C:\tmp\foo.aln"), "foo");
    assert_eq!(get_base_name("foo.txt"), "foo.txt");
    assert_eq!(
        parse_file_name("/tmp/foo.fa"),
        ("/tmp".to_string(), "foo.fa".to_string())
    );
    assert_eq!(
        parse_file_name("foo.fa"),
        (".".to_string(), "foo.fa".to_string())
    );
    let list_dir = std::env::temp_dir().join(format!("muscle_rs_listdir_{}", std::process::id()));
    std::fs::create_dir_all(&list_dir).unwrap();
    std::fs::write(list_dir.join("b.txt"), b"b").unwrap();
    std::fs::write(list_dir.join("a.txt"), b"a").unwrap();
    let read_dir_names = read_dir_l291(list_dir.to_str().unwrap());
    assert_eq!(read_dir_names, vec![".", "..", "a.txt", "b.txt"]);
    assert_eq!(read_dir_l261(list_dir.to_str().unwrap()), read_dir_names);
    let mut listed = mylistdir_l1122(list_dir.to_str().unwrap());
    listed.sort();
    assert_eq!(listed, vec![".", "..", "a.txt", "b.txt"]);
    let mut listed2 = mylistdir_l1096(list_dir.to_str().unwrap());
    listed2.sort();
    assert_eq!(listed2, vec![".", "..", "a.txt", "b.txt"]);
    std::fs::remove_file(list_dir.join("a.txt")).unwrap();
    std::fs::remove_file(list_dir.join("b.txt")).unwrap();
    std::fs::remove_dir(&list_dir).unwrap();
    assert_eq!(get_elapsed_time_str().len(), 5);
    assert_eq!(get_max_ram_str(), " 0.0b");
    let mut dir = "out".to_string();
    dirize(&mut dir);
    assert_eq!(dir, "out/");
    dirize(&mut dir);
    assert_eq!(dir, "out/");
    assert_eq!(char_vec_to_str(b"abcXYZ"), "abcXYZ");
    assert_eq!(
        make_replicate_file_name("out/@.efa", TREEPERM::TP_ACB, 17),
        "out/acb.17.efa"
    );
    assert_eq!(
        make_replicate_file_name("@", TREEPERM::TP_None, 0),
        "none.0"
    );
    assert_eq!(make_replicate_file_name_n("rep@.efa", 12), "rep12.efa");
    assert_eq!(make_replicate_file_name_n("rep.efa", 12), "rep.efa12");
    assert_eq!(make_replicate_file_name_n("@/@", 7), "7/7");
    assert_eq!(mystrsave("abc\ndef"), "abc\ndef");
    assert!(get_version_string().starts_with("muscle 5.3."));
    assert!(get_version_string().contains(get_platform()));
    let version = print_version();
    assert!(version.starts_with(&get_version_string()));
    assert!(version.contains("\nBuilt "));
    assert_eq!(cmd_version(), format!("{}\n", print_version()));
    let log_file = std::env::temp_dir().join(format!("muscle_rs_log_{}.txt", std::process::id()));
    set_log_file_name(log_file.to_str().unwrap());
    log("abc");
    log("123");
    set_log_file_name("");
    assert_eq!(std::fs::read_to_string(&log_file).unwrap(), "abc123");
    std::fs::remove_file(&log_file).unwrap();
    assert_eq!(log_int(123, uint::MAX), "123");
    assert_eq!(log_int(12345, uint::MAX), "12345 (12.3k)");
    assert_eq!(log_int(7, 3), "  7");
    assert_eq!(logu(uint::MAX, 3, 2), "    *");
    assert_eq!(logu(42, 4, 1), "   42");
    assert_eq!(logf(f32::MAX, 3, 1), "   *");
    assert_eq!(logf(1.25, 5, 2), "   1.25");
    let cli = <MuscleCli as clap::Parser>::try_parse_from([
        "muscle",
        "--msastats",
        "input.fa",
        "--max_gap_fract",
        "0.25",
        "--quiet",
    ])
    .unwrap();
    assert_eq!(cli.msastats.as_deref(), Some("input.fa"));
    assert_eq!(cli.max_gap_fract, Some(0.25));
    assert!(cli.quiet);
    let cli_opts = <MuscleCli as clap::Parser>::try_parse_from([
        "muscle",
        "--efa-bestcols",
        "ensemble.efa",
        "--output",
        "out.fa",
        "--indir",
        "inputs",
        "--outdir",
        "outputs",
        "--db",
        "db.fa",
        "--minconf",
        "0.75",
        "--maxcols",
        "12",
        "--minea",
        "0.9",
        "--maxpd",
        "1.5",
        "--paircount",
        "100",
        "--threads",
        "2",
        "--linkage",
        "max",
        "--tsvout",
        "out.tsv",
        "--centroids",
        "centroids.fa",
        "--joins",
        "joins.tsv",
        "--guidetreein",
        "in.nwk",
        "--guidetreeout",
        "out.nwk",
        "--max-gap-fract-row",
        "0.4",
        "--prefix",
        "p_",
        "--suffix",
        ".fa",
        "--nodes",
        "nodes.tsv",
        "--html",
        "conf.html",
        "--jalview",
        "conf.features",
        "--label2",
        "B",
        "--labels2",
        "labels.tsv",
        "--subtreeout",
        "sub.nwk",
        "--supertreeout",
        "super.nwk",
        "--blosumparamset",
        "2",
        "--kmerdist",
        "33",
        "--testdir",
        "testdir",
        "--output2",
        "out2.tsv",
        "--output3",
        "out3.tsv",
        "--output4",
        "out4.tsv",
        "--center",
        "0.25",
        "--s-is",
        "0.1",
        "--s-il",
        "0.2",
        "--m-is",
        "0.3",
        "--m-il",
        "0.4",
        "--is-is",
        "0.5",
        "--il-il",
        "0.6",
        "--perturb",
        "7",
        "--consiters",
        "4",
        "--refineiters",
        "5",
        "--minsuper",
        "6",
        "--perm",
        "abc",
        "--distmxin",
        "dist.tsv",
        "--shrub-size",
        "9",
        "--super5-minea1",
        "0.7",
        "--super4-minea1",
        "0.6",
        "--super4-minea2",
        "0.85",
        "--super6-maxpd1",
        "1.2",
        "--nt",
        "--amino",
        "--confseq1",
        "--right",
        "--stratified",
        "--diversified",
        "--input-order",
        "--muscle3-randomorder",
        "--mega",
    ])
    .unwrap();
    assert_eq!(cli_opts.efa_bestcols.as_deref(), Some("ensemble.efa"));
    assert_eq!(cli_opts.indir.as_deref(), Some("inputs"));
    assert_eq!(cli_opts.outdir.as_deref(), Some("outputs"));
    assert_eq!(cli_opts.db.as_deref(), Some("db.fa"));
    assert_eq!(cli_opts.minconf, Some(0.75));
    assert_eq!(cli_opts.maxcols, Some(12));
    assert_eq!(cli_opts.minea, Some(0.9));
    assert_eq!(cli_opts.maxpd, Some(1.5));
    assert_eq!(cli_opts.paircount, Some(100));
    assert_eq!(cli_opts.threads, Some(2));
    assert_eq!(cli_opts.linkage.as_deref(), Some("max"));
    assert_eq!(cli_opts.tsvout.as_deref(), Some("out.tsv"));
    assert_eq!(cli_opts.centroids.as_deref(), Some("centroids.fa"));
    assert_eq!(cli_opts.joins.as_deref(), Some("joins.tsv"));
    assert_eq!(cli_opts.guidetreein.as_deref(), Some("in.nwk"));
    assert_eq!(cli_opts.guidetreeout.as_deref(), Some("out.nwk"));
    assert_eq!(cli_opts.max_gap_fract_row, Some(0.4));
    assert_eq!(cli_opts.prefix.as_deref(), Some("p_"));
    assert_eq!(cli_opts.suffix.as_deref(), Some(".fa"));
    assert_eq!(cli_opts.nodes.as_deref(), Some("nodes.tsv"));
    assert_eq!(cli_opts.html.as_deref(), Some("conf.html"));
    assert_eq!(cli_opts.jalview.as_deref(), Some("conf.features"));
    assert_eq!(cli_opts.label2.as_deref(), Some("B"));
    assert_eq!(cli_opts.labels2.as_deref(), Some("labels.tsv"));
    assert_eq!(cli_opts.subtreeout.as_deref(), Some("sub.nwk"));
    assert_eq!(cli_opts.supertreeout.as_deref(), Some("super.nwk"));
    assert_eq!(cli_opts.blosumparamset, Some(2));
    assert_eq!(cli_opts.kmerdist.as_deref(), Some("33"));
    assert_eq!(cli_opts.testdir.as_deref(), Some("testdir"));
    assert_eq!(cli_opts.output2.as_deref(), Some("out2.tsv"));
    assert_eq!(cli_opts.output3.as_deref(), Some("out3.tsv"));
    assert_eq!(cli_opts.output4.as_deref(), Some("out4.tsv"));
    assert_eq!(cli_opts.center, Some(0.25));
    assert_eq!(cli_opts.s_is, Some(0.1));
    assert_eq!(cli_opts.s_il, Some(0.2));
    assert_eq!(cli_opts.m_is, Some(0.3));
    assert_eq!(cli_opts.m_il, Some(0.4));
    assert_eq!(cli_opts.is_is, Some(0.5));
    assert_eq!(cli_opts.il_il, Some(0.6));
    assert_eq!(cli_opts.perturb, Some(7));
    assert_eq!(cli_opts.consiters, Some(4));
    assert_eq!(cli_opts.refineiters, Some(5));
    assert_eq!(cli_opts.minsuper, Some(6));
    assert_eq!(cli_opts.perm.as_deref(), Some("abc"));
    assert_eq!(cli_opts.distmxin.as_deref(), Some("dist.tsv"));
    assert_eq!(cli_opts.shrub_size, Some(9));
    assert_eq!(cli_opts.super5_minea1, Some(0.7));
    assert_eq!(cli_opts.super4_minea1, Some(0.6));
    assert_eq!(cli_opts.super4_minea2, Some(0.85));
    assert_eq!(cli_opts.super6_maxpd1, Some(1.2));
    assert!(cli_opts.nt);
    assert!(cli_opts.amino);
    assert!(cli_opts.confseq1);
    assert!(cli_opts.right);
    assert!(cli_opts.stratified);
    assert!(cli_opts.diversified);
    assert!(cli_opts.input_order);
    assert!(cli_opts.muscle3_randomorder);
    assert!(cli_opts.mega);
    let two_cmds = <MuscleCli as clap::Parser>::try_parse_from([
        "muscle",
        "--msastats",
        "a.fa",
        "--qscore",
        "b.fa",
    ])
    .unwrap();
    assert!(two_cmds.msastats.is_some());
    assert!(two_cmds.qscore.is_some());
    let cli_msa =
        std::env::temp_dir().join(format!("muscle_rs_cli_msastats_{}.fa", std::process::id()));
    std::fs::write(&cli_msa, b">a\nAC-\n>b\nA-G\n").unwrap();
    let stats_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-msastats", cli_msa.to_str().unwrap(), "-quiet"])
        .output()
        .unwrap();
    assert!(stats_cmd.status.success());
    let stats_out = String::from_utf8(stats_cmd.stdout).unwrap();
    assert!(stats_out.contains("         2  Sequences"));
    assert!(stats_out.contains("         3  Columns"));
    std::fs::remove_file(&cli_msa).unwrap();

    let cli_derep_out = std::env::temp_dir().join(format!(
        "muscle_rs_cli_rdrp_derep_{}.fa",
        std::process::id()
    ));
    let derep_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-derep",
            "muscle/test_data/rdrp/rdrp.fa",
            "-output",
            cli_derep_out.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(derep_cmd.status.success());
    let cli_derep_text = std::fs::read_to_string(&cli_derep_out).unwrap();
    assert_eq!(
        cli_derep_text
            .lines()
            .filter(|line| line.starts_with('>'))
            .count(),
        4494
    );
    std::fs::remove_file(&cli_derep_out).unwrap();
    let derep_no_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-derep", "muscle/test_data/rdrp/rdrp.fa", "-quiet"])
        .output()
        .unwrap();
    assert!(derep_no_output_cmd.status.success());

    let mut rdrp_all = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut rdrp_all, "muscle/test_data/rdrp/rdrp.fa", true);
    let mut rdrp_align_text = String::new();
    for seq in rdrp_all.seqs.iter().take(3) {
        rdrp_align_text.push_str(&seq_to_fasta_l2561(
            &sequence_get_seq_as_string(seq),
            &seq.label,
        ));
    }
    let cli_rdrp_align_in = std::env::temp_dir().join(format!(
        "muscle_rs_cli_rdrp_align_{}.fa",
        std::process::id()
    ));
    let cli_rdrp_align_out = std::env::temp_dir().join(format!(
        "muscle_rs_cli_rdrp_align_{}.afa",
        std::process::id()
    ));
    std::fs::write(&cli_rdrp_align_in, rdrp_align_text).unwrap();
    let align_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-align",
            cli_rdrp_align_in.to_str().unwrap(),
            "-output",
            cli_rdrp_align_out.to_str().unwrap(),
            "-consiters",
            "1",
            "-refineiters",
            "0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(align_cmd.status.success());
    let align_stats_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-msastats", cli_rdrp_align_out.to_str().unwrap(), "-quiet"])
        .output()
        .unwrap();
    assert!(align_stats_cmd.status.success());
    let align_stats = String::from_utf8(align_stats_cmd.stdout).unwrap();
    assert!(align_stats.contains("         3  Sequences"));
    assert!(align_stats.contains("       450  Columns"));
    let cli_rdrp_uclust_out = std::env::temp_dir().join(format!(
        "muscle_rs_cli_rdrp_uclust_{}.fa",
        std::process::id()
    ));
    let uclust_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-uclust",
            cli_rdrp_align_in.to_str().unwrap(),
            "-output",
            cli_rdrp_uclust_out.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(uclust_cmd.status.success());
    let uclust_text = std::fs::read_to_string(&cli_rdrp_uclust_out).unwrap();
    assert_eq!(
        uclust_text
            .lines()
            .filter(|line| line.starts_with('>'))
            .count(),
        3
    );
    assert!(uclust_text.starts_with(">AB000906.1_Infectious_flacherie_virus_A\n"));
    let uclust_no_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-uclust", cli_rdrp_align_in.to_str().unwrap(), "-quiet"])
        .output()
        .unwrap();
    assert!(uclust_no_output_cmd.status.success());
    let searchpd_missing_maxpd_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-searchpd",
            cli_rdrp_align_in.to_str().unwrap(),
            "-db",
            cli_rdrp_align_in.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!searchpd_missing_maxpd_cmd.status.success());
    assert!(
        String::from_utf8(searchpd_missing_maxpd_cmd.stderr)
            .unwrap()
            .contains("Must set -maxpd")
    );
    let searchpd_missing_db_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-searchpd",
            cli_rdrp_align_in.to_str().unwrap(),
            "-maxpd",
            "0.1",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!searchpd_missing_db_cmd.status.success());
    assert!(
        String::from_utf8(searchpd_missing_db_cmd.stderr)
            .unwrap()
            .contains("Must set -db")
    );
    let searchpd_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-searchpd",
            cli_rdrp_align_in.to_str().unwrap(),
            "-db",
            cli_rdrp_align_in.to_str().unwrap(),
            "-maxpd",
            "0.1",
            "-output",
            cli_rdrp_uclust_out.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!searchpd_output_cmd.status.success());
    assert!(
        String::from_utf8(searchpd_output_cmd.stderr)
            .unwrap()
            .contains("Use -tsvout not -output")
    );
    let uclustpd_missing_maxpd_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-uclustpd", cli_rdrp_align_in.to_str().unwrap(), "-quiet"])
        .output()
        .unwrap();
    assert!(!uclustpd_missing_maxpd_cmd.status.success());
    assert!(
        String::from_utf8(uclustpd_missing_maxpd_cmd.stderr)
            .unwrap()
            .contains("Must set -maxpd")
    );
    let uclustpd_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-uclustpd",
            cli_rdrp_align_in.to_str().unwrap(),
            "-maxpd",
            "0.1",
            "-output",
            cli_rdrp_uclust_out.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!uclustpd_output_cmd.status.success());
    assert!(
        String::from_utf8(uclustpd_output_cmd.stderr)
            .unwrap()
            .contains("Use -tsvout not -output")
    );
    let uclustpd2_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-uclustpd2",
            cli_rdrp_align_in.to_str().unwrap(),
            "-maxpd",
            "0.1",
            "-output",
            cli_rdrp_uclust_out.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!uclustpd2_output_cmd.status.success());
    assert!(
        String::from_utf8(uclustpd2_output_cmd.stderr)
            .unwrap()
            .contains("Use -output1/2")
    );
    let pprog_tree_missing_guide_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-pprog_tree", cli_rdrp_align_in.to_str().unwrap(), "-quiet"])
        .output()
        .unwrap();
    assert!(!pprog_tree_missing_guide_cmd.status.success());
    assert!(
        String::from_utf8(pprog_tree_missing_guide_cmd.stderr)
            .unwrap()
            .contains("Must set -guidetreein")
    );
    let transalnref_missing_input2_cmd =
        std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args([
                "-transalnref",
                cli_rdrp_align_in.to_str().unwrap(),
                "-label",
                "AB000906.1_Infectious_flacherie_virus_A",
                "-output",
                cli_rdrp_uclust_out.to_str().unwrap(),
                "-quiet",
            ])
            .output()
            .unwrap();
    assert!(!transalnref_missing_input2_cmd.status.success());
    assert!(
        String::from_utf8(transalnref_missing_input2_cmd.stderr)
            .unwrap()
            .contains("Must set -input2")
    );
    let letterconf_missing_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-letterconf_html",
            cli_rdrp_align_in.to_str().unwrap(),
            "-ref",
            cli_rdrp_align_in.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!letterconf_missing_output_cmd.status.success());
    assert!(
        String::from_utf8(letterconf_missing_output_cmd.stderr)
            .unwrap()
            .contains("Must set -ref")
    );
    for args in [
        vec!["-eadistmx", cli_rdrp_align_in.to_str().unwrap(), "-quiet"],
        vec!["-maxcc", cli_rdrp_align_in.to_str().unwrap(), "-quiet"],
        vec!["-resample", cli_rdrp_align_in.to_str().unwrap(), "-quiet"],
        vec!["-m3ensemble", cli_rdrp_align_in.to_str().unwrap(), "-quiet"],
    ] {
        let missing_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args(args)
            .output()
            .unwrap();
        assert!(!missing_output_cmd.status.success());
        assert!(
            String::from_utf8(missing_output_cmd.stderr)
                .unwrap()
                .contains("Must set -output")
        );
    }
    let m3ensemble_n_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-m3ensemble",
            cli_rdrp_align_in.to_str().unwrap(),
            "-output",
            cli_rdrp_uclust_out.to_str().unwrap(),
            "-n",
            "2",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(!m3ensemble_n_cmd.status.success());
    for (flag, value, msg) in [
        ("-diversified", None, "-diversified not supported"),
        ("-stratified", None, "-stratified not supported"),
        ("-replicates", Some("2"), "-replicates not supported"),
    ] {
        let mut args = vec![
            "-super5",
            cli_rdrp_align_in.to_str().unwrap(),
            "-output",
            cli_rdrp_uclust_out.to_str().unwrap(),
            flag,
            "-quiet",
        ];
        if let Some(value) = value {
            args.insert(args.len() - 1, value);
        }
        let super5_unsupported_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args(args)
            .output()
            .unwrap();
        assert!(!super5_unsupported_cmd.status.success());
        assert!(
            String::from_utf8(super5_unsupported_cmd.stderr)
                .unwrap()
                .contains(msg)
        );
    }
    let spatter_missing_warmup_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-spatter", cli_rdrp_align_in.to_str().unwrap(), "-quiet"])
        .output()
        .unwrap();
    assert!(!spatter_missing_warmup_cmd.status.success());
    assert!(
        String::from_utf8(spatter_missing_warmup_cmd.stderr)
            .unwrap()
            .contains("Must set -warmup_pct")
    );
    for args in [
        ["-test", "x", "-quiet"],
        ["-test_mega", "x", "-quiet"],
        ["-build_guide_tree", "x", "-quiet"],
    ] {
        let no_output_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args(args)
            .output()
            .unwrap();
        assert!(no_output_cmd.status.success());
        assert!(no_output_cmd.stdout.is_empty());
    }
    let swtest_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-swtest", "x", "-quiet"])
        .output()
        .unwrap();
    assert!(swtest_cmd.status.success());
    assert!(
        String::from_utf8(swtest_cmd.stdout)
            .unwrap()
            .contains("         1  PS score ok\n")
    );
    let swtestmm_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args(["-swtestmm", "x", "-quiet"])
        .output()
        .unwrap();
    assert!(swtestmm_cmd.status.success());
    let swtestmm_stdout = String::from_utf8(swtestmm_cmd.stdout).unwrap();
    assert!(swtestmm_stdout.contains("      1000  Tests\n"));
    assert!(swtestmm_stdout.contains("      1000  Agree\n"));
    std::fs::remove_file(&cli_rdrp_align_in).unwrap();
    std::fs::remove_file(&cli_rdrp_align_out).unwrap();
    std::fs::remove_file(&cli_rdrp_uclust_out).unwrap();

    let banner = print_banner();
    assert!(banner.starts_with('\n'));
    assert!(banner.contains(&get_version_string()));
    assert!(banner.contains(" RAM, "));
    assert!(banner.contains("(C) Copyright 2004-2021 Robert C. Edgar."));
    let help_text = help();
    assert!(help_text.starts_with(&banner));
    assert!(help_text.contains("Align FASTA input, write aligned FASTA (AFA) output:"));
    assert!(help_text.contains("header line \"<PERM.SEED\":"));
    assert_eq!(usage(), help_text);
    let compiler = compiler_info();
    assert!(compiler.contains(" bits\n"));
    assert!(compiler.contains("sizeof(int) = 4\n"));
    assert!(compiler.contains("sizeof(void *) = "));
    assert!(compiler.contains("pack(1)\n"));
    {
        let mut argv = G_ARGV.lock().unwrap();
        argv.clear();
        argv.push("muscle".to_string());
        argv.push("-align".to_string());
        argv.push("in.fa".to_string());
    }
    assert_eq!(get_cmd_line(), "muscle -align in.fa");
    assert_eq!(print_cmd_line(), "muscle -align in.fa \n");
    let log_info = log_program_info_and_cmd_line();
    assert!(log_info.contains(" built unknown unknown\n"));
    assert!(log_info.ends_with("muscle -align in.fa \n"));
    let log_done = log_elapsed_time_and_ram();
    assert!(log_done.contains("Elapsed time "));
    assert!(log_done.contains("Max memory "));
    G_ARGV.lock().unwrap().clear();
    assert!(try_flag_opt("quiet"));
    assert!(!try_flag_opt("threads"));
    assert_eq!(try_uns_opt("threads", "12"), Some(12));
    assert_eq!(try_uns_opt("quiet", "12"), None);
    assert_eq!(try_float_opt("max_gap_fract", "0.25"), Some(0.25));
    assert_eq!(try_float_opt("threads", "0.25"), None);
    assert_eq!(try_str_opt("output", "out.fa"), Some("out.fa".to_string()));
    assert_eq!(try_str_opt("quiet", "out.fa"), None);
    assert!(
        check_used_opt(true, false, "threads")
            .unwrap()
            .contains("Option -threads not used")
    );
    assert!(check_used_opt(true, true, "threads").is_none());
    assert!(check_used_opts(false).is_empty());

    let load_input_file =
        std::env::temp_dir().join(format!("muscle_rs_load_input_{}.fa", std::process::id()));
    std::fs::write(&load_input_file, b">a\nA-C\n>b\nDE.\n").unwrap();
    let loaded_input = load_input(load_input_file.to_str().unwrap(), false);
    assert_eq!(
        loaded_input
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["AC".to_string(), "DE".to_string()]
    );
    assert_eq!(get_global_ms_seq_count(), 2);
    std::fs::remove_file(&load_input_file).unwrap();

    let strings_file =
        std::env::temp_dir().join(format!("muscle_rs_read_strings_{}.txt", std::process::id()));
    std::fs::write(&strings_file, b"alpha\r\nbeta\nlast").unwrap();
    assert_eq!(
        read_strings_from_file(strings_file.to_str().unwrap()),
        vec!["alpha".to_string(), "beta".to_string(), "last".to_string()]
    );
    std::fs::remove_file(&strings_file).unwrap();
    let args_file =
        std::env::temp_dir().join(format!("muscle_rs_my_cmd_line_{}.txt", std::process::id()));
    std::fs::write(&args_file, b"-align in.fa\n-output out.fa\n").unwrap();
    my_cmd_line(&[
        "muscle".to_string(),
        "file:".to_string(),
        args_file.to_str().unwrap().to_string(),
        "-threads".to_string(),
        "2".to_string(),
    ]);
    assert_eq!(
        get_cmd_line(),
        "muscle -align in.fa -output out.fa -threads 2"
    );
    std::fs::remove_file(&args_file).unwrap();
    G_ARGV.lock().unwrap().clear();

    let strings_out_file = std::env::temp_dir().join(format!(
        "muscle_rs_strings_to_file_{}.txt",
        std::process::id()
    ));
    strings_to_file(
        strings_out_file.to_str().unwrap(),
        &["one".to_string(), "two".to_string()],
    );
    assert_eq!(
        std::fs::read_to_string(&strings_out_file).unwrap(),
        "one\ntwo\n"
    );
    std::fs::remove_file(&strings_out_file).unwrap();
}

#[test]
fn float_helpers_match_cpp_intent() {
    assert!(is_valid_float_str_l1191("1.25"));
    assert!(is_valid_float_str_l1199("-1e-3"));
    assert!(!is_valid_float_str_l1191(""));
    assert!(!is_valid_float_str_l1191("1.2x"));
    assert_eq!(str_to_float_l1204("1.5", false), 1.5);
    assert_eq!(str_to_float_l1209("*", true), f64::MAX);
    assert_eq!(str_to_mem_bytes("123"), 123.0);
    assert_eq!(hscore(&[0.1, 0.5, 0.9], &[1.0, 2.0, 3.0], 0.0), 1.0);
    assert_eq!(hscore(&[0.1, 0.5, 0.9], &[1.0, 2.0, 3.0], 0.1), 1.0);
    assert!((hscore(&[0.1, 0.5, 0.9], &[1.0, 2.0, 3.0], 0.3) - 1.5).abs() < 1e-12);
    assert_eq!(hscore(&[0.1, 0.5, 0.9], &[1.0, 2.0, 3.0], 0.9), 3.0);
    assert_eq!(test_l21(), "x=0 y=1\n");
    assert_eq!(cmd_test_l27(), cmd_test_l117());
    assert_eq!(cmd_test_l4(), "");
    assert_eq!(cmd_test_mega(), "");
    assert_eq!(cmd_testlog(), "");
    assert_eq!(cmd_testfb(), "");
    assert_eq!(test2_l4(0.25, 0.5), "P1=0.25 P2=0.5 ok\n");
    assert_eq!(test_exp(), "");
    let cmd_line_err = std::panic::catch_unwind(|| cmd_line_err("bad flag")).unwrap_err();
    let cmd_line_err = cmd_line_err
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| cmd_line_err.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(cmd_line_err, "\n\nInvalid command line\nbad flag\n\n");
    assert_eq!(test_l3(16), "     16.0b  ok\n");
    let malloc_log = cmd_test_malloc();
    assert_eq!(malloc_log.lines().count(), 9);
    assert!(malloc_log.contains("18.4Gb"));

    set_gaps(0.85, 0.10, 0.0, 0.097557);
    let gaps = *VITERBI_MEGA_GAPS.lock().unwrap();
    assert_eq!(gaps.l_open_a, 0.0);
    assert_eq!(gaps.l_open_b, 0.0);
    assert_eq!(gaps.l_ext_a, 0.097557);
    assert_eq!(gaps.l_ext_b, 0.097557);
    assert_eq!(gaps.r_open_a, 0.0);
    assert_eq!(gaps.r_open_b, 0.0);
    assert_eq!(gaps.r_ext_a, 0.097557);
    assert_eq!(gaps.r_ext_b, 0.097557);
    assert_eq!(gaps.open_a, 0.85);
    assert_eq!(gaps.open_b, 0.85);
    assert_eq!(gaps.ext_a, 0.10);
    assert_eq!(gaps.ext_b, 0.10);

    assert!(myfeq(10.0, 10.09));
    assert!(!myfeq(10.0, 10.2));
    assert!(myfeq(-1000.0, -2000.0));
    assert_eq!(logx(-9e9_f32), "        *");
    assert_eq!(logx(f32::MAX), "        &");
    assert_eq!(logx(-8e8_f32), "         ");
    assert_eq!(logx(12.345_f32), "    12.35");
    assert_eq!(
        log_mx("SM", &[vec![1.0, -9e9_f32], vec![f32::MAX, -8e8_f32]]),
        "\nLogMx(SM)\n              0        1\n  0 |      1.00        *\n  1 |         &         \n\n"
    );
    assert_eq!(
        log_tb_mx("TB", &[vec!['M', 'D'], vec!['I', 'X']]),
        "\nLogMx(TB)\n        0  1\n  0 | MD\n  1 | IX\n\n"
    );
    assert_eq!(
        log_dist_mx(
            "D",
            &[
                vec![0.0, f32::MAX, 1_234_567.0],
                vec![LOG_ZERO, 1.25, 0.00125],
            ],
        ),
        "\nLogDistMx(D)\n[    0]          0        *  1.23e+06\n[    1]          .     1.25  0.00125\n"
    );
    assert_eq!(
        log_tom_mx("T", &[1.0, 2.0, 1_234_567.0, 0.00125], 1, 1),
        "\nTom T: LX=1 LY=1\n                  0           1\n[  0]             1           2\n[  1]      1.23e+06     0.00125\n"
    );
    assert_eq!(
        log_flat_mx1("F1", &[1.0, 2.0, 1_234_567.0, 0.00125], 1, 1),
        "\nFlat1 F1: LX=1 LY=1\n                  0           1\n[  0]             1           2\n[  1]      1.23e+06     0.00125\n"
    );
    assert_eq!(
        log_flat_mx("F", &[1.0, 2.0, 1_234_567.0, 0.00125], 2, 2),
        "\nFlat F: LX=2 LY=2\n                  0           1\n[  0]             1           2\n[  1]      1.23e+06     0.00125\n"
    );
    let mut flat_mxs = vec![0.0_f32; (HMMSTATE_COUNT as usize) * 4];
    flat_mxs[0] = INVALID_LOG;
    flat_mxs[1] = OUT_OF_BAND_LOG;
    flat_mxs[2] = UNINIT_LOG;
    flat_mxs[3] = LOG_ZERO;
    flat_mxs[4] = 1_234_567.0;
    flat_mxs[5] = 0.00125;
    let flat_mxs_log = log_flat_mxs("S", &flat_mxs, 1, 1);
    assert!(flat_mxs_log.contains("Flat S[0]: LX=1 LY=1"));
    assert!(flat_mxs_log.contains("  *ERR*"));
    assert!(flat_mxs_log.contains("#"));
    assert!(flat_mxs_log.contains("-"));
    assert!(flat_mxs_log.contains("."));
    assert!(flat_mxs_log.contains("1.23e+06"));
    assert!(flat_mxs_log.contains(" 0.00125"));
    cmp_mx(
        'M',
        &[vec![0.0, 0.0], vec![0.0, 10.0]],
        &[vec![0.0, 0.0], vec![0.0, 10.05]],
    );
}

#[test]
fn scoretest_letter_counts_match_amino_mapping() {
    let (gap_count, counts) = get_letter_counts("ACD.-yya");
    assert_eq!(gap_count, 2);
    assert_eq!(counts.len(), 20);
    assert_eq!(counts[0], 2);
    assert_eq!(counts[1], 1);
    assert_eq!(counts[2], 1);
    assert_eq!(counts[19], 2);
    assert_eq!(counts.iter().sum::<uint>(), 6);

    let mx_file =
        std::env::temp_dir().join(format!("muscle_rs_subst_mx_{}.tsv", std::process::id()));
    let letters = "ACDEFGHIKLMNPQRSTVWY".chars().collect::<Vec<_>>();
    let mut text = String::from(" ");
    for c in &letters {
        text.push('\t');
        text.push(*c);
    }
    text.push('\n');
    for (i, c) in letters.iter().enumerate() {
        text.push(*c);
        for j in 0..20 {
            text.push('\t');
            text.push_str(&(i as i32 - j as i32).to_string());
        }
        text.push('\n');
    }
    std::fs::write(&mx_file, text).unwrap();
    let mx = read_subst_mx_letter_from_file(mx_file.to_str().unwrap());
    assert_eq!(mx[0][0], 0.0);
    assert_eq!(mx[1][0], 1.0);
    assert_eq!(mx[0][1], -1.0);
    assert_eq!(mx[19][18], 1.0);
    std::fs::remove_file(&mx_file).unwrap();

    set_alpha_l209(ALPHA::ALPHA_Amino);
    let (_, single_scoretest_log) = inner_test(
        "C-",
        "CC",
        1,
        1,
        0.5,
        0.0,
        2,
        1,
        1,
        0,
        0,
        2,
        2,
        0,
        0,
        0,
        &[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    assert!(single_scoretest_log.contains("Norm=1, MulOccs=1 Center=0.5 AddCenter=0"));
    assert!(single_scoretest_log.contains("PPA:\n"));
    assert!(single_scoretest_log.contains("Score ="));
    let pair_log = test1_l87("C-", "CC", 0.5);
    assert!(pair_log.contains("ColA = C-\nColB = CC\n"));
    assert!(pair_log.contains(" == Equivalent OK ==\n"));
    let cmd_log = cmd_scoretest();
    assert_eq!(cmd_log.matches(" == Equivalent OK ==").count(), 4);
}

#[test]
fn treeperm_round_trip_matches_cpp_names() {
    for (s, tp) in [
        ("none", TREEPERM::TP_None),
        ("abc", TREEPERM::TP_ABC),
        ("acb", TREEPERM::TP_ACB),
        ("bca", TREEPERM::TP_BCA),
        ("all", TREEPERM::TP_All),
    ] {
        assert_eq!(str_to_treeperm(s), tp);
        assert_eq!(treeperm_to_str(tp), s);
    }
}

#[test]
fn simple_cluster_matches_cpp_join_order_and_tree_export() {
    let labels = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];
    let dist_mx = vec![
        vec![0.0, 2.0, 8.0, 9.0],
        vec![2.0, 0.0, 7.0, 6.0],
        vec![8.0, 7.0, 0.0, 4.0],
        vec![9.0, 6.0, 4.0, 0.0],
    ];
    let mut sc = SimpleCluster {
        dist_mx: dist_mx.clone(),
        labels: labels.clone(),
        linkage: "min".to_string(),
        dist_is_similarity: false,
        sizes: vec![1, 1, 1, 1],
        pending: vec![0, 1, 2, 3],
        ..SimpleCluster::default()
    };
    assert_eq!(simple_cluster_get_dist(&sc, 0, 1), 2.0);
    assert_eq!(simple_cluster_get_size(&sc, 2), 1);
    assert_eq!(simple_cluster_find_closest_pair(&sc), (0, 1, 2.0));
    assert_eq!(simple_cluster_calc_new_dist(&sc, 0, 1, 3), 6.0);
    simple_cluster_run(&mut sc, &dist_mx, &labels, "min", false);
    assert_eq!(sc.parents, vec![4, 4, 5, 5, 6, 6, uint::MAX]);
    assert_eq!(sc.lefts[4], 0);
    assert_eq!(sc.rights[4], 1);
    assert_eq!(sc.lefts[5], 2);
    assert_eq!(sc.rights[5], 3);
    assert_eq!(sc.lefts[6], 4);
    assert_eq!(sc.rights[6], 5);
    assert_eq!(sc.sizes, vec![1, 1, 1, 1, 2, 2, 4]);
    assert_eq!(sc.heights[4], 1.0);
    assert_eq!(sc.heights[5], 2.0);
    assert_eq!(sc.heights[6], 3.0);
    assert_eq!(sc.lengths[4], 2.0);
    assert_eq!(sc.lengths[5], 1.0);
    assert_eq!(simple_cluster_get_label(&sc, 5), "Int1");
    let mut log_sc = sc.clone();
    log_sc.dist_mx[0][2] = 1_234_567.0;
    log_sc.dist_mx[2][0] = 1_234_567.0;
    let pre_log = simple_cluster_log_me(&log_sc);
    assert!(pre_log.contains("1.23e+06"));
    assert!(!pre_log.contains("1234567.000"));
    let log = simple_cluster_log_me(&sc);
    assert!(log.contains("Pending (1)  6"));
    assert!(log.contains("     3       1       0       2"));
    assert!(log.contains(">Int2"));

    let mut tree = Tree::default();
    simple_cluster_get_tree(&sc, &mut tree);
    assert_eq!(
        tree_get_leaf_labels(&tree),
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string()
        ]
    );
    assert!((tree_get_edge_length(&tree, 0, 4) - 1.0).abs() < 1e-6);
    assert!((tree_get_edge_length(&tree, 4, 6) - 2.0).abs() < 1e-6);
    let mut ms = MultiSequence::default();
    multi_sequence_from_strings(
        &mut ms,
        &labels,
        &vec![
            "AAAA".to_string(),
            "CCCC".to_string(),
            "GGGG".to_string(),
            "TTTT".to_string(),
        ],
    );
    let mut cw = ClustalWeights::default();
    let weights = clustal_weights_run(&mut cw, &ms, &tree);
    assert_eq!(
        cw.node_to_subtree_size[..tree.node_count as usize],
        [1, 1, 1, 1, 2, 2, 4]
    );
    assert!((weights[0] - 2.0 / 9.0).abs() < 1e-6);
    assert!((weights[1] - 2.0 / 9.0).abs() < 1e-6);
    assert!((weights[2] - 2.5 / 9.0).abs() < 1e-6);
    assert!((weights[3] - 2.5 / 9.0).abs() < 1e-6);
    assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-6);

    let mut sim_sc = SimpleCluster {
        dist_mx: vec![
            vec![0.0, 0.7, 0.2],
            vec![0.7, 0.0, 0.9],
            vec![0.2, 0.9, 0.0],
        ],
        pending: vec![0, 1, 2],
        dist_is_similarity: true,
        ..SimpleCluster::default()
    };
    assert_eq!(simple_cluster_find_closest_pair(&sim_sc), (1, 2, 0.9));
    sim_sc.linkage = "biased".to_string();
    sim_sc.sizes = vec![1, 1, 1];
    assert!((simple_cluster_calc_new_dist(&sim_sc, 0, 1, 2) - 0.2175).abs() < 1e-6);
}

#[test]
fn newick_token_type_names_match_cpp_macro_strings() {
    for (ntt, s) in [
        (NEWICKTOKENTYPE::NTT_Unknown, "Unknown"),
        (NEWICKTOKENTYPE::NTT_Lparen, "Lparen"),
        (NEWICKTOKENTYPE::NTT_Rparen, "Rparen"),
        (NEWICKTOKENTYPE::NTT_Colon, "Colon"),
        (NEWICKTOKENTYPE::NTT_Comma, "Comma"),
        (NEWICKTOKENTYPE::NTT_Semicolon, "Semicolon"),
        (NEWICKTOKENTYPE::NTT_String, "String"),
        (
            NEWICKTOKENTYPE::NTT_SingleQuotedString,
            "SingleQuotedString",
        ),
        (
            NEWICKTOKENTYPE::NTT_DoubleQuotedString,
            "DoubleQuotedString",
        ),
        (NEWICKTOKENTYPE::NTT_Comment, "Comment"),
    ] {
        assert_eq!(tree_ntt_str(ntt), s);
    }
}

#[test]
fn text_file_character_and_generic_token_reads_match_cpp_state() {
    let mut file = TextFile::default();
    text_file_init(&mut file, b"abc", "mem");
    assert_eq!(text_file_get_char(&mut file), Some(b'a'));
    assert_eq!(text_file_get_char(&mut file), Some(b'b'));
    assert_eq!(text_file_get_char(&mut file), Some(b'c'));
    assert_eq!(text_file_get_char(&mut file), Some(b'\n'));
    assert_eq!(text_file_get_char(&mut file), None);

    text_file_init(&mut file, b"  alpha{beta}", "mem");
    assert_eq!(
        text_file_get_token(&mut file, 32, "{}"),
        Some("alpha".into())
    );
    assert_eq!(text_file_get_token(&mut file, 32, "{}"), Some("{".into()));
    assert_eq!(
        text_file_get_token(&mut file, 32, "{}"),
        Some("beta".into())
    );
    assert_eq!(text_file_get_token(&mut file, 32, "{}"), Some("}".into()));

    let stdio_path =
        std::env::temp_dir().join(format!("muscle_rs_stdio_file_{}.txt", std::process::id()));
    std::fs::write(&stdio_path, b"abcdef").unwrap();
    let stdio_name = stdio_path.to_str().unwrap();
    let mut f = open_stdio_file(stdio_name);
    assert_eq!(get_stdio_file_size32(&mut f), 6);
    assert_eq!(get_stdio_file_size64(&mut f), 6);
    set_stdio_file_pos(&mut f, 2);
    assert_eq!(get_stdio_file_pos32(&mut f), 2);
    assert_eq!(read_stdio_file_no_fail(&mut f, 3), b"cde");
    assert_eq!(get_stdio_file_pos64(&mut f), 5);
    assert_eq!(read_stdio_file_l395(&mut f, 1, 2), b"bc");
    assert_eq!(read_stdio_file64_l408(&mut f, 3, 2), b"de");
    set_stdio_file_pos64(&mut f, 4);
    assert_eq!(read_stdio_file_l423(&mut f, 2), b"ef");
    set_stdio_file_pos64(&mut f, 0);
    assert_eq!(read_stdio_file64_l435(&mut f, 3), b"abc");
    set_stdio_file_pos64(&mut f, 2);
    assert_eq!(read_all_stdio_file(&mut f), b"abcdef");
    assert_eq!(get_stdio_file_pos64(&mut f), 2);
    assert_eq!(read_all_stdio_file64_l476(&mut f), b"abcdef");
    assert_eq!(get_stdio_file_pos64(&mut f), 2);
    drop(f);
    assert_eq!(read_all_stdio_file64_l463(stdio_name), b"abcdef");
    assert_eq!(read_all_stdio_file32(stdio_name), b"abcdef");

    let stdio_out =
        std::env::temp_dir().join(format!("muscle_rs_stdio_out_{}.txt", std::process::id()));
    let stdio_out_name = stdio_out.to_str().unwrap();
    let mut out = create_stdio_file(stdio_out_name).unwrap();
    write_stdio_file_l578(&mut out, b"abef");
    write_stdio_file_l558(&mut out, 2, b"cd");
    write_stdio_file64(&mut out, b"gh");
    write_stdio_file_str(&mut out, "ij");
    flush_stdio_file(&mut out);
    close_stdio_file(Some(out));
    assert_eq!(std::fs::read(stdio_out_name).unwrap(), b"abcdghij");
    assert!(create_stdio_file("").is_none());
    std::fs::remove_file(&stdio_path).unwrap();
    std::fs::remove_file(&stdio_out).unwrap();

    let lines_path =
        std::env::temp_dir().join(format!("muscle_rs_stdio_lines_{}.txt", std::process::id()));
    std::fs::write(&lines_path, b"one\r\nA\tB\nlast").unwrap();
    let mut lines = open_stdio_file(lines_path.to_str().unwrap());
    assert_eq!(
        progress_file_init(&mut lines, Some("Reading")),
        "  0.1% Reading"
    );
    set_stdio_file_pos64(&mut lines, 8);
    assert_eq!(
        progress_file_step(&mut lines, Some("Still reading")),
        Some(" 61.5% Still reading".to_string())
    );
    assert_eq!(get_progress_level_str(), " 61.5% Still reading");
    assert_eq!(progress_file_step(&mut lines, None), None);
    assert_eq!(progress_file_done(Some("Done")), "100.0% Done");
    assert_eq!(
        progress_step(0, 10, "Step"),
        Some(" 10.0% Step".to_string())
    );
    assert_eq!(
        progress_step(4, 10, "Step"),
        Some(" 50.0% Step".to_string())
    );
    assert_eq!(
        progress_step64(0, 10, "Big"),
        Some("  0.1% Big".to_string())
    );
    assert_eq!(
        progress_step64(5, 10, "Big"),
        Some(" 50.0% Big".to_string())
    );
    assert_eq!(progress_callback(0, 4), "  25.0% Processing");
    assert_eq!(progress_callback(3, 4), " 100.0% Processing");
    set_stdio_file_pos64(&mut lines, 0);
    assert_eq!(
        read_line_stdio_file_l605(&mut lines, 16),
        Some("one".to_string())
    );
    assert_eq!(
        read_tabbed_line(&mut lines, 2),
        vec!["A".to_string(), "B".to_string()]
    );
    assert_eq!(
        read_line_stdio_file_l650(&mut lines),
        Some("last".to_string())
    );
    assert_eq!(read_line_stdio_file_l650(&mut lines), None);
    std::fs::remove_file(&lines_path).unwrap();

    let rename_from =
        std::env::temp_dir().join(format!("muscle_rs_rename_from_{}.txt", std::process::id()));
    let rename_mid =
        std::env::temp_dir().join(format!("muscle_rs_rename_mid_{}.txt", std::process::id()));
    let rename_to =
        std::env::temp_dir().join(format!("muscle_rs_rename_to_{}.txt", std::process::id()));
    std::fs::write(&rename_from, b"from").unwrap();
    rename_stdio_file(rename_from.to_str().unwrap(), rename_mid.to_str().unwrap());
    assert!(!stdio_file_exists(rename_from.to_str().unwrap()));
    assert_eq!(std::fs::read(&rename_mid).unwrap(), b"from");
    std::fs::write(&rename_to, b"old").unwrap();
    move_stdio_file(rename_mid.to_str().unwrap(), rename_to.to_str().unwrap());
    assert!(!stdio_file_exists(rename_mid.to_str().unwrap()));
    assert_eq!(std::fs::read(&rename_to).unwrap(), b"from");
    delete_stdio_file(rename_to.to_str().unwrap());
    assert!(!stdio_file_exists(rename_to.to_str().unwrap()));
    close_stdio_file(None);
}

#[test]
fn fasta_sequence_reader_matches_cpp_record_rules() {
    let _guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    *FASTA_UPPER.lock().unwrap() = true;
    *FASTA_ALLOW_DIGITS.lock().unwrap() = true;

    let mut file = TextFile::default();
    text_file_init(
        &mut file,
        b">empty\n\n>seq one\na-c.d 12\n!ignored\n>seq2\nxx\n",
        "fasta",
    );
    assert_eq!(
        get_fasta_seq(&mut file, true),
        Some(("seq one".to_string(), "ACD12IGNORED".to_string()))
    );
    assert_eq!(
        get_fasta_seq(&mut file, true),
        Some(("seq2".to_string(), "XX".to_string()))
    );
    assert_eq!(get_fasta_seq(&mut file, true), None);

    text_file_init(&mut file, b">lower\nac-.9\n", "fasta");
    *FASTA_UPPER.lock().unwrap() = false;
    assert_eq!(
        get_fasta_seq(&mut file, false),
        Some(("lower".to_string(), "ac-.9".to_string()))
    );
    *FASTA_UPPER.lock().unwrap() = true;
}

#[test]
fn tree_newick_tokenizer_matches_cpp_comments_and_quotes() {
    let mut file = TextFile::default();
    text_file_init(&mut file, b" ( 'a b':0.1,[ignored] \"c\":2);", "tree");

    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_Lparen, "(".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_String, "a b".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_Colon, ":".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_String, "0.1".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_Comma, ",".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_String, "c".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_Colon, ":".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_String, "2".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_Rparen, ")".into())
    );
    assert_eq!(
        tree_get_token(&mut file, 64),
        (NEWICKTOKENTYPE::NTT_Semicolon, ";".into())
    );
}

#[test]
fn diagonal_helpers_match_cpp_formulas() {
    assert_eq!(get_diag_range(5, 4, 3), (2, 0, 4, 2));
    assert_eq!(get_diag_range(5, 4, 5), (0, 0, 3, 3));
    let b = get_diag_box(5, 4, 3, 5);
    assert_eq!(b.la, 5);
    assert_eq!(b.lb, 4);
    assert_eq!(b.dlo, 3);
    assert_eq!(b.dhi, 5);
    assert_eq!(
        (b.dlo_mini, b.dlo_minj, b.dlo_maxi, b.dlo_maxj),
        (2, 0, 4, 2)
    );
    assert_eq!(
        (b.dhi_mini, b.dhi_minj, b.dhi_maxi, b.dhi_maxj),
        (0, 0, 3, 3)
    );
    assert_eq!(get_diag_lo_hi(5, 4, "MIMD"), (5, 6));
    assert_eq!(get_diag_lo_hi(5, 4, "IID"), (uint::MAX, uint::MAX));
    test1_l96(5, 3, 4, 1, 0, 3, 2);
    assert_eq!(test2_l88(5, 4, 3, 5), b);
    assert_eq!(test_diag_box(), "\nALL OK\n");
}

#[test]
fn random_generator_matches_cpp_sequence() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    reset_rand(1);
    assert_eq!(
        (0..10).map(|_| randu32()).collect::<Vec<_>>(),
        vec![
            3_038_371_955,
            1_260_633_549,
            2_188_579_518,
            1_409_240_068,
            593_640_035,
            2_685_225_289,
            1_715_023_808,
            3_437_273_325,
            3_122_878_812,
            1_226_659_007,
        ]
    );

    reset_rand(42);
    assert_eq!(
        (0..5).map(|_| rand_int32()).collect::<Vec<_>>(),
        vec![
            4_254_789_784,
            2_512_986_608,
            2_228_124_101,
            3_587_908_383,
            1_287_365_884,
        ]
    );

    reset_rand(1);
    assert_eq!(randu64(), 5_414_379_868_233_785_459);
    reset_rand(1);
    let dist = get_random_dist_mx(3);
    assert_eq!(dist[0][0], 0.0);
    assert_eq!(dist[1][1], 0.0);
    assert_eq!(dist[2][2], 0.0);
    assert_eq!(dist[1][0], dist[0][1]);
    assert_eq!(dist[2][0], dist[0][2]);
    assert_eq!(dist[2][1], dist[1][2]);
    assert!((dist[1][0] - 0.551).abs() < 1e-6);
}

#[test]
fn simple_lcg_initializer_matches_cpp_sequence() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    slcg_srand(1);
    assert_eq!(slcg_rand(), 373_929_026);
    assert_eq!(slcg_rand(), 1_844_513_277);
}

#[test]
fn pair_generation_matches_cpp_ordering() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    assert_eq!(
        get_all_pairs_l3(4),
        (vec![0, 0, 0, 1, 1, 2], vec![1, 2, 3, 2, 3, 3])
    );
    assert_eq!(
        get_all_pairs_l18(2, 3),
        (vec![0, 0, 0, 1, 1, 1], vec![0, 1, 2, 0, 1, 2])
    );
    assert_eq!(get_pairs(2, 3, uint::MAX), get_all_pairs_l18(2, 3));

    reset_rand(1);
    let pairs = get_pairs(10, 10, 5);
    assert_eq!(pairs, (vec![2, 2, 5, 7, 8], vec![4, 7, 9, 2, 5]));

    let mut path = String::new();
    let dist = get_prot_dist_seq_pair(
        b"AC",
        2,
        b"AGC",
        3,
        Some(&mut path),
        |seqi, li, seqj, lj| {
            assert_eq!(&seqi[..li as usize], b"AC");
            assert_eq!(&seqj[..lj as usize], b"AGC");
            PathInfo {
                path: "BYB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(row_x, "A-C");
            assert_eq!(row_y, "AGC");
            assert_eq!(col_count, 3);
            1.25
        },
    );
    assert_eq!(dist, 1.25);
    assert_eq!(path, "BYB");

    let mut tt = 0.1_f64;
    let (mut p, mut dp, mut d2p, mut q, mut elambdat) = (0.0, 0.0, 0.0, 0.0, 0.0);
    re_predict(
        0,
        0,
        &mut tt,
        &mut p,
        &mut dp,
        &mut d2p,
        &mut q,
        &mut elambdat,
    );
    assert!((p - 0.06794298102677006).abs() < 1e-15);
    assert!((dp - -0.08287498345031888).abs() < 1e-15);
    assert!((d2p - 0.11996286072582593).abs() < 1e-15);
    assert!((q - 0.00224845633927084).abs() < 1e-15);
    assert!((elambdat - 0.93695528027260366).abs() < 1e-15);
    assert_eq!(tt, 0.1);
    assert!(
        (get_prot_dist_l111("ACDEFGHIKLMNPQRSTVWY", "ACDEFGHIKLMNPQRSTVWY") - 0.00001).abs()
            < 1e-12
    );
    assert!(
        (get_prot_dist_l111("ACDEFGHIKLMNPQRSTVWY", "YYYYYYYYYYYYYYYYYYYY") - 6.803538204535586)
            .abs()
            < 1e-12
    );
    assert_eq!(get_prot_dist_l111("----", "XXXX"), -1.0);

    let search_dist = align_and_prot_dist(
        b"AC",
        2,
        b"AGC",
        3,
        |seqi, li, seqj, lj| {
            assert_eq!(&seqi[..li as usize], b"AC");
            assert_eq!(&seqj[..lj as usize], b"AGC");
            PathInfo {
                path: "BYB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(row_x, "A-C");
            assert_eq!(row_y, "AGC");
            assert_eq!(col_count, 3);
            1.75
        },
    );
    assert_eq!(search_dist, 1.75);

    let mut mfa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut mfa,
        &["a".to_string(), "b".to_string()],
        &["AC".to_string(), "AG".to_string()],
    );
    let mfa_pair_dist = get_prot_dist_pair_from_mfa(
        &mfa,
        0,
        1,
        |seqi, li, seqj, lj| {
            assert_eq!(&seqi[..li as usize], b"AC");
            assert_eq!(&seqj[..lj as usize], b"AG");
            PathInfo {
                path: "BB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(row_x, "AC");
            assert_eq!(row_y, "AG");
            assert_eq!(col_count, 2);
            0.5
        },
    );
    assert_eq!(mfa_pair_dist, 0.5);

    let mut mfa1 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut mfa1,
        &["a1".to_string(), "a2".to_string()],
        &["AA".to_string(), "CC".to_string()],
    );
    let mut mfa2 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut mfa2,
        &["b1".to_string(), "b2".to_string()],
        &["GG".to_string(), "TT".to_string()],
    );
    let mut mfa_calls = Vec::new();
    let avg = get_prot_dist_mfa_pair(&mfa1, &mfa2, uint::MAX, |seq1, l1, seq2, l2| {
        let s1 = std::str::from_utf8(&seq1[..l1 as usize])
            .unwrap()
            .to_string();
        let s2 = std::str::from_utf8(&seq2[..l2 as usize])
            .unwrap()
            .to_string();
        mfa_calls.push((s1, s2));
        mfa_calls.len() as f64
    });
    assert_eq!(
        mfa_calls,
        vec![
            ("AA".to_string(), "GG".to_string()),
            ("AA".to_string(), "TT".to_string()),
            ("CC".to_string(), "GG".to_string()),
            ("CC".to_string(), "TT".to_string()),
        ]
    );
    assert_eq!(avg, 2.5);

    let protdists_in =
        std::env::temp_dir().join(format!("muscle_rs_protdists_{}.fa", std::process::id()));
    let protdists_out =
        std::env::temp_dir().join(format!("muscle_rs_protdists_{}.tsv", std::process::id()));
    std::fs::write(&protdists_in, b">pd_a\nEFIL\n>pd_b\nEFKL\n>pd_c\nPQRS\n").unwrap();
    let mut protdist_pairs = Vec::new();
    let protdist_rows = cmd_protdists(
        protdists_in.to_str().unwrap(),
        protdists_out.to_str().unwrap(),
        |seqi, li, seqj, lj| {
            protdist_pairs.push((
                std::str::from_utf8(&seqi[..li as usize])
                    .unwrap()
                    .to_string(),
                std::str::from_utf8(&seqj[..lj as usize])
                    .unwrap()
                    .to_string(),
            ));
            PathInfo {
                path: "BBBB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(col_count, 4);
            match (row_x, row_y) {
                ("EFKL", "EFIL") => 1.25,
                ("PQRS", "EFIL") => 0.0001234,
                ("PQRS", "EFKL") => 12_345.0,
                _ => panic!("unexpected rows {row_x} {row_y}"),
            }
        },
    );
    assert_eq!(
        protdist_pairs,
        vec![
            ("EFKL".to_string(), "EFIL".to_string()),
            ("PQRS".to_string(), "EFIL".to_string()),
            ("PQRS".to_string(), "EFKL".to_string()),
        ]
    );
    assert_eq!(
        protdist_rows,
        "pd_b\tpd_a\t1.25\npd_c\tpd_a\t0.0001234\npd_c\tpd_b\t1.234e+04\n"
    );
    assert_eq!(
        std::fs::read_to_string(&protdists_out).unwrap(),
        protdist_rows
    );
    std::fs::remove_file(&protdists_in).unwrap();
    std::fs::remove_file(&protdists_out).unwrap();

    let search_query = std::env::temp_dir().join(format!(
        "muscle_rs_searchpd_query_{}.fa",
        std::process::id()
    ));
    let search_db =
        std::env::temp_dir().join(format!("muscle_rs_searchpd_db_{}.fa", std::process::id()));
    let search_out =
        std::env::temp_dir().join(format!("muscle_rs_searchpd_{}.tsv", std::process::id()));
    std::fs::write(&search_query, b">q1\nEF-IL\n>q2\nPQRS\n").unwrap();
    std::fs::write(&search_db, b">d1\nEFKL\n>d2\nWYVV\n").unwrap();
    let mut search_pairs = Vec::new();
    let search_rows = cmd_searchpd(
        search_query.to_str().unwrap(),
        search_db.to_str().unwrap(),
        2.0,
        search_out.to_str().unwrap(),
        |seqi, li, seqj, lj| {
            search_pairs.push((
                std::str::from_utf8(&seqi[..li as usize])
                    .unwrap()
                    .to_string(),
                std::str::from_utf8(&seqj[..lj as usize])
                    .unwrap()
                    .to_string(),
            ));
            PathInfo {
                path: "BBBB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(col_count, 4);
            match (row_x, row_y) {
                ("EFIL", "EFKL") => 1.25,
                ("EFIL", "WYVV") => 2.5,
                ("PQRS", "EFKL") => 0.0001234,
                ("PQRS", "WYVV") => 12_345.0,
                _ => panic!("unexpected search rows {row_x} {row_y}"),
            }
        },
    );
    assert_eq!(
        search_pairs,
        vec![
            ("EFIL".to_string(), "EFKL".to_string()),
            ("EFIL".to_string(), "WYVV".to_string()),
            ("PQRS".to_string(), "EFKL".to_string()),
            ("PQRS".to_string(), "WYVV".to_string()),
        ]
    );
    assert_eq!(search_rows, "q1\td1\t1.25\nq2\td1\t0.000123\n");
    assert_eq!(std::fs::read_to_string(&search_out).unwrap(), search_rows);
    std::fs::remove_file(&search_query).unwrap();
    std::fs::remove_file(&search_db).unwrap();
    std::fs::remove_file(&search_out).unwrap();
}

#[test]
fn quartile_helpers_match_cpp_indexing() {
    let q = get_quarts(&[10, 1, 5, 3, 9]);
    assert_eq!(q.min, 1);
    assert_eq!(q.lo_q, 3);
    assert_eq!(q.med, 5);
    assert_eq!(q.hi_q, 9);
    assert_eq!(q.max, 10);
    assert_eq!(q.total, 28);
    assert_eq!(q.avg, 5.6);
    assert_eq!(get_quarts(&[]), Quarts::default());

    let qf = get_quarts_float(&[3.0, 1.0, 5.0, 9.0]);
    assert_eq!(qf.min, 1.0);
    assert_eq!(qf.lo_q, 3.0);
    assert_eq!(qf.med, 5.0);
    assert_eq!(qf.hi_q, 9.0);
    assert_eq!(qf.max, 9.0);
    assert_eq!(qf.total, 18.0);
    assert_eq!(qf.avg, 4.5);
    assert!((qf.std_dev - 2.95804).abs() < 1e-4);
}

#[test]
fn integer_power_helpers_match_cpp_behavior() {
    assert_eq!(myipow(2, 10), 1024);
    assert_eq!(myipow64(10, 12), 1_000_000_000_000);
}

#[test]
fn fasta_and_shuffle_helpers_match_cpp_ordering() {
    assert_eq!(seq_to_fasta_l2561("ACGT", "seq1"), ">seq1\nACGT\n");
    assert_eq!(
        sequence_log_new_delete_counts(),
        "Sequence::LogNewDeleteCounts new=0, delete=0\n"
    );
    let mut counted_seq = sequence_new_sequence();
    sequence_from_string(&mut counted_seq, "counted", "AC");
    assert_eq!(sequence_log_me(&counted_seq), "AC  >counted (2)\n");
    let mut counted_file = TextFile::default();
    sequence_write_mfa(&counted_seq, &mut counted_file);
    assert_eq!(
        String::from_utf8(counted_file.data.clone()).unwrap(),
        ">counted\nAC\n"
    );
    sequence_delete_sequence(counted_seq);
    assert_eq!(
        sequence_log_new_delete_counts(),
        "Sequence::LogNewDeleteCounts new=1, delete=1\n"
    );
    assert_eq!(seq_to_fasta_l2571(b"", Some("empty")), "");
    let long = "A".repeat(81);
    assert_eq!(
        seq_to_fasta_l2566(&long, "long"),
        format!(">long\n{}\nA\n", "A".repeat(80))
    );

    let _guard = RNG_TEST_LOCK.lock().unwrap();
    reset_rand(1);
    let mut v = vec![0, 1, 2, 3, 4];
    shuffle(&mut v);
    assert_eq!(v, vec![3, 2, 4, 1, 0]);
}

#[test]
fn trans_aln_paths_and_extensions_match_cpp_state_machine() {
    let mut msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut msa,
        &["m1".to_string(), "m2".to_string()],
        &["A-C".to_string(), "AGC".to_string()],
    );
    let mut fresh = MultiSequence::default();
    multi_sequence_from_strings(
        &mut fresh,
        &["f1".to_string(), "f2".to_string()],
        &["ATC".to_string(), "GC".to_string()],
    );
    let pw_paths = vec!["BXB".to_string(), "YBB".to_string()];
    let mut ta = TransAln::default();
    trans_aln_init(&mut ta, &msa, &fresh, &[0, 1], &pw_paths);

    assert_eq!(trans_aln_get_fresh_count(&ta), 2);
    assert_eq!(trans_aln_get_msa_count(&ta), 2);
    assert_eq!(trans_aln_get_msa_path(&ta, 0), "MGM");
    assert_eq!(trans_aln_get_msa_path(&ta, 1), "MMM");
    assert_eq!(trans_aln_get_t_path1(&ta, 0), "FIgF");
    assert_eq!(trans_aln_get_t_path1(&ta, 1), "GFF");
    assert_eq!(ta.msa_col_to_max_inserts, vec![0, 1, 0, 0]);
    assert_eq!(ta.extended_msa_col_count, 4);
    assert_eq!(trans_aln_get_t_path2(&ta, 0), "FIgF");
    assert_eq!(trans_aln_get_t_path2(&ta, 1), "GFiF");
    assert_eq!(ta.m_path, "MiMM");
    assert_eq!(
        sequence_get_seq_as_string(&trans_aln_extend_msa_seq(&ta, 0)),
        "A--C"
    );
    assert_eq!(
        sequence_get_seq_as_string(&trans_aln_extend_msa_seq(&ta, 1)),
        "A-GC"
    );
    assert_eq!(
        sequence_get_seq_as_string(&trans_aln_extend_fresh_seq(&ta, 0)),
        "AT-C"
    );
    assert_eq!(
        sequence_get_seq_as_string(&trans_aln_extend_fresh_seq(&ta, 1)),
        "-G-C"
    );

    trans_aln_make_extended_msa(&mut ta);
    let extended = ta.extended_msa.as_ref().unwrap();
    assert_eq!(
        extended
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A--C", "A-GC", "AT-C", "-G-C"]
    );
    assert_eq!(
        trans_aln_log_t_path1_aln(&ta, 0),
        "\nFIgF\nAT.C  >f1\nA..C  >m1\n"
    );
    assert_eq!(
        trans_aln_log_m_path_aln(&ta, 1, true),
        "\nMiMM\nA.GC  [M] >m2\n"
    );

    let trans_input =
        std::env::temp_dir().join(format!("muscle_rs_transaln_in_{}.fa", std::process::id()));
    let trans_ref =
        std::env::temp_dir().join(format!("muscle_rs_transaln_ref_{}.fa", std::process::id()));
    let trans_out =
        std::env::temp_dir().join(format!("muscle_rs_transaln_out_{}.fa", std::process::id()));
    std::fs::write(&trans_input, b">f1\nATC\n>f2\nGC\n").unwrap();
    std::fs::write(&trans_ref, b">m1\nA-C\n>m2\nAGC\n").unwrap();
    let mut trans_calls = Vec::new();
    let (cmd_ta, cmd_extended) = cmd_transaln(
        trans_input.to_str().unwrap(),
        trans_ref.to_str().unwrap(),
        trans_out.to_str().unwrap(),
        |input_label, ref_label, path| {
            trans_calls.push((input_label.to_string(), ref_label.to_string()));
            *path = match (input_label, ref_label) {
                ("f1", "m1") => "BXB".to_string(),
                ("f2", "m2") => "YBB".to_string(),
                _ => panic!("unexpected transaln labels {input_label} {ref_label}"),
            };
        },
    );
    assert_eq!(
        trans_calls,
        vec![
            ("f1".to_string(), "m1".to_string()),
            ("f2".to_string(), "m2".to_string()),
        ]
    );
    assert_eq!(cmd_ta.fresh_index_to_msa_index, vec![0, 1]);
    assert_eq!(cmd_ta.pw_paths, vec!["BXB".to_string(), "YBB".to_string()]);
    assert_eq!(
        cmd_extended
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A--C", "A-GC", "AT-C", "-G-C"]
    );
    assert_eq!(
        std::fs::read_to_string(&trans_out).unwrap(),
        ">m1\nA--C\n>m2\nA-GC\n>f1\nAT-C\n>f2\n-G-C\n"
    );
    std::fs::remove_file(&trans_input).unwrap();
    std::fs::remove_file(&trans_out).unwrap();

    let trans_refadd = std::env::temp_dir().join(format!(
        "muscle_rs_transalnref_add_{}.fa",
        std::process::id()
    ));
    let trans_refout = std::env::temp_dir().join(format!(
        "muscle_rs_transalnref_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(&trans_refadd, b">f1\nATC\n").unwrap();
    let mut trans_ref_calls = Vec::new();
    let (ref_ta, ref_extended, ref_log) = cmd_transalnref(
        trans_ref.to_str().unwrap(),
        trans_refadd.to_str().unwrap(),
        "m1",
        trans_refout.to_str().unwrap(),
        |seqr, lr, seqa, la| {
            trans_ref_calls.push((
                std::str::from_utf8(&seqr[..lr as usize])
                    .unwrap()
                    .to_string(),
                std::str::from_utf8(&seqa[..la as usize])
                    .unwrap()
                    .to_string(),
            ));
            PathInfo {
                path: "MIM".to_string(),
                ..PathInfo::default()
            }
        },
    );
    assert_eq!(trans_ref_calls, vec![("AC".to_string(), "ATC".to_string())]);
    assert_eq!(ref_ta.fresh_index_to_msa_index, vec![0]);
    assert_eq!(ref_ta.pw_paths, vec!["BXB".to_string()]);
    assert_eq!(
        ref_extended
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A--C", "A-GC", "AT-C"]
    );
    assert_eq!(
        std::fs::read_to_string(&trans_refout).unwrap(),
        ">m1\nA--C\n>m2\nA-GC\n>f1\nAT-C\n"
    );
    assert_eq!(ref_log, "ref m1, add f1 (66.7% id)\nDone.\n");
    std::fs::remove_file(&trans_ref).unwrap();
    std::fs::remove_file(&trans_refadd).unwrap();
    std::fs::remove_file(&trans_refout).unwrap();
}

#[test]
fn sw_tester_fix_gaps_and_stats_match_cpp_helpers() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    reset_rand(1);
    let mut aln_bar = "--A|---|--C".to_string();
    sw_tester_fix_gaps(&mut aln_bar);
    let rows = split(&aln_bar, '|');
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.len() == 3));
    for col in 0..3 {
        assert!(rows.iter().any(|row| row.as_bytes()[col] != b'-'));
    }
    reset_rand(1);
    assert_eq!(sw_tester_get_random_seq(6, false), "SLWKSL");
    reset_rand(1);
    let gapped = sw_tester_get_random_seq(12, true);
    assert_eq!(gapped.len(), 12);
    assert!(gapped.contains('-'));
    assert!(gapped.chars().all(|c| c == '-' || AMINO_ALPHA.contains(c)));

    let swt = SWTester {
        n: 7,
        n_agree: 5,
        n_score_diff: 1,
        n_path_diff: 2,
        n_pos_diff: 3,
        n_ps_score_ok: 4,
        n_ps_score_diff: 6,
        ..SWTester::default()
    };
    assert_eq!(
        sw_tester_stats(&swt),
        "\n         7  Tests\n         5  Agree\n         1  Score diff\n         2  Path diff\n         3  Pos diff\n         4  PS score ok\n         6  PS score diff\n"
    );

    let mut runx = SWTester::default();
    sw_tester_run_x(
        &mut runx,
        "AB|CD",
        "WXYZ",
        |s, lo_a, lo_b, path| {
            assert_eq!(s.rows_a, vec!["AB".to_string(), "CD".to_string()]);
            *lo_a = 0;
            *lo_b = 1;
            *path = "MI".to_string();
            2.5
        },
        |lo_a, lo_b, path| {
            assert_eq!(lo_a, 0);
            assert_eq!(lo_b, 1);
            assert_eq!(path, "MI");
            2.5
        },
    );
    assert_eq!(runx.a, "AB|CD");
    assert_eq!(runx.b, "WXYZ");
    assert_eq!(runx.x_score, 2.5);
    assert_eq!(runx.x_lo_a, 0);
    assert_eq!(runx.x_lo_b, 1);
    assert_eq!(runx.x_path, "MI");
    assert_eq!(runx.n_ps_score_ok, 1);
    assert_eq!(runx.n_ps_score_diff, 0);

    sw_tester_run_x(
        &mut runx,
        "AA",
        "BB",
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_lo_a, _lo_b, _path| 0.5,
    );
    assert_eq!(runx.n_ps_score_ok, 1);
    assert_eq!(runx.n_ps_score_diff, 1);

    let mut runy = SWTester::default();
    sw_tester_run_y(&mut runy, "AC", "GT", |_s, lo_a, lo_b, path| {
        *lo_a = 1;
        *lo_b = 1;
        *path = "M".to_string();
        4.0
    });
    assert_eq!(runy.a, "AC");
    assert_eq!(runy.b, "GT");
    assert_eq!(runy.y_score, 4.0);
    assert_eq!(runy.y_lo_a, 1);
    assert_eq!(runy.y_lo_b, 1);
    assert_eq!(runy.y_path, "M");

    let mut runxab = SWTester::default();
    let mut ps_calls = Vec::new();
    let runxab_log = sw_tester_run_xab(
        &mut runxab,
        "XName",
        "AB",
        "CD",
        true,
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            3.0
        },
        |lo_a, lo_b, path, trace| {
            ps_calls.push((lo_a, lo_b, path.to_string(), trace));
            if trace { 2.75 } else { 3.0 }
        },
    )
    .unwrap();
    assert_eq!(
        ps_calls,
        vec![
            (0, 0, "M".to_string(), false),
            (0, 0, "M".to_string(), true),
        ]
    );
    assert_eq!(runxab.n_ps_score_ok, 1);
    assert!(runxab_log.contains("\nRunXAB(XName)\nA AB\nB CD\n"));
    assert!(runxab_log.contains("ScoreSW 3, ScorePS 2.75   M\n"));

    let mut agree = SWTester {
        x_score: 1.0,
        y_score: 1.0,
        x_lo_a: 2,
        y_lo_a: 2,
        x_lo_b: 3,
        y_lo_b: 3,
        x_path: "MB".to_string(),
        y_path: "MB".to_string(),
        ..SWTester::default()
    };
    assert_eq!(sw_tester_cmp_xy(&mut agree), None);
    assert_eq!(agree.n, 1);
    assert_eq!(agree.n_agree, 1);

    let mut diff = SWTester {
        a: "ABC".to_string(),
        b: "ABD".to_string(),
        x_score: 4.0,
        y_score: 3.0,
        x_lo_a: 1,
        y_lo_a: 2,
        x_lo_b: 0,
        y_lo_b: 0,
        x_path: "MB".to_string(),
        y_path: "MI".to_string(),
        ..SWTester::default()
    };
    let log = sw_tester_cmp_xy(&mut diff).unwrap();
    assert!(log.starts_with("@SCOREDIFF"));
    assert!(log.contains("A: ABC\nB: ABD\n"));
    assert!(log.contains("  4/3  loa 1,2  lob 0,0  MB,MI\n"));
    assert_eq!(diff.n, 1);
    assert_eq!(diff.n_agree, 0);
    assert_eq!(diff.n_score_diff, 1);
    assert_eq!(diff.n_path_diff, 1);
    assert_eq!(diff.n_pos_diff, 1);

    let mut runxy = SWTester::default();
    let runxy_log = sw_tester_run_xy(
        &mut runxy,
        "AC",
        "GT",
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            2.0
        },
        |_s, lo_a, lo_b, path| {
            *lo_a = 1;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_lo_a, _lo_b, _path| 2.0,
    )
    .unwrap();
    assert!(runxy_log.starts_with("@SCOREDIFF"));
    assert_eq!(runxy.n, 1);
    assert_eq!(runxy.n_score_diff, 1);
    assert_eq!(runxy.n_pos_diff, 1);

    let mut runab = SWTester::default();
    let runab_log = sw_tester_run_ab(
        &mut runab,
        "AC",
        "GT",
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            3.0
        },
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            3.0
        },
        |_lo_a, _lo_b, _path| 3.0,
    );
    assert_eq!(runab_log, None);
    assert_eq!(runab.n, 1);
    assert_eq!(runab.n_agree, 1);

    reset_rand(1);
    let mut random_seq_tester = SWTester::default();
    let random_seq_log = sw_tester_run_random_seqs(
        &mut random_seq_tester,
        3,
        3,
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_lo_a, _lo_b, _path| 1.0,
    );
    assert_eq!(random_seq_log, None);
    assert_eq!(random_seq_tester.n, 1);
    assert_eq!(random_seq_tester.n_agree, 1);
    assert_eq!(random_seq_tester.a.len(), 3);
    assert_eq!(random_seq_tester.b.len(), 3);

    reset_rand(1);
    let mut random_iter_tester = SWTester::default();
    let random_iter_log = sw_tester_run_random_seqs_iters(
        &mut random_iter_tester,
        3,
        3,
        2,
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_lo_a, _lo_b, _path| 1.0,
    );
    assert_eq!(random_iter_log, "");
    assert_eq!(random_iter_tester.n, 2);
    assert_eq!(random_iter_tester.n_agree, 2);

    reset_rand(1);
    let mut random_msa_tester = SWTester::default();
    let random_msa_log = sw_tester_run_random_msa_seq(
        &mut random_msa_tester,
        2,
        2,
        3,
        3,
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_lo_a, _lo_b, _path| 1.0,
    );
    assert_eq!(random_msa_log, None);
    assert_eq!(random_msa_tester.n, 1);
    assert!(random_msa_tester.a.contains('|'));

    reset_rand(1);
    let mut random_msa_iter_tester = SWTester::default();
    let random_msa_iter_log = sw_tester_run_random_msa_seq_iters(
        &mut random_msa_iter_tester,
        2,
        2,
        3,
        3,
        2,
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_s, lo_a, lo_b, path| {
            *lo_a = 0;
            *lo_b = 0;
            *path = "M".to_string();
            1.0
        },
        |_lo_a, _lo_b, _path| 1.0,
    );
    assert_eq!(random_msa_iter_log, "");
    assert_eq!(random_msa_iter_tester.n, 2);
    assert_eq!(random_msa_iter_tester.n_agree, 2);

    let bug_log = bug_l13();
    assert!(bug_log.contains("         X  SWer\n         Y  SWer\n"));
    assert!(bug_log.contains("         1  Tests\n"));
    assert!(bug_log.contains("         1  Agree\n"));

    let bug_mm_log = bug_swtestmm_l13();
    assert!(bug_mm_log.contains("         X  SWer\n         Y  SWer\n"));
    assert!(bug_mm_log.contains("         1  Tests\n"));
    assert!(bug_mm_log.contains("         1  Agree\n"));

    assert_eq!(
        cmd_swtest(),
        "\n         X  SWer\n         Y  SWer\n         1  Tests\n         1  Agree\n         0  Score diff\n         0  Path diff\n         0  Pos diff\n         1  PS score ok\n         0  PS score diff\n"
    );
    reset_rand(1);
    let swtestmm_log = cmd_swtestmm();
    assert!(swtestmm_log.contains("      1000  Tests\n"));
    assert!(swtestmm_log.contains("      1000  Agree\n"));
    assert!(swtestmm_log.contains("         0  Score diff\n"));
}

#[test]
fn log_tba_matches_cpp_traceback_display_mapping() {
    let a = vec![vec![0.0, 1234.0], vec![-2.5, 0.00125]];
    let tb = vec![vec!['D', 'L'], vec!['U', 'M']];
    assert_eq!(
        log_tba(&a, &tb, 1, 1),
        "\n                0         1\n[  0]           0  1.23e+03\n[  1]        -2.5   0.00125\n\n          0   1\n[  0]    B   Y \n[  1]    X   M \n"
    );
}

#[test]
fn platform_helpers_have_cpp_shape() {
    assert_eq!(cmd_build_guide_tree(), "");
    let mut io_buffer = alloc_buffer();
    assert_eq!(io_buffer.len(), 32_000);
    free_buffer(&mut io_buffer);
    assert!(io_buffer.is_empty());
    set_pcb();
    mysleep_l952(0);
    mysleep_l957(0);
    assert!(!get_platform().is_empty());
    assert!(!myisatty(-1));
    assert_eq!(get_struct_pack(), 1);
    assert!(get_cpu_core_count() >= 1);
    assert!(get_requested_thread_count() >= 1);
    assert!(get_usable_mem_bytes() >= 0.0);
    assert!(get_phys_mem_bytes_l979() >= 0.0);
    assert!(get_phys_mem_bytes_l990() >= 0.0);
    assert!(get_phys_mem_bytes_l1079() >= 0.0);
    assert!(get_mem_use_bytes_l964() >= 0.0);
    assert!(get_mem_use_bytes_l1008() >= 0.0);
    assert!(get_mem_use_bytes_l1060() >= 0.0);
    assert_eq!(get_mem_use_bytes_l1089(), 0.0);
    assert_eq!(get_size_from_str("file.cpp:12=123.5"), 123.5);
    assert_eq!(get_size_from_str("7.25"), 7.25);
    assert_eq!(get_size_from_str("x=12.5kb"), 12.5);
    assert_eq!(get_size_from_str("x=abc"), 0.0);
    let seek_file = std::env::temp_dir().join(format!("muscle_rs_seek_{}.txt", std::process::id()));
    std::fs::write(&seek_file, b"abcdef").unwrap();
    let mut seek_handle = std::fs::File::open(&seek_file).unwrap();
    assert_eq!(fseeko(&mut seek_handle, 2, 0), 0);
    assert_eq!(fseeko(&mut seek_handle, 1, 1), 0);
    assert_eq!(fseeko(&mut seek_handle, -1, 2), 0);
    assert_eq!(fseeko(&mut seek_handle, 0, 99), -1);
    std::fs::remove_file(&seek_file).unwrap();
    let seen_threads = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen_threads2 = seen_threads.clone();
    run_threads(move |thread_index| {
        seen_threads2.lock().unwrap().push(thread_index);
    });
    let mut seen = seen_threads.lock().unwrap().clone();
    seen.sort_unstable();
    assert_eq!(seen, (0..get_requested_thread_count()).collect::<Vec<_>>());
    let gtb_mem = get_dp_mem_l18();
    assert_eq!(gtb_mem.max_la, 0);
    assert_eq!(gtb_mem.max_lb, 0);
    assert!(gtb_mem.tb_bit.is_empty());
    let uclustpd_mem = get_dp_mem_l17();
    assert_eq!(uclustpd_mem.max_la, 0);
    assert_eq!(uclustpd_mem.max_lb, 0);
    assert!(uclustpd_mem.buffer1.is_empty());
    let args_file =
        std::env::temp_dir().join(format!("muscle_rs_args_file_{}.txt", std::process::id()));
    std::fs::write(&args_file, "  -a  12 # comment\n\n-b\txyz\n# full\n-c\n").unwrap();
    assert_eq!(
        get_args_from_file(args_file.to_str().unwrap()),
        vec![
            "-a".to_string(),
            "12".to_string(),
            "-b".to_string(),
            "xyz".to_string(),
            "-c".to_string()
        ]
    );
    std::fs::remove_file(&args_file).unwrap();
}

#[test]
fn original_cpp_binary_matches_rust_on_stable_real_data_commands() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let Some(cpp_bin) = original_muscle_bin() else {
        eprintln!(
            "skipping original C++ parity test: MUSCLE_CPP_BIN not set and default binary absent"
        );
        return;
    };

    let stats_ref = cmd_msastats("muscle/test_data/ref_alns/BB11001", Some(0.5));
    let stats_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-msastats",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "0.5",
        ])
        .output()
        .unwrap();
    assert!(
        stats_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&stats_cpp.stdout),
        String::from_utf8_lossy(&stats_cpp.stderr)
    );
    assert_eq!(
        original_command_body(&stats_cpp),
        normalize_command_text(&stats_ref)
    );

    let qscore2_ref = cmd_qscore2(
        "muscle/test_data/ref_alns/BB11001",
        "muscle/test_data/ref_alns/BB11001",
        1.0,
    );
    let qscore2_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-qscore2",
            "muscle/test_data/ref_alns/BB11001",
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        qscore2_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&qscore2_cpp.stdout),
        String::from_utf8_lossy(&qscore2_cpp.stderr)
    );
    assert_eq!(
        original_command_body(&qscore2_cpp),
        normalize_command_text(&qscore2_ref)
    );

    let root = std::env::temp_dir().join(format!(
        "muscle_rs_original_cpp_parity_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();

    let cmp_test = root.join("cmp_test.fa");
    let cmp_ref = root.join("cmp_ref.fa");
    std::fs::write(&cmp_test, b">seq1\nAC-G\n>seq2\nAT-G\n").unwrap();
    std::fs::write(&cmp_ref, b">seq1\nAC-G\n>seq2\nAT-G\n").unwrap();

    let cmp_ref_msas_ref_full =
        cmd_cmp_ref_msas(cmp_test.to_str().unwrap(), cmp_ref.to_str().unwrap(), 1.0).1;
    let cmp_ref_msas_ref = format!("{}\n", cmp_ref_msas_ref_full.lines().next().unwrap());
    let cmp_ref_msas_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-cmp_ref_msas",
            cmp_test.to_str().unwrap(),
            "-ref",
            cmp_ref.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        cmp_ref_msas_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&cmp_ref_msas_cpp.stdout),
        String::from_utf8_lossy(&cmp_ref_msas_cpp.stderr)
    );
    assert_eq!(
        original_command_body(&cmp_ref_msas_cpp),
        normalize_command_text(&cmp_ref_msas_ref)
    );
    let cmp_ref_msas_rust = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-cmp_ref_msas",
            cmp_test.to_str().unwrap(),
            "-ref",
            cmp_ref.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        cmp_ref_msas_rust.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&cmp_ref_msas_rust.stdout),
        String::from_utf8_lossy(&cmp_ref_msas_rust.stderr)
    );
    assert_eq!(
        String::from_utf8(cmp_ref_msas_rust.stdout).unwrap(),
        original_command_body(&cmp_ref_msas_cpp)
    );

    let cpp_cmp_msa = root.join("cpp.cmp_msa.html");
    let rust_cmp_msa = root.join("rust.cmp_msa.html");
    let cmp_msa_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-cmp_msa",
            cmp_test.to_str().unwrap(),
            "-ref",
            cmp_ref.to_str().unwrap(),
            "-output",
            cpp_cmp_msa.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        cmp_msa_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&cmp_msa_cpp.stdout),
        String::from_utf8_lossy(&cmp_msa_cpp.stderr)
    );
    let cmp_msa_rust = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-cmp_msa",
            cmp_test.to_str().unwrap(),
            "-ref",
            cmp_ref.to_str().unwrap(),
            "-output",
            rust_cmp_msa.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        cmp_msa_rust.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&cmp_msa_rust.stdout),
        String::from_utf8_lossy(&cmp_msa_rust.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&rust_cmp_msa).unwrap(),
        std::fs::read_to_string(&cpp_cmp_msa).unwrap()
    );

    let cpp_a2m = root.join("cpp.a2m");
    let rust_a2m = root.join("rust.a2m");
    let make_a2m_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-make_a2m",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_a2m.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        make_a2m_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&make_a2m_cpp.stdout),
        String::from_utf8_lossy(&make_a2m_cpp.stderr)
    );
    cmd_make_a2m(
        "muscle/test_data/ref_alns/BB11001",
        rust_a2m.to_str().unwrap(),
        0.5,
        false,
    );
    assert_eq!(
        std::fs::read_to_string(&rust_a2m).unwrap(),
        std::fs::read_to_string(&cpp_a2m).unwrap()
    );

    let cpp_strip = root.join("cpp.strip.fa");
    let rust_strip = root.join("rust.strip.fa");
    let strip_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-strip_gappy",
            "muscle/test_data/ref_alns/BB11005",
            "-output",
            cpp_strip.to_str().unwrap(),
            "-max_gap_fract",
            "0.5",
            "-max_gap_fract_row",
            "0.5",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        strip_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&strip_cpp.stdout),
        String::from_utf8_lossy(&strip_cpp.stderr)
    );
    cmd_strip_gappy(
        "muscle/test_data/ref_alns/BB11005",
        rust_strip.to_str().unwrap(),
        0.5,
        0.5,
    );
    assert_eq!(
        std::fs::read_to_string(&rust_strip).unwrap(),
        std::fs::read_to_string(&cpp_strip).unwrap()
    );

    let cpp_squeeze = root.join("cpp.squeeze.fa");
    let rust_squeeze = root.join("rust.squeeze.fa");
    let squeeze_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-squeeze_inserts",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_squeeze.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        squeeze_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&squeeze_cpp.stdout),
        String::from_utf8_lossy(&squeeze_cpp.stderr)
    );
    let squeezed = cmd_squeeze_inserts(
        "muscle/test_data/ref_alns/BB11001",
        rust_squeeze.to_str().unwrap(),
    );
    assert_eq!(squeezed.seqs.len(), 4);
    assert_eq!(
        std::fs::read_to_string(&rust_squeeze).unwrap(),
        std::fs::read_to_string(&cpp_squeeze).unwrap()
    );

    let cpp_trim = root.join("cpp.trim.fa");
    let rust_trim = root.join("rust.trim.fa");
    let trim_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-trimtoref",
            "muscle/test_data/ref_alns/BB11001",
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_trim.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        trim_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&trim_cpp.stdout),
        String::from_utf8_lossy(&trim_cpp.stderr)
    );
    let trimmed = cmd_trimtoref(
        "muscle/test_data/ref_alns/BB11001",
        "muscle/test_data/ref_alns/BB11001",
        rust_trim.to_str().unwrap(),
    );
    assert_eq!(trimmed.seqs.len(), 4);
    assert_eq!(
        std::fs::read_to_string(&rust_trim).unwrap(),
        std::fs::read_to_string(&cpp_trim).unwrap()
    );

    let efa_paths = root.join("efa_paths.txt");
    let efa_file = root.join("input.efa");
    std::fs::write(
        &efa_paths,
        b"muscle/test_data/ref_alns/BB11001\nmuscle/test_data/ref_alns/BB11001\n",
    )
    .unwrap();
    cmd_fa2efa(
        efa_paths.to_str().unwrap(),
        efa_file.to_str().unwrap(),
        true,
        true,
    );

    let qscore_efa_ref = cmd_qscore_efa(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        1.0,
    );
    let qscore_efa_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-qscore_efa",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        qscore_efa_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&qscore_efa_cpp.stdout),
        String::from_utf8_lossy(&qscore_efa_cpp.stderr)
    );
    assert_eq!(
        original_command_body(&qscore_efa_cpp),
        normalize_command_text(&qscore_efa_ref)
    );
    let qscore_efa_rust = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-qscore_efa",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        qscore_efa_rust.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&qscore_efa_rust.stdout),
        String::from_utf8_lossy(&qscore_efa_rust.stderr)
    );
    assert_eq!(
        String::from_utf8(qscore_efa_rust.stdout).unwrap(),
        original_command_body(&qscore_efa_cpp)
    );

    let efastats_ref = cmd_efastats(
        efa_file.to_str().unwrap(),
        1.0,
        Some("muscle/test_data/ref_alns/BB11001"),
    );
    let efastats_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-efastats",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        efastats_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&efastats_cpp.stdout),
        String::from_utf8_lossy(&efastats_cpp.stderr)
    );
    let mut efastats_cpp_body = String::new();
    for line in original_command_body(&efastats_cpp).lines() {
        if line.contains("Pairwise dists") {
            continue;
        }
        if let Some(pos) = line.find("4 seqs, 2 MSAs") {
            efastats_cpp_body.push_str(&line[pos..]);
        } else {
            efastats_cpp_body.push_str(line);
        }
        efastats_cpp_body.push('\n');
    }
    assert_eq!(efastats_cpp_body, normalize_command_text(&efastats_ref));
    let efastats_rust = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-efastats",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        efastats_rust.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&efastats_rust.stdout),
        String::from_utf8_lossy(&efastats_rust.stderr)
    );
    assert_eq!(
        String::from_utf8(efastats_rust.stdout).unwrap(),
        normalize_command_text(&efastats_ref)
    );

    let efastats_no_ref = cmd_efastats(efa_file.to_str().unwrap(), 1.0, None);
    let efastats_no_ref_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-efastats",
            efa_file.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        efastats_no_ref_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&efastats_no_ref_cpp.stdout),
        String::from_utf8_lossy(&efastats_no_ref_cpp.stderr)
    );
    let mut efastats_no_ref_cpp_body = String::new();
    for line in original_command_body(&efastats_no_ref_cpp).lines() {
        if line.contains("Pairwise dists") {
            continue;
        }
        if let Some(pos) = line.find("4 seqs, 2 MSAs") {
            efastats_no_ref_cpp_body.push_str(&line[pos..]);
        } else {
            efastats_no_ref_cpp_body.push_str(line);
        }
        efastats_no_ref_cpp_body.push('\n');
    }
    assert_eq!(
        efastats_no_ref_cpp_body,
        normalize_command_text(&efastats_no_ref)
    );

    let disperse_ref = cmd_disperse(efa_file.to_str().unwrap(), 1.0);
    let disperse_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-disperse",
            efa_file.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        disperse_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&disperse_cpp.stdout),
        String::from_utf8_lossy(&disperse_cpp.stderr)
    );
    let mut disperse_cpp_body = String::new();
    for line in original_command_body(&disperse_cpp).lines() {
        if line.contains("Pairwise dists") {
            continue;
        }
        disperse_cpp_body.push_str(line);
        disperse_cpp_body.push('\n');
    }
    assert_eq!(disperse_cpp_body, normalize_command_text(&disperse_ref));
    let disperse_rust = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-disperse",
            efa_file.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
        ])
        .output()
        .unwrap();
    assert!(
        disperse_rust.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&disperse_rust.stdout),
        String::from_utf8_lossy(&disperse_rust.stderr)
    );
    assert_eq!(
        String::from_utf8(disperse_rust.stdout).unwrap(),
        normalize_command_text(&disperse_ref)
    );

    let cpp_explode_prefix = root.join("cpp.explode.");
    let rust_explode_prefix = root.join("rust.explode.");
    let explode_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-efa_explode",
            efa_file.to_str().unwrap(),
            "-prefix",
            cpp_explode_prefix.to_str().unwrap(),
            "-suffix",
            ".fa",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        explode_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&explode_cpp.stdout),
        String::from_utf8_lossy(&explode_cpp.stderr)
    );
    let exploded = cmd_efa_explode(
        efa_file.to_str().unwrap(),
        Some(rust_explode_prefix.to_str().unwrap()),
        Some(".fa"),
    );
    assert_eq!(exploded.len(), 2);
    for suffix in ["BB11001.0.fa", "BB11001.1.fa"] {
        assert_eq!(
            std::fs::read_to_string(root.join(format!("rust.explode.{suffix}"))).unwrap(),
            std::fs::read_to_string(root.join(format!("cpp.explode.{suffix}"))).unwrap()
        );
    }

    let cpp_resample = root.join("cpp.resample.efa");
    let rust_resample = root.join("rust.resample.efa");
    let resample_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-resample",
            efa_file.to_str().unwrap(),
            "-output",
            cpp_resample.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-minconf",
            "1.0",
            "-replicates",
            "2",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        resample_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&resample_cpp.stdout),
        String::from_utf8_lossy(&resample_cpp.stderr)
    );
    reset_rand(1);
    let resampled = cmd_resample(
        efa_file.to_str().unwrap(),
        rust_resample.to_str().unwrap(),
        1.0,
        1.0,
        2,
    );
    assert_eq!(resampled.len(), 2);
    assert_eq!(
        std::fs::read_to_string(&rust_resample).unwrap(),
        std::fs::read_to_string(&cpp_resample).unwrap()
    );

    let cpp_trimtoref_efa = root.join("cpp.trimtoref.efa");
    let rust_trimtoref_efa = root.join("rust.trimtoref.efa");
    let trimtoref_efa_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-trimtoref_efa",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_trimtoref_efa.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        trimtoref_efa_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&trimtoref_efa_cpp.stdout),
        String::from_utf8_lossy(&trimtoref_efa_cpp.stderr)
    );
    let trimmed_efa = cmd_trimtoref_efa(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        rust_trimtoref_efa.to_str().unwrap(),
    );
    assert_eq!(trimmed_efa.msas.len(), 2);
    assert_eq!(
        std::fs::read_to_string(&rust_trimtoref_efa).unwrap(),
        std::fs::read_to_string(&cpp_trimtoref_efa).unwrap()
    );

    let cpp_addconf = root.join("cpp.addconf.efa");
    let rust_addconf = root.join("rust.addconf.efa");
    let addconf_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-addconfseq",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_addconf.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        addconf_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&addconf_cpp.stdout),
        String::from_utf8_lossy(&addconf_cpp.stderr)
    );
    let addconf = cmd_addconfseq(
        efa_file.to_str().unwrap(),
        rust_addconf.to_str().unwrap(),
        Some("muscle/test_data/ref_alns/BB11001"),
        None,
        false,
    );
    assert!(addconf.contains(">_conf_\n"));
    assert_eq!(
        std::fs::read_to_string(&rust_addconf).unwrap(),
        std::fs::read_to_string(&cpp_addconf).unwrap()
    );

    let cpp_addconf_one = root.join("cpp.addconf_one.efa");
    let rust_addconf_one = root.join("rust.addconf_one.efa");
    let addconf_one_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-addconfseq",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_addconf_one.to_str().unwrap(),
            "-label",
            "custom_conf",
            "-confseq1",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        addconf_one_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&addconf_one_cpp.stdout),
        String::from_utf8_lossy(&addconf_one_cpp.stderr)
    );
    let addconf_one = cmd_addconfseq(
        efa_file.to_str().unwrap(),
        rust_addconf_one.to_str().unwrap(),
        Some("muscle/test_data/ref_alns/BB11001"),
        Some("custom_conf"),
        true,
    );
    assert!(addconf_one.contains(">custom_conf\n"));
    assert!(!addconf_one.contains(">custom_conf2\n"));
    assert_eq!(
        std::fs::read_to_string(&rust_addconf_one).unwrap(),
        std::fs::read_to_string(&cpp_addconf_one).unwrap()
    );

    let cpp_addletter = root.join("cpp.addletter.fa");
    let rust_addletter = root.join("rust.addletter.fa");
    let addletter_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-addletterconfseq",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_addletter.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        addletter_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&addletter_cpp.stdout),
        String::from_utf8_lossy(&addletter_cpp.stderr)
    );
    let addletter = cmd_addletterconfseq(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        rust_addletter.to_str().unwrap(),
        1.0,
    );
    assert!(addletter.contains(">_letterconf_\n"));
    assert_eq!(
        std::fs::read_to_string(&rust_addletter).unwrap(),
        std::fs::read_to_string(&cpp_addletter).unwrap()
    );
    let rust_addletter_cli = root.join("rust.addletter_cli.fa");
    let addletter_rust_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-addletterconfseq",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            rust_addletter_cli.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        addletter_rust_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&addletter_rust_cli.stdout),
        String::from_utf8_lossy(&addletter_rust_cli.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&rust_addletter_cli).unwrap(),
        std::fs::read_to_string(&cpp_addletter).unwrap()
    );

    let cpp_letterconf = root.join("cpp.letterconf.fa");
    let rust_letterconf = root.join("rust.letterconf.fa");
    let cpp_letterconf_jalview = root.join("cpp.letterconf.features");
    let rust_letterconf_jalview = root.join("rust.letterconf.features");
    let letterconf_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-letterconf",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_letterconf.to_str().unwrap(),
            "-jalview",
            cpp_letterconf_jalview.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        letterconf_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&letterconf_cpp.stdout),
        String::from_utf8_lossy(&letterconf_cpp.stderr)
    );
    let letterconf = cmd_letterconf(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        rust_letterconf.to_str().unwrap(),
        "",
        rust_letterconf_jalview.to_str().unwrap(),
        1.0,
    );
    assert_eq!(letterconf.seqs.len(), 4);
    assert_eq!(
        std::fs::read_to_string(&rust_letterconf).unwrap(),
        std::fs::read_to_string(&cpp_letterconf).unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(&rust_letterconf_jalview).unwrap(),
        std::fs::read_to_string(&cpp_letterconf_jalview).unwrap()
    );

    let letterconf_html_input = root.join("letterconf_input.fa");
    let letterconf_html_ref = root.join("letterconf_ref.fa");
    let cpp_letterconf_html2 = root.join("cpp.letterconf2.html");
    let rust_letterconf_html2 = root.join("rust.letterconf2.html");
    std::fs::write(&letterconf_html_input, b">a\n9-\n>b\n8-\n").unwrap();
    std::fs::write(&letterconf_html_ref, b">a\nAC\n>b\nTG\n").unwrap();
    let letterconf_html_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-letterconf_html",
            letterconf_html_input.to_str().unwrap(),
            "-ref",
            letterconf_html_ref.to_str().unwrap(),
            "-output",
            cpp_letterconf_html2.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        letterconf_html_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&letterconf_html_cpp.stdout),
        String::from_utf8_lossy(&letterconf_html_cpp.stderr)
    );
    cmd_letterconf_html(
        letterconf_html_input.to_str().unwrap(),
        letterconf_html_ref.to_str().unwrap(),
        rust_letterconf_html2.to_str().unwrap(),
    );
    assert_eq!(
        std::fs::read_to_string(&rust_letterconf_html2).unwrap(),
        std::fs::read_to_string(&cpp_letterconf_html2).unwrap()
    );

    let relabel_in = root.join("relabel.fa");
    let relabel_labels = root.join("relabel.tsv");
    let cpp_relabel = root.join("cpp.relabel.fa");
    let rust_relabel = root.join("rust.relabel.fa");
    std::fs::write(&relabel_in, b">old1\nACGT\n>old2\nTGCA\n>keep\nNNNN\n").unwrap();
    std::fs::write(&relabel_labels, b"old1\tnew1\nold2\tnew2\n").unwrap();
    let relabel_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-relabel",
            relabel_in.to_str().unwrap(),
            "-labels2",
            relabel_labels.to_str().unwrap(),
            "-output",
            cpp_relabel.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        relabel_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&relabel_cpp.stdout),
        String::from_utf8_lossy(&relabel_cpp.stderr)
    );
    let relabeled = cmd_relabel(
        relabel_in.to_str().unwrap(),
        relabel_labels.to_str().unwrap(),
        rust_relabel.to_str().unwrap(),
    );
    assert!(relabeled.contains(">new1\n"));
    assert_eq!(
        std::fs::read_to_string(&rust_relabel).unwrap(),
        std::fs::read_to_string(&cpp_relabel).unwrap()
    );
    let rust_relabel_cli = root.join("rust.relabel_cli.fa");
    let relabel_rust_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-relabel",
            relabel_in.to_str().unwrap(),
            "--labels2",
            relabel_labels.to_str().unwrap(),
            "-output",
            rust_relabel_cli.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        relabel_rust_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&relabel_rust_cli.stdout),
        String::from_utf8_lossy(&relabel_rust_cli.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&rust_relabel_cli).unwrap(),
        std::fs::read_to_string(&cpp_relabel).unwrap()
    );

    let rust_consseq = root.join("rust.consseq.fa");
    set_alpha_l209(ALPHA::ALPHA_Amino);
    cmd_consseq(
        "muscle/test_data/ref_alns/BB11001",
        rust_consseq.to_str().unwrap(),
        Some("BB11001_cons"),
    );
    let cpp_strip_cols = root.join("cpp.strip_cols.fa");
    let rust_strip_cols = root.join("rust.strip_cols.fa");
    let strip_cols_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-strip_gappy_cols",
            "muscle/test_data/ref_alns/BB11005",
            "-output",
            cpp_strip_cols.to_str().unwrap(),
            "-max_gap_fract",
            "0.5",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        strip_cols_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&strip_cols_cpp.stdout),
        String::from_utf8_lossy(&strip_cols_cpp.stderr)
    );
    let gappy_cols = cmd_strip_gappy_cols(
        "muscle/test_data/ref_alns/BB11005",
        rust_strip_cols.to_str().unwrap(),
        0.5,
    );
    assert!(gappy_cols > 0);
    assert_eq!(
        std::fs::read_to_string(&rust_strip_cols).unwrap(),
        std::fs::read_to_string(&cpp_strip_cols).unwrap()
    );

    let cpp_strip_rows = root.join("cpp.strip_rows.fa");
    let rust_strip_rows = root.join("rust.strip_rows.fa");
    let strip_rows_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-strip_gappy_rows",
            "muscle/test_data/ref_alns/BB11005",
            "-output",
            cpp_strip_rows.to_str().unwrap(),
            "-max_gap_fract",
            "0.5",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        strip_rows_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&strip_rows_cpp.stdout),
        String::from_utf8_lossy(&strip_rows_cpp.stderr)
    );
    let gappy_rows = cmd_strip_gappy_rows(
        "muscle/test_data/ref_alns/BB11005",
        rust_strip_rows.to_str().unwrap(),
        0.5,
    );
    assert!(gappy_rows <= 4);
    assert_eq!(
        std::fs::read_to_string(&rust_strip_rows).unwrap(),
        std::fs::read_to_string(&cpp_strip_rows).unwrap()
    );

    let cpp_a2m_refseq = root.join("cpp.a2m_refseq.fa");
    let rust_a2m_refseq = root.join("rust.a2m_refseq.fa");
    let a2m_refseq_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-make_a2m_refseq",
            "muscle/test_data/ref_alns/BB11001",
            "-label",
            "1j46_A",
            "-output",
            cpp_a2m_refseq.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        a2m_refseq_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&a2m_refseq_cpp.stdout),
        String::from_utf8_lossy(&a2m_refseq_cpp.stderr)
    );
    cmd_make_a2m_refseq(
        "muscle/test_data/ref_alns/BB11001",
        rust_a2m_refseq.to_str().unwrap(),
        Some("1j46_A"),
        true,
    );
    assert_eq!(
        std::fs::read_to_string(&rust_a2m_refseq).unwrap(),
        std::fs::read_to_string(&cpp_a2m_refseq).unwrap()
    );

    let cpp_bestconf = root.join("cpp.bestconf.fa");
    let rust_bestconf = root.join("rust.bestconf.fa");
    let bestconf_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-efa_bestconf",
            efa_file.to_str().unwrap(),
            "-output",
            cpp_bestconf.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        bestconf_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&bestconf_cpp.stdout),
        String::from_utf8_lossy(&bestconf_cpp.stderr)
    );
    let (_bestconf, best_total, best_median) =
        cmd_efa_bestconf(efa_file.to_str().unwrap(), rust_bestconf.to_str().unwrap());
    assert_eq!((best_total, best_median), (0, 0));
    assert_eq!(
        std::fs::read_to_string(&rust_bestconf).unwrap(),
        std::fs::read_to_string(&cpp_bestconf).unwrap()
    );

    let cpp_bestcols = root.join("cpp.bestcols.fa");
    let rust_bestcols = root.join("rust.bestcols.fa");
    let bestcols_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-efa_bestcols",
            efa_file.to_str().unwrap(),
            "-output",
            cpp_bestcols.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-minconf",
            "1.0",
            "-maxcols",
            "8",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        bestcols_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&bestcols_cpp.stdout),
        String::from_utf8_lossy(&bestcols_cpp.stderr)
    );
    let bestcols = cmd_efa_bestcols(
        efa_file.to_str().unwrap(),
        rust_bestcols.to_str().unwrap(),
        1.0,
        1.0,
        8,
    );
    assert_eq!(bestcols.seqs.len(), 4);
    assert_eq!(multi_sequence_get_col_count(&bestcols), 8);
    assert_eq!(
        std::fs::read_to_string(&rust_bestcols).unwrap(),
        std::fs::read_to_string(&cpp_bestcols).unwrap()
    );

    let cpp_colscore = root.join("cpp.colscore.tsv");
    let rust_colscore = root.join("rust.colscore.tsv");
    let colscore_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-colscore_efa",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-output",
            cpp_colscore.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        colscore_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&colscore_cpp.stdout),
        String::from_utf8_lossy(&colscore_cpp.stderr)
    );
    let colscore = cmd_colscore_efa(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        rust_colscore.to_str().unwrap(),
        1.0,
    );
    assert!(colscore.starts_with("meantc\t1.0000\n"));
    assert_eq!(
        std::fs::read_to_string(&rust_colscore).unwrap(),
        std::fs::read_to_string(&cpp_colscore).unwrap()
    );

    let subst_msa = root.join("subst.fa");
    let subst_list = root.join("subst_list.txt");
    let cpp_subst = root.join("cpp.subst.tsv");
    let rust_subst = root.join("rust.subst.tsv");
    std::fs::write(
        &subst_msa,
        format!(">s1\n{AMINO_ALPHA}\n>s2\n{AMINO_ALPHA}\n"),
    )
    .unwrap();
    std::fs::write(&subst_list, format!("{}\n", subst_msa.to_string_lossy())).unwrap();
    let subst_cpp = std::process::Command::new(&cpp_bin)
        .args([
            "-make_substmx",
            subst_list.to_str().unwrap(),
            "-output",
            cpp_subst.to_str().unwrap(),
            "-label",
            "TESTMX",
            "-minpctid",
            "100",
            "-maxpctid",
            "100",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        subst_cpp.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&subst_cpp.stdout),
        String::from_utf8_lossy(&subst_cpp.stderr)
    );
    let subst_log = cmd_make_substmx(
        subst_list.to_str().unwrap(),
        rust_subst.to_str().unwrap(),
        Some("TESTMX"),
        Some(100),
        Some(100),
    );
    assert!(subst_log.contains("Score matrix"));
    assert_eq!(
        std::fs::read_to_string(&rust_subst).unwrap(),
        std::fs::read_to_string(&cpp_subst).unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn clap_command_fields_are_counted_and_dispatched_exhaustively() {
    let source = include_str!("../src/generated.rs");
    let cli_struct_start = source.find("pub struct MuscleCli").unwrap();
    let cli_struct_end = source[cli_struct_start..]
        .find("#[track_caller]\npub fn main()")
        .map(|offset| cli_struct_start + offset)
        .unwrap();
    let cli_struct = &source[cli_struct_start..cli_struct_end];
    let command_struct_end = cli_struct.find("    pub output: Option<String>,").unwrap();
    let command_struct = &cli_struct[..command_struct_end];

    let mut command_fields = std::collections::BTreeSet::<String>::new();
    for line in command_struct.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pub ") {
            if let Some((name, ty)) = rest.split_once(':') {
                if ty.trim() == "Option<String>," {
                    command_fields.insert(name.to_string());
                }
            }
        }
    }
    assert_eq!(command_fields.len(), 100);

    let count_start = source.find("let cmd_count = [").unwrap();
    let count_end = source[count_start..]
        .find("]\n    .iter()")
        .map(|offset| count_start + offset)
        .unwrap();
    let cmd_count_block = &source[count_start..count_end];

    let mut counted_fields = std::collections::BTreeSet::<String>::new();
    for part in cmd_count_block.split("cli.").skip(1) {
        if let Some((name, rest)) = part.split_once(".is_some()") {
            if rest.starts_with(',') || rest.starts_with('\n') {
                counted_fields.insert(name.to_string());
            }
        }
    }
    assert_eq!(counted_fields, command_fields);

    let dispatch_start = source.find("let out = if").unwrap();
    let dispatch_end = source[dispatch_start..]
        .find("} else if cmd_count == 1")
        .map(|offset| dispatch_start + offset)
        .unwrap();
    let dispatch_block = &source[dispatch_start..dispatch_end];

    let mut dispatched_fields = std::collections::BTreeSet::<String>::new();
    for field in &command_fields {
        let cli_ref = format!("cli.{field}");
        if dispatch_block.contains(&cli_ref) {
            dispatched_fields.insert(field.clone());
        }
    }
    assert_eq!(dispatched_fields, command_fields);
}

#[test]
fn alpha_helpers_match_cpp_classification() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    assert_eq!(get_alpha_size(ALPHA::ALPHA_Amino), 20);
    assert_eq!(get_alpha_size(ALPHA::ALPHA_Nucleo), 4);
    set_alpha_l209(ALPHA::ALPHA_Amino);
    assert_eq!(get_wildcard_char(), 'X');
    let _ = init_probcons();
    {
        let ins = PAIR_HMM_INS_SCORE.lock().unwrap();
        assert!(ins[b'A' as usize].is_finite());
        assert_ne!(ins[b'A' as usize], 0.0);
    }
    set_alpha_l209(ALPHA::ALPHA_Nucleo);
    assert_eq!(get_wildcard_char(), 'N');
    assert!(is_nucleo('A'));
    assert!(is_nucleo('u'));
    assert!(is_nucleo('Y'));
    assert!(!is_nucleo('Z'));
    assert!(is_dna('T'));
    assert!(!is_dna('U'));
    assert!(is_rna('U'));
    assert!(!is_rna('T'));

    let mut alpha_seq = Seq::default();
    seq_from_string(&mut alpha_seq, "TU", "alpha");
    init_arrays();
    set_gap_char('.');
    set_gap_char('-');
    set_alpha_dna();
    assert_eq!(seq_get_letter(&alpha_seq, 0), 3);
    assert_eq!(seq_get_letter(&alpha_seq, 1), INVALID_LETTER);
    init_arrays();
    set_gap_char('.');
    set_gap_char('-');
    set_alpha_rna();
    assert_eq!(seq_get_letter(&alpha_seq, 0), 3);
    assert_eq!(seq_get_letter(&alpha_seq, 1), 3);

    clear_invalid_letter_warning();
    invalid_letter_warning('!', '?');
    invalid_letter_warning('Z', '?');
    assert_eq!(
        report_invalid_letters(),
        Some("Invalid letters found: !Z".to_string())
    );

    let mut msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut msa,
        &["n0".to_string(), "n1".to_string(), "n2".to_string()],
        &["A--G".to_string(), "AG-G".to_string(), "AGTG".to_string()],
    );
    set_alpha_l209(ALPHA::ALPHA_Nucleo);
    assert_eq!(get_cons_char(&msa, 0), 'A');
    assert_eq!(get_cons_char(&msa, 1), 'G');
    assert_eq!(get_cons_char(&msa, 2), '-');
    assert_eq!(get_consensus_sequence(&msa), "AGG");

    let fasta_file =
        std::env::temp_dir().join(format!("muscle_rs_consseq_in_{}.fa", std::process::id()));
    let cons_file =
        std::env::temp_dir().join(format!("muscle_rs_consseq_out_{}.fa", std::process::id()));
    std::fs::write(&fasta_file, b"\n>s1\nA-c.\n>s1\nagt.\n").unwrap();
    let mut loaded = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut loaded, fasta_file.to_str().unwrap(), false);
    assert_eq!(loaded.seqs.len(), 2);
    assert_eq!(loaded.seqs[0].label, "s1");
    assert_eq!(sequence_get_seq_as_string(&loaded.seqs[0]), "A-c.");
    assert_eq!(loaded.seqs[1].label, "s1 dupelabel1");

    let mut stripped = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut stripped, fasta_file.to_str().unwrap(), true);
    assert_eq!(sequence_get_seq_as_string(&stripped.seqs[0]), "AC");

    cmd_consseq(
        fasta_file.to_str().unwrap(),
        cons_file.to_str().unwrap(),
        Some("C"),
    );
    assert_eq!(std::fs::read_to_string(&cons_file).unwrap(), ">C\nAGC\n");
    cmd_consseq(fasta_file.to_str().unwrap(), "", Some("C"));
    std::fs::remove_file(&fasta_file).unwrap();
    std::fs::remove_file(&cons_file).unwrap();
}

#[test]
fn enum_grid_and_path_helpers_match_cpp_order() {
    let mut indexes = vec![0, 0];
    assert!(get_next_enum_grid(&[2, 3], &mut indexes));
    assert_eq!(indexes, vec![1, 0]);
    assert!(get_next_enum_grid(&[2, 3], &mut indexes));
    assert_eq!(indexes, vec![0, 1]);
    assert!(get_next_enum_grid(&[2, 3], &mut indexes));
    assert_eq!(indexes, vec![1, 1]);

    assert_eq!(get_na("MDIIM"), 3);
    assert_eq!(get_nb("MDIIM"), 4);
    assert_eq!(enum_paths_global(1, 1), vec!["M".to_string()]);
    assert_eq!(
        enum_paths_global(2, 1),
        vec!["MD".to_string(), "DM".to_string()]
    );
    assert_eq!(
        enum_paths_local_l57(0, 0, 1, 0, 0, 1),
        vec![(0, 0, "M".to_string())]
    );
    assert_eq!(on_path_global("MD"), "MD\n");
    assert_eq!(
        on_path_local(1, 2, "MDIM"),
        "  1 ..   3,    2 ..   4  MDIM\n"
    );
    let enum_log = cmd_test_l117();
    assert!(enum_log.starts_with("  0 ..   0,    0 ..   0  M\n"));
    assert!(enum_log.contains("  0 ..   2,    0 ..   2  MMM\n"));
}

#[test]
fn object_type_string_helpers_match_cpp_tables() {
    assert_eq!(obj_type_to_str(ObjType::OT_SeqInfo), "SeqInfo");
    assert_eq!(obj_type_to_str(ObjType::OT_PathInfo), "PathInfo");
    assert_eq!(obj_type_to_str(ObjType::OTCount), "OT_??");
    assert_eq!(obj_type_to_str2(ObjType::OT_SeqInfo), "SI");
    assert_eq!(obj_type_to_str2(ObjType::OT_PathInfo), "??");

    let empty_mgr = obj_mgr_obj_mgr();
    assert_eq!(empty_mgr.free.len(), 0);
    assert_eq!(empty_mgr.busy.len(), 0);
    obj_mgr_validate_type(ObjType::OT_SeqInfo);
    let fresh = obj_mgr_alloc_new(ObjType::OT_SeqInfo);
    assert_eq!(fresh.type_, ObjType::OT_SeqInfo);
    assert_eq!(fresh.ref_count, 0);
    let mut obj = obj_mgr_thread_get_obj(ObjType::OT_SeqInfo);
    assert_eq!(obj.ref_count, 1);
    assert!(obj_mgr_get_busy_count(ObjType::OT_SeqInfo) >= 1);
    obj_mgr_up(&mut obj);
    assert_eq!(obj.ref_count, 2);
    obj_mgr_down(&mut obj);
    assert_eq!(obj.ref_count, 1);
    obj_mgr_down(&mut obj);
    assert_eq!(obj.ref_count, 0);
    assert!(obj_mgr_get_free_count(ObjType::OT_SeqInfo) >= 1);
    let recycled = obj_mgr_static_get_obj(ObjType::OT_SeqInfo);
    assert_eq!(recycled.ref_count, 1);
    assert!(obj_mgr_get_max_ref_count(ObjType::OT_SeqInfo) >= 1);
    assert!(obj_mgr_get_total_mem(ObjType::OT_SeqInfo) >= std::mem::size_of::<Obj>() as f32);
    obj_mgr_validate();
    obj_mgr_update_global_stats();
    obj_mgr_thread_update_global_stats();
    let log = obj_mgr_log_global_stats();
    assert!(log.contains("SeqInfo free "));
    assert!(log.contains("PathInfo free "));
    let mgr = obj_mgr_get_obj_mgr();
    assert!(mgr.busy.contains_key(&ObjType::OT_SeqInfo));
}

#[test]
fn hmm_path_feature_and_seq_gap_helpers_match_cpp_logic() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    set_alpha_amino();

    assert_eq!(hmmtrans_to_str(HMMTRANS::HMMTRANS_START_M), "START_M");
    assert_eq!(hmmtrans_to_str(HMMTRANS::HMMTRANS_IL_M), "IL_M");
    let hmm_lines = [
        "HMM\tnt",
        "T.START_M\t0.5",
        "T.START_IS\t0.1",
        "T.START_IL\t0.2",
        "T.M_M\t0.5",
        "T.M_IS\t0.1",
        "T.M_IL\t0.2",
        "T.IS_IS\t0.3",
        "T.IS_M\t0.7",
        "T.IL_IL\t0.4",
        "T.IL_M\t0.6",
        "E.AA\t1",
        "E.CA\t1",
        "E.CC\t1",
        "E.GA\t1",
        "E.GC\t1",
        "E.GG\t1",
        "E.TA\t1",
        "E.TC\t1",
        "E.TG\t1",
        "E.TT\t1",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    let hmm = hmm_params_from_strings(&hmm_lines);
    assert_eq!(hmm.alpha, NT_ALPHA);
    assert_eq!(hmm.var, DEFAULT_PERTURB_VAR);
    assert!(!hmm.logs);
    hmm_params_assert_probs_valid(&hmm);
    assert!(myfeq(hmm.emits[0][3] as f64, 1.0 / 16.0));
    assert_eq!(hmm_params_get_probs(&hmm), hmm);
    let scores = hmm_params_get_scores(&hmm);
    assert!(scores.logs);
    assert!(myfeq(
        scores.trans[HMMTRANS::HMMTRANS_IS_M as usize] as f64,
        hmm.trans[HMMTRANS::HMMTRANS_IS_M as usize].ln() as f64
    ));
    let round_trip = hmm_params_scores_to_probs(&scores);
    assert!(myfeq(
        round_trip.trans[HMMTRANS::HMMTRANS_M_M as usize] as f64,
        hmm.trans[HMMTRANS::HMMTRANS_M_M as usize] as f64
    ));
    let affine = hmm_params_to_single_affine_probs(&hmm);
    assert!(myfeq(
        affine.trans[HMMTRANS::HMMTRANS_M_IS as usize] as f64,
        affine.trans[HMMTRANS::HMMTRANS_M_IL as usize] as f64
    ));
    let def_lines = hmm_params_get_default_hmm_params(true);
    assert_eq!(def_lines, hmm_params_get_default_hmm_params_nucleo());
    assert_eq!(def_lines[0], "HMM\tnt");
    assert_eq!(def_lines.last().unwrap(), "E.TT\t0.12");
    let amino_lines = hmm_params_get_default_hmm_params(false);
    assert_eq!(amino_lines, hmm_params_get_default_hmm_params_amino());
    assert_eq!(amino_lines[0], "HMM\taa");
    assert_eq!(amino_lines.len(), 221);
    assert_eq!(amino_lines[11], "E.AA\t0.023731");
    assert_eq!(amino_lines.last().unwrap(), "E.YY\t0.0099931");
    let amino_hmm = hmm_params_from_defaults(false);
    hmm_params_assert_probs_valid(&amino_hmm);
    assert_eq!(amino_hmm.alpha, AMINO_ALPHA);
    assert!(amino_hmm.emits[0][19] > 0.0);
    hmm_params_to_pair_hmm(&amino_hmm);
    let params_report = pair_hmm_write_params_report_l13();
    assert!(params_report.contains("const float InitProb_IM = 0.6;"));
    assert!(params_report.contains("const float InsProbs[20] ="));
    assert!(params_report.contains("const float EmitScores[20][20] ="));
    let mut sci_hmm = amino_hmm.clone();
    let tiny_emit = 0.00001_f32;
    let other_emit = (1.0 - tiny_emit) / 399.0;
    for row in &mut sci_hmm.emits {
        row.fill(other_emit);
    }
    sci_hmm.emits[0][0] = tiny_emit;
    hmm_params_to_pair_hmm(&sci_hmm);
    let sci_report = pair_hmm_write_params_report_l13();
    assert!(sci_report.contains("/* A */ {  1e-05 "));
    let sci_hmm_file =
        std::env::temp_dir().join(format!("muscle_rs_hmm_sci_{}.txt", std::process::id()));
    let sci_hmm_text = hmm_params_to_file(&sci_hmm, sci_hmm_file.to_str().unwrap()).unwrap();
    assert!(sci_hmm_text.contains("E.AA\t1e-05\n"));
    std::fs::remove_file(&sci_hmm_file).unwrap();
    let def_hmm = hmm_params_from_defaults(true);
    hmm_params_assert_probs_valid(&def_hmm);
    assert_eq!(def_hmm.alpha, NT_ALPHA);
    let hmm_file = std::env::temp_dir().join(format!("muscle_rs_hmm_{}.txt", std::process::id()));
    let hmm_file_name = hmm_file.to_str().unwrap();
    let hmm_text = hmm_params_to_file(&def_hmm, hmm_file_name).unwrap();
    assert!(hmm_text.starts_with("HMM\tnt\nT.START_M\t0.6\n"));
    let parsed_hmm = hmm_params_from_file(hmm_file_name);
    hmm_params_assert_probs_valid(&parsed_hmm);
    assert_eq!(parsed_hmm.alpha, def_hmm.alpha);
    assert!(myfeq(
        parsed_hmm.trans[HMMTRANS::HMMTRANS_M_M as usize] as f64,
        def_hmm.trans[HMMTRANS::HMMTRANS_M_M as usize] as f64
    ));
    assert!(myfeq(
        parsed_hmm.emits[0][0] as f64,
        def_hmm.emits[0][0] as f64
    ));
    std::fs::remove_file(&hmm_file).unwrap();
    assert!(hmm_params_to_file(&def_hmm, "").is_none());
    let mut updated_hmm = def_hmm.clone();
    hmm_params_cmd_line_update(
        &mut updated_hmm,
        Some(0.11),
        None,
        Some(0.22),
        None,
        None,
        Some(0.33),
    );
    hmm_params_assert_probs_valid(&updated_hmm);
    assert!(updated_hmm.trans[HMMTRANS::HMMTRANS_START_IS as usize] > 0.0);
    assert!(updated_hmm.trans[HMMTRANS::HMMTRANS_M_IS as usize] > 0.0);
    hmm_params_to_pair_hmm(&def_hmm);
    {
        let ins = PAIR_HMM_INS_SCORE.lock().unwrap();
        assert_eq!(ins[b'U' as usize], ins[b'T' as usize]);
        assert_eq!(ins[b'u' as usize], ins[b't' as usize]);
    }
    {
        let mat = PAIR_HMM_MATCH_SCORE.lock().unwrap();
        assert_eq!(
            mat[b'U' as usize][b'A' as usize],
            mat[b'T' as usize][b'A' as usize]
        );
        assert_eq!(
            mat[b'A' as usize][b'U' as usize],
            mat[b'T' as usize][b'A' as usize]
        );
    }
    let mut perturbed = def_hmm.clone();
    hmm_params_perturb_probs(&mut perturbed, 7);
    hmm_params_assert_probs_valid(&perturbed);
    let (mean_t, mean_e) = hmm_params_compare(&def_hmm, &perturbed);
    assert!(mean_t > 0.0);
    assert!(mean_e > 0.0);
    let unchanged = perturbed.clone();
    hmm_params_perturb_probs(&mut perturbed, 0);
    assert_eq!(perturbed, unchanged);
    assert_eq!(run1(0, true), "Iter 0, trans 0.000000, emit 0.000000\n");
    let perturb_report = run1(7, true);
    assert!(perturb_report.starts_with("Iter 7, trans "));
    assert!(perturb_report.contains(", emit "));
    reset_rand(7);
    let mut p = 0.5_f32;
    perturb(&mut p, 0.25);
    assert!((p - 0.5075).abs() < 1e-6);
    let perturb_cmd_report = cmd_perturbhmm(2, true);
    assert!(perturb_cmd_report.starts_with("Iter 0, trans 0.000000, emit 0.000000\n"));
    assert!(perturb_cmd_report.contains("Iter 1, trans "));
    assert_eq!(get_feature_char(true, b'A' as uint), 0);
    assert_eq!(get_feature_char(false, 0), b'A');
    assert_eq!(get_feature_char(false, 25), b'Z');
    assert_eq!(get_feature_char(false, 26), b'{');

    let mut align_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut align_input,
        &["a".to_string(), "b".to_string()],
        &["AC".to_string(), "AG".to_string()],
    );
    set_alpha_l209(ALPHA::ALPHA_Nucleo);
    let mut align_mpc = MPCFlat::default();
    let mut align_updated = false;
    let (align_hp, align_out) = align(
        &mut align_mpc,
        &align_input,
        0,
        TREEPERM::TP_ACB,
        true,
        true,
        |hp| {
            align_updated = true;
            hp.var = 0.125;
        },
        |mpc, input_seqs| {
            assert_eq!(mpc.tree_perm, TREEPERM::TP_ACB);
            input_seqs.clone()
        },
    );
    assert!(align_updated);
    assert_eq!(align_hp.alpha, NT_ALPHA);
    assert!((align_hp.var - 0.125).abs() < f32::EPSILON);
    assert_eq!(align_mpc.tree_perm, TREEPERM::TP_ACB);
    assert_eq!(align_out, "<acb.0\n>a\nAC\n>b\nAG\n");
    let mut skipped_update = false;
    let (_skipped_hp, skipped_out) = align(
        &mut align_mpc,
        &align_input,
        0,
        TREEPERM::TP_None,
        false,
        false,
        |_hp| skipped_update = true,
        |_mpc, _input_seqs| panic!("Align should return immediately for null output"),
    );
    assert!(!skipped_update);
    assert!(skipped_out.is_empty());

    let cmd_dir = std::env::temp_dir().join(format!("muscle_rs_cmd_align_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_dir).unwrap();
    let cmd_input = cmd_dir.join("input.fa");
    let cmd_output = cmd_dir.join("out.fa");
    std::fs::write(&cmd_input, b">a\nA-A\n>b\nA-G\n").unwrap();
    let mut cmd_update_count = 0;
    let (cmd_mpc, cmd_files, cmd_log) = cmd_align(
        cmd_input.to_str().unwrap(),
        cmd_output.to_str().unwrap(),
        None,
        Some(5),
        Some(7),
        false,
        false,
        None,
        Some(3),
        Some(TREEPERM::TP_ABC),
        |hp| {
            cmd_update_count += 1;
            hp.var = 0.25;
        },
        |mpc, input_seqs| {
            assert_eq!(mpc.consistency_iter_count, 5);
            assert_eq!(mpc.refine_iter_count, 7);
            assert_eq!(mpc.tree_perm, TREEPERM::TP_ABC);
            input_seqs.clone()
        },
        |_seq_count| panic!("cmd_align should not dispatch to Super5 here"),
    );
    assert_eq!(cmd_update_count, 1);
    assert_eq!(cmd_mpc.tree_perm, TREEPERM::TP_ABC);
    assert_eq!(cmd_files, vec![cmd_output.to_string_lossy().to_string()]);
    assert!(cmd_log.is_empty());
    assert_eq!(
        std::fs::read_to_string(&cmd_output).unwrap(),
        ">a\nAA\n>b\nAG\n"
    );
    let rep_output = cmd_dir.join("rep.efa");
    let mut rep_calls = Vec::new();
    let (_rep_mpc, rep_files, rep_log) = cmd_align(
        cmd_input.to_str().unwrap(),
        rep_output.to_str().unwrap(),
        None,
        None,
        None,
        false,
        false,
        Some(2),
        None,
        None,
        |_hp| {},
        |mpc, input_seqs| {
            rep_calls.push(mpc.tree_perm);
            input_seqs.clone()
        },
        |_seq_count| panic!("cmd_align should not dispatch to Super5 here"),
    );
    assert_eq!(rep_calls, vec![TREEPERM::TP_None, TREEPERM::TP_ABC]);
    assert_eq!(rep_files, vec![rep_output.to_string_lossy().to_string()]);
    assert_eq!(rep_log, "Replicate 1/2, none.0\nReplicate 2/2, abc.1\n");
    assert_eq!(
        std::fs::read_to_string(&rep_output).unwrap(),
        "<none.0\n>a\nAA\n>b\nAG\n<abc.1\n>a\nAA\n>b\nAG\n"
    );

    let (_super_mpc, super_files, super_log) = cmd_align(
        cmd_input.to_str().unwrap(),
        "ignored.fa",
        Some(2),
        None,
        None,
        false,
        false,
        None,
        None,
        None,
        |_hp| panic!("cmd_align minsuper should not align"),
        |_mpc, _input_seqs| panic!("cmd_align minsuper should not run MPC"),
        |seq_count| {
            assert_eq!(seq_count, 2);
            (vec!["super5.fa".to_string()], "super5-log\n".to_string())
        },
    );
    assert_eq!(super_files, vec!["super5.fa".to_string()]);
    assert_eq!(super_log, "2 seqs, running Super5 algorithm\nsuper5-log\n");
    std::fs::remove_dir_all(&cmd_dir).unwrap();

    assert_eq!(invert_path("BXY"), "BYX");
    validate_path("BxY", 2, 2);
    let mut seq = Seq::default();
    seq_from_string(&mut seq, "A-C.G", "s1");
    assert_eq!(seq_get_ungapped_length(&seq), 3);
    assert!(seq_has_gap(&seq));
    seq_from_string(&mut seq, "A.C", "s2");
    assert!(seq_has_gap(&seq));
    seq_from_string(&mut seq, "ACG", "s3");
    assert!(!seq_has_gap(&seq));
}

#[test]
fn seq_methods_match_cpp_vector_and_fasta_behavior() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    let mut s = Seq::default();
    seq_from_string(&mut s, "a-C.d", "seq1");
    assert_eq!(s.name.as_deref(), Some("seq1"));
    assert_eq!(s.chars, vec!['a', '-', 'C', '.', 'd']);
    assert_eq!(seq_get_ungapped_length(&s), 3);
    assert!(seq_has_gap(&s));
    let mut extracted = MultiSequence::default();
    seq_extract_ungapped(&s, &mut extracted);
    assert_eq!(msa_get_seq_name(&extracted, 0), "seq1");
    assert_eq!(msa_get_row_str(&extracted, 0), "aCd");

    seq_to_upper(&mut s);
    assert_eq!(s.chars, vec!['A', '-', 'C', '.', 'D']);
    assert_eq!(seq_get_letter(&s, 0), 0);
    assert_eq!(seq_get_letter(&s, 2), 1);
    assert_eq!(seq_log_me(&s), ">seq1\nA-C.D\n");

    let mut copied = Seq::default();
    seq_copy(&mut copied, &s);
    assert!(seq_eq(&copied, &s));

    let mut reversed = Seq::default();
    seq_copy_reversed(&mut reversed, &s);
    assert_eq!(reversed.chars, vec!['D', '.', 'C', '-', 'A']);
    assert_eq!(reversed.name.as_deref(), Some("seq1"));

    let mut stripped = s.clone();
    seq_strip_gaps(&mut stripped);
    assert_eq!(stripped.chars, vec!['A', 'C', 'D']);

    let mut spaced = Seq {
        chars: vec!['A', ' ', '-', '\t', 'c', '.'],
        name: Some("spaced".into()),
        id: uint::MAX,
    };
    seq_strip_gaps_and_whitespace(&mut spaced);
    assert_eq!(spaced.chars, vec!['A', 'c']);

    let mut same_gap = Seq::default();
    seq_from_string(&mut same_gap, "a.C-D", "other");
    assert!(seq_eq_ignore_case(&s, &same_gap));
    let mut ungapped_same = Seq::default();
    seq_from_string(&mut ungapped_same, "acd", "ungapped");
    assert!(seq_eq_ignore_case_and_gaps(&s, &ungapped_same));
    assert!(!seq_eq(&s, &same_gap));

    let mut fasta = TextFile::default();
    seq_to_fasta_file(&s, &mut fasta);
    assert_eq!(
        String::from_utf8(fasta.data.clone()).unwrap(),
        ">seq1\nA-C.D\n"
    );

    let mut input = TextFile::default();
    text_file_init(
        &mut input,
        b">first label\nac gt\nA-C.\n>second\nTT\n",
        "mem.fa",
    );
    let mut first = Seq::default();
    assert!(!seq_from_fasta_file(&mut first, &mut input));
    assert_eq!(first.name.as_deref(), Some("first label"));
    assert_eq!(first.chars, vec!['A', 'C', 'G', 'T', 'A', 'C']);
    let mut second = Seq::default();
    assert!(!seq_from_fasta_file(&mut second, &mut input));
    assert_eq!(second.name.as_deref(), Some("second"));
    assert_eq!(second.chars, vec!['T', 'T']);
    let mut eof = Seq::default();
    assert!(seq_from_fasta_file(&mut eof, &mut input));
}

#[test]
fn seq_info_methods_match_cpp_buffer_semantics() {
    let mut si = seq_info_seq_info();
    assert_eq!(si.index, uint::MAX);
    assert_eq!(si.l, 0);
    assert_eq!(si.orf_nuc_l, uint::MAX);

    seq_info_set_copy(&mut si, 7, "seq1", b"AC-NnX");
    assert_eq!(si.index, 7);
    assert_eq!(si.label, "seq1");
    assert_eq!(si.l, 6);
    assert_eq!(&si.seq, b"AC-NnX");
    assert_eq!(si.max_l, 8192);
    assert_eq!(si.max_label_bytes, 133);
    assert_eq!(seq_info_get_mem_bytes(&si), 8325);
    assert_eq!(seq_info_get_il(&si), 6);
    assert_eq!(seq_info_get_n_count(&si), 2);
    assert_eq!(seq_info_get_wildcard_count(&si, false), 2);
    assert_eq!(seq_info_get_wildcard_count(&si, true), 4);
    assert_eq!(seq_info_to_fasta(&si, "out"), ">out\nAC-NnX\n");
    assert_eq!(seq_info_to_fastx(&si, "out"), ">out\nAC-NnX\n");

    let mut rev = seq_info_seq_info();
    seq_info_get_reverse(&si, &mut rev);
    assert_eq!(rev.index, 7);
    assert_eq!(rev.label, "seq1");
    assert_eq!(&rev.seq, b"XnN-CA");
    assert!(rev.rev_comp);

    seq_info_strip_gaps(&mut si);
    assert_eq!(&si.seq, b"ACNnX");
    assert_eq!(si.l, 5);
    seq_info_strip_left(&mut si, 2);
    assert_eq!(&si.seq, b"NnX");
    seq_info_strip_right(&mut si, 1);
    assert_eq!(&si.seq, b"Nn");
    seq_info_truncate_length(&mut si, 1);
    assert_eq!(&si.seq, b"N");

    seq_info_pad(&mut si, 4, 'Q', '\0');
    assert_eq!(&si.seq, b"NQQQ");
    seq_info_on_zero_ref_count(&mut si);
    assert_eq!(si.index, uint::MAX);
    assert_eq!(si.l, 0);
    assert!(!si.is_orf);

    seq_info_init(&mut si, 3);
    assert_eq!(si.index, 3);
    seq_info_set_ptrs(&mut si, 9, "ptr", b"TT");
    assert_eq!(si.index, 9);
    assert_eq!(si.label, "ptr");
    assert_eq!(&si.seq, b"TT");

    let mut copied = seq_info_seq_info();
    seq_info_copy(&mut copied, &si);
    assert_eq!(copied.index, si.index);
    assert_eq!(copied.label, si.label);
    assert_eq!(copied.seq, si.seq);
    assert_eq!(copied.l, si.l);
    seq_info_destructor_seq_info(&mut copied);
    assert_eq!(copied, seq_info_seq_info());
    assert!(seq_info_log_me(&si).contains(">ptr\nTT\n"));
}

#[test]
fn path_info_string_operations_match_cpp_logic() {
    let mut pi = PathInfo::default();
    path_info_append_ms(&mut pi, 2);
    path_info_append_ds(&mut pi, 1);
    path_info_append_is(&mut pi, 3);
    path_info_append_char(&mut pi, b'M');
    assert_eq!(pi.path, "MMDIIIM");
    assert_eq!(path_info_get_counts(&pi), (7, 3, 1, 3));

    path_info_reverse(&mut pi);
    assert_eq!(pi.path, "MIIIDMM");
    assert_eq!(path_info_get_left_i_count(&pi), 0);
    path_info_append_path(
        &mut pi,
        &PathInfo {
            path: "DI".to_string(),
            buffer_bytes: 0,
        },
    );
    assert_eq!(pi.path, "MIIIDMMDI");
    path_info_prepend_path(
        &mut pi,
        &PathInfo {
            path: "II".to_string(),
            buffer_bytes: 0,
        },
    );
    assert_eq!(pi.path, "IIMIIIDMMDI");
    assert_eq!(path_info_trim_left_is(&mut pi), 2);
    assert_eq!(pi.path, "MIIIDMMDI");
    path_info_trim_right_is(&mut pi);
    assert_eq!(pi.path, "MIIIDMMD");
    assert_eq!(
        path_info_to_ops(&pi),
        ("MDIMI".to_string(), vec![1, 3, 1, 2, 1])
    );
    path_info_set_empty(&mut pi);
    assert!(pi.path.is_empty());
    assert_eq!(pi.buffer_bytes, 4096 + 128);

    path_info_alloc(&mut pi, 100);
    assert_eq!(pi.buffer_bytes, 4096 + 128);
    path_info_realloc(&mut pi, 5000);
    assert_eq!(pi.buffer_bytes, 5000);
    path_info_alloc2(&mut pi, 10, 20);
    assert_eq!(pi.buffer_bytes, 5000);
    pi.path = "MDI".to_string();
    pi.buffer_bytes = 4 * 4096;
    path_info_free_if_big(&mut pi);
    assert!(pi.path.is_empty());
    assert_eq!(pi.buffer_bytes, 0);
    path_info_alloc2(&mut pi, 10, 20);
    assert_eq!(pi.buffer_bytes, 31 + 128);

    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_SM: byte = 0x10;
    const TRACEBITS_UNINIT: byte = !0x1f;

    let mut mem = XDPMem {
        tb_bit: vec![vec![0; 3]; 3],
        ..XDPMem::default()
    };
    let mut pi_mm = PathInfo::default();
    trace_back_bit_mem(&mem, 2, 2, b'M', &mut pi_mm);
    assert_eq!(pi_mm.path, "MM");

    mem.tb_bit[1][0] = TRACEBITS_DM;
    let mut pi_dm = PathInfo::default();
    trace_back_bit_mem(&mem, 2, 1, b'M', &mut pi_dm);
    assert_eq!(pi_dm.path, "DM");

    mem.tb_bit = vec![vec![0; 3]; 3];
    mem.tb_bit[0][1] = TRACEBITS_IM;
    let mut pi_im = PathInfo::default();
    trace_back_bit_mem(&mem, 1, 2, b'M', &mut pi_im);
    assert_eq!(pi_im.path, "IM");

    mem.tb_bit = vec![vec![0; 3]; 3];
    mem.tb_bit[0][0] = TRACEBITS_SM;
    let (leni1, lenj1, path1) = trace_back_bit_sw(&mem, 2, 2, 1, 1);
    assert_eq!((leni1, lenj1, path1), (1, 1, "M".to_string()));
    mem.tb_bit[1][1] = 0;
    let (leni2, lenj2, path2) = trace_back_bit_sw(&mem, 3, 3, 2, 2);
    assert_eq!((leni2, lenj2, path2), (2, 2, "MM".to_string()));

    mem.tb_bit[0][1] = TRACEBITS_DM;
    mem.tb_bit[1][0] = TRACEBITS_IM;
    mem.tb_bit[1][1] = TRACEBITS_UNINIT;
    let tb_log = log_tbsw("demo", &mem, 2, 2);
    assert!(tb_log.starts_with("TBM demo\n  0 | SD\n  1 | I*\n\nTBD demo\n"));
    assert!(tb_log.contains("\nTBI\n  0 | II\n  1 | I*\n"));
}

#[test]
fn profile_position_score_matches_cpp_loop_order() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    let mut ppa = ProfPos3::default();
    ppa.sort_order[0] = 3;
    ppa.sort_order[1] = 1;
    ppa.sort_order[2] = 4;
    ppa.freqs[3] = 0.25;
    ppa.freqs[1] = 0.50;
    ppa.freqs[4] = 0.0;
    ppa.freqs[0] = 1.0;

    let mut ppb = ProfPos3::default();
    ppb.aa_scores[3] = 8.0;
    ppb.aa_scores[1] = -2.0;
    ppb.aa_scores[0] = 100.0;
    assert_eq!(score_prof_pos2(&ppa, &ppb), 1.0);

    let mut nwp_a = ProfPos3::default();
    nwp_a.sort_order[0] = 0;
    nwp_a.sort_order[1] = 1;
    nwp_a.freqs[0] = 1.0;
    nwp_a.aa_scores[0] = 5.0;
    nwp_a.gap_open_score = -1.0;
    nwp_a.gap_close_score = -1.0;
    let mut nwp_b = nwp_a.clone();
    nwp_b.aa_scores[0] = 5.0;
    let mut cm = CacheMem3::default();
    let (nw_score, nw_path) = nw_small3(
        &mut cm,
        &Profile3 {
            pps: vec![nwp_a.clone()],
        },
        &Profile3 { pps: vec![nwp_b] },
    );
    assert_eq!(nw_path, "M");
    assert!((nw_score - 3.0).abs() < 1e-6, "{nw_score}");
    assert_eq!(cm.cache_tb.len(), 2);

    let mut counts = [0.0_f32; 20];
    counts[3] = 0.5;
    counts[1] = 0.5;
    counts[4] = 0.25;
    let order = sort_counts(&counts);
    assert_eq!(&order[..6], &[1, 3, 4, 0, 2, 5]);

    let mut pp = ProfPos3::default();
    prof_pos3_set_start_dimers(&mut pp);
    assert!(init_pp_start());
    assert_eq!((pp.ll, pp.lg, pp.gl, pp.gg), (1.0, 0.0, 0.0, 0.0));

    let mut dpa = ProfPos3::default();
    dpa.ll = 0.1;
    dpa.lg = 0.2;
    dpa.gl = 0.3;
    dpa.gg = 0.4;
    dpa.freqs[0] = 0.2;
    dpa.freqs[1] = 0.4;
    let mut dpb = ProfPos3::default();
    dpb.ll = 0.5;
    dpb.lg = 0.6;
    dpb.gl = 0.7;
    dpb.gg = 0.8;
    dpb.freqs[0] = 0.6;
    dpb.freqs[1] = 0.8;
    let wa = 0.25_f32;
    let wb = 0.75_f32;
    let assert_dimers = |got: &ProfPos3, want: (f32, f32, f32, f32)| {
        assert!((got.ll - want.0).abs() < 1e-6, "ll {}", got.ll);
        assert!((got.lg - want.1).abs() < 1e-6, "lg {}", got.lg);
        assert!((got.gl - want.2).abs() < 1e-6, "gl {}", got.gl);
        assert!((got.gg - want.3).abs() < 1e-6, "gg {}", got.gg);
    };
    let mut mixed = ProfPos3::default();
    set_dimers_mm(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.4, 0.5, 0.6, 0.7));
    set_dimers_md(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.025, 0.95, 0.075, 1.15));
    set_dimers_dd(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.025, 0.05, 0.075, 0.85));
    set_dimers_mi(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.375, 0.55, 0.525, 0.75));
    set_dimers_dm(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.025, 0.05, 0.975, 1.15));
    set_dimers_im(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.375, 0.45, 0.625, 0.75));
    set_dimers_id(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.0, 0.9, 0.1, 1.2));
    set_dimers_di(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.0, 0.1, 0.9, 1.2));
    set_dimers_ii(&dpa, wa, &dpb, wb, &mut mixed);
    assert_dimers(&mixed, (0.375, 0.45, 0.525, 0.85));
    set_freqs1(&dpa, wa, &mut mixed);
    assert!((mixed.freqs[0] - 0.05).abs() < 1e-6);
    assert!((mixed.freqs[1] - 0.1).abs() < 1e-6);
    set_freqs2(&dpa, wa, &dpb, wb, &mut mixed);
    assert!((mixed.freqs[0] - 0.5).abs() < 1e-6);
    assert!((mixed.freqs[1] - 0.7).abs() < 1e-6);

    pp.ll = 0.25;
    pp.gl = 0.5;
    prof_pos3_set_occ(&mut pp);
    assert_eq!(pp.f_occ, 0.75);

    let mut letter_counts = vec![0_u32; 20];
    letter_counts[0] = 2;
    letter_counts[1] = 1;
    prof_pos3_set_freqs2(&mut pp, 4, 1, 1, 2, 0, &letter_counts);
    assert_eq!(
        (pp.ll, pp.lg, pp.gl, pp.gg, pp.f_occ),
        (0.25, 0.25, 0.5, 0.0, 0.75)
    );
    assert_eq!(pp.freqs[0], 0.5);
    assert_eq!(pp.freqs[1], 0.25);

    let mut subst = [[0.0_f32; 20]; 20];
    for (i, row) in subst.iter_mut().enumerate() {
        for (j, score) in row.iter_mut().enumerate() {
            *score = i as f32 + j as f32;
        }
    }
    prof_pos3_set_aa_scores(&mut pp, &subst);
    assert_eq!(pp.sort_order[0], 0);
    assert_eq!(pp.sort_order[1], 1);
    assert!((pp.aa_scores[0] - 0.25).abs() < 1e-6);
    assert!((pp.aa_scores[2] - 1.75).abs() < 1e-6);
    let mut prof_a = Profile3::default();
    let mut prof_b = Profile3::default();
    let mut msa_a = MultiSequence::default();
    let mut msa_b = MultiSequence::default();
    multi_sequence_from_strings(&mut msa_a, &["a".to_string()], &["A".to_string()]);
    multi_sequence_from_strings(&mut msa_b, &["b".to_string()], &["C".to_string()]);
    profile3_from_msa(&mut prof_a, &msa_a, &subst, -2.0, &[1.0]);
    profile3_from_msa(&mut prof_b, &msa_b, &subst, -2.0, &[1.0]);
    let prof_ab = align_two_profs_given_path(&prof_a, 1.0, &prof_b, 1.0, &subst, -2.0, "M");
    assert_eq!(prof_ab.pps.len(), 1);
    assert!((prof_ab.pps[0].freqs[0] - 0.5).abs() < 1e-6);
    assert!((prof_ab.pps[0].freqs[1] - 0.5).abs() < 1e-6);
    assert_eq!(prof_ab.pps[0].ll, 1.0);

    let mut msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut msa,
        &["a".to_string(), "b".to_string(), "c".to_string()],
        &["AC.".to_string(), "-C.".to_string(), "A-.".to_string()],
    );
    prof_pos3_set_freqs(&mut pp, &msa, 1, &[0.2, 0.3, 0.5]);
    assert!(!pp.all_gaps);
    assert!((pp.f_occ - 0.5).abs() < 1e-6);
    assert!((pp.freqs[1] - 0.5).abs() < 1e-6);
    assert!((pp.ll - 0.2).abs() < 1e-6);
    assert!((pp.gl - 0.3).abs() < 1e-6);
    assert!((pp.lg - 0.5).abs() < 1e-6);
    assert_eq!(pp.gg, 0.0);

    prof_pos3_set_aa_scores(&mut pp, &subst);
    pp.gap_open_score = -1.25;
    pp.gap_close_score = -0.75;
    assert_eq!(
        prof_pos3_to_tsv(&pp),
        "\t0.2\t0.5\t0.3\t0\t0.5\t-1.25\t-0.75\t0\t0.5\t0.5\t1\t0\t1.5\t0\t2\t0\t2.5\t0\t3\t0\t3.5\t0\t4\t0\t4.5\t0\t5\t0\t5.5\t0\t6\t0\t6.5\t0\t7\t0\t7.5\t0\t8\t0\t8.5\t0\t9\t0\t9.5\t0\t10\n"
    );
    assert_eq!(
        prof_pos3_log_me(&pp),
        " LL=0.200 LG=0.500 GL=0.300 GG=0.000 fOcc=0.500\n Freqs:  C=0.5\nScores:  C=1 A=0.5 D=1.5 E=2 F=2.5 G=3 H=3.5 I=4 K=4.5\n    L=5 M=5.5 N=6 P=6.5 Q=7 R=7.5 S=8 T=8.5 V=9 W=9.5 Y=10\n"
    );

    let mut prof = Profile3 {
        pps: vec![ProfPos3::default(), ProfPos3::default()],
    };
    prof.pps[0].ll = 0.4;
    prof.pps[0].lg = 0.1;
    prof.pps[0].gl = 0.2;
    prof.pps[0].gg = 0.3;
    prof.pps[0].f_occ = 0.6;
    prof.pps[0].freqs[0] = 0.25;
    prof.pps[0].aa_scores[0] = 1.5;
    prof.pps[1].ll = 0.5;
    prof.pps[1].lg = 0.1;
    prof.pps[1].gl = 0.2;
    prof.pps[1].gg = 0.2;
    prof.pps[1].f_occ = 0.7;
    profile3_validate(&prof);
    profile3_set_gap_scores(&mut prof, -2.0);
    assert!((prof.pps[0].gap_open_score + 0.6).abs() < 1e-6);
    assert!((prof.pps[0].gap_close_score + 0.8).abs() < 1e-6);
    assert!((prof.pps[1].gap_open_score + 0.9).abs() < 1e-6);
    assert!((prof.pps[1].gap_close_score + 0.7).abs() < 1e-6);
    assert_eq!(profile3_get_pp(&prof, 1).f_occ, 0.7);
    assert_eq!(profile3_to_tsv_l256(&prof).lines().count(), 3);
    profile3_to_tsv_l249(&prof, "");

    let mut prof2 = prof.clone();
    prof2.pps[0].freqs[0] = 0.5;
    prof2.pps[0].aa_scores[0] = 2.0;
    prof2.pps[1].gap_close_score = -1.1;
    let (diff_count, diff_log) = profile3_log_diffs(&prof, &prof2);
    assert_eq!(diff_count, 3);
    assert!(diff_log.contains("Col 0 Freqs[0=A] 0.25 0.5\n"));
    assert!(diff_log.contains("Col 0 AAScores[0=A] 1.5 2\n"));
    assert!(diff_log.contains("Col 1 GapCloseScore: -0.7 -1.1\n"));
    let (length_diff_count, length_diff_log) = profile3_log_diffs(&prof, &Profile3 { pps: vec![] });
    assert_eq!(length_diff_count, 1);
    assert_eq!(length_diff_log, "Lengths differ 2, 0\n");
    assert!(profile3_get_self_score(&prof) > 0.0);

    let mut built = Profile3::default();
    profile3_from_msa(&mut built, &msa, &subst, -2.0, &[0.2, 0.3, 0.5]);
    assert_eq!(built.pps.len(), 3);
    assert!(profile3_log_me(&built, Some(&msa)).contains("  Pos  Occ"));
    profile3_clear(&mut built);
    assert!(built.pps.is_empty());

    let prof_cmd_in =
        std::env::temp_dir().join(format!("muscle_rs_profile3_cmd_{}.fa", std::process::id()));
    let prof_cmd_out =
        std::env::temp_dir().join(format!("muscle_rs_profile3_cmd_{}.tsv", std::process::id()));
    std::fs::write(&prof_cmd_in, b">pa\nEF.\n>pb\n-F.\n>pc\nEI.\n").unwrap();
    let (cmd_prof, cmd_log, cmd_tsv) = cmd_build_prof3(
        prof_cmd_in.to_str().unwrap(),
        prof_cmd_out.to_str().unwrap(),
        &subst,
        -2.0,
    );
    assert_eq!(cmd_prof.pps.len(), 3);
    assert!(cmd_log.contains("  Pos  Occ"));
    assert_eq!(std::fs::read_to_string(&prof_cmd_out).unwrap(), cmd_tsv);
    let (cmd_prof_no_output, _, cmd_tsv_no_output) =
        cmd_build_prof3(prof_cmd_in.to_str().unwrap(), "", &subst, -2.0);
    assert_eq!(cmd_prof_no_output.pps.len(), 3);
    assert_eq!(cmd_tsv_no_output, cmd_tsv);
    let (_self_prof, self_score, self_log) =
        cmd_msaselfscore3(prof_cmd_in.to_str().unwrap(), &subst, -2.0);
    assert!(self_score > 0.0);
    assert!(self_log.starts_with("MSASelfScore3=12.667, MSA="));
    assert!(self_log.ends_with(&format!(
        "MSA={}\n",
        base_name(prof_cmd_in.to_str().unwrap())
    )));
    std::fs::remove_file(&prof_cmd_in).unwrap();
    std::fs::remove_file(&prof_cmd_out).unwrap();
}

#[test]
fn sequence_mapping_and_msa_format_helpers_match_cpp_logic() {
    assert_eq!(
        sequence_get_pos_to_col("ATGCC---GT--CA"),
        vec![0, 1, 2, 3, 4, 8, 9, 12, 13]
    );
    assert_eq!(sequence_get_col_to_pos("A-C."), vec![0, uint::MAX, 1, 2]);
    assert_eq!(sequence_copy_delete_gaps("A-C.G-"), "AC.G");
    let mut seq_obj = Sequence::default();
    sequence_from_string(&mut seq_obj, "seq1", "ACGT");
    assert_eq!(seq_obj.label, "seq1");
    assert_eq!(seq_obj.char_vec, vec!['A', 'C', 'G', 'T']);
    assert_eq!(sequence_get_seq_as_string(&seq_obj), "ACGT");
    let cloned = sequence_clone(&seq_obj);
    assert_eq!(cloned, seq_obj);
    let gapped = sequence_add_gaps_path(&seq_obj, "MDMIB", 'D');
    assert_eq!(gapped.label, "seq1");
    assert_eq!(sequence_get_seq_as_string(&gapped), "ACG-T");
    let mut msa_a = MultiSequence::default();
    multi_sequence_from_strings(
        &mut msa_a,
        &["a1".to_string(), "a2".to_string()],
        &["AC".to_string(), "GT".to_string()],
    );
    let mut msa_b = MultiSequence::default();
    multi_sequence_from_strings(&mut msa_b, &["b1".to_string()], &["XY".to_string()]);
    let mut joined = MultiSequence::default();
    align_two_ms_as_given_path(&msa_a, &msa_b, "MDI", &mut joined);
    assert_eq!(joined.seqs.len(), 3);
    assert_eq!(sequence_get_seq_as_string(&joined.seqs[0]), "AC-");
    assert_eq!(sequence_get_seq_as_string(&joined.seqs[1]), "GT-");
    assert_eq!(sequence_get_seq_as_string(&joined.seqs[2]), "X-Y");

    let mut msa_x = MultiSequence::default();
    multi_sequence_from_strings(&mut msa_x, &["x1".to_string()], &["AB".to_string()]);
    let mut msa_y = MultiSequence::default();
    multi_sequence_from_strings(&mut msa_y, &["y1".to_string()], &["CD".to_string()]);
    let mut xy = MultiSequence::default();
    align_ms_as_by_path(&msa_x, &msa_y, "BXY", &mut xy);
    assert_eq!(sequence_get_seq_as_string(&xy.seqs[0]), "AB-");
    assert_eq!(sequence_get_seq_as_string(&xy.seqs[1]), "C-D");

    let mut cols_x2 = MultiSequence::default();
    let mut cols_y2 = MultiSequence::default();
    let mut cols_path = String::new();
    let mut merge_map = Vec::new();
    multi_sequence_from_strings(&mut msa_x, &["x1".to_string()], &["ABCD".to_string()]);
    multi_sequence_from_strings(&mut msa_y, &["y1".to_string()], &["WXYZ".to_string()]);
    align_ms_as_by_cols(
        &msa_x,
        &msa_y,
        &[1, 3],
        &[0, 2],
        &mut cols_path,
        &mut merge_map,
        &mut cols_x2,
        &mut cols_y2,
    );
    assert_eq!(cols_path, "xMmMy");
    assert_eq!(merge_map, vec![1, 3]);
    assert_eq!(sequence_get_seq_as_string(&cols_x2.seqs[0]), "ABCD.");
    assert_eq!(sequence_get_seq_as_string(&cols_y2.seqs[0]), ".WXYZ");

    let mut mpc = MPCFlat::default();
    let mut combined = MultiSequence::default();
    multi_sequence_from_strings(
        &mut combined,
        &["ma".to_string(), "mb".to_string()],
        &["AC".to_string(), "AG".to_string()],
    );
    mpc_flat_init_seqs(&mut mpc, &combined);
    mpc_flat_init_pairs(&mut mpc);
    mpc_flat_alloc_pair_count(&mut mpc, 1);
    mpc_flat_init_dist_mx(&mut mpc);
    let mut sparse_post = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse_post, &[1.0, 0.0, 0.0, 1.0], 2, 2);
    mpc.sparse_posts1[0] = Some(sparse_post);
    let mut mpc_msa1 = MultiSequence::default();
    let mut mpc_msa2 = MultiSequence::default();
    multi_sequence_from_strings(&mut mpc_msa1, &["ma".to_string()], &["AC".to_string()]);
    multi_sequence_from_strings(&mut mpc_msa2, &["mb".to_string()], &["AG".to_string()]);
    let (aligned_mpc, aligned_score, aligned_path) =
        mpc_flat_align_alns(&mut mpc, &mpc_msa1, &mpc_msa2);
    assert_eq!(aligned_path, "BB");
    assert_eq!(aligned_score, 2.0);
    assert_eq!(
        aligned_mpc
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["AC".to_string(), "AG".to_string()]
    );

    let pi = PathInfo {
        path: "MDMIM".to_string(),
        buffer_bytes: 0,
    };
    assert!((get_fract_id_path("AbCd", "aXYd", &pi) - (2.0 / 3.0)).abs() < 1e-6);
    let pi = PathInfo {
        path: "M".to_string(),
        buffer_bytes: 0,
    };
    assert_eq!(get_fract_id_path("-", ".", &pi), 0.0);
    assert_eq!(
        make_aln_rows_l4("AC", "TGA", "BYYX"),
        ("A--C".to_string(), "TGA-".to_string())
    );
    let pi_rows = PathInfo {
        path: "MDM".to_string(),
        buffer_bytes: 0,
    };
    assert_eq!(
        make_aln_rows_l45(b"ACG", 3, b"TA", 2, &pi_rows),
        ("ACG".to_string(), "T-A".to_string())
    );
    assert_eq!(log_aln_l126("AC", "TGA", "BYYX"), "\nA--C\nTGA-\n");
    assert_eq!(log_aln_l136(b"ACG", 3, b"TA", 2, &pi_rows), "\nACG\nT-A\n");

    let mut seqi = Sequence::default();
    let mut seqj = Sequence::default();
    sequence_from_string(&mut seqi, "i", "Ab-.D");
    sequence_from_string(&mut seqj, "j", "aC-.d");
    assert!((get_fract_id(&seqi, &seqj) - (2.0 / 3.0)).abs() < 1e-6);
    assert_eq!(
        make_aln_rows_l85(&seqi, &seqj, "BXYBBB"),
        ("Ab--.D".to_string(), "a-C-.d".to_string())
    );
    assert_eq!(
        log_aln_l149(&seqi, &seqj, "BXYBBB"),
        "\ni           Ab--.D\nj           a-C-.d\n"
    );
    let (pos_to_col_x, pos_to_col_y, col_to_pos_x, col_to_pos_y) = path_to_col_vecs("BXYBBB");
    assert_eq!(pos_to_col_x, vec![0, 1, 3, 4, 5]);
    assert_eq!(pos_to_col_y, vec![0, 2, 3, 4, 5]);
    assert_eq!(col_to_pos_x, vec![0, 1, uint::MAX, 2, 3, 4]);
    assert_eq!(col_to_pos_y, vec![0, uint::MAX, 1, 2, 3, 4]);
    assert_eq!(
        write_annot_row(b"AbCd", b"aXYd", "MDIM", 0, 0, 0, 3),
        "      |   \n"
    );
    assert_eq!(
        write_a_row(b"AbCd", "MDIM", 0, 0, 3, "labA"),
        ("    1 Ab-C 3  labA\n".to_string(), 3)
    );
    assert_eq!(
        write_b_row(b"aXYd", "MDIM", 0, 0, 3, "labB"),
        ("    1 a-XY 3  labB\n".to_string(), 3)
    );
    assert_eq!(
        write_aln_pretty(b"AbCd", b"aXYd", "MDIM"),
        "    1 Ab-C 3  \n      |   \n    1 a-XY 3  \n\n"
    );
    assert_eq!(
        write_local_aln("labA", b"AbCd", "labB", b"aXYd", 0, 0, "MDIM"),
        "    1 Ab-C 3  labA\n      |   \n    1 a-XY 3  labB\n\n"
    );

    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let seqs = vec!["A-C.".to_string(), "....".to_string(), "G-T.".to_string()];
    let mut msa = MultiSequence::default();
    multi_sequence_from_strings(&mut msa, &labels, &seqs);
    assert_eq!(msa_get_char(&msa, 0, 2), 'C');
    assert!(msa_is_gap(&msa, 0, 1));
    assert_eq!(msa_get_seq_name(&msa, 2), "c");
    assert_eq!(msa_get_seq_label(&msa, 1), "b");
    assert_eq!(msa_get_row_str(&msa, 0), "A-C.");
    assert!(msa_has_gap(&msa));
    assert!(msa_is_legal_letter(19));
    assert!(!msa_is_legal_letter(20));
    assert_eq!(msa_get_upper_lower_gap_count(&msa, 0), (2, 0, 1, 1, 0));
    assert_eq!(msa_get_gap_count(&msa, 1), 3);
    assert!(msa_is_gap_column(&msa, 1));
    assert_eq!(msa_get_char_count(&msa, 0, 2), 2);
    let ungapped_seq = msa_get_seq(&msa, 0);
    assert_eq!(ungapped_seq.name.as_deref(), Some("a"));
    assert_eq!(ungapped_seq.chars, vec!['A', 'C']);
    let mut renamed = msa_copy(&msa);
    msa_set_seq_name(&mut renamed, 0, "renamed");
    assert_eq!(msa_get_seq_name(&renamed, 0), "renamed");
    msa_copy_col(&mut renamed, 0, 2);
    assert_eq!(msa_get_char(&renamed, 0, 2), 'A');
    msa_set_char(&mut renamed, 1, 4, 'X');
    assert_eq!(msa_get_char(&renamed, 1, 4), 'X');
    let mut deleted = msa_copy(&msa);
    msa_delete_col(&mut deleted, 1);
    assert_eq!(msa_get_row_str(&deleted, 0), "AC.");
    msa_delete_columns(&mut deleted, 1, 2);
    assert_eq!(msa_get_row_str(&deleted, 0), "A");
    assert_eq!(msa_get_seq_index_l367(&msa, "C", true), 2);
    assert_eq!(msa_get_seq_index_l367(&msa, "missing", false), uint::MAX);
    assert_eq!(msa_get_seq_index_l377(&msa, "B"), Some(1));
    assert_eq!(msa_get_seq_index_l377(&msa, "missing"), None);
    let (labels_from_msa, label_map) = msa_get_label_to_seq_index(&msa);
    assert_eq!(labels_from_msa, labels);
    assert_eq!(label_map.get("c"), Some(&2));
    let from_sequence = msa_from_sequence(&seq_obj);
    assert_eq!(msa_get_row_str(&from_sequence, 0), "ACGT");
    let from_multi = msa_from_multi_sequence(&msa);
    assert_eq!(from_multi, msa);
    let from_seq = msa_from_seq(&ungapped_seq);
    assert_eq!(msa_get_row_str(&from_seq, 0), "AC");
    let mut copy_target = MultiSequence {
        seqs: vec![Sequence {
            label: "dst".to_string(),
            char_vec: vec!['?', '?', '?', '?'],
        }],
        owners: vec![true],
        dupe_labels_ok: false,
        id_count: 0,
        id_to_seq_index: Vec::new(),
        seq_index_to_id: Vec::new(),
    };
    msa_copy_seq(&mut copy_target, 0, &msa, 2);
    assert_eq!(msa_get_seq_buffer(&copy_target, 0), &['G', '-', 'T', '.']);
    let mut deleted_seq = msa_copy(&msa);
    msa_delete_seq(&mut deleted_seq, 1);
    assert_eq!(
        deleted_seq
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["a".to_string(), "c".to_string()]
    );
    assert!(msa_is_empty_col(&msa, 1));
    assert!(msa_seqs_eq(&msa, 0, &from_seq, 0));
    assert_eq!(msa_get_ungapped_seq_length(&msa, 0), 2);
    let (pwid, pos_count) = msa_get_pwid(&msa, 0, 2);
    assert_eq!(pos_count, 2);
    assert_eq!(pwid, 0.0);
    assert_eq!(msa_get_occ(&msa, 0), 2.0 / 3.0);
    assert_eq!(msa_get_ungapped_seq_str(&msa, 0), "AC");
    assert!(msa_column_has_gap(&msa, 1));
    let mut id_msa = msa_copy(&msa);
    msa_set_id_count(&mut id_msa, 7);
    msa_set_seq_id(&mut id_msa, 0, 3);
    msa_set_seq_id(&mut id_msa, 1, 5);
    msa_set_seq_id(&mut id_msa, 2, 6);
    assert_eq!(msa_get_seq_index_l733(&id_msa, 6), 2);
    assert_eq!(msa_get_seq_index_l742(&id_msa, 5), Some(1));
    assert_eq!(msa_get_seq_index_l742(&id_msa, 4), None);
    assert_eq!(msa_get_seq_id(&id_msa, 0), 3);
    let subset_by_index = msa_subset_by_ids(&id_msa, &[6, 3]);
    assert_eq!(
        subset_by_index
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["c".to_string(), "a".to_string()]
    );
    assert_eq!(msa_get_seq_id(&subset_by_index, 0), 6);
    let mut sized = msa_msa();
    msa_set_id_count(&mut sized, 2);
    msa_set_size(&mut sized, 2, 3);
    assert_eq!(sized.seqs.len(), 2);
    assert_eq!(msa_get_row_str(&sized, 0), "???");
    msa_set_seq_count(&mut sized, 1);
    assert_eq!(sized.seqs.len(), 1);
    assert_eq!(sized.seqs[0].char_vec.len(), 500);
    let mut cache = msa_msa();
    msa_expand_cache(&mut cache, 2, 3);
    assert_eq!(cache.seqs.len(), 2);
    msa_destructor_msa(&mut cache);
    assert!(cache.seqs.is_empty());
    let mut appended_seq = msa_copy(&msa);
    msa_append_seq(&mut appended_seq, &['T', 'T', '-', '.'], "d");
    assert_eq!(msa_get_row_str(&appended_seq, 3), "TT-.");
    assert_eq!(msa_get_pos_to_col(&msa, 0), vec![0, 2]);
    assert_eq!(msa_get_col_to_pos1(&msa, 0), vec![1, -1, 2, -2]);
    assert_eq!(
        msa_get_col_to_pos(&msa, 0),
        vec![0, uint::MAX, 1, uint::MAX]
    );
    assert!(msa_col_is_upper(&msa, 0, 1.0));
    assert!(!msa_col_is_upper(&msa, 3, 1.0));
    assert!(msa_col_is_aligned(&msa, 0));
    assert!(!msa_col_is_aligned(&msa, 1));
    let all_gap_deleted = msa_delete_all_gap_cols(&msa);
    assert_eq!(
        all_gap_deleted
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["AC".to_string(), "..".to_string(), "GT".to_string()]
    );
    let seq_range = msa_from_seq_range(&msa, 1, 2);
    assert_eq!(
        seq_range
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["b".to_string(), "c".to_string()]
    );
    let col_range = msa_from_col_range(&msa, 1, 2);
    assert_eq!(
        col_range
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["-C".to_string(), "..".to_string(), "-T".to_string()]
    );
    let subset = msa_from_seq_subset(&msa, &[2, 0]);
    assert_eq!(subset.seqs[0].label, "c");
    assert_eq!(subset.seqs[1].label, "a");
    let mut no_gap_cols = msa.clone();
    delete_gapped_cols(&mut no_gap_cols);
    assert_eq!(
        no_gap_cols
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["AC".to_string(), "..".to_string(), "GT".to_string()]
    );
    let mut same_letters = MultiSequence::default();
    multi_sequence_from_strings(
        &mut same_letters,
        &labels,
        &vec!["ac".to_string(), "".to_string(), "gt".to_string()],
    );
    assert_msa_eq_ignore_case_and_gaps(&msa, &same_letters);
    let mut exact_reordered = MultiSequence::default();
    multi_sequence_from_strings(
        &mut exact_reordered,
        &vec!["c".to_string(), "a".to_string(), "b".to_string()],
        &vec!["G-T.".to_string(), "A-C.".to_string(), "....".to_string()],
    );
    assert_msa_eq(&msa, &exact_reordered);

    let mut suffix = MultiSequence::default();
    multi_sequence_from_strings(
        &mut suffix,
        &vec!["b".to_string(), "a".to_string()],
        &vec!["TT".to_string(), "GG".to_string()],
    );
    let cat = msa_cat(&msa, &suffix);
    assert_eq!(
        cat.seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec![
            "A-C.GG".to_string(),
            "....TT".to_string(),
            "G-T.--".to_string()
        ]
    );
    let mut appended = msa_from_seq_range(&msa, 0, 2);
    msa_append(&mut appended, &suffix);
    assert_eq!(
        appended
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A-C.GG".to_string(), "....TT".to_string()]
    );
    let mut gap_cols = MultiSequence::default();
    multi_sequence_from_strings(
        &mut gap_cols,
        &["g1".to_string(), "g2".to_string()],
        &["A--G".to_string(), "TT--".to_string()],
    );
    delete_all_gap_columns(&mut gap_cols);
    assert_eq!(
        gap_cols
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A-G".to_string(), "TT-".to_string()]
    );
    let mut test_trim = MultiSequence::default();
    multi_sequence_from_strings(
        &mut test_trim,
        &["a".to_string(), "b".to_string()],
        &["A-CG".to_string(), "TT-G".to_string()],
    );
    let mut ref_trim = MultiSequence::default();
    multi_sequence_from_strings(
        &mut ref_trim,
        &["a".to_string(), "b".to_string()],
        &["AcG".to_string(), "TTt".to_string()],
    );
    let trimmed = trim_to_ref(&test_trim, &ref_trim);
    assert_eq!(
        trimmed
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A-G".to_string(), "TT-".to_string()]
    );
    let trim_in =
        std::env::temp_dir().join(format!("muscle_rs_trimtoref_in_{}.fa", std::process::id()));
    let trim_ref =
        std::env::temp_dir().join(format!("muscle_rs_trimtoref_ref_{}.fa", std::process::id()));
    let trim_out =
        std::env::temp_dir().join(format!("muscle_rs_trimtoref_out_{}.fa", std::process::id()));
    std::fs::write(&trim_in, b">a\nA-CG\n>b\nTT-G\n").unwrap();
    std::fs::write(&trim_ref, b">a\nAcG\n>b\nTTt\n").unwrap();
    let trimmed_cmd = cmd_trimtoref(
        trim_in.to_str().unwrap(),
        trim_ref.to_str().unwrap(),
        trim_out.to_str().unwrap(),
    );
    assert_eq!(
        trimmed_cmd
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["A-G".to_string(), "TT-".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(&trim_out).unwrap(),
        ">a\nA-G\n>b\nTT-\n"
    );
    std::fs::remove_file(&trim_in).unwrap();
    std::fs::remove_file(&trim_ref).unwrap();
    std::fs::remove_file(&trim_out).unwrap();

    let efa_in = std::env::temp_dir().join(format!(
        "muscle_rs_trimtoref_efa_in_{}.efa",
        std::process::id()
    ));
    let ref_in = std::env::temp_dir().join(format!(
        "muscle_rs_trimtoref_efa_ref_{}.fa",
        std::process::id()
    ));
    let efa_out = std::env::temp_dir().join(format!(
        "muscle_rs_trimtoref_efa_out_{}.efa",
        std::process::id()
    ));
    std::fs::write(
        &efa_in,
        b"<rep1\n>a\nA-CG\n>b\nTT-G\n<rep2\n>a\nAC-G\n>b\nT-TG\n",
    )
    .unwrap();
    std::fs::write(&ref_in, b">a\nAcG\n>b\nTTt\n").unwrap();
    let trimmed_efa = cmd_trimtoref_efa(
        efa_in.to_str().unwrap(),
        ref_in.to_str().unwrap(),
        efa_out.to_str().unwrap(),
    );
    assert_eq!(trimmed_efa.msa_names, vec!["rep1", "rep2"]);
    assert_eq!(
        std::fs::read_to_string(&efa_out).unwrap(),
        "<rep1\n>a\nA-G\n>b\nTT-\n<rep2\n>a\nA-G\n>b\nTT-\n"
    );
    let trimmed_efa_no_output =
        cmd_trimtoref_efa(efa_in.to_str().unwrap(), ref_in.to_str().unwrap(), "");
    assert_eq!(trimmed_efa_no_output.msa_names, vec!["rep1", "rep2"]);
    std::fs::remove_file(&efa_in).unwrap();
    std::fs::remove_file(&ref_in).unwrap();
    std::fs::remove_file(&efa_out).unwrap();

    let explode_in =
        std::env::temp_dir().join(format!("muscle_rs_explode_in_{}.efa", std::process::id()));
    let explode_prefix = std::env::temp_dir()
        .join(format!("muscle_rs_explode_{}_", std::process::id()))
        .to_string_lossy()
        .to_string();
    std::fs::write(&explode_in, b"<r1\n>a\nac\n>b\nT-\n<r2\n>b\nT-\n>a\nac\n").unwrap();
    let exploded = cmd_efa_explode(
        explode_in.to_str().unwrap(),
        Some(&explode_prefix),
        Some(".fa"),
    );
    assert_eq!(exploded.len(), 2);
    assert_eq!(
        std::fs::read_to_string(&exploded[0]).unwrap(),
        ">a\nAC\n>b\nT-\n"
    );
    assert_eq!(
        std::fs::read_to_string(&exploded[1]).unwrap(),
        ">a\nAC\n>b\nT-\n"
    );
    std::fs::remove_file(&explode_in).unwrap();
    std::fs::remove_file(&exploded[0]).unwrap();
    std::fs::remove_file(&exploded[1]).unwrap();

    let fa2efa_msa1 =
        std::env::temp_dir().join(format!("muscle_rs_fa2efa_1_{}.fa", std::process::id()));
    let fa2efa_msa2 =
        std::env::temp_dir().join(format!("muscle_rs_fa2efa_2_{}.fa", std::process::id()));
    let fa2efa_paths =
        std::env::temp_dir().join(format!("muscle_rs_fa2efa_paths_{}.txt", std::process::id()));
    let fa2efa_out =
        std::env::temp_dir().join(format!("muscle_rs_fa2efa_out_{}.efa", std::process::id()));
    std::fs::write(&fa2efa_msa1, b">a\nAC\n>b\nT-\n").unwrap();
    std::fs::write(&fa2efa_msa2, b">b\nT-\n>a\nac\n").unwrap();
    std::fs::write(
        &fa2efa_paths,
        format!(
            "{}\n{}\n",
            fa2efa_msa1.to_string_lossy(),
            fa2efa_msa2.to_string_lossy()
        ),
    )
    .unwrap();
    let fa2efa = cmd_fa2efa(
        fa2efa_paths.to_str().unwrap(),
        fa2efa_out.to_str().unwrap(),
        true,
        true,
    );
    assert_eq!(
        fa2efa.msa_names,
        vec![
            base_name(fa2efa_msa1.to_str().unwrap()).to_string() + ".0",
            base_name(fa2efa_msa2.to_str().unwrap()).to_string() + ".1",
        ]
    );
    assert_eq!(ensemble_get_ix_subset(&fa2efa, 0.5), vec![0, 1, 2, 3]);
    assert_eq!(ensemble_get_ab_to_count(&fa2efa, 0), vec![0, 0, 2]);
    assert_eq!(ensemble_get_ab_to_count_all(&fa2efa), vec![0, 0, 0]);
    {
        let _guard = RNG_TEST_LOCK.lock().unwrap();
        reset_rand(1);
        let subsampled = ensemble_subsample_with_replacement_l577(&fa2efa, &[0, 1, 2, 3], 4);
        assert_eq!(
            subsampled
                .seqs
                .iter()
                .map(sequence_get_seq_as_string)
                .collect::<Vec<_>>(),
            vec!["CCAA".to_string(), "--TT".to_string()]
        );
        reset_rand(1);
        let subsampled_by_gap = ensemble_subsample_with_replacement_l569(&fa2efa, 0.5, 4);
        assert_eq!(
            subsampled_by_gap
                .seqs
                .iter()
                .map(sequence_get_seq_as_string)
                .collect::<Vec<_>>(),
            vec!["CCAA".to_string(), "--TT".to_string()]
        );
    }
    assert_eq!(
        ensemble_get_hi_qual_unique_ixs(&fa2efa, 0.5, 1.0),
        vec![0, 1]
    );
    assert_eq!(ensemble_get_median_hi_qual_col_count(&fa2efa, 0.5, 1.0), 2);
    assert_eq!(
        std::fs::read_to_string(&fa2efa_out).unwrap(),
        format!(
            "<{}.0\n>a\nAC\n>b\nT-\n<{}.1\n>a\nAC\n>b\nT-\n",
            base_name(fa2efa_msa1.to_str().unwrap()),
            base_name(fa2efa_msa2.to_str().unwrap())
        )
    );
    let bestconf_out =
        std::env::temp_dir().join(format!("muscle_rs_bestconf_out_{}.fa", std::process::id()));
    let (bestconf_log, best_total, best_median) =
        cmd_efa_bestconf(fa2efa_out.to_str().unwrap(), bestconf_out.to_str().unwrap());
    assert_eq!(best_total, 0);
    assert_eq!(best_median, 0);
    assert!(bestconf_log.contains("2 seqs, 2 MSAs, avg cols 2.0\n"));
    assert!(bestconf_log.contains("Best MSA, median 1 ("));
    assert_eq!(
        std::fs::read_to_string(&bestconf_out).unwrap(),
        ">a\nAC\n>b\nT-\n"
    );
    let bestcols_out =
        std::env::temp_dir().join(format!("muscle_rs_bestcols_out_{}.fa", std::process::id()));
    let bestcols = cmd_efa_bestcols(
        fa2efa_out.to_str().unwrap(),
        bestcols_out.to_str().unwrap(),
        1.0,
        0.5,
        uint::MAX,
    );
    assert_eq!(
        bestcols
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["CA".to_string(), "-T".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(&bestcols_out).unwrap(),
        ">a\nCA\n>b\n-T\n"
    );
    let qscore_log = cmd_qscore_efa(
        fa2efa_out.to_str().unwrap(),
        fa2efa_msa1.to_str().unwrap(),
        1.0,
    );
    assert!(qscore_log.contains("Q=1.0000 TC=1.0000"));
    let qscore_efa_cli_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-qscore_efa",
            fa2efa_out.to_str().unwrap(),
            "-ref",
            fa2efa_msa1.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(qscore_efa_cli_cmd.status.success());
    assert_eq!(
        String::from_utf8(qscore_efa_cli_cmd.stdout).unwrap(),
        qscore_log
    );
    let colscore_out =
        std::env::temp_dir().join(format!("muscle_rs_colscore_out_{}.txt", std::process::id()));
    let colscore_log = cmd_colscore_efa(
        fa2efa_out.to_str().unwrap(),
        fa2efa_msa1.to_str().unwrap(),
        colscore_out.to_str().unwrap(),
        0.5,
    );
    assert_eq!(
        std::fs::read_to_string(&colscore_out).unwrap(),
        colscore_log
    );
    assert!(colscore_log.starts_with("meantc\t1.0000\n"));
    assert!(colscore_log.contains("bin\t10\t4\t4\t1.0000\n"));
    assert_eq!(
        cmd_colscore_efa(
            fa2efa_out.to_str().unwrap(),
            fa2efa_msa1.to_str().unwrap(),
            "",
            0.5,
        ),
        colscore_log
    );
    let efastats_log = cmd_efastats(
        fa2efa_out.to_str().unwrap(),
        0.5,
        Some(fa2efa_msa1.to_str().unwrap()),
    );
    assert!(efastats_log.contains("2 seqs, 2 MSAs, avg cols 2.0"));
    assert!(efastats_log.contains("D_LP 0, D_Cols 0, CC 1"));
    assert!(efastats_log.contains("E_LP 0.0000, E_Cols 0.0000"));
    let addconf_out =
        std::env::temp_dir().join(format!("muscle_rs_addconf_out_{}.efa", std::process::id()));
    let addconf = cmd_addconfseq(
        fa2efa_out.to_str().unwrap(),
        addconf_out.to_str().unwrap(),
        Some(fa2efa_msa1.to_str().unwrap()),
        None,
        false,
    );
    assert_eq!(std::fs::read_to_string(&addconf_out).unwrap(), addconf);
    assert!(addconf.contains(">_conf_\n++\n>_conf_2\n++\n>a\nAC\n>b\nT-\n"));
    let addconf_no_output = cmd_addconfseq(
        fa2efa_out.to_str().unwrap(),
        "",
        Some(fa2efa_msa1.to_str().unwrap()),
        None,
        false,
    );
    assert_eq!(addconf_no_output, addconf);
    let letterconf_out = std::env::temp_dir().join(format!(
        "muscle_rs_letterconf_out_{}.fa",
        std::process::id()
    ));
    let letterconf = cmd_letterconf(
        fa2efa_out.to_str().unwrap(),
        fa2efa_msa1.to_str().unwrap(),
        letterconf_out.to_str().unwrap(),
        "",
        "",
        1.0,
    );
    assert_eq!(
        letterconf
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["^^".to_string(), "^-".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(&letterconf_out).unwrap(),
        ">a\n^^\n>b\n^-\n"
    );
    let letterconf_html_out = std::env::temp_dir().join(format!(
        "muscle_rs_letterconf_html_{}.html",
        std::process::id()
    ));
    let letterconf_html_in = std::env::temp_dir().join(format!(
        "muscle_rs_letterconf_html_in_{}.fa",
        std::process::id()
    ));
    std::fs::write(&letterconf_html_in, b">a\n9-\n>b\n9-\n").unwrap();
    cmd_letterconf_html(
        letterconf_html_in.to_str().unwrap(),
        fa2efa_msa1.to_str().unwrap(),
        letterconf_html_out.to_str().unwrap(),
    );
    let letterconf_html = std::fs::read_to_string(&letterconf_html_out).unwrap();
    assert!(letterconf_html.contains("<html"));
    assert!(letterconf_html.contains("a"));
    assert!(letterconf_html.contains("b"));
    assert!(letterconf_html.contains("Style9"));
    std::fs::remove_file(&letterconf_html_out).unwrap();
    std::fs::remove_file(&letterconf_html_in).unwrap();

    let addletter_out =
        std::env::temp_dir().join(format!("muscle_rs_addletter_out_{}.fa", std::process::id()));
    let addletter = cmd_addletterconfseq(
        fa2efa_out.to_str().unwrap(),
        fa2efa_msa1.to_str().unwrap(),
        addletter_out.to_str().unwrap(),
        1.0,
    );
    assert_eq!(std::fs::read_to_string(&addletter_out).unwrap(), addletter);
    assert_eq!(addletter, ">_letterconf_\n^/\n>a\nAC\n>b\nT-\n");
    assert_eq!(
        cmd_addletterconfseq(
            fa2efa_out.to_str().unwrap(),
            fa2efa_msa1.to_str().unwrap(),
            "",
            1.0,
        ),
        addletter
    );
    let maxcc_out =
        std::env::temp_dir().join(format!("muscle_rs_maxcc_out_{}.fa", std::process::id()));
    let (maxcc, maxcc_index, maxcc_avg) =
        cmd_maxcc(fa2efa_out.to_str().unwrap(), maxcc_out.to_str().unwrap());
    assert_eq!(maxcc_index, 1);
    assert!((maxcc_avg - 2.0).abs() < 1e-12);
    assert_eq!(
        maxcc
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["AC".to_string(), "T-".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(&maxcc_out).unwrap(),
        ">a\nAC\n>b\nT-\n"
    );
    assert!(cmd_disperse(fa2efa_out.to_str().unwrap(), 0.5).contains("D_LP=0 D_Cols=0"));
    let resample_out =
        std::env::temp_dir().join(format!("muscle_rs_resample_out_{}.efa", std::process::id()));
    {
        let _guard = RNG_TEST_LOCK.lock().unwrap();
        reset_rand(1);
        let reps = cmd_resample(
            fa2efa_out.to_str().unwrap(),
            resample_out.to_str().unwrap(),
            0.5,
            1.0,
            2,
        );
        assert_eq!(reps.len(), 2);
        assert_eq!(
            reps[0]
                .1
                .seqs
                .iter()
                .map(sequence_get_seq_as_string)
                .collect::<Vec<_>>(),
            vec!["CC".to_string(), "--".to_string()]
        );
        assert_eq!(
            reps[1]
                .1
                .seqs
                .iter()
                .map(sequence_get_seq_as_string)
                .collect::<Vec<_>>(),
            vec!["AA".to_string(), "TT".to_string()]
        );
    }
    let resample_low_qual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cmd_resample(
            fa2efa_out.to_str().unwrap(),
            resample_out.to_str().unwrap(),
            0.00012345,
            1.2345,
            1,
        )
    }));
    let resample_panic = resample_low_qual.unwrap_err();
    let resample_panic = resample_panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| resample_panic.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(
        resample_panic,
        "All columns low qual (max fract 0.000123, min conf 1.23)"
    );
    assert_eq!(
        std::fs::read_to_string(&resample_out).unwrap(),
        "<resampled.1\n>a\nCC\n>b\n--\n<resampled.2\n>a\nAA\n>b\nTT\n"
    );

    let relabel_in =
        std::env::temp_dir().join(format!("muscle_rs_relabel_in_{}.fa", std::process::id()));
    let relabel_labels = std::env::temp_dir().join(format!(
        "muscle_rs_relabel_labels_{}.txt",
        std::process::id()
    ));
    let relabel_out =
        std::env::temp_dir().join(format!("muscle_rs_relabel_out_{}.fa", std::process::id()));
    std::fs::write(&relabel_in, b">old1\nAC\n>old2\nTG\n>keep\nNN\n").unwrap();
    std::fs::write(&relabel_labels, b"old1\tnew1\nold2\tnew2\n").unwrap();
    assert_eq!(
        cmd_relabel(
            relabel_in.to_str().unwrap(),
            relabel_labels.to_str().unwrap(),
            relabel_out.to_str().unwrap(),
        ),
        ">new1\nAC\n>new2\nTG\n>keep\nNN\n"
    );
    assert_eq!(
        std::fs::read_to_string(&relabel_out).unwrap(),
        ">new1\nAC\n>new2\nTG\n>keep\nNN\n"
    );
    assert_eq!(
        cmd_relabel(
            relabel_in.to_str().unwrap(),
            relabel_labels.to_str().unwrap(),
            "",
        ),
        ">new1\nAC\n>new2\nTG\n>keep\nNN\n"
    );

    let a2m_in =
        std::env::temp_dir().join(format!("muscle_rs_make_a2m_in_{}.fa", std::process::id()));
    let a2m_out =
        std::env::temp_dir().join(format!("muscle_rs_make_a2m_out_{}.fa", std::process::id()));
    std::fs::write(&a2m_in, b">s1\nAC-G\n>s2\nA.-G\n>s3\nATTT\n").unwrap();
    assert_eq!(
        cmd_make_a2m(
            a2m_in.to_str().unwrap(),
            a2m_out.to_str().unwrap(),
            0.34,
            false,
        ),
        ">s1\nACG\n>s2\nA-G\n>s3\nATtT\n"
    );
    assert_eq!(
        std::fs::read_to_string(&a2m_out).unwrap(),
        ">s1\nACG\n>s2\nA-G\n>s3\nATtT\n"
    );
    assert_eq!(
        cmd_make_a2m(a2m_in.to_str().unwrap(), "", 0.34, false),
        ">s1\nACG\n>s2\nA-G\n>s3\nATtT\n"
    );
    std::fs::remove_file(&fa2efa_msa1).unwrap();
    std::fs::remove_file(&fa2efa_msa2).unwrap();
    std::fs::remove_file(&fa2efa_paths).unwrap();
    std::fs::remove_file(&fa2efa_out).unwrap();
    std::fs::remove_file(&bestconf_out).unwrap();
    std::fs::remove_file(&bestcols_out).unwrap();
    std::fs::remove_file(&colscore_out).unwrap();
    std::fs::remove_file(&addconf_out).unwrap();
    std::fs::remove_file(&letterconf_out).unwrap();
    std::fs::remove_file(&addletter_out).unwrap();
    std::fs::remove_file(&maxcc_out).unwrap();
    std::fs::remove_file(&resample_out).unwrap();
    std::fs::remove_file(&relabel_in).unwrap();
    std::fs::remove_file(&relabel_labels).unwrap();
    std::fs::remove_file(&relabel_out).unwrap();
    std::fs::remove_file(&a2m_in).unwrap();
    std::fs::remove_file(&a2m_out).unwrap();
    let mut all_gaps = MultiSequence::default();
    multi_sequence_from_strings(
        &mut all_gaps,
        &["g1".to_string(), "g2".to_string()],
        &["--".to_string(), "..".to_string()],
    );
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            delete_all_gap_columns(&mut all_gaps);
        }))
        .is_err()
    );
    let lines = vec![
        ">x".to_string(),
        "AC GT".to_string(),
        ">y".to_string(),
        "A-C.".to_string(),
    ];
    let parsed = msa_from_strings(&lines);
    assert_eq!(parsed.seqs[0].label, "x");
    assert_eq!(sequence_get_seq_as_string(&parsed.seqs[0]), "ACGT");
    assert_eq!(parsed.seqs[1].label, "y");
    assert_eq!(sequence_get_seq_as_string(&parsed.seqs[1]), "A-C.");
    let mut text_file = text_file_text_file_l63(b">m\nAC\n>n\nGT\n", "mem.fa");
    let parsed_from_file = msa_from_file(&mut text_file);
    assert_eq!(parsed_from_file.seqs[0].label, "m");
    assert_eq!(msa_get_row_str(&parsed_from_file, 1), "GT");
    let fasta_text = msa_to_file(&parsed_from_file);
    assert_eq!(fasta_text, ">m\nAC\n>n\nGT\n");
    let log_text = msa_log_me(&parsed_from_file);
    assert!(log_text.contains("           m           AC"));
    let rebuilt = msa_from_strings2(
        &["a".to_string(), "b".to_string()],
        &["ACGT".to_string(), "TGCA".to_string()],
    );
    assert_eq!(msa_to_fasta_file_l124(&rebuilt), ">a\nACGT\n>b\nTGCA\n");
    let long_row = "A".repeat(61);
    let long_msa = msa_from_strings2(&["long".to_string()], std::slice::from_ref(&long_row));
    assert_eq!(
        msa_to_fasta_file_l112(&long_msa),
        format!(">long\n{}\n", "A".repeat(61))
    );
    std::fs::create_dir_all(".tmp").unwrap();
    let msa_file = ".tmp/msa_from_fasta_case.fa";
    std::fs::write(msa_file, ">s1\naC\n>s2\nTg\n").unwrap();
    let loaded_upper = msa_from_fasta_file_l95(msa_file);
    assert_eq!(sequence_get_seq_as_string(&loaded_upper.seqs[0]), "AC");
    let loaded_case = msa_from_fasta_file_preserve_case(msa_file);
    assert_eq!(sequence_get_seq_as_string(&loaded_case.seqs[0]), "aC");
    let msa_out = ".tmp/msa_to_fasta.fa";
    msa_to_fasta_file_l103(&rebuilt, msa_out);
    assert_eq!(
        std::fs::read_to_string(msa_out).unwrap(),
        ">a\nACGT\n>b\nTGCA\n"
    );

    assert_eq!(fmt_char(b'A', 4), "A   ");
    assert_eq!(fmt_int(0, 3), ".  ");
    assert_eq!(fmt_int(42, 4), "42  ");
    assert_eq!(fmt_int0(0, 3), "0  ");
    assert_eq!(fmt_pad(3), "   ");
    assert_eq!(get_q_char(1.0), '|');
    assert_eq!(get_q_char(0.9), ':');
    assert_eq!(get_q_char(0.5), '.');
    assert_eq!(get_q_char(0.01), '@');
    assert_eq!(get_q_char(0.0), '*');
    assert_eq!(delete_not_upper("AbC-dE.F"), "AC-EF");
    let subst_dir =
        std::env::temp_dir().join(format!("muscle_rs_make_substmx_{}", std::process::id()));
    std::fs::create_dir_all(&subst_dir).unwrap();
    let subst_msa = subst_dir.join("one.fa");
    let subst_list = subst_dir.join("list.txt");
    let subst_out = subst_dir.join("mx.tsv");
    std::fs::write(
        &subst_msa,
        format!(">s1\n{AMINO_ALPHA}\n>s2\n{AMINO_ALPHA}\n"),
    )
    .unwrap();
    std::fs::write(&subst_list, format!("{}\n", subst_msa.to_string_lossy())).unwrap();
    let subst_log = cmd_make_substmx(
        subst_list.to_str().unwrap(),
        subst_out.to_str().unwrap(),
        Some("TESTMX"),
        Some(100),
        Some(100),
    );
    assert!(subst_log.contains("A  0.050000  4\n"));
    assert!(subst_log.contains("Sum = 1.000000"));
    assert!(subst_log.contains("100\t1\t1\n"));
    assert!(subst_log.contains("99\t0\t0\n"));
    assert!(subst_log.contains("Score matrix"));
    let subst_text = std::fs::read_to_string(&subst_out).unwrap();
    assert!(subst_text.starts_with("TESTMX\tA\tC\tD\tE"));
    assert!(subst_text.contains("A\t4.322"));
    assert!(
        std::panic::catch_unwind(|| cmd_make_substmx(
            subst_list.to_str().unwrap(),
            subst_out.to_str().unwrap(),
            None,
            Some(90),
            None,
        ))
        .is_err()
    );
    std::fs::remove_dir_all(&subst_dir).unwrap();

    let a2m_in = std::env::temp_dir().join(format!(
        "muscle_rs_make_a2m_refseq_in_{}.fa",
        std::process::id()
    ));
    let a2m_out = std::env::temp_dir().join(format!(
        "muscle_rs_make_a2m_refseq_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(&a2m_in, b">ref\nA-C.G\n>other\ntac-g\n").unwrap();
    cmd_make_a2m_refseq(
        a2m_in.to_str().unwrap(),
        a2m_out.to_str().unwrap(),
        None,
        false,
    );
    assert_eq!(
        std::fs::read_to_string(&a2m_out).unwrap(),
        ">ref\nACG\n>other\nTaCG\n"
    );
    cmd_make_a2m_refseq(
        a2m_in.to_str().unwrap(),
        a2m_out.to_str().unwrap(),
        Some("other"),
        false,
    );
    assert_eq!(
        std::fs::read_to_string(&a2m_out).unwrap(),
        ">other\nTACG\n>ref\nA-CG\n"
    );
    cmd_make_a2m_refseq(a2m_in.to_str().unwrap(), "", Some("other"), false);
    std::fs::remove_file(&a2m_in).unwrap();
    std::fs::remove_file(&a2m_out).unwrap();

    let qscore_ref =
        std::env::temp_dir().join(format!("muscle_rs_qscore_ref_{}.fa", std::process::id()));
    let qscore_test =
        std::env::temp_dir().join(format!("muscle_rs_qscore_test_{}.fa", std::process::id()));
    std::fs::write(&qscore_ref, b">r1\nAC-G\n>r2\nAT-G\n").unwrap();
    std::fs::write(&qscore_test, b">r1\nAC-G\n>r2\nAT-G\n").unwrap();
    assert_eq!(
        cmd_qscore(
            qscore_test.to_str().unwrap(),
            qscore_ref.to_str().unwrap(),
            false
        ),
        (1.0, 1.0)
    );
    let (old_perfect_q, old_perfect_tc, old_perfect_log) =
        cmd_qscore_oldcode(qscore_test.to_str().unwrap(), qscore_ref.to_str().unwrap());
    assert_eq!((old_perfect_q, old_perfect_tc), (1.0, 1.0));
    assert!(old_perfect_log.contains("Q=1, TC=1"));
    assert!(!old_perfect_log.contains("Q=1.000"));
    std::fs::write(&qscore_test, b">x\nAC-G\n>y\nAT-G\n").unwrap();
    assert_eq!(
        cmd_qscore(
            qscore_test.to_str().unwrap(),
            qscore_ref.to_str().unwrap(),
            true
        ),
        (1.0, 1.0)
    );
    std::fs::write(&qscore_test, b">r1\nA-CG\n>r2\nAT-G\n").unwrap();
    assert_eq!(
        cmd_qscore(
            qscore_test.to_str().unwrap(),
            qscore_ref.to_str().unwrap(),
            false
        ),
        (2.0 / 3.0, 2.0 / 3.0)
    );
    let (old_q, old_tc, old_log) =
        cmd_qscore_oldcode(qscore_test.to_str().unwrap(), qscore_ref.to_str().unwrap());
    assert!((old_q - (2.0 / 3.0)).abs() < 1e-12);
    assert!((old_tc - (2.0 / 3.0)).abs() < 1e-12);
    assert!(old_log.contains("Q=0.667, TC=0.667"));
    std::fs::remove_file(&qscore_ref).unwrap();
    std::fs::remove_file(&qscore_test).unwrap();

    {
        let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
        *state = MakeSubstMxState {
            aln_count: 0,
            pct_id_counts: vec![0; 101],
            letter_counts: vec![0; 20],
            total_letters: 0,
            letter_pair_counts: vec![vec![0; 20]; 20],
            total_pairs: 0,
            min_pct_id: 0,
            max_pct_id: 100,
        };
    }
    add_pair('A', 'C');
    add_pair('-', 'A');
    add_pair('A', '-');
    add_pair('A', 'b');
    {
        let state = MAKE_SUBST_MX_STATE.lock().unwrap();
        assert_eq!(state.total_letters, 2);
        assert_eq!(state.total_pairs, 2);
        assert_eq!(state.letter_counts[0], 1);
        assert_eq!(state.letter_counts[1], 1);
        assert_eq!(state.letter_pair_counts[0][1], 1);
        assert_eq!(state.letter_pair_counts[1][0], 1);
    }

    {
        let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
        *state = MakeSubstMxState {
            aln_count: 0,
            pct_id_counts: vec![0; 101],
            letter_counts: vec![0; 20],
            total_letters: 0,
            letter_pair_counts: vec![vec![0; 20]; 20],
            total_pairs: 0,
            min_pct_id: 0,
            max_pct_id: 100,
        };
    }
    add_rows("AC-D", "ADCD");
    {
        let state = MAKE_SUBST_MX_STATE.lock().unwrap();
        assert_eq!(state.aln_count, 1);
        assert_eq!(state.pct_id_counts[50], 1);
        assert_eq!(state.total_letters, 12);
        assert_eq!(state.total_pairs, 12);
        assert_eq!(state.letter_counts[0], 4);
        assert_eq!(state.letter_counts[1], 2);
        assert_eq!(state.letter_counts[2], 6);
        assert_eq!(state.letter_pair_counts[0][0], 4);
        assert_eq!(state.letter_pair_counts[1][2], 2);
        assert_eq!(state.letter_pair_counts[2][1], 2);
        assert_eq!(state.letter_pair_counts[2][2], 4);
    }

    {
        let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
        *state = MakeSubstMxState {
            aln_count: 0,
            pct_id_counts: vec![0; 101],
            letter_counts: vec![0; 20],
            total_letters: 0,
            letter_pair_counts: vec![vec![0; 20]; 20],
            total_pairs: 0,
            min_pct_id: 80,
            max_pct_id: 100,
        };
    }
    add_rows("AC-D", "ADCD");
    {
        let state = MAKE_SUBST_MX_STATE.lock().unwrap();
        assert_eq!(state.aln_count, 1);
        assert_eq!(state.pct_id_counts.iter().sum::<uint>(), 0);
        assert_eq!(state.total_letters, 6);
        assert_eq!(state.total_pairs, 6);
    }

    assert_eq!(logize_str("1"), "           .");
    assert_eq!(expize_str("0"), "     1.00000");
    assert_eq!(logize_str("2.718281828459045"), "     1.00000");
    assert_eq!(expize_str("20"), "   4.852e+08");
    let mut mx_base = MxBase::default();
    mx_base_on_ctor(&mut mx_base);
    mx_base_alloc(&mut mx_base, 2, 3, "base");
    assert_eq!(mx_base.name, "base");
    assert_eq!(mx_base.row_count, 2);
    assert_eq!(mx_base.col_count, 3);
    assert!(mx_base.allocated_row_count >= 18);
    assert!(mx_base.allocated_col_count >= 19);
    mx_base.data[0][0] = 0.0;
    mx_base.data[0][1] = 2.0;
    mx_base.data[0][2] = 1_234_567.0;
    mx_base.data[1][0] = UNINIT;
    let log = mx_base_log_me(&mx_base, true, 0);
    assert!(log.contains("base Rows 2/"));
    assert!(log.contains("     2.00000"));
    assert!(log.contains("   1.235e+06"));
    let log_exp = mx_base_log_me(&mx_base, true, OPT_EXP);
    assert!(log_exp.starts_with("\nExp base"));
    mx_base_alloc(&mut mx_base, 40, 2, "grown");
    assert_eq!(mx_base.name, "grown");
    assert_eq!(mx_base.row_count, 40);
    let counts = mx_base_log_counts();
    assert!(counts.contains("MxBase::LogCounts()"));
    assert!(counts.contains("Allocs"));
    mx_base_on_dtor(&mut mx_base);
}

#[test]
fn multi_sequence_string_and_length_helpers_match_cpp_logic() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    let labels = vec!["s1".to_string(), "s2".to_string(), "s3".to_string()];
    let seqs = vec!["A-C.".to_string(), "....".to_string(), "ACG.".to_string()];
    let mut ms = MultiSequence::default();
    multi_sequence_from_strings(&mut ms, &labels, &seqs);

    assert_eq!(ms.seqs.len(), 3);
    assert_eq!(ms.owners, vec![true, true, true]);
    multi_sequence_assert_seq_ids(&ms);
    let ms_log = multi_sequence_log_me(&ms);
    assert!(ms_log.contains("MultiSequence::LogMe("));
    assert!(ms_log.ends_with("A-C.  >s1 (4)\n....  >s2 (4)\nACG.  >s3 (4)\n"));
    assert!(multi_sequence_is_aligned(&ms));
    assert_eq!(multi_sequence_get_col_count(&ms), 4);
    assert_eq!(multi_sequence_get_seq_length(&ms, 0), 4);
    assert_eq!(multi_sequence_get_seq_index(&ms, "s2", true), 1);
    assert_eq!(
        progress_log_input_summary("input.fa", &ms),
        "\nInput  input.fa\nSeqs   3\nMinL   4\nMaxL   4\n\n"
    );
    assert_eq!(
        progress_log_msa_summary("MSA summary", &ms),
        "\nMSA summary\nSeqs   3\nCols   4\n\n"
    );
    assert_eq!(
        multi_sequence_get_seq_index(&ms, "missing", false),
        uint::MAX
    );
    assert!(multi_sequence_col_is_all_gaps(&ms, 3));
    assert!(!multi_sequence_col_is_all_gaps(&ms, 0));
    assert_eq!(multi_sequence_get_length_order(&ms), vec![0, 1, 2]);
    assert_eq!(multi_sequence_get_mean_seq_length(&ms), 4.0);
    assert_eq!(multi_sequence_get_max_seq_length(&ms), 4);
    assert_eq!(multi_sequence_get_min_seq_length(&ms), 4);
    let msa_copy = multi_sequence_to_msa(&ms);
    assert_eq!(msa_copy, ms);
    assert_eq!(
        show_seq_stats(&ms),
        "Input: 3 seqs, avg length 4, max 4, min 4\n\n"
    );

    let projected = multi_sequence_project_l3(&ms, &[2, 0, 2]);
    assert_eq!(projected.seqs.len(), 2);
    assert_eq!(projected.owners, vec![true, true]);
    assert_eq!(projected.seqs[0].label, "s1");
    assert_eq!(sequence_get_seq_as_string(&projected.seqs[0]), "A-C.");
    assert_eq!(projected.seqs[1].label, "s3");
    assert_eq!(sequence_get_seq_as_string(&projected.seqs[1]), "ACG.");

    let mut index_set = std::collections::BTreeSet::new();
    index_set.insert(1);
    index_set.insert(2);
    let projected = multi_sequence_project_l16(&ms, &index_set);
    assert_eq!(projected.seqs[0].label, "s2");
    assert_eq!(sequence_get_seq_as_string(&projected.seqs[0]), "....");
    assert_eq!(projected.seqs[1].label, "s3");
    assert_eq!(sequence_get_seq_as_string(&projected.seqs[1]), "ACG.");

    let uneven_seqs = vec!["AC".to_string(), "ACGT".to_string(), "A".to_string()];
    multi_sequence_from_strings(&mut ms, &labels, &uneven_seqs);
    assert!(!multi_sequence_is_aligned(&ms));
    assert_eq!(multi_sequence_get_length_order(&ms), vec![1, 0, 2]);
    assert_eq!(multi_sequence_get_mean_seq_length(&ms), 7.0 / 3.0);
    assert_eq!(multi_sequence_get_max_seq_length(&ms), 4);
    assert_eq!(multi_sequence_get_min_seq_length(&ms), 1);

    let mut copied = MultiSequence::default();
    multi_sequence_copy(&mut copied, &ms);
    assert_eq!(copied.seqs, ms.seqs);
    assert_eq!(copied.owners, vec![true, true, true]);
    multi_sequence_clear(&mut copied);
    assert!(copied.seqs.is_empty());
    assert!(copied.owners.is_empty());

    let nucleo_labels = vec!["n1".to_string(), "n2".to_string()];
    let nucleo_seqs = vec!["ACGTACGT".to_string(), "UUUUAAAA".to_string()];
    multi_sequence_from_strings(&mut ms, &nucleo_labels, &nucleo_seqs);
    reset_rand(1);
    assert!(multi_sequence_guess_is_nucleo(&ms));

    let amino_seqs = vec!["FFFFFFFF".to_string(), "PPPPPPPP".to_string()];
    multi_sequence_from_strings(&mut ms, &nucleo_labels, &amino_seqs);
    reset_rand(1);
    assert!(!multi_sequence_guess_is_nucleo(&ms));

    let derep_labels = vec![
        "r1".to_string(),
        "r2".to_string(),
        "r3".to_string(),
        "r4".to_string(),
    ];
    let derep_seqs = vec![
        "AcgT".to_string(),
        "acgt".to_string(),
        "AAAA".to_string(),
        "ACGA".to_string(),
    ];
    let mut derep_ms = MultiSequence::default();
    multi_sequence_from_strings(&mut derep_ms, &derep_labels, &derep_seqs);
    let mut derep = Derep::default();
    derep_run(&mut derep, &derep_ms, false);
    derep_validate(&derep);
    assert_eq!(derep.slot_count, 19);
    assert_eq!(derep.rep_seq_indexes, vec![0, 2, 3]);
    assert_eq!(derep.seq_index_to_rep_seq_index, vec![0, 0, 2, 3]);
    assert!(derep_seqs_eq(&derep, 0, 1));
    assert!(!derep_seqs_eq(&derep, 0, 2));
    assert_eq!(derep_search(&derep, 1), 0);
    let mut unique_seqs = MultiSequence::default();
    derep_get_unique_seqs(&derep, &mut unique_seqs);
    assert_eq!(unique_seqs.owners, vec![false, false, false]);
    assert_eq!(
        unique_seqs
            .seqs
            .iter()
            .map(sequence_get_seq_as_string)
            .collect::<Vec<_>>(),
        vec!["AcgT".to_string(), "AAAA".to_string(), "ACGA".to_string()]
    );
    let rep_to_dupes = derep_get_rep_label_to_dupe_labels(&derep);
    assert_eq!(rep_to_dupes.get("r1"), Some(&vec!["r2".to_string()]));

    let derep_in =
        std::env::temp_dir().join(format!("muscle_rs_derep_in_{}.fa", std::process::id()));
    let derep_out =
        std::env::temp_dir().join(format!("muscle_rs_derep_out_{}.fa", std::process::id()));
    std::fs::write(&derep_in, b">r1\nAcgT\n>r2\nacgt\n>r3\nAAAA\n").unwrap();
    cmd_derep(derep_in.to_str().unwrap(), derep_out.to_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(&derep_out).unwrap(),
        ">r1\nAcgT\n>r3\nAAAA\n"
    );
    cmd_derep(derep_in.to_str().unwrap(), "");
    std::fs::remove_file(&derep_in).unwrap();
    std::fs::remove_file(&derep_out).unwrap();

    let rdrp_input = "muscle/test_data/rdrp/rdrp.fa";
    let rdrp_out =
        std::env::temp_dir().join(format!("muscle_rs_rdrp_derep_{}.fa", std::process::id()));
    cmd_derep(rdrp_input, rdrp_out.to_str().unwrap());
    let rdrp_text = std::fs::read_to_string(&rdrp_out).unwrap();
    assert_eq!(
        rdrp_text
            .lines()
            .filter(|line| line.starts_with('>'))
            .count(),
        4494
    );
    assert!(rdrp_text.starts_with(">AB000906.1_Infectious_flacherie_virus_A\n"));
    std::fs::remove_file(&rdrp_out).unwrap();

    let strip_in =
        std::env::temp_dir().join(format!("muscle_rs_strip_rows_in_{}.fa", std::process::id()));
    let strip_out = std::env::temp_dir().join(format!(
        "muscle_rs_strip_rows_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(&strip_in, b">keep1\nAC-G\n>drop\nA---\n>keep2\n....\n").unwrap();
    assert_eq!(
        cmd_strip_gappy_rows(
            strip_in.to_str().unwrap(),
            strip_out.to_str().unwrap(),
            0.75
        ),
        1
    );
    assert_eq!(
        std::fs::read_to_string(&strip_out).unwrap(),
        ">keep1\nAC-G\n>drop\nA---\n"
    );
    assert_eq!(
        cmd_strip_gappy_rows(strip_in.to_str().unwrap(), "", 0.75),
        1
    );
    std::fs::remove_file(&strip_in).unwrap();
    std::fs::remove_file(&strip_out).unwrap();

    let strip_cols_in =
        std::env::temp_dir().join(format!("muscle_rs_strip_cols_in_{}.fa", std::process::id()));
    let strip_cols_out = std::env::temp_dir().join(format!(
        "muscle_rs_strip_cols_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(&strip_cols_in, b">a\nA-C.\n>b\nAT-.\n>c\nAGTT\n").unwrap();
    assert_eq!(
        cmd_strip_gappy_cols(
            strip_cols_in.to_str().unwrap(),
            strip_cols_out.to_str().unwrap(),
            0.5
        ),
        1
    );
    assert_eq!(
        std::fs::read_to_string(&strip_cols_out).unwrap(),
        ">a\nA-C\n>b\nAT-\n>c\nAGT\n"
    );
    assert_eq!(
        cmd_strip_gappy_cols(strip_cols_in.to_str().unwrap(), "", 0.5),
        1
    );
    std::fs::remove_file(&strip_cols_in).unwrap();
    std::fs::remove_file(&strip_cols_out).unwrap();

    let stats_in =
        std::env::temp_dir().join(format!("muscle_rs_msastats_in_{}.fa", std::process::id()));
    std::fs::write(&stats_in, b">a\nA-C.\n>b\nAT-.\n>c\naGTT\n").unwrap();
    assert_eq!(
        cmd_msastats(stats_in.to_str().unwrap(), Some(0.5)),
        concat!(
            "         3  Sequences\n",
            "         4  Columns\n",
            "       2.7  Mean seq length  min 2, median 2, max 4\n",
            "      33.0  Mean col gap pct, min 0, median 33, max 66\n",
            "         1  Cols with no gaps (25.0% of cols)\n",
            "         3  Cols with <50.0% gaps (75.0% of cols)\n",
            "         3  Upper-case (0 lower, 1 mixed)\n",
            "     50.0%  Dash gaps\n\n",
        )
    );
    std::fs::remove_file(&stats_in).unwrap();

    let strip_both_in =
        std::env::temp_dir().join(format!("muscle_rs_strip_both_in_{}.fa", std::process::id()));
    let strip_both_out = std::env::temp_dir().join(format!(
        "muscle_rs_strip_both_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(
        &strip_both_in,
        b">keep\nA-C.\n>drop_dash\n----\n>keep_dot\nA...\n",
    )
    .unwrap();
    assert_eq!(
        cmd_strip_gappy(
            strip_both_in.to_str().unwrap(),
            strip_both_out.to_str().unwrap(),
            0.67,
            0.5
        ),
        (2, 1)
    );
    assert_eq!(
        std::fs::read_to_string(&strip_both_out).unwrap(),
        ">keep\nAC\n>keep_dot\nA.\n"
    );
    assert_eq!(
        cmd_strip_gappy(strip_both_in.to_str().unwrap(), "", 0.67, 0.5),
        (2, 1)
    );
    std::fs::remove_file(&strip_both_in).unwrap();
    std::fs::remove_file(&strip_both_out).unwrap();

    let anchors_in = std::env::temp_dir().join(format!(
        "muscle_rs_strip_anchors_in_{}.fa",
        std::process::id()
    ));
    let anchors_out = std::env::temp_dir().join(format!(
        "muscle_rs_strip_anchors_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(&anchors_in, b">a\nAC1G2T\n>b\nTT1A2C\n").unwrap();
    assert_eq!(
        cmd_strip_anchors(anchors_in.to_str().unwrap(), anchors_out.to_str().unwrap()),
        2
    );
    assert_eq!(
        std::fs::read_to_string(&anchors_out).unwrap(),
        ">a\nACGT\n>b\nTTAC\n"
    );
    assert_eq!(cmd_strip_anchors(anchors_in.to_str().unwrap(), ""), 2);
    std::fs::remove_file(&anchors_in).unwrap();
    std::fs::remove_file(&anchors_out).unwrap();

    let uclust_labels = vec!["s0".to_string(), "s1".to_string(), "s2".to_string()];
    let uclust_seqs = vec!["AA".to_string(), "CC".to_string(), "GG".to_string()];
    let mut uclust_ms = MultiSequence::default();
    multi_sequence_from_strings(&mut uclust_ms, &uclust_labels, &uclust_seqs);
    let uclust = UClustPD {
        input_seqs: Some(uclust_ms),
        subset_seq_indexes: vec![0, 1, 2],
        max_pd: 1.0,
        thread_count: 1,
        pending_subset_indexes: Vec::new(),
        centroid_seq_indexes: vec![0, 2],
        centroid_index_to_member_subset_indexes: vec![vec![0, 1], vec![2]],
        subset_index_to_centroid_index: vec![0, 0, 1],
        subset_index_to_dist: vec![0.0, 0.3, 0.0],
    };
    assert_eq!(u_clust_pd_get_label(&uclust, 1), "s1");
    assert_eq!(u_clust_pd_get_byte_seq(&uclust, 2), (b"GG".to_vec(), 2));
    let mut pd_path = String::new();
    let pd_dist = u_clust_pd_get_prot_dist_pair(
        &uclust,
        0,
        1,
        Some(&mut pd_path),
        |seqi, li, seqj, lj| {
            assert_eq!(&seqi[..li as usize], b"AA");
            assert_eq!(&seqj[..lj as usize], b"CC");
            PathInfo {
                path: "BB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(row_x, "AA");
            assert_eq!(row_y, "CC");
            assert_eq!(col_count, 2);
            0.75
        },
    );
    assert_eq!(pd_dist, 0.75);
    assert_eq!(pd_path, "BB");
    let mut search_all_calls = Vec::new();
    assert_eq!(
        u_clust_pd_search_all(&uclust, 0, |seq_index1, seq_index2| {
            search_all_calls.push((seq_index1, seq_index2));
            if seq_index2 == 1 { 0.2 } else { 2.0 }
        }),
        1
    );
    assert_eq!(search_all_calls, vec![(0, 1), (0, 2)]);
    let mut search_calls = Vec::new();
    let (best_centroid, best_dist) =
        u_clust_pd_search(&uclust, 1, &[0, 1], |seq_index1, seq_index2| {
            search_calls.push((seq_index1, seq_index2));
            if seq_index2 == 0 { 0.8 } else { 0.3 }
        });
    assert_eq!(best_centroid, 1);
    assert!((best_dist - 0.3).abs() < f64::EPSILON);
    assert_eq!(search_calls, vec![(1, 0), (1, 2)]);
    let gtb = GTBNode {
        builder: Some(GTBuilder {
            target_seed_count: 32,
            max_all: 1024,
            seqs: uclust.input_seqs.clone(),
        }),
        seq_indexes: vec![0, 1, 2],
        ..GTBNode::default()
    };
    assert_eq!(gtb_node_get_label(&gtb, 1), "s1");
    assert_eq!(gtb_node_get_byte_seq(&gtb, 2), b"GG".to_vec());
    let gtb_dist = gtb_node_get_prot_dist(
        &gtb,
        0,
        1,
        |seqi, li, seqj, lj| {
            assert_eq!(&seqi[..li as usize], b"AA");
            assert_eq!(&seqj[..lj as usize], b"CC");
            PathInfo {
                path: "BB".to_string(),
                ..PathInfo::default()
            }
        },
        |row_x, row_y, col_count| {
            assert_eq!(row_x, "AA");
            assert_eq!(row_y, "CC");
            assert_eq!(col_count, 2);
            0.66
        },
    );
    assert_eq!(gtb_dist, 0.66);
    let mut gtb_all = gtb.clone();
    let mut gtb_pairs = Vec::new();
    gtb_node_do_all(
        &mut gtb_all,
        |node, seq_indexi, seq_indexj| {
            gtb_pairs.push((seq_indexi, seq_indexj));
            let label_i = gtb_node_get_label(node, seq_indexi);
            let label_j = gtb_node_get_label(node, seq_indexj);
            (label_i.len() + label_j.len()) as f64 + f64::from(seq_indexi + seq_indexj)
        },
        |upgma, tree| {
            assert_eq!(
                upgma.labels,
                vec!["s0".to_string(), "s1".to_string(), "s2".to_string()]
            );
            tree.node_count = upgma.labels.len() as uint;
        },
    );
    assert_eq!(gtb_pairs, vec![(1, 0), (2, 0), (2, 1)]);
    assert_eq!(gtb_all.upgma.dist_mx[0][1], 5.0);
    assert_eq!(gtb_all.upgma.dist_mx[2][1], 7.0);
    assert_eq!(gtb_all.tree.node_count, 3);
    let mut gtb_seed = GTBNode {
        builder: Some(GTBuilder {
            target_seed_count: 1,
            max_all: 3,
            seqs: uclust.input_seqs.clone(),
        }),
        seq_indexes: vec![0, 1, 2],
        ..GTBNode::default()
    };
    let mut child_sizes = Vec::new();
    gtb_node_run(
        &mut gtb_seed,
        |_node, seq_indexi, seq_indexj| f64::from(seq_indexi + seq_indexj),
        |upgma, tree| {
            assert_eq!(upgma.labels, vec!["Seed0.s0".to_string()]);
            tree.node_count = 1;
        },
        |child| child_sizes.push(child.seq_indexes.len() as uint),
    );
    assert_eq!(gtb_seed.seed_seq_indexes, vec![0]);
    assert_eq!(gtb_seed.children.len(), 1);
    assert_eq!(gtb_seed.children[0].seq_indexes, vec![0, 1, 2]);
    assert_eq!(child_sizes, vec![3]);
    assert_eq!(gtb_seed.tree.node_count, 1);
    assert_eq!(u_clust_pd_get_cluster_size(&uclust, 0), 2);
    assert_eq!(u_clust_pd_to_tsv_l269(&uclust), "0\ts0\n0\ts1\n1\ts2\n");
    let uclust_tsv = std::env::temp_dir().join(format!(
        "muscle_rs_uclustpd_clusters_{}.tsv",
        std::process::id()
    ));
    u_clust_pd_to_tsv_l260(&uclust, uclust_tsv.to_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(&uclust_tsv).unwrap(),
        "0\ts0\n0\ts1\n1\ts2\n"
    );
    std::fs::remove_file(&uclust_tsv).unwrap();

    let mut cluster_mfa = MultiSequence::default();
    u_clust_pd_get_cluster_mfa(&uclust, 0, &mut cluster_mfa);
    assert_eq!(
        cluster_mfa
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["s0".to_string(), "s1".to_string()]
    );
    assert_eq!(cluster_mfa.owners, vec![false, false]);
    let cluster_mfas = u_clust_pd_get_cluster_mf_as(&uclust);
    assert_eq!(cluster_mfas.len(), 2);
    assert_eq!(sequence_get_seq_as_string(&cluster_mfas[1].seqs[0]), "GG");
    assert_eq!(u_clust_pd_get_cluster_sizes(&uclust), vec![2, 1]);
    assert_eq!(
        u_clust_pd_log_stats(&uclust),
        "\n3 seqs, 2 clusters, avg size 1.0, median 1, singletons 1\n    Cluster  [    0]   size 2\n    Cluster  [    1]   size 1\n"
    );
    let candidate_uc = UClustPD {
        input_seqs: uclust.input_seqs.clone(),
        subset_seq_indexes: (0..17).collect(),
        centroid_seq_indexes: vec![0, 8],
        centroid_index_to_member_subset_indexes: vec![(0..8).collect(), (8..17).collect()],
        ..UClustPD::default()
    };
    reset_rand(1);
    let mut candidate_searches = Vec::new();
    let candidates =
        u_clust_pd_select_candidate_good_centroids(&candidate_uc, |_uc, subset_index| {
            candidate_searches.push(subset_index);
            if subset_index < 8 { 20 } else { 1 }
        });
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0] < 8);
    assert_eq!(candidates[1], 8);
    assert_eq!(candidate_searches.len(), 2);
    let mut run_uc = UClustPD::default();
    let run_log = u_clust_pd_run(
        &mut run_uc,
        uclust.input_seqs.as_ref().unwrap(),
        &[0, 1, 2],
        1.0,
        1,
        |uc, seq_index, centroids| {
            let mut best = (uint::MAX, f64::from(f32::MAX));
            for centroid_index in centroids {
                let centroid_seq_index = uc.centroid_seq_indexes[*centroid_index as usize];
                let d = if seq_index == centroid_seq_index {
                    0.0
                } else if (seq_index == 1 && centroid_seq_index == 0)
                    || (seq_index == 0 && centroid_seq_index == 1)
                {
                    0.25
                } else {
                    2.0
                };
                if d <= uc.max_pd && d < best.1 {
                    best = (*centroid_index, d);
                }
            }
            best
        },
    );
    assert!(run_log.starts_with("Iter [1] 0 clusters, (assigned 0%, remaining 3)\n"));
    assert!(run_log.contains("Iter [2] 1 clusters, (assigned 66.7%, remaining 1)\n"));
    assert_eq!(run_uc.pending_subset_indexes, Vec::<uint>::new());
    assert_eq!(run_uc.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(run_uc.subset_index_to_centroid_index, vec![0, 0, 1]);
    assert_eq!(run_uc.subset_index_to_dist, vec![0.0, 0.25, 0.0]);
    assert_eq!(
        run_uc.centroid_index_to_member_subset_indexes,
        vec![vec![0, 1], vec![2]]
    );
    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut uc_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut uc_input,
        &["u0".to_string(), "u1".to_string(), "u2".to_string()],
        &["AAAA".to_string(), "AAAT".to_string(), "GGGG".to_string()],
    );
    set_global_input_ms(&uc_input);
    let mut run_u = UClust::default();
    let u_log = u_clust_run(&mut run_u, &uc_input, 0.9, |label1, label2| {
        if (label1 == "u1" && label2 == "u0") || (label1 == "u0" && label2 == "u1") {
            (0.95, "BBBB".to_string())
        } else {
            (0.1, "XXXX".to_string())
        }
    });
    assert!(u_log.starts_with("UCLUST 3 seqs EE<0.10, 0 centroids, 0 members\n"));
    assert_eq!(run_u.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(run_u.seq_index_to_centroid_seq_index, vec![0, 0, 2]);
    assert_eq!(
        run_u.seq_index_to_path,
        vec![String::new(), "BBBB".to_string(), String::new()]
    );
    let mut run_centroids = MultiSequence::default();
    u_clust_get_centroid_seqs(&run_u, &mut run_centroids);
    assert_eq!(
        run_centroids
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["u0".to_string(), "u2".to_string()]
    );
    let uclust_fasta = std::env::temp_dir().join(format!(
        "muscle_rs_uclustpd_centroids_{}.fa",
        std::process::id()
    ));
    u_clust_pd_centroids_to_fasta(&uclust, uclust_fasta.to_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(&uclust_fasta).unwrap(),
        ">s0\nAA\n>s2\nGG\n"
    );
    std::fs::remove_file(&uclust_fasta).unwrap();

    let uclustpd_in =
        std::env::temp_dir().join(format!("muscle_rs_uclustpd_cmd_{}.fa", std::process::id()));
    let uclustpd_out =
        std::env::temp_dir().join(format!("muscle_rs_uclustpd_cmd_{}.tsv", std::process::id()));
    std::fs::write(
        &uclustpd_in,
        b">pd_a\nEFKLEF\n>pd_b\nEFKLEQ\n>pd_c\nPQRSWY\n",
    )
    .unwrap();
    let mut cmd_uclustpd_calls = Vec::new();
    let (cmd_ud, cmd_ud_log, cmd_ud_stats) = cmd_uclustpd(
        uclustpd_in.to_str().unwrap(),
        uclustpd_out.to_str().unwrap(),
        0.3,
        1,
        |uc, seq_indexi, seq_indexj| {
            let labeli = u_clust_pd_get_label(uc, seq_indexi);
            let labelj = u_clust_pd_get_label(uc, seq_indexj);
            cmd_uclustpd_calls.push((labeli.clone(), labelj.clone()));
            if (labeli == "pd_b" && labelj == "pd_a") || (labeli == "pd_a" && labelj == "pd_b") {
                0.25
            } else {
                0.9
            }
        },
    );
    assert!(cmd_ud_log.starts_with("Iter [1] 0 clusters"));
    assert!(cmd_ud_stats.contains("3 seqs, 2 clusters"));
    assert_eq!(cmd_ud.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(cmd_ud.subset_index_to_centroid_index, vec![0, 0, 1]);
    assert_eq!(
        std::fs::read_to_string(&uclustpd_out).unwrap(),
        "0\tpd_a\n0\tpd_b\n1\tpd_c\n"
    );
    assert!(cmd_uclustpd_calls.contains(&("pd_b".to_string(), "pd_a".to_string())));
    std::fs::remove_file(&uclustpd_in).unwrap();
    std::fs::remove_file(&uclustpd_out).unwrap();

    let uclustpd2_in =
        std::env::temp_dir().join(format!("muscle_rs_uclustpd2_cmd_{}.fa", std::process::id()));
    let uclustpd2_out1 = std::env::temp_dir().join(format!(
        "muscle_rs_uclustpd2_cmd_{}_1.tsv",
        std::process::id()
    ));
    let uclustpd2_out2 = std::env::temp_dir().join(format!(
        "muscle_rs_uclustpd2_cmd_{}_2.tsv",
        std::process::id()
    ));
    let uclustpd2_centroids = std::env::temp_dir().join(format!(
        "muscle_rs_uclustpd2_cmd_{}_centroids.fa",
        std::process::id()
    ));
    std::fs::write(
        &uclustpd2_in,
        b">p2_a\nEFKLEF\n>p2_b\nEFKLEQ\n>p2_c\nPQRSWY\n",
    )
    .unwrap();
    let mut cmd_uclustpd2_calls = Vec::new();
    let (cmd_ud2, selected, reordered, run_log1, stats1, run_log2, stats2) = cmd_uclustpd2(
        uclustpd2_in.to_str().unwrap(),
        uclustpd2_out1.to_str().unwrap(),
        uclustpd2_out2.to_str().unwrap(),
        uclustpd2_centroids.to_str().unwrap(),
        0.3,
        1,
        |uc, seq_indexi, seq_indexj| {
            let labeli = u_clust_pd_get_label(uc, seq_indexi);
            let labelj = u_clust_pd_get_label(uc, seq_indexj);
            cmd_uclustpd2_calls.push((labeli.clone(), labelj.clone()));
            if (labeli == "p2_b" && labelj == "p2_a") || (labeli == "p2_a" && labelj == "p2_b") {
                0.25
            } else {
                0.9
            }
        },
        |_uc, _subset_index| 0,
    );
    assert!(run_log1.starts_with("Iter [1] 0 clusters"));
    assert!(run_log2.starts_with("Iter [1] 0 clusters"));
    assert!(stats1.contains("3 seqs, 2 clusters"));
    assert!(stats2.contains("3 seqs, 2 clusters"));
    assert!(selected.is_empty());
    assert_eq!(reordered, vec![0, 1, 2]);
    assert_eq!(cmd_ud2.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(
        std::fs::read_to_string(&uclustpd2_out1).unwrap(),
        "0\tp2_a\n0\tp2_b\n1\tp2_c\n"
    );
    assert_eq!(
        std::fs::read_to_string(&uclustpd2_out2).unwrap(),
        "0\tp2_a\n0\tp2_b\n1\tp2_c\n"
    );
    assert_eq!(
        std::fs::read_to_string(&uclustpd2_centroids).unwrap(),
        ">p2_a\nEFKLEF\n>p2_c\nPQRSWY\n"
    );
    assert!(cmd_uclustpd2_calls.contains(&("p2_b".to_string(), "p2_a".to_string())));
    std::fs::remove_file(&uclustpd2_in).unwrap();
    std::fs::remove_file(&uclustpd2_out1).unwrap();
    std::fs::remove_file(&uclustpd2_out2).unwrap();
    std::fs::remove_file(&uclustpd2_centroids).unwrap();

    let mut mpc = MPCFlat::default();
    mpc_flat_init_seqs(&mut mpc, &derep_ms);
    assert_eq!(mpc_flat_get_seq_count(&mpc), 4);
    assert_eq!(mpc.labels, derep_labels);
    assert_eq!(mpc_flat_get_my_input_seq_index(&mpc, "r3"), 2);
    assert_eq!(mpc_flat_get_label(&mpc, 1), "r2");
    assert_eq!(mpc_flat_get_seq_length(&mpc, 0), 4);
    assert_eq!(
        sequence_get_seq_as_string(&mpc_flat_get_sequence(&mpc, 3)),
        "ACGA"
    );
    assert_eq!(mpc_flat_get_byte_ptr(&mpc, 0), b"AcgT");

    mpc_flat_init_pairs(&mut mpc);
    assert_eq!(
        mpc.pairs,
        vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
    );
    assert_eq!(mpc_flat_get_pair_index(&mpc, 1, 3), 4);
    assert_eq!(mpc_flat_get_pair(&mpc, 5), (2, 3));
    mpc_flat_alloc_pair_count(&mut mpc, 6);
    assert_eq!(mpc.sparse_posts1.len(), 6);
    assert_eq!(mpc.sparse_posts2.len(), 6);
    my_sparse_mx_from_post(
        mpc_flat_get_sparse_post(&mut mpc, 4),
        &[0.0, 0.5, 0.02, 0.0],
        2,
        2,
    );
    assert_eq!(mpc.sparse_posts1[4].as_ref().unwrap().vec_size, 2);
    my_sparse_mx_from_post(
        mpc_flat_get_updated_sparse_post(&mut mpc, 4),
        &[0.2, 0.0, 0.0, 0.3],
        2,
        2,
    );
    assert_eq!(mpc.sparse_posts2[4].as_ref().unwrap().vec_size, 2);
    let mut cons_calls = Vec::new();
    mpc_flat_cons_iter(&mut mpc, 0, |mpc, pair_index| {
        cons_calls.push(pair_index);
        mpc_flat_get_updated_sparse_post(mpc, pair_index).lx = pair_index + 10;
    });
    assert_eq!(cons_calls, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(mpc.sparse_posts1[0].as_ref().unwrap().lx, 10);

    let mut small_mpc = MPCFlat::default();
    let mut two_seq = MultiSequence::default();
    multi_sequence_from_strings(
        &mut two_seq,
        &["x".to_string(), "y".to_string()],
        &["AA".to_string(), "CC".to_string()],
    );
    mpc_flat_init_seqs(&mut small_mpc, &two_seq);
    small_mpc.consistency_iter_count = 3;
    let mut skipped_consistency = Vec::new();
    mpc_flat_consistency(&mut small_mpc, |_mpc, pair_index| {
        skipped_consistency.push(pair_index)
    });
    assert!(skipped_consistency.is_empty());

    mpc.consistency_iter_count = 2;
    let mut consistency_calls = Vec::new();
    mpc_flat_consistency(&mut mpc, |mpc, pair_index| {
        consistency_calls.push(pair_index);
        mpc_flat_get_updated_sparse_post(mpc, pair_index).ly += 1;
    });
    assert_eq!(consistency_calls, vec![0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5]);
    mpc_flat_init_dist_mx(&mut mpc);
    assert_eq!(mpc.dist_mx.len(), 4);
    assert_eq!(mpc.dist_mx[0][0], 0.0);
    assert_eq!(mpc.dist_mx[0][1], f32::MAX);

    let mut label_to_index = std::collections::HashMap::new();
    for (i, label) in derep_labels.iter().enumerate() {
        label_to_index.insert(label.clone(), i as uint);
    }
    make_guide_tree_from_join_order(&[0, 2, 4], &[1, 3, 5], &label_to_index, &mut mpc.guide_tree);
    mpc_flat_calc_join_order(&mut mpc);
    assert_eq!(mpc.join_indexes1.len(), 3);
    assert_eq!(mpc.join_indexes2.len(), 3);
    validate_join_order(&mpc.join_indexes1, &mpc.join_indexes2);

    mpc.tree_perm = TREEPERM::TP_None;
    mpc.dist_mx = vec![
        vec![1.0, 0.75, 0.20, 0.10],
        vec![0.75, 1.0, 0.30, 0.15],
        vec![0.20, 0.30, 1.0, 0.80],
        vec![0.10, 0.15, 0.80, 1.0],
    ];
    let mut guide_run_labels = Vec::new();
    let mut guide_run_dist = Vec::new();
    mpc_flat_calc_guide_tree(&mut mpc, |upgma, tree| {
        guide_run_labels = upgma.labels.clone();
        guide_run_dist = upgma.dist_mx.clone();
        make_guide_tree_from_join_order(&[0, 2, 4], &[1, 3, 5], &label_to_index, tree);
    });
    assert_eq!(guide_run_labels, derep_labels);
    assert_eq!(guide_run_dist[0][1], 0.25);
    assert!((guide_run_dist[2][3] - 0.20).abs() < 1e-6);
    assert_eq!(tree_get_leaf_labels(&mpc.guide_tree).len(), 4);

    let mut super4_mpc = MPCFlat::default();
    let super4_phases = std::cell::RefCell::new(Vec::new());
    mpc_flat_run_super4(
        &mut super4_mpc,
        &derep_ms,
        |mpc| {
            super4_phases
                .borrow_mut()
                .push(("post".to_string(), mpc.pairs.len() as uint));
            mpc.dist_mx[0][1] = 0.7;
            mpc.dist_mx[1][0] = 0.7;
        },
        |mpc| {
            super4_phases
                .borrow_mut()
                .push(("cons".to_string(), mpc.sparse_posts1.len() as uint));
        },
        |mpc| {
            super4_phases
                .borrow_mut()
                .push(("tree".to_string(), mpc.labels.len() as uint));
            make_guide_tree_from_join_order(
                &[0, 2, 4],
                &[1, 3, 5],
                &label_to_index,
                &mut mpc.guide_tree,
            );
        },
    );
    assert_eq!(
        *super4_phases.borrow(),
        vec![
            ("post".to_string(), 6),
            ("cons".to_string(), 6),
            ("tree".to_string(), 4),
        ]
    );
    assert_eq!(super4_mpc.labels, derep_labels);
    assert_eq!(super4_mpc.pairs.len(), 6);
    assert_eq!(super4_mpc.sparse_posts1.len(), 6);
    assert_eq!(super4_mpc.dist_mx[0][0], 0.0);
    assert_eq!(super4_mpc.dist_mx[0][1], 0.7);
    assert_eq!(tree_get_leaf_labels(&super4_mpc.guide_tree).len(), 4);

    let mut run_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut run_input,
        &[
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ],
        &[
            "AA".to_string(),
            "CC".to_string(),
            "aa".to_string(),
            "GG".to_string(),
        ],
    );
    let mut run_mpc = MPCFlat {
        consistency_iter_count: 1,
        ..MPCFlat::default()
    };
    let run_phases = std::cell::RefCell::new(Vec::new());
    mpc_flat_run(
        &mut run_mpc,
        &run_input,
        true,
        |mpc| {
            run_phases
                .borrow_mut()
                .push(("post".to_string(), mpc.pairs.len() as uint));
        },
        |mpc| {
            run_phases
                .borrow_mut()
                .push(("tree".to_string(), mpc.labels.len() as uint));
            let label_to_index = mpc
                .label_to_index
                .iter()
                .map(|(label, index)| (label.clone(), *index))
                .collect::<std::collections::HashMap<_, _>>();
            make_guide_tree_from_join_order(&[0, 3], &[1, 2], &label_to_index, &mut mpc.guide_tree);
            for leaf_id in 0..3 {
                tree_set_leaf_id(&mut mpc.guide_tree, leaf_id, leaf_id);
            }
        },
        |mpc| {
            run_phases
                .borrow_mut()
                .push(("cons".to_string(), mpc.sparse_posts1.len() as uint));
        },
        |mpc| {
            run_phases
                .borrow_mut()
                .push(("prog".to_string(), mpc.join_indexes1.len() as uint));
            let mut out = MultiSequence::default();
            multi_sequence_from_strings(
                &mut out,
                &["d".to_string(), "b".to_string(), "a".to_string()],
                &["GG".to_string(), "CC".to_string(), "AA".to_string()],
            );
            mpc.msa = Some(out);
        },
        |mpc| {
            run_phases.borrow_mut().push((
                "refine".to_string(),
                mpc.msa.as_ref().unwrap().seqs.len() as uint,
            ));
        },
    );
    assert_eq!(
        *run_phases.borrow(),
        vec![
            ("post".to_string(), 3),
            ("tree".to_string(), 3),
            ("cons".to_string(), 3),
            ("prog".to_string(), 2),
            ("refine".to_string(), 3),
        ]
    );
    assert_eq!(run_mpc.weights, vec![1.0, 1.0, 1.0]);
    assert_eq!(
        run_mpc
            .msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "c", "b", "d"]
    );

    let mut run_single_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut run_single_input,
        &["u1".to_string(), "u2".to_string()],
        &["AA".to_string(), "aa".to_string()],
    );
    let mut run_single_mpc = MPCFlat::default();
    let run_single_called = std::cell::Cell::new(false);
    mpc_flat_run(
        &mut run_single_mpc,
        &run_single_input,
        true,
        |_mpc| run_single_called.set(true),
        |_mpc| run_single_called.set(true),
        |_mpc| run_single_called.set(true),
        |_mpc| run_single_called.set(true),
        |_mpc| run_single_called.set(true),
    );
    assert!(!run_single_called.get());
    assert_eq!(run_single_mpc.msa.as_ref().unwrap(), &run_single_input);

    let mut refine_skip = MPCFlat::default();
    mpc_flat_init_seqs(&mut refine_skip, &derep_ms);
    refine_skip.msa = Some(derep_ms.clone());
    let mut skip_align_calls = 0;
    mpc_flat_refine_iter(
        &mut refine_skip,
        || 0,
        |_mpc, _msa1, _msa2| {
            skip_align_calls += 1;
            MultiSequence::default()
        },
    );
    assert_eq!(skip_align_calls, 0);
    assert_eq!(
        refine_skip
            .msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["r1", "r2", "r3", "r4"]
    );

    let mut refine_mpc = MPCFlat::default();
    mpc_flat_init_seqs(&mut refine_mpc, &derep_ms);
    refine_mpc.msa = Some(derep_ms.clone());
    let split_values = std::cell::RefCell::new(vec![0, 1, 0, 1].into_iter());
    let mut refine_calls = Vec::new();
    mpc_flat_refine_iter(
        &mut refine_mpc,
        || split_values.borrow_mut().next().unwrap(),
        |_mpc, msa1, msa2| {
            refine_calls.push((
                msa1.seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>(),
                msa2.seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>(),
            ));
            let mut joined = msa1.clone();
            for seq in &msa2.seqs {
                joined.seqs.push(seq.clone());
                joined.owners.push(true);
            }
            joined
        },
    );
    assert_eq!(
        refine_calls,
        vec![(
            vec!["r1".to_string(), "r3".to_string()],
            vec!["r2".to_string(), "r4".to_string()]
        )]
    );
    assert_eq!(
        refine_mpc
            .msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["r1", "r3", "r2", "r4"]
    );
    let mut refine_loop_mpc = MPCFlat::default();
    mpc_flat_init_seqs(&mut refine_loop_mpc, &derep_ms);
    refine_loop_mpc.msa = Some(derep_ms.clone());
    refine_loop_mpc.refine_iter_count = 2;
    let split_loop_values = std::cell::RefCell::new(vec![0, 1, 0, 1, 0, 1, 0, 1].into_iter());
    let mut refine_loop_calls = 0;
    mpc_flat_refine(
        &mut refine_loop_mpc,
        || split_loop_values.borrow_mut().next().unwrap(),
        |_mpc, msa1, msa2| {
            refine_loop_calls += 1;
            let mut joined = msa1.clone();
            for seq in &msa2.seqs {
                joined.seqs.push(seq.clone());
                joined.owners.push(true);
            }
            joined
        },
    );
    assert_eq!(refine_loop_calls, 2);

    let mut shuffled_msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut shuffled_msa,
        &[
            "r3".to_string(),
            "r1".to_string(),
            "r4".to_string(),
            "r2".to_string(),
        ],
        &[
            "GG".to_string(),
            "AA".to_string(),
            "TT".to_string(),
            "CC".to_string(),
        ],
    );
    mpc.msa = Some(shuffled_msa);
    let label_to_msa = mpc_flat_get_label_to_msa_seq_index(&mpc);
    assert_eq!(label_to_msa["r3"], 0);
    assert_eq!(label_to_msa["r2"], 3);
    mpc_flat_sort_msa(&mut mpc, true);
    assert_eq!(
        mpc.msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["r1", "r2", "r3", "r4"]
    );
    mpc_flat_sort_msa(&mut mpc, false);
    assert_eq!(
        mpc.msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        tree_get_leaf_labels(&mpc.guide_tree)
    );

    let mut dupe_map = std::collections::BTreeMap::new();
    dupe_map.insert("r1".to_string(), vec!["r1_dup".to_string()]);
    dupe_map.insert(
        "r3".to_string(),
        vec!["r3_dup1".to_string(), "r3_dup2".to_string()],
    );
    mpc_flat_insert_dupes(&mut mpc, &dupe_map);
    let inserted = mpc.msa.as_ref().unwrap();
    assert_eq!(inserted.seqs.len(), 7);
    assert!(inserted.seqs.iter().any(|seq| seq.label == "r1_dup"));
    let r3_seq = inserted
        .seqs
        .iter()
        .find(|seq| seq.label == "r3")
        .unwrap()
        .char_vec
        .clone();
    assert_eq!(
        inserted
            .seqs
            .iter()
            .find(|seq| seq.label == "r3_dup2")
            .unwrap()
            .char_vec,
        r3_seq
    );

    let mut mpc_prog = MPCFlat::default();
    mpc_flat_init_seqs(&mut mpc_prog, &derep_ms);
    mpc_prog.join_indexes1 = vec![0, 2, 4];
    mpc_prog.join_indexes2 = vec![1, 3, 5];
    let mut prog_calls = Vec::new();
    mpc_flat_progressive_align(&mut mpc_prog, |_mpc, msa1, msa2| {
        prog_calls.push((
            msa1.seqs
                .iter()
                .map(|seq| seq.label.clone())
                .collect::<Vec<_>>(),
            msa2.seqs
                .iter()
                .map(|seq| seq.label.clone())
                .collect::<Vec<_>>(),
        ));
        let mut joined = msa1.clone();
        for seq in &msa2.seqs {
            joined.seqs.push(seq.clone());
            joined.owners.push(true);
        }
        joined
    });
    assert_eq!(
        prog_calls,
        vec![
            (vec!["r1".to_string()], vec!["r2".to_string()]),
            (vec!["r3".to_string()], vec!["r4".to_string()]),
            (
                vec!["r1".to_string(), "r2".to_string()],
                vec!["r3".to_string(), "r4".to_string()]
            ),
        ]
    );
    assert!(mpc_prog.prog_msas.is_empty());
    assert_eq!(
        mpc_prog
            .msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["r1", "r2", "r3", "r4"]
    );

    mpc.prog_msas.push(Some(derep_ms.clone()));
    mpc_flat_free_prog_ms_as(&mut mpc);
    assert!(mpc.prog_msas.is_empty());
    mpc_flat_free_sparse_posts(&mut mpc);
    assert!(mpc.sparse_posts1.is_empty());
    assert!(mpc.sparse_posts2.is_empty());
    mpc_flat_alloc_pair_count(&mut mpc, 3);
    mpc_flat_clear(&mut mpc);
    assert!(mpc.my_input_seqs.is_none());
    assert!(mpc.labels.is_empty());
    assert!(mpc.sparse_posts1.is_empty());

    let mut same1 = MultiSequence::default();
    let mut same2 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut same1,
        &["a".to_string(), "b".to_string()],
        &["A-C.G".to_string(), "TT--".to_string()],
    );
    multi_sequence_from_strings(
        &mut same2,
        &["b".to_string(), "a".to_string(), "extra".to_string()],
        &["T.T".to_string(), "ACG".to_string(), "XX".to_string()],
    );
    assert_seqs_eq("test", 7, &same1, &same2);

    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_global_input_ms(&derep_ms);
    let (dupe_gsis, dupe_rep_gsis) = derep_get_dupe_gs_is(&derep);
    assert_eq!(dupe_gsis, vec![1]);
    assert_eq!(dupe_rep_gsis, vec![0]);

    set_global_input_ms(&same1);
    assert_eq!(get_global_ms_seq_count(), 2);
    assert_eq!(get_gsi_count(), 2);
    assert!((get_global_ms_mean_seq_length() - 4.5).abs() < 1e-12);
    assert_eq!(get_gsi_by_label("a"), 0);
    assert_eq!(get_gsi_by_label("b"), 1);
    assert_eq!(get_label_by_gsi(1), "b");
    assert_eq!(get_seq_length_by_gsi(0), 5);
    assert_eq!(get_seq_length_by_global_label("b"), 4);
    assert_eq!(sequence_get_seq_as_string(&get_sequence_by_gsi(0)), "A-C.G");
    assert_eq!(
        sequence_get_seq_as_string(&get_sequence_by_global_label("b")),
        "TT--"
    );
    assert_eq!(get_byte_seq_by_gsi(1), b"TT--");
    assert_eq!(get_global_byte_seq_by_label("a"), b"A-C.G");

    let mut tmp_seq = Sequence::default();
    sequence_from_string(&mut tmp_seq, "tmp", "QQ");
    add_global_tmp_seq(&tmp_seq);
    assert_eq!(
        sequence_get_seq_as_string(&get_global_input_seq_by_label("tmp")),
        "QQ"
    );
    let mut same2_exact = MultiSequence::default();
    multi_sequence_from_strings(
        &mut same2_exact,
        &["b".to_string(), "a".to_string()],
        &["T.T".to_string(), "ACG".to_string()],
    );
    assert_seqs_eq_input("test", 8, &same2_exact);
    let before = get_assert_same_seqs_ok_count();
    assert_same_seqs("test", 9, &same1, &same2_exact);
    assert_eq!(get_assert_same_seqs_ok_count(), before + 1);
    assert_same_labels("test", 10, &same1);

    let mut part1 = MultiSequence::default();
    let mut part2 = MultiSequence::default();
    let mut joined = MultiSequence::default();
    multi_sequence_from_strings(&mut part1, &["a".to_string()], &["AAA".to_string()]);
    multi_sequence_from_strings(&mut part2, &["b".to_string()], &["TTT".to_string()]);
    multi_sequence_from_strings(
        &mut joined,
        &["b".to_string(), "a".to_string()],
        &["TTT".to_string(), "AAA".to_string()],
    );
    let before = get_assert_same_seqs_ok_count();
    assert_same_seqs_vec_l91("test", 11, &joined, &[&part1, &part2]);
    assert_eq!(get_assert_same_seqs_ok_count(), before + 2);
    let before = get_assert_same_seqs_ok_count();
    assert_same_seqs_vec_l111("test", 12, &joined, &[part1.clone(), part2.clone()]);
    assert_eq!(get_assert_same_seqs_ok_count(), before + 2);
    let before = get_assert_same_seqs_ok_count();
    assert_same_seqs_join("test", 13, &part1, &part2, &joined);
    assert_eq!(get_assert_same_seqs_ok_count(), before + 2);
}

#[test]
fn sweeper_and_tree_label_helpers_match_cpp_logic() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    let mut s = Sweeper::default();
    sweeper_set_param_names(&mut s, &["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(s.param_count, 3);
    assert_eq!(s.param_names, vec!["a", "b", "c"]);
    s.param_values_vec = vec![vec![1.0, 2.0, 3.0], vec![4.0, 6.0, 3.0]];
    s.scores = vec![0.5, 0.75];
    s.qs = vec![0.1, 0.2];
    s.tcs = vec![0.3, 0.9];

    assert_eq!(sweeper_get_params(&s, 0), &[1.0, 2.0, 3.0]);
    assert!((sweeper_get_param_dist(&s, 0, 1) - 5.0).abs() < 1e-6);
    assert_eq!(sweeper_get_score_l552(&s, 1), 0.75);
    assert_eq!(sweeper_get_q(&s, 1), 0.2);
    assert_eq!(sweeper_get_tc(&s, 1), 0.9);
    assert!((sweeper_get_score_diff(&s, 1, 0) - 0.25).abs() < 1e-6);
    assert!((sweeper_get_score_dist(&s, 0, 1) - 0.25).abs() < 1e-6);
    assert_eq!(sweeper_get_sort_order(&s), vec![1, 0]);
    assert_eq!(
        sweeper_log_indexes(&s, &[1, 0]),
        " Score       Q      TC             a             b             c\n0.7500  0.2000  0.9000         4         6         3\n0.5000  0.1000  0.3000         1         2         3\n"
    );
    assert_eq!(
        sweeper_log_top(&s, 1),
        "Top params:\n Score       Q      TC         a         b         c\n0.7500  0.2000  0.9000           4           6           3\n"
    );
    let mut run_s = Sweeper::default();
    sweeper_set_param_names(&mut run_s, &["x".to_string(), "y".to_string()]);
    run_s.grid_coords = vec![1, 0];
    run_s.grid_sizes = vec![3, 2];
    let fev1 = sweeper_run1(&mut run_s, &[1.25, 2.5], 0.75, 0.8);
    assert_eq!(
        fev1,
        "1\tscore=0.80000001\tQ=0.75\tTC=0.8\tx=1.25\ty=2.5\tgridcoord0=1/3\tgridcoord1=0/2\tnewbest=yes\n"
    );
    let fev2 = sweeper_run1(&mut run_s, &[3.0, 4.0], 0.5, 0.8);
    assert!(fev2.ends_with("\tnewbest=yes\n"));
    assert_eq!(run_s.best_score, 0.8);
    assert_eq!(run_s.best_indexes, vec![1, 2]);
    let sweeper_fev_file =
        std::env::temp_dir().join(format!("muscle_rs_sweeper_fev_{}.txt", std::process::id()));
    let mut file_s = Sweeper {
        grid_counter: uint::MAX,
        spatter_iter: uint::MAX,
        ..Sweeper::default()
    };
    sweeper_set_param_names(&mut file_s, &["p".to_string()]);
    sweeper_set_fev(&mut file_s, sweeper_fev_file.to_str().unwrap());
    let file_fev = sweeper_run1(&mut file_s, &[1.0], 0.2, 0.3);
    assert_eq!(
        std::fs::read_to_string(&sweeper_fev_file).unwrap(),
        file_fev
    );
    std::fs::remove_file(&sweeper_fev_file).unwrap();

    reset_rand(1);
    assert!((sweeper_get_random_value(&s, 10.0, 20.0) - 14.984955).abs() < 1e-5);
    reset_rand(1);
    assert_eq!(
        sweeper_get_random_values(&s, &[0.0, 10.0, -1.0], &[1.0, 20.0, 1.0])
            .iter()
            .map(|x| (x * 1000.0).round() as i32)
            .collect::<Vec<_>>(),
        vec![498, 18295, -974]
    );
    assert_eq!(sweeper_get_grid_value(&s, 2.0, 10.0, 0, 1), 2.0);
    assert_eq!(sweeper_get_grid_value(&s, 2.0, 10.0, 2, 5), 6.0);

    let mut grid_s = Sweeper::default();
    sweeper_set_param_names(&mut grid_s, &["x".to_string(), "y".to_string()]);
    let grid_fev = sweeper_explore_grid(
        &mut grid_s,
        &[0.0, 10.0],
        &[2.0, 20.0],
        &[3, 2],
        |_s, values| {
            let q = values[0] as f64;
            let tc = values[1] as f64 / 20.0;
            (q, tc)
        },
    );
    assert_eq!(
        grid_s.param_values_vec,
        vec![
            vec![0.0, 10.0],
            vec![1.0, 10.0],
            vec![2.0, 10.0],
            vec![0.0, 20.0],
            vec![1.0, 20.0],
            vec![2.0, 20.0],
        ]
    );
    assert_eq!(grid_s.grid_coords, vec![0, 0]);
    assert_eq!(grid_s.grid_counter, uint::MAX);
    assert_eq!(grid_s.grid_count, uint::MAX);
    assert_eq!(grid_s.best_indexes, vec![4, 5, 6]);
    assert!(grid_fev.contains("\tgridcoord0=2/3\tgridcoord1=1/2\tnewbest=yes\n"));

    let mut random_s = Sweeper::default();
    sweeper_set_param_names(&mut random_s, &["r".to_string()]);
    reset_rand(1);
    let random_fev = sweeper_explore_random(&mut random_s, &[10.0], &[20.0], 2, |_s, values| {
        (values[0] as f64, (values[0] / 20.0) as f64)
    });
    assert_eq!(
        random_s
            .param_values_vec
            .iter()
            .map(|values| (values[0] * 1000.0).round() as i32)
            .collect::<Vec<_>>(),
        vec![14985, 18295]
    );
    assert!(random_fev.starts_with(
        "1\tscore=0.74924773\tQ=14.984955\tTC=0.74924773\tr=14.984955\tnewbest=yes\n"
    ));

    s.param_values_vec = vec![
        vec![0.0, 0.0, 0.0],
        vec![1.0, 0.0, 0.0],
        vec![10.0, 0.0, 0.0],
        vec![0.0, 3.0, 4.0],
    ];
    s.scores = vec![1.0, 0.95, 0.9, 0.97];
    s.qs = vec![0.1, 0.2, 0.3, 0.4];
    s.tcs = vec![1.0, 0.95, 0.9, 0.97];
    assert_eq!(sweeper_get_sort_order(&s), vec![0, 3, 1, 2]);
    assert_eq!(
        sweeper_get_distinct_top_indexes(&s, 3, 0.08, 2.0),
        vec![0, 3, 2]
    );
    assert_eq!(sweeper_get_cloud(&s, 0, 2, 0.08, 5.0), vec![3, 1]);
    s.max_distinct_score_drop = 0.08;
    s.min_distinct_param_dist = 2.0;
    let distinct_log = sweeper_log_distinct_top(&s, &[0.5, 0.5, 0.5], 2, 0.08, 5.0, 3);
    assert!(distinct_log.starts_with("Distinct top (3 of 3)\n"));
    assert!(distinct_log.contains(" n  Cloud         Q        TC  ParamDst  ScoreDff"));
    assert!(distinct_log.contains(" 0          0.10000   1.00000"));
    assert!(distinct_log.contains(" 0      0   0.40000   0.97000   5.00000   0.03000"));
    assert!(distinct_log.contains(" 1          0.40000   0.97000"));
    assert!(distinct_log.contains(" 2          0.30000   0.90000"));

    s.spatter_deltas = vec![1.0, 0.04, 2.0];
    s.min_delta = 0.05;
    sweeper_spatter_update_deltas_shrink(&mut s, 0.5);
    assert_eq!(s.spatter_deltas, vec![0.5, 0.04, 1.0]);
    assert_eq!(sweeper_printf_spatter_deltas(&s), "  a+0.5  b+0.04  c+1\n");
    assert_eq!(
        sweeper_log_spatter_deltas(&s),
        "Deltas:  a=0.5  b=0.04  c=1\n"
    );

    reset_rand(1);
    let spattered = sweeper_get_spattered_values(&s, &[10.0, 20.0, 30.0], &[1.0, 2.0, 3.0]);
    assert_eq!(
        spattered
            .iter()
            .map(|x| (x * 1000.0).round() as i32)
            .collect::<Vec<_>>(),
        vec![10490, 21408, 28025]
    );

    let mut spatter_iter_s = Sweeper::default();
    sweeper_set_param_names(&mut spatter_iter_s, &["p".to_string()]);
    spatter_iter_s.param_values_vec = vec![vec![10.0], vec![20.0]];
    spatter_iter_s.scores = vec![1.0, 0.9];
    spatter_iter_s.qs = vec![1.0, 0.9];
    spatter_iter_s.tcs = vec![1.0, 0.9];
    spatter_iter_s.best_score = 1.0;
    spatter_iter_s.spatter_tries_per_iter = 2;
    spatter_iter_s.max_distinct_score_drop = 1.0;
    spatter_iter_s.min_distinct_param_dist = 0.1;
    spatter_iter_s.spatter_deltas = vec![0.0];
    reset_rand(1);
    let (improved, spatter_iter_fev) =
        sweeper_spatter_iter(&mut spatter_iter_s, |_s, values| (values[0] as f64, 0.5));
    assert!(!improved);
    assert_eq!(spatter_iter_s.spatter_seed_indexes, vec![0, 1]);
    assert_eq!(spatter_iter_s.spatter_try, 2);
    assert_eq!(
        spatter_iter_s.param_values_vec[2..],
        [vec![10.0], vec![10.0]]
    );
    assert!(spatter_iter_fev.contains("\tp=10"));

    let mut explore_s = Sweeper::default();
    sweeper_set_param_names(&mut explore_s, &["p".to_string()]);
    explore_s.start_max_distinct_score_drop = 1.0;
    explore_s.end_max_distinct_score_drop = 0.1;
    explore_s.start_min_distinct_param_dist = 0.1;
    explore_s.end_min_distinct_param_dist = 0.0;
    reset_rand(1);
    let explore_fev = sweeper_explore_spatter(
        &mut explore_s,
        &[vec![2.0], vec![1.0]],
        &[0.0],
        1,
        1,
        1,
        0.5,
        |_s, values| (values[0] as f64, values[0] as f64),
    );
    assert_eq!(explore_s.spatter_seed_indexes, vec![0, 1]);
    assert_eq!(explore_s.spatter_tries_per_iter, 1);
    assert_eq!(explore_s.spatter_failed_iter_count, 1);
    assert_eq!(explore_s.spatter_deltas, vec![0.0]);
    assert!(explore_fev.starts_with("1\tscore=2\tQ=2\tTC=2\tp=2\tnewbest=yes\n"));

    assert_eq!(
        get_max_string(&["b".to_string(), "aa".to_string(), "z".to_string()]),
        "z"
    );
    assert!(compare_labels(
        &["b".to_string(), "z".to_string()],
        &["a".to_string(), "y".to_string()]
    ));
    assert!(!compare_labels(&["b".to_string()], &["c".to_string()]));
}

#[test]
fn super6_split_and_prepare_clusters_preserve_sequence_order() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let labels = vec![
        "seq_a".to_string(),
        "seq_b".to_string(),
        "seq_c".to_string(),
        "seq_d".to_string(),
        "seq_e".to_string(),
    ];
    let seqs = vec![
        "AAAA".to_string(),
        "AAAT".to_string(),
        "AATT".to_string(),
        "ATTT".to_string(),
        "TTTT".to_string(),
    ];
    let mut input = MultiSequence::default();
    multi_sequence_from_strings(&mut input, &labels, &seqs);
    set_global_input_ms(&input);

    let mut split = Vec::new();
    super6_split_big_mfa_random(&input, 2, &mut split);
    assert_eq!(split.len(), 3);
    assert_eq!(
        split
            .iter()
            .map(|mfa| {
                mfa.seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec!["seq_a".to_string(), "seq_b".to_string()],
            vec!["seq_c".to_string(), "seq_d".to_string()],
            vec!["seq_e".to_string()],
        ]
    );
    assert_eq!(split[0].owners, vec![false, false]);

    let mut s6 = Super6 {
        max_cluster_size: 2,
        input_seqs: Some(input.clone()),
        cluster_mfas: vec![input],
        ..Super6::default()
    };
    assert_eq!(
        super6_prepare_clusters(&mut s6),
        "1 clusters pass 1\n3 clusters pass 2\n"
    );
    assert_eq!(s6.cluster_labels, vec!["Cluster0", "Cluster1", "Cluster2"]);
    assert_eq!(
        s6.cluster_mfas
            .iter()
            .map(|mfa| {
                mfa.seqs
                    .iter()
                    .map(|seq| sequence_get_seq_as_string(seq))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec!["AAAA".to_string(), "AAAT".to_string()],
            vec!["AATT".to_string(), "ATTT".to_string()],
            vec!["TTTT".to_string()],
        ]
    );

    let mut dist_calls = Vec::new();
    super6_calc_cluster_dist_mx(&mut s6, |mfa1, mfa2, k| {
        assert_eq!(k, 8);
        let label1 = mfa1.seqs[0].label.clone();
        let label2 = mfa2.seqs[0].label.clone();
        dist_calls.push((label1, label2));
        mfa1.seqs.len() as f32 * 10.0 + mfa2.seqs.len() as f32
    });
    assert_eq!(
        dist_calls,
        vec![
            ("seq_c".to_string(), "seq_a".to_string()),
            ("seq_e".to_string(), "seq_a".to_string()),
            ("seq_e".to_string(), "seq_c".to_string()),
        ]
    );
    assert_eq!(
        s6.cluster_dist_mx,
        vec![
            vec![0.0, 22.0, 12.0],
            vec![22.0, 0.0, 12.0],
            vec![12.0, 12.0, 0.0],
        ]
    );
    let mut single_s6 = Super6 {
        cluster_labels: vec!["Cluster0".to_string()],
        cluster_dist_mx: vec![vec![0.0]],
        ..Super6::default()
    };
    super6_make_guide_tree(&mut single_s6, |_u, _tree| {
        panic!("single-cluster Super6 should not run UPGMA")
    });
    assert_eq!(single_s6.guide_tree.node_count, 1);
    assert_eq!(
        tree_get_leaf_name(&single_s6.guide_tree, 0),
        Some("Cluster0")
    );
    assert_eq!(single_s6.guide_tree.ids[0], 0);

    let mut opt_s6 = Super6::default();
    super6_set_opts(&mut opt_s6, Some(0.25));
    assert!((opt_s6.max_pd_pass1 - 0.25).abs() < f64::EPSILON);
    assert_eq!(opt_s6.max_cluster_size, 500);
    assert_eq!(opt_s6.target_pair_count_cluster_dist, 8);
    assert_eq!(opt_s6.target_pair_count, 2000);

    let mut aligned_labels = Vec::new();
    let align_log = super6_align_clusters(&mut s6, |mpc, cluster_mfa| {
        assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
        aligned_labels.push(cluster_mfa.seqs[0].label.clone());
        let mut msa = cluster_mfa.clone();
        for seq in &mut msa.seqs {
            seq.char_vec.push('-');
        }
        msa
    });
    assert_eq!(
        align_log,
        "\nAlign cluster 1 / 3 (2 seqs)\n\n\nAlign cluster 2 / 3 (2 seqs)\n\nAlign cluster 3 / 3 (1 seq)\n"
    );
    assert_eq!(
        aligned_labels,
        vec![
            "seq_a".to_string(),
            "seq_c".to_string(),
            "seq_e".to_string()
        ]
    );
    assert_eq!(
        s6.cluster_msas
            .iter()
            .map(|msa| sequence_get_seq_as_string(&msa.seqs[0]))
            .collect::<Vec<_>>(),
        vec![
            "AAAA-".to_string(),
            "AATT-".to_string(),
            "TTTT-".to_string()
        ]
    );

    s6.target_pair_count = 17;
    super6_init_pp(&mut s6);
    assert_eq!(s6.pp.target_pair_count, 17);
    assert_eq!(s6.pp.input_msa_count, 3);
    assert_eq!(s6.pp.msa_labels[..3], s6.cluster_labels[..]);
    assert_eq!(p_prog_get_msa_label(&s6.pp, 2), "Cluster2");
    assert_eq!(
        sequence_get_seq_as_string(&p_prog_get_msa(&s6.pp, 1).seqs[0]),
        "AATT-"
    );

    let mut run_s6 = Super6 {
        max_pd_pass1: 0.42,
        max_cluster_size: 2,
        target_pair_count: 19,
        ..Super6::default()
    };
    let mut run_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut run_input,
        &[
            "seq_a".to_string(),
            "seq_b".to_string(),
            "seq_c".to_string(),
            "seq_d".to_string(),
            "seq_e".to_string(),
        ],
        &[
            "AAAA".to_string(),
            "AAAT".to_string(),
            "AATT".to_string(),
            "ATTT".to_string(),
            "TTTT".to_string(),
        ],
    );
    set_global_input_ms(&run_input);
    let mut run_dist_calls = Vec::new();
    let mut guide_labels = Vec::new();
    let mut pp_input_msa_count = 0;
    let run_log = super6_run(
        &mut run_s6,
        &run_input,
        |ucpd, input_seqs, all_seq_indexes, max_pd| {
            assert_eq!(all_seq_indexes, &[0, 1, 2, 3, 4]);
            assert!((max_pd - 0.42).abs() < f64::EPSILON);
            ucpd.input_seqs = Some(input_seqs.clone());
            ucpd.subset_seq_indexes = all_seq_indexes.to_vec();
            vec![input_seqs.clone()]
        },
        |mfa1, mfa2, k| {
            assert_eq!(k, 8);
            run_dist_calls.push((mfa1.seqs[0].label.clone(), mfa2.seqs[0].label.clone()));
            mfa1.seqs.len() as f32 * 10.0 + mfa2.seqs.len() as f32
        },
        |guide_tree, labels, dist_mx| {
            guide_labels = labels.to_vec();
            assert_eq!(dist_mx[0][1], 22.0);
            guide_tree.node_count = labels.len() as uint;
        },
        |pp, guide_tree| {
            assert_eq!(guide_tree.node_count, 3);
            pp_input_msa_count = pp.input_msa_count;
        },
        |mpc, cluster_mfa| {
            assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
            let mut msa = cluster_mfa.clone();
            for seq in &mut msa.seqs {
                seq.char_vec.push('-');
            }
            msa
        },
    );
    assert_eq!(
        run_log,
        "1 clusters pass 1\n3 clusters pass 2\n\nAlign cluster 1 / 3 (2 seqs)\n\n\nAlign cluster 2 / 3 (2 seqs)\n\nAlign cluster 3 / 3 (1 seq)\n"
    );
    assert_eq!(
        run_dist_calls,
        vec![
            ("seq_c".to_string(), "seq_a".to_string()),
            ("seq_e".to_string(), "seq_a".to_string()),
            ("seq_e".to_string(), "seq_c".to_string()),
        ]
    );
    assert_eq!(guide_labels, vec!["Cluster0", "Cluster1", "Cluster2"]);
    assert_eq!(pp_input_msa_count, 3);
    assert_eq!(run_s6.pp.target_pair_count, 19);

    let cmd_dir = std::env::temp_dir().join(format!("muscle_rs_cmd_super6_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_dir).unwrap();
    let cmd_input = cmd_dir.join("input.fa");
    let cmd_output = cmd_dir.join("out.fa");
    std::fs::write(&cmd_input, b">aa\nA-A\n>cc\nC-C\n").unwrap();
    let mut cmd_seen = Vec::new();
    let (cmd_s6, cmd_log) = cmd_super6(
        cmd_input.to_str().unwrap(),
        cmd_output.to_str().unwrap(),
        Some(true),
        Some(0.25),
        |s6, input_seqs| {
            assert!((s6.max_pd_pass1 - 0.25).abs() < f64::EPSILON);
            cmd_seen = input_seqs
                .seqs
                .iter()
                .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
                .collect::<Vec<_>>();
            let mut final_msa = MultiSequence::default();
            multi_sequence_from_strings(
                &mut final_msa,
                &["aa".to_string(), "cc".to_string()],
                &["AA".to_string(), "CC".to_string()],
            );
            s6.pp.input_msa_count = 1;
            s6.pp.msas = vec![Some(final_msa)];
            "cmd-super6\n".to_string()
        },
    );
    assert_eq!(
        cmd_seen,
        vec![
            ("aa".to_string(), "AA".to_string()),
            ("cc".to_string(), "CC".to_string())
        ]
    );
    assert_eq!(cmd_log, "cmd-super6\n");
    assert_eq!(cmd_s6.pp.input_msa_count, 1);
    assert_eq!(
        std::fs::read_to_string(&cmd_output).unwrap(),
        ">aa\nAA\n>cc\nCC\n"
    );
    std::fs::remove_dir_all(&cmd_dir).unwrap();
}

#[test]
fn super4_leaf_state_methods_match_cpp_sequence_handling() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Nucleo);
    let labels = vec![
        "seq_a".to_string(),
        "seq_b".to_string(),
        "seq_c".to_string(),
        "seq_d".to_string(),
        "seq_e".to_string(),
    ];
    let seqs = vec![
        "AAAA".to_string(),
        "AAAT".to_string(),
        "AATT".to_string(),
        "ATTT".to_string(),
        "TTTT".to_string(),
    ];
    let mut input = MultiSequence::default();
    multi_sequence_from_strings(&mut input, &labels, &seqs);
    set_global_input_ms(&input);

    let mut split = Vec::new();
    super4_split_big_mfa_random(&input, 2, &mut split);
    assert_eq!(split.len(), 3);
    assert_eq!(
        split
            .iter()
            .map(|mfa| {
                mfa.seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec!["seq_a".to_string(), "seq_b".to_string()],
            vec!["seq_c".to_string(), "seq_d".to_string()],
            vec!["seq_e".to_string()],
        ]
    );
    assert_eq!(split[0].owners, vec![false, false]);

    let mut clustered_split = Vec::new();
    let mut split_s4 = Super4::default();
    super4_split_big_mfa(
        &mut split_s4,
        &input,
        2,
        0.7,
        &mut clustered_split,
        |_ec, big, min_ea| {
            assert_eq!(min_ea, 0.7);
            let mut first = MultiSequence::default();
            for seq in big.seqs.iter().take(3) {
                first.seqs.push(seq.clone());
                first.owners.push(false);
            }
            let mut second = MultiSequence::default();
            for seq in big.seqs.iter().skip(3) {
                second.seqs.push(seq.clone());
                second.owners.push(false);
            }
            vec![first, second]
        },
    );
    assert_eq!(
        clustered_split
            .iter()
            .map(|mfa| {
                mfa.seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec!["seq_a".to_string(), "seq_b".to_string()],
            vec!["seq_d".to_string(), "seq_e".to_string()],
            vec!["seq_c".to_string()],
        ]
    );

    let mut cluster_s4 = Super4 {
        max_cluster_size: 2,
        min_ea_pass1: 0.5,
        min_ea_pass2: 0.8,
        input_seqs: Some(input.clone()),
        ..Super4::default()
    };
    let cluster_log = super4_cluster_input(&mut cluster_s4, |_ec, big, min_ea| {
        if (min_ea - 0.5).abs() < 1e-6 {
            let mut first = MultiSequence::default();
            for seq in big.seqs.iter().take(3) {
                first.seqs.push(seq.clone());
                first.owners.push(false);
            }
            let mut second = MultiSequence::default();
            for seq in big.seqs.iter().skip(3) {
                second.seqs.push(seq.clone());
                second.owners.push(false);
            }
            vec![first, second]
        } else {
            assert!((min_ea - 0.8).abs() < 1e-6);
            vec![big.clone()]
        }
    });
    assert_eq!(cluster_log, "2 clusters pass 1\n3 clusters pass 2\n");
    assert_eq!(
        cluster_s4
            .cluster_mfas
            .iter()
            .map(|mfa| {
                mfa.seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec!["seq_a".to_string(), "seq_b".to_string()],
            vec!["seq_d".to_string(), "seq_e".to_string()],
            vec!["seq_c".to_string()],
        ]
    );
    assert_eq!(
        cluster_s4.cluster_labels,
        vec![
            "Cluster0".to_string(),
            "Cluster1".to_string(),
            "Cluster2".to_string()
        ]
    );
    let mut align_s4 = Super4 {
        cluster_mfas: cluster_s4.cluster_mfas.clone(),
        ..Super4::default()
    };
    let mut aligned_inputs = Vec::new();
    let align_log = super4_align_clusters(&mut align_s4, |mpc, cluster_mfa| {
        assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
        aligned_inputs.push(cluster_mfa.seqs[0].label.clone());
        let mut msa = cluster_mfa.clone();
        for seq in &mut msa.seqs {
            seq.char_vec.push('-');
        }
        msa
    });
    assert_eq!(
        align_log,
        "\nAlign cluster 1 / 3 (2 seqs)\n\n\nAlign cluster 2 / 3 (2 seqs)\n\nAlign cluster 3 / 3 (1 seq)\n"
    );
    assert_eq!(
        aligned_inputs,
        vec![
            "seq_a".to_string(),
            "seq_d".to_string(),
            "seq_c".to_string()
        ]
    );
    assert_eq!(align_s4.cluster_msas.len(), 3);
    assert_eq!(
        align_s4
            .cluster_msas
            .iter()
            .map(|msa| sequence_get_seq_as_string(&msa.seqs[0]))
            .collect::<Vec<_>>(),
        vec![
            "AAAA-".to_string(),
            "ATTT-".to_string(),
            "AATT-".to_string()
        ]
    );

    let mut s4 = Super4 {
        target_pair_count: 23,
        cluster_msas: split.clone(),
        cluster_labels: vec![
            "Cluster0".to_string(),
            "Cluster1".to_string(),
            "Cluster2".to_string(),
        ],
        final_msa: split[0].clone(),
        final_msa_none: split[0].clone(),
        final_msa_abc: split[1].clone(),
        final_msa_acb: split[2].clone(),
        final_msa_bca: split[0].clone(),
        guide_tree_none: {
            let mut t = Tree::default();
            tree_create_rooted(&mut t);
            t
        },
        ..Super4::default()
    };
    super4_init_pp(&mut s4);
    assert_eq!(s4.pp.target_pair_count, 23);
    assert_eq!(s4.pp.input_msa_count, 3);
    assert_eq!(s4.pp.msa_labels[..3], s4.cluster_labels[..]);
    assert_eq!(p_prog_get_msa_label(&s4.pp, 1), "Cluster1");
    assert_eq!(
        sequence_get_seq_as_string(&p_prog_get_msa(&s4.pp, 2).seqs[0]),
        "TTTT"
    );

    super4_get_consensus_seqs(&mut s4);
    assert_eq!(
        s4.consensus_seqs
            .seqs
            .iter()
            .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
            .collect::<Vec<_>>(),
        vec![
            ("Cluster0".to_string(), "AAAA".to_string()),
            ("Cluster1".to_string(), "AAUU".to_string()),
            ("Cluster2".to_string(), "UUUU".to_string()),
        ]
    );
    assert_eq!(
        sequence_get_seq_as_string(&get_global_input_seq_by_label("Cluster1")),
        "AAUU"
    );

    let mut coarse_s4 = Super4 {
        target_pair_count: 29,
        max_cluster_size: 2,
        min_ea_pass1: 0.5,
        min_ea_pass2: 0.8,
        input_seqs: Some(input.clone()),
        ..Super4::default()
    };
    let mut coarse_dist_labels = Vec::new();
    let mut coarse_guide_labels = Vec::new();
    let coarse_log = super4_coarse_align(
        &mut coarse_s4,
        |_ec, big, min_ea| {
            if (min_ea - 0.5).abs() < 1e-6 {
                vec![big.clone()]
            } else {
                assert!((min_ea - 0.8).abs() < 1e-6);
                vec![big.clone()]
            }
        },
        |mpc, cluster_mfa| {
            assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
            let mut msa = cluster_mfa.clone();
            for seq in &mut msa.seqs {
                seq.char_vec.push('-');
            }
            msa
        },
        |consensus_seqs| {
            coarse_dist_labels = consensus_seqs
                .seqs
                .iter()
                .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
                .collect::<Vec<_>>();
            vec![
                vec![0.0, 0.9, 0.8],
                vec![0.9, 0.0, 0.7],
                vec![0.8, 0.7, 0.0],
            ]
        },
        |guide_tree, labels, dist_mx| {
            coarse_guide_labels = labels.to_vec();
            assert!((dist_mx[1][2] - 0.3).abs() < f32::EPSILON);
            guide_tree.node_count = labels.len() as uint;
        },
    );
    assert_eq!(
        coarse_log,
        "1 clusters pass 1\n3 clusters pass 2\n\nAlign cluster 1 / 3 (2 seqs)\n\n\nAlign cluster 2 / 3 (2 seqs)\n\nAlign cluster 3 / 3 (1 seq)\n"
    );
    assert_eq!(
        coarse_dist_labels,
        vec![
            ("Cluster0".to_string(), "AAAA".to_string()),
            ("Cluster1".to_string(), "AAUU".to_string()),
            ("Cluster2".to_string(), "UUUU".to_string()),
        ]
    );
    assert_eq!(
        coarse_guide_labels,
        vec!["Cluster0", "Cluster1", "Cluster2"]
    );
    assert_eq!(coarse_s4.guide_tree_none.node_count, 3);
    assert_eq!(coarse_s4.pp.input_msa_count, 3);
    assert_eq!(coarse_s4.pp.target_pair_count, 29);

    let mut single_s4 = Super4 {
        cluster_labels: vec!["Cluster0".to_string()],
        dist_mx: vec![vec![0.0]],
        ..Super4::default()
    };
    super4_make_guide_tree(&mut single_s4, |_u, _tree| {
        panic!("single-cluster Super4 should not run UPGMA")
    });
    assert_eq!(single_s4.guide_tree_none.node_count, 1);
    assert_eq!(
        tree_get_leaf_name(&single_s4.guide_tree_none, 0),
        Some("Cluster0")
    );
    assert_eq!(single_s4.guide_tree_none.ids[0], 0);

    let mut run_s4 = Super4::default();
    let mut pp_tree_node_counts = Vec::new();
    let run_log = super4_run(
        &mut run_s4,
        &input,
        TREEPERM::TP_All,
        |s4| {
            s4.target_pair_count = 31;
            s4.max_cluster_size = 2;
            s4.min_ea_pass1 = 0.5;
            s4.min_ea_pass2 = 0.8;
        },
        |_ec, big, min_ea| {
            assert!((min_ea - 0.5).abs() < 1e-6 || (min_ea - 0.8).abs() < 1e-6);
            vec![big.clone()]
        },
        |mpc, cluster_mfa| {
            assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
            let mut msa = cluster_mfa.clone();
            for seq in &mut msa.seqs {
                seq.char_vec.push('-');
            }
            msa
        },
        |_consensus_seqs| {
            vec![
                vec![0.0, 0.9, 0.8],
                vec![0.9, 0.0, 0.7],
                vec![0.8, 0.7, 0.0],
            ]
        },
        |guide_tree, labels, _dist_mx| {
            guide_tree.node_count = labels.len() as uint;
        },
        |none, abc, acb, bca, labels_a, labels_b, labels_c| {
            assert_eq!(none.node_count, 3);
            abc.node_count = 101;
            acb.node_count = 102;
            bca.node_count = 103;
            labels_a.push("A".to_string());
            labels_b.push("B".to_string());
            labels_c.push("C".to_string());
        },
        |pp, guide_tree| {
            assert_eq!(pp.input_msa_count, 3);
            pp_tree_node_counts.push(guide_tree.node_count);
            let label = format!("tree_{}", guide_tree.node_count);
            let mut msa = MultiSequence::default();
            multi_sequence_from_strings(&mut msa, &[label], &["AC".to_string()]);
            msa
        },
    );
    assert!(
        run_log.contains("Guide tree (default)\nGuide tree ABC\nGuide tree ACB\nGuide tree BCA\n")
    );
    assert_eq!(pp_tree_node_counts, vec![3, 101, 102, 103]);
    assert!(run_s4.cluster_msas.is_empty());
    assert_eq!(
        sequence_get_seq_as_string(&run_s4.final_msa_none.seqs[0]),
        "AC"
    );
    assert_eq!(run_s4.final_msa_abc.seqs[0].label, "tree_101");
    assert_eq!(run_s4.final_msa_acb.seqs[0].label, "tree_102");
    assert_eq!(run_s4.final_msa_bca.seqs[0].label, "tree_103");

    let mut opt_s4 = Super4::default();
    super4_set_opts(
        &mut opt_s4,
        Some(17),
        Some(0.25),
        Some(0.75),
        Some(3),
        Some(4),
    );
    assert_eq!(opt_s4.target_pair_count, 17);
    assert_eq!(opt_s4.max_cluster_size, 17);
    assert!((opt_s4.min_ea_pass1 - 0.25).abs() < f32::EPSILON);
    assert!((opt_s4.min_ea_pass2 - 0.75).abs() < f32::EPSILON);
    assert_eq!(opt_s4.mpc.consistency_iter_count, 3);
    assert_eq!(opt_s4.mpc.refine_iter_count, 4);

    let cmd_dir = std::env::temp_dir().join(format!("muscle_rs_cmd_super4_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_dir).unwrap();
    let cmd_input = cmd_dir.join("input.fa");
    let cmd_output = cmd_dir.join("out.fa");
    std::fs::write(&cmd_input, b">aa\nA-A\n>cc\nC-C\n").unwrap();
    let mut cmd_seen = Vec::new();
    let (cmd_s4, cmd_log) = cmd_super4(
        cmd_input.to_str().unwrap(),
        cmd_output.to_str().unwrap(),
        Some(true),
        Some(TREEPERM::TP_ACB),
        false,
        |s4, input_seqs, tp| {
            assert_eq!(tp, TREEPERM::TP_ACB);
            cmd_seen = input_seqs
                .seqs
                .iter()
                .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
                .collect::<Vec<_>>();
            multi_sequence_from_strings(
                &mut s4.final_msa,
                &["aa".to_string(), "cc".to_string()],
                &["AA".to_string(), "CC".to_string()],
            );
            "cmd-super4\n".to_string()
        },
    );
    assert_eq!(
        cmd_seen,
        vec![
            ("aa".to_string(), "AA".to_string()),
            ("cc".to_string(), "CC".to_string())
        ]
    );
    assert_eq!(cmd_log, "cmd-super4\n");
    assert_eq!(cmd_s4.final_msa.seqs.len(), 2);
    assert_eq!(
        std::fs::read_to_string(&cmd_output).unwrap(),
        ">aa\nAA\n>cc\nCC\n"
    );
    std::fs::remove_dir_all(&cmd_dir).unwrap();

    super4_delete_cluster_ms_as(&mut s4);
    assert!(s4.cluster_msas.is_empty());

    super4_clear_trees_and_ms_as(&mut s4);
    assert!(s4.final_msa.seqs.is_empty());
    assert!(s4.final_msa_none.seqs.is_empty());
    assert!(s4.final_msa_abc.seqs.is_empty());
    assert!(s4.final_msa_acb.seqs.is_empty());
    assert!(s4.final_msa_bca.seqs.is_empty());
    assert_eq!(s4.guide_tree_none.node_count, 0);
}

#[test]
fn super5_dupe_and_centroid_vecs_match_cpp_index_mapping() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let labels = vec![
        "seq_a".to_string(),
        "seq_b".to_string(),
        "seq_c".to_string(),
        "seq_d".to_string(),
    ];
    let seqs = vec![
        "AAAA".to_string(),
        "AAAT".to_string(),
        "AAAA".to_string(),
        "TTTT".to_string(),
    ];
    let mut input = MultiSequence::default();
    multi_sequence_from_strings(&mut input, &labels, &seqs);
    set_global_input_ms(&input);

    let unique_labels = vec![
        "seq_a".to_string(),
        "seq_b".to_string(),
        "seq_d".to_string(),
    ];
    let unique_seqs = vec!["AAAA".to_string(), "AAAT".to_string(), "TTTT".to_string()];
    let mut unique = MultiSequence::default();
    multi_sequence_from_strings(&mut unique, &unique_labels, &unique_seqs);

    let mut s5 = Super5 {
        input_seqs: Some(input.clone()),
        unique_seqs: Some(unique.clone()),
        d: Derep {
            input_seqs: Some(input),
            seq_index_to_rep_seq_index: vec![0, 1, 0, 3],
            rep_seq_indexes: vec![0, 1, 3],
            rep_seq_index_to_seq_indexes: vec![vec![0, 2], vec![1], Vec::new(), vec![3]],
            ..Derep::default()
        },
        u: UClust {
            input_seqs: Some(unique),
            centroid_seq_indexes: vec![0, 2],
            seq_index_to_centroid_seq_index: vec![0, 0, 2],
            seq_index_to_path: vec![String::new(), "MB".to_string(), String::new()],
            ..UClust::default()
        },
        ..Super5::default()
    };

    super5_set_dupe_vecs(&mut s5);
    assert_eq!(s5.dupe_gs_is, vec![2]);
    assert_eq!(s5.dupe_rep_gs_is, vec![0]);
    assert_eq!(s5.is_dupe, vec![false, false, true, false]);
    assert_eq!(s5.dupe_rep_gsi_to_member_gs_is[0], vec![2]);

    super5_set_centroid_vecs(&mut s5);
    assert_eq!(s5.centroid_gs_is, vec![0, 3]);
    assert_eq!(s5.member_gs_is, vec![1]);
    assert_eq!(s5.member_centroid_gs_is, vec![0]);
    assert_eq!(s5.is_centroid, vec![true, false, false, true]);
    assert_eq!(s5.is_member, vec![false, true, false, false]);
    assert_eq!(
        s5.gsi_to_centroid_gsi,
        vec![uint::MAX, 0, uint::MAX, uint::MAX]
    );
    assert_eq!(s5.centroid_gsi_to_member_gs_is[0], vec![1]);
    assert_eq!(s5.gsi_to_member_count, vec![1, 0, 0, 0]);
    assert_eq!(s5.gsi_to_member_centroid_path[1], "MB");

    let mut centroid_seqs = MultiSequence::default();
    u_clust_get_centroid_seqs(&s5.u, &mut centroid_seqs);
    assert_eq!(
        centroid_seqs
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["seq_a".to_string(), "seq_d".to_string()]
    );
    assert_eq!(centroid_seqs.owners, vec![false, false]);

    let make_input = s5.input_seqs.clone().unwrap();
    let mut made_s5 = Super5 {
        min_ea_pass1: 0.6,
        ..Super5::default()
    };
    super5_make_centroid_seqs(
        &mut made_s5,
        &make_input,
        |d, input_seqs| {
            d.input_seqs = Some(input_seqs.clone());
            d.seq_index_to_rep_seq_index = vec![0, 1, 0, 3];
            d.rep_seq_indexes = vec![0, 1, 3];
            d.rep_seq_index_to_seq_indexes = vec![vec![0, 2], vec![1], Vec::new(), vec![3]];
            let mut unique = MultiSequence::default();
            for seq in [
                &input_seqs.seqs[0],
                &input_seqs.seqs[1],
                &input_seqs.seqs[3],
            ] {
                unique.seqs.push(seq.clone());
                unique.owners.push(false);
            }
            unique
        },
        |u, unique_seqs, min_ea| {
            assert_eq!(min_ea, 0.6);
            u.input_seqs = Some(unique_seqs.clone());
            u.centroid_seq_indexes = vec![0, 2];
            u.seq_index_to_centroid_seq_index = vec![0, 0, 2];
            u.seq_index_to_path = vec![String::new(), "MB".to_string(), String::new()];
            let mut centroids = MultiSequence::default();
            u_clust_get_centroid_seqs(u, &mut centroids);
            centroids
        },
    );
    assert_eq!(made_s5.dupe_gs_is, vec![2]);
    assert_eq!(made_s5.centroid_gs_is, vec![0, 3]);
    assert_eq!(made_s5.member_gs_is, vec![1]);
    assert_eq!(
        made_s5
            .centroid_seqs
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["seq_a".to_string(), "seq_d".to_string()]
    );
    assert_eq!(made_s5.centroid_seqs_seq_index_to_gsi, vec![0, 3]);
    assert_eq!(
        made_s5.gsi_to_centroid_seqs_seq_index,
        vec![0, uint::MAX, uint::MAX, 1]
    );
    made_s5.centroid_msa = made_s5.centroid_seqs.clone();
    super5_set_centroid_msa_vecs(&mut made_s5);
    made_s5.gsi_to_member_centroid_path[1] = "BBBB".to_string();
    super5_align_members(&mut made_s5);
    assert_eq!(
        made_s5
            .extended_msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
            .collect::<Vec<_>>(),
        vec![
            ("seq_a".to_string(), "AAAA".to_string()),
            ("seq_d".to_string(), "TTTT".to_string()),
            ("seq_b".to_string(), "AAAT".to_string()),
        ]
    );
    assert_eq!(
        super5_align_dupes(&mut made_s5),
        "Inserting 1 dupes... done.\n"
    );
    assert_eq!(
        made_s5
            .extended_msa
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
            .collect::<Vec<_>>(),
        vec![
            ("seq_a".to_string(), "AAAA".to_string()),
            ("seq_d".to_string(), "TTTT".to_string()),
            ("seq_b".to_string(), "AAAT".to_string()),
            ("seq_c".to_string(), "AAAA".to_string()),
        ]
    );

    let mut run_s5 = Super5 {
        min_ea_pass1: 0.6,
        ..Super5::default()
    };
    let mut super4_perms = Vec::new();
    super5_run(
        &mut run_s5,
        &make_input,
        TREEPERM::TP_All,
        false,
        |d, input_seqs| {
            d.input_seqs = Some(input_seqs.clone());
            d.seq_index_to_rep_seq_index = vec![0, 1, 0, 3];
            d.rep_seq_indexes = vec![0, 1, 3];
            d.rep_seq_index_to_seq_indexes = vec![vec![0, 2], vec![1], Vec::new(), vec![3]];
            let mut unique = MultiSequence::default();
            for seq in [
                &input_seqs.seqs[0],
                &input_seqs.seqs[1],
                &input_seqs.seqs[3],
            ] {
                unique.seqs.push(seq.clone());
                unique.owners.push(false);
            }
            unique
        },
        |u, unique_seqs, min_ea| {
            assert_eq!(min_ea, 0.6);
            u.input_seqs = Some(unique_seqs.clone());
            u.centroid_seq_indexes = vec![0, 2];
            u.seq_index_to_centroid_seq_index = vec![0, 0, 2];
            u.seq_index_to_path = vec![String::new(), "BBBB".to_string(), String::new()];
            let mut centroids = MultiSequence::default();
            u_clust_get_centroid_seqs(u, &mut centroids);
            centroids
        },
        |s4, centroid_seqs, perm| {
            super4_perms.push(perm);
            assert_eq!(
                centroid_seqs
                    .seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>(),
                vec!["seq_a".to_string(), "seq_d".to_string()]
            );
            s4.final_msa_none = centroid_seqs.clone();
            s4.final_msa_abc = centroid_seqs.clone();
            s4.final_msa_acb = centroid_seqs.clone();
            s4.final_msa_bca = centroid_seqs.clone();
        },
    );
    assert_eq!(super4_perms, vec![TREEPERM::TP_All]);
    assert_eq!(
        run_s5
            .final_msa_none
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec![
            "seq_a".to_string(),
            "seq_c".to_string(),
            "seq_b".to_string(),
            "seq_d".to_string()
        ]
    );
    assert_eq!(
        run_s5
            .final_msa_bca
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec![
            "seq_a".to_string(),
            "seq_c".to_string(),
            "seq_b".to_string(),
            "seq_d".to_string()
        ]
    );

    let mut opt_s5 = Super5::default();
    super5_set_opts(&mut opt_s5, Some(0.33));
    assert!((opt_s5.min_ea_pass1 - 0.33).abs() < f32::EPSILON);

    let cmd_dir = std::env::temp_dir().join(format!("muscle_rs_cmd_super5_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_dir).unwrap();
    let cmd_input = cmd_dir.join("input.fa");
    let cmd_output = cmd_dir.join("out.fa");
    std::fs::write(&cmd_input, b">aa\nA-A\n>cc\nC-C\n").unwrap();
    let mut cmd_seen = Vec::new();
    let (cmd_s5, cmd_files, cmd_log) = cmd_super5(
        cmd_input.to_str().unwrap(),
        cmd_output.to_str().unwrap(),
        Some(true),
        Some(TREEPERM::TP_None),
        None,
        Some(0.44),
        true,
        false,
        false,
        false,
        false,
        |s5, input_seqs, perm, input_order| {
            assert_eq!(perm, TREEPERM::TP_None);
            assert!(input_order);
            assert!((s5.min_ea_pass1 - 0.44).abs() < f32::EPSILON);
            cmd_seen = input_seqs
                .seqs
                .iter()
                .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
                .collect::<Vec<_>>();
            let mut final_msa = MultiSequence::default();
            multi_sequence_from_strings(
                &mut final_msa,
                &["aa".to_string(), "cc".to_string()],
                &["AA".to_string(), "CC".to_string()],
            );
            s5.final_msa = Some(final_msa);
            "cmd-super5\n".to_string()
        },
    );
    assert_eq!(
        cmd_seen,
        vec![
            ("aa".to_string(), "AA".to_string()),
            ("cc".to_string(), "CC".to_string())
        ]
    );
    assert_eq!(cmd_files, vec![cmd_output.to_string_lossy().to_string()]);
    assert_eq!(cmd_log, "cmd-super5\n");
    assert!(cmd_s5.final_msa.is_none());
    assert_eq!(
        std::fs::read_to_string(&cmd_output).unwrap(),
        ">aa\nAA\n>cc\nCC\n"
    );

    let all_pattern = cmd_dir.join("perm_@.fa");
    let (_cmd_all_s5, all_files, all_log) = cmd_super5(
        cmd_input.to_str().unwrap(),
        all_pattern.to_str().unwrap(),
        Some(true),
        Some(TREEPERM::TP_All),
        Some(7),
        None,
        false,
        false,
        false,
        false,
        false,
        |s5, _input_seqs, perm, input_order| {
            assert_eq!(perm, TREEPERM::TP_All);
            assert!(!input_order);
            multi_sequence_from_strings(
                &mut s5.final_msa_none,
                &["none".to_string()],
                &["AA".to_string()],
            );
            multi_sequence_from_strings(
                &mut s5.final_msa_abc,
                &["abc".to_string()],
                &["CC".to_string()],
            );
            multi_sequence_from_strings(
                &mut s5.final_msa_acb,
                &["acb".to_string()],
                &["GG".to_string()],
            );
            multi_sequence_from_strings(
                &mut s5.final_msa_bca,
                &["bca".to_string()],
                &["TT".to_string()],
            );
            "cmd-super5-all\n".to_string()
        },
    );
    assert_eq!(all_log, "cmd-super5-all\n");
    assert_eq!(
        all_files
            .iter()
            .map(|file_name| std::path::Path::new(file_name)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string())
            .collect::<Vec<_>>(),
        vec![
            "perm_none.7.fa".to_string(),
            "perm_abc.7.fa".to_string(),
            "perm_acb.7.fa".to_string(),
            "perm_bca.7.fa".to_string(),
        ]
    );
    assert_eq!(
        std::fs::read_to_string(&all_files[2]).unwrap(),
        ">acb\nGG\n"
    );
    std::fs::remove_dir_all(&cmd_dir).unwrap();

    s5.final_msa_none = s5.input_seqs.clone().unwrap();
    s5.final_msa_abc = s5.final_msa_none.clone();
    s5.final_msa_acb = s5.final_msa_none.clone();
    s5.final_msa_bca = s5.final_msa_none.clone();
    tree_create_rooted(&mut s5.guide_tree_none);
    super5_clear_trees_and_ms_as(&mut s5);
    assert!(s5.final_msa_none.seqs.is_empty());
    assert!(s5.final_msa_abc.seqs.is_empty());
    assert!(s5.final_msa_acb.seqs.is_empty());
    assert!(s5.final_msa_bca.seqs.is_empty());
    assert_eq!(s5.guide_tree_none.node_count, 0);
}

#[test]
fn ea_cluster_mfa_helpers_match_cpp_cluster_order() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    let labels = vec![
        "seq_a".to_string(),
        "seq_b".to_string(),
        "seq_c".to_string(),
    ];
    let seqs = vec!["AAAA".to_string(), "AAAT".to_string(), "GGGG".to_string()];
    let mut input = MultiSequence::default();
    multi_sequence_from_strings(&mut input, &labels, &seqs);
    set_global_input_ms(&input);
    MEGA_STATE.lock().unwrap().loaded = false;
    let (flat_ea, flat_path) = align_pair_flat("seq_a", "seq_b");
    assert!(!flat_path.is_empty());
    assert_eq!(ea_cluster_align_seq_pair("seq_a", "seq_b"), flat_ea);

    let mut centroid_only = MultiSequence::default();
    multi_sequence_from_strings(
        &mut centroid_only,
        &["seq_a".to_string()],
        &["AAAA".to_string()],
    );
    let mut s5 = Super5 {
        input_seqs: Some(input.clone()),
        centroid_seqs: Some(centroid_only.clone()),
        centroid_msa: Some(centroid_only),
        is_dupe: vec![false, false, true],
        is_centroid: vec![true, false, false],
        is_member: vec![false, true, false],
        centroid_gs_is: vec![0],
        gsi_to_member_count: vec![1, 0, 0],
        gsi_to_centroid_gsi: vec![uint::MAX, 0, uint::MAX],
        centroid_gsi_to_member_gs_is: vec![vec![1], vec![], vec![]],
        dupe_rep_gsi_to_member_gs_is: vec![vec![2], vec![], vec![]],
        ..Super5::default()
    };
    super5_set_centroid_seqs_vecs(&mut s5);
    super5_set_centroid_msa_vecs(&mut s5);
    super5_validate_vecs(&s5);
    assert_eq!(s5.centroid_seqs_seq_index_to_gsi, vec![0]);
    assert_eq!(s5.centroid_msa_seq_index_to_gsi, vec![0]);
    assert_eq!(
        super5_log_clusters(&s5),
        "  GSI  Cat   CSSI   CMSI   MbCt   Cent\n    0  Cnt      0      0      1      *  >seq_a <1> =2\n    1  Mem      *      *      0      0  >seq_b  >> seq_a\n    2  Dup      *      *      0      *  >seq_c\n"
    );
    let mut labels_from_centroid = Vec::new();
    super5_append_labels_from_centroid(&s5, 0, &mut labels_from_centroid);
    assert_eq!(
        labels_from_centroid,
        vec![
            "seq_a".to_string(),
            "seq_c".to_string(),
            "seq_b".to_string()
        ]
    );
    assert_eq!(
        super5_get_labels_in_guide_tree_order(&s5),
        labels_from_centroid
    );
    let mut s5_aln = MultiSequence::default();
    multi_sequence_from_strings(
        &mut s5_aln,
        &[
            "seq_c".to_string(),
            "seq_a".to_string(),
            "seq_b".to_string(),
        ],
        &["GGGG".to_string(), "AAAA".to_string(), "AAAT".to_string()],
    );
    assert_eq!(
        super5_get_label_to_aln_seq_index(&s5, &s5_aln)
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            ("seq_a".to_string(), 1),
            ("seq_b".to_string(), 2),
            ("seq_c".to_string(), 0)
        ]
    );
    super5_sort_msa_by_guide_tree(&s5, &mut s5_aln);
    assert_eq!(
        s5_aln
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["seq_a", "seq_c", "seq_b"]
    );
    super5_sort_msa_by_input_order(&s5, &mut s5_aln);
    assert_eq!(
        s5_aln
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["seq_a", "seq_b", "seq_c"]
    );

    let mut ec = EACluster {
        input_seqs: Some(input.clone()),
        us: USorter::default(),
        centroid_seq_indexes: vec![0, 2],
        centroid_index_to_seq_indexes: vec![vec![0, 1], vec![2]],
        seq_index_to_centroid_index: vec![0, 0, 1],
        cluster_mfas: Vec::new(),
    };

    ea_cluster_validate(&ec);
    ea_cluster_make_cluster_mf_as(&mut ec);
    assert_eq!(ec.cluster_mfas.len(), 2);
    assert_eq!(ec.cluster_mfas[0].seqs[0].label, "seq_a");
    assert_eq!(ec.cluster_mfas[0].seqs[1].label, "seq_b");
    assert_eq!(ec.cluster_mfas[0].seqs.len(), 2);
    assert_eq!(ec.cluster_mfas[1].seqs[0].label, "seq_c");
    assert_eq!(ec.cluster_mfas[1].seqs.len(), 1);

    let copies = ea_cluster_get_cluster_mf_as(&ec);
    assert_eq!(copies.len(), 2);
    assert_eq!(copies[0].seqs[1].label, "seq_b");
    assert_eq!(sequence_get_seq_as_string(&copies[1].seqs[0]), "GGGG");

    let pattern = format!("/tmp/muscle_rs_ea_cluster_{}_@.afa", std::process::id());
    let file_names = ea_cluster_write_mf_as(&ec, &pattern);
    assert_eq!(file_names.len(), 2);
    assert!(file_names[0].ends_with("_1.afa"));
    assert!(file_names[1].ends_with("_2.afa"));
    assert_eq!(
        std::fs::read_to_string(&file_names[0]).unwrap(),
        ">seq_a\nAAAA\n>seq_b\nAAAT\n"
    );
    assert_eq!(
        std::fs::read_to_string(&file_names[1]).unwrap(),
        ">seq_c\nGGGG\n"
    );
    let _ = std::fs::remove_file(&file_names[0]);
    let _ = std::fs::remove_file(&file_names[1]);

    let long_seq = "ACDEFGHIKLMNPQRSTVWY".repeat(5);
    let mut long_mfa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut long_mfa,
        &["long".to_string()],
        std::slice::from_ref(&long_seq),
    );
    let long_ec = EACluster {
        cluster_mfas: vec![long_mfa],
        ..EACluster::default()
    };
    let long_pattern = format!(
        "/tmp/muscle_rs_ea_cluster_long_{}_@.afa",
        std::process::id()
    );
    let long_files = ea_cluster_write_mf_as(&long_ec, &long_pattern);
    assert_eq!(
        std::fs::read_to_string(&long_files[0]).unwrap(),
        format!(">long\n{}\n{}\n", &long_seq[..80], &long_seq[80..])
    );
    let _ = std::fs::remove_file(&long_files[0]);

    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut run_ec = EACluster::default();
    let mut run_calls = Vec::new();
    ea_cluster_run(&mut run_ec, &input, 0.9, |label1, label2| {
        run_calls.push((label1.to_string(), label2.to_string()));
        if (label1 == "seq_b" && label2 == "seq_a") || (label1 == "seq_a" && label2 == "seq_b") {
            0.95
        } else {
            0.1
        }
    });
    assert_eq!(run_ec.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(run_ec.seq_index_to_centroid_index, vec![0, 0, 1]);
    assert_eq!(
        run_ec.centroid_index_to_seq_indexes,
        vec![vec![0, 1], vec![2]]
    );
    assert_eq!(run_ec.cluster_mfas.len(), 2);
    assert!(run_calls.contains(&("seq_b".to_string(), "seq_a".to_string())));

    let mut best_ec = EACluster::default();
    best_ec.input_seqs = Some(input.clone());
    u_sorter_init(&mut best_ec.us);
    best_ec.centroid_seq_indexes = vec![0];
    best_ec.centroid_index_to_seq_indexes = vec![vec![0]];
    best_ec.seq_index_to_centroid_index = vec![0, uint::MAX, uint::MAX, uint::MAX];
    u_sorter_add_seq(&mut best_ec.us, b"AAAA", 0);
    let mut best_ea = 0.0;
    let best = ea_cluster_get_best_centroid(&best_ec, 1, 0.9, &mut best_ea, |label1, label2| {
        assert_eq!(label1, "seq_b");
        assert_eq!(label2, "seq_a");
        0.93
    });
    assert_eq!(best, 0);
    assert_eq!(best_ea, 0.93);

    let mut sparse_posts = Vec::new();
    let mut dist_calls = Vec::new();
    let (ea_dist_mx, ea_rows) =
        calc_ea_dist_mx(&input, Some(&mut sparse_posts), |label1, label2, path| {
            dist_calls.push((label1.to_string(), label2.to_string()));
            *path = "BB".to_string();
            let mut sparse = MySparseMx::default();
            sparse.lx = label1.len() as uint;
            sparse.ly = label2.len() as uint;
            if label1 == "seq_a" && label2 == "seq_b" {
                (0.75, Some(sparse))
            } else {
                (0.25, Some(sparse))
            }
        });
    assert_eq!(
        dist_calls,
        vec![
            ("seq_a".to_string(), "seq_b".to_string()),
            ("seq_a".to_string(), "seq_c".to_string()),
            ("seq_b".to_string(), "seq_c".to_string()),
        ]
    );
    assert_eq!(sparse_posts.len(), 3);
    assert_eq!(ea_dist_mx[0][0], 1.0);
    assert_eq!(ea_dist_mx[0][1], 0.75);
    assert_eq!(ea_dist_mx[1][0], 0.75);
    assert_eq!(ea_dist_mx[1][2], 0.25);
    assert!(ea_rows.starts_with("seq_a\tseq_b\t0.75\n"));

    let eadist_in =
        std::env::temp_dir().join(format!("muscle_rs_eadist_{}.fa", std::process::id()));
    let eadist_out =
        std::env::temp_dir().join(format!("muscle_rs_eadist_{}.tsv", std::process::id()));
    std::fs::write(&eadist_in, b">seq_a\nAA--AA\n>seq_b\nAA-AT\n>seq_c\nGGGG\n").unwrap();
    let mut cmd_dist_calls = Vec::new();
    let cmd_dist_mx = cmd_eadistmx(
        eadist_in.to_str().unwrap(),
        eadist_out.to_str().unwrap(),
        |label1, label2, path| {
            cmd_dist_calls.push((label1.to_string(), label2.to_string()));
            *path = "BB".to_string();
            if label1 == "seq_a" && label2 == "seq_b" {
                (0.875, None)
            } else {
                (0.125, None)
            }
        },
    );
    assert_eq!(
        cmd_dist_calls,
        vec![
            ("seq_a".to_string(), "seq_b".to_string()),
            ("seq_a".to_string(), "seq_c".to_string()),
            ("seq_b".to_string(), "seq_c".to_string()),
        ]
    );
    assert_eq!(cmd_dist_mx[0][1], 0.875);
    assert_eq!(
        std::fs::read_to_string(&eadist_out).unwrap(),
        "seq_a\tseq_b\t0.875\nseq_a\tseq_c\t0.125\nseq_b\tseq_c\t0.125\n"
    );
    std::fs::remove_file(&eadist_in).unwrap();
    std::fs::remove_file(&eadist_out).unwrap();

    let eacluster_in =
        std::env::temp_dir().join(format!("muscle_rs_eacluster_{}.fa", std::process::id()));
    let eacluster_pattern = format!("/tmp/muscle_rs_eacluster_cmd_{}_@.afa", std::process::id());
    std::fs::write(
        &eacluster_in,
        b">seq_a\nEFILL\n>seq_b\nEFILM\n>seq_c\nWYVVP\n",
    )
    .unwrap();
    let mut cmd_cluster_calls = Vec::new();
    let (cmd_ec, cmd_cluster_files) = cmd_eacluster(
        eacluster_in.to_str().unwrap(),
        0.9,
        &eacluster_pattern,
        |label1, label2| {
            cmd_cluster_calls.push((label1.to_string(), label2.to_string()));
            if (label1 == "seq_b" && label2 == "seq_a") || (label1 == "seq_a" && label2 == "seq_b")
            {
                0.95
            } else {
                0.1
            }
        },
    );
    assert_eq!(cmd_ec.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(cmd_ec.seq_index_to_centroid_index, vec![0, 0, 1]);
    assert_eq!(cmd_cluster_files.len(), 2);
    assert_eq!(
        std::fs::read_to_string(&cmd_cluster_files[0]).unwrap(),
        ">seq_a\nEFILL\n>seq_b\nEFILM\n"
    );
    assert_eq!(
        std::fs::read_to_string(&cmd_cluster_files[1]).unwrap(),
        ">seq_c\nWYVVP\n"
    );
    assert!(cmd_cluster_calls.contains(&("seq_b".to_string(), "seq_a".to_string())));
    std::fs::remove_file(&eacluster_in).unwrap();
    for file_name in cmd_cluster_files {
        std::fs::remove_file(file_name).unwrap();
    }

    let eesort_query =
        std::env::temp_dir().join(format!("muscle_rs_eesort_query_{}.fa", std::process::id()));
    let eesort_db =
        std::env::temp_dir().join(format!("muscle_rs_eesort_db_{}.fa", std::process::id()));
    let eesort_fa =
        std::env::temp_dir().join(format!("muscle_rs_eesort_{}.fa", std::process::id()));
    let eesort_tsv =
        std::env::temp_dir().join(format!("muscle_rs_eesort_{}.tsv", std::process::id()));
    std::fs::write(&eesort_query, b">q1\nEFIL\n>q2\nPQRS\n").unwrap();
    std::fs::write(&eesort_db, b">db_a\nEF--IL\n>db_b\nPQ-RS\n>db_c\nWYVV\n").unwrap();
    let mut eesort_calls = Vec::new();
    let (ees, order, tsv_out, fa_out) = cmd_eesort(
        eesort_query.to_str().unwrap(),
        eesort_db.to_str().unwrap(),
        eesort_fa.to_str().unwrap(),
        eesort_tsv.to_str().unwrap(),
        |query_label, db_label, path| {
            eesort_calls.push((query_label.to_string(), db_label.to_string()));
            *path = "BBBB".to_string();
            if query_label == "q1" {
                match db_label {
                    "db_a" => 0.125,
                    "db_b" => 10.0,
                    "db_c" => 1234.0,
                    _ => panic!("unexpected db label {db_label}"),
                }
            } else {
                -1.0
            }
        },
    );
    assert_eq!(ees, vec![0.125, 10.0, 1234.0]);
    assert_eq!(order, vec![2, 1, 0]);
    assert_eq!(
        eesort_calls,
        vec![
            ("q1".to_string(), "db_a".to_string()),
            ("q2".to_string(), "db_a".to_string()),
            ("q1".to_string(), "db_b".to_string()),
            ("q2".to_string(), "db_b".to_string()),
            ("q1".to_string(), "db_c".to_string()),
            ("q2".to_string(), "db_c".to_string()),
        ]
    );
    assert_eq!(tsv_out, "1.23e+03\tdb_c\n10\tdb_b\n0.125\tdb_a\n");
    assert_eq!(std::fs::read_to_string(&eesort_tsv).unwrap(), tsv_out);
    assert_eq!(fa_out, ">db_c\nWYVV\n>db_b\nPQRS\n>db_a\nEFIL\n");
    assert_eq!(std::fs::read_to_string(&eesort_fa).unwrap(), fa_out);
    std::fs::remove_file(&eesort_query).unwrap();
    std::fs::remove_file(&eesort_db).unwrap();
    std::fs::remove_file(&eesort_fa).unwrap();
    std::fs::remove_file(&eesort_tsv).unwrap();

    let uclust_in =
        std::env::temp_dir().join(format!("muscle_rs_uclust_cmd_{}.fa", std::process::id()));
    let uclust_out =
        std::env::temp_dir().join(format!("muscle_rs_uclust_cmd_{}.afa", std::process::id()));
    std::fs::write(
        &uclust_in,
        b">seq_a\nEFKL--EFKL\n>seq_b\nEFKL-EFQI\n>seq_c\nPQRSWXYZ\n",
    )
    .unwrap();
    let mut cmd_uclust_calls = Vec::new();
    let (cmd_u, cmd_centroids) = cmd_uclust(
        uclust_in.to_str().unwrap(),
        uclust_out.to_str().unwrap(),
        0.9,
        |label1, label2| {
            cmd_uclust_calls.push((label1.to_string(), label2.to_string()));
            if (label1 == "seq_b" && label2 == "seq_a") || (label1 == "seq_a" && label2 == "seq_b")
            {
                (0.95, "BBBB".to_string())
            } else {
                (0.1, "XXXX".to_string())
            }
        },
    );
    assert_eq!(cmd_u.centroid_seq_indexes, vec![0, 2]);
    assert_eq!(cmd_u.seq_index_to_centroid_seq_index, vec![0, 0, 2]);
    assert_eq!(
        cmd_centroids
            .seqs
            .iter()
            .map(|seq| sequence_get_seq_as_string(seq))
            .collect::<Vec<_>>(),
        vec!["EFKLEFKL".to_string(), "PQRSWXYZ".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(&uclust_out).unwrap(),
        ">seq_a\nEFKLEFKL\n>seq_c\nPQRSWXYZ\n"
    );
    assert!(cmd_uclust_calls.contains(&("seq_b".to_string(), "seq_a".to_string())));
    std::fs::remove_file(&uclust_in).unwrap();
    std::fs::remove_file(&uclust_out).unwrap();

    ea_cluster_clear(&mut ec);
    assert!(ec.centroid_seq_indexes.is_empty());
    assert!(ec.centroid_index_to_seq_indexes.is_empty());
    assert!(ec.seq_index_to_centroid_index.is_empty());
    assert!(ec.cluster_mfas.is_empty());
}

#[test]
fn tree_leaf_and_neighbor_accessors_match_cpp_slots() {
    let mut t = Tree {
        node_count: 5,
        cache_count: 5,
        neighbor1: vec![NULL_NEIGHBOR, 0, 0, 1, 1],
        neighbor2: vec![1, 3, NULL_NEIGHBOR, NULL_NEIGHBOR, NULL_NEIGHBOR],
        neighbor3: vec![2, 4, NULL_NEIGHBOR, NULL_NEIGHBOR, NULL_NEIGHBOR],
        edge_length1: vec![0.0, 0.1, 0.2, 0.3, 0.4],
        edge_length2: vec![1.1, 1.3, 0.0, 0.0, 0.0],
        edge_length3: vec![1.2, 1.4, 0.0, 0.0, 0.0],
        height: vec![0.0; 5],
        has_edge_length1: vec![false, true, true, true, true],
        has_edge_length2: vec![true, true, false, false, false],
        has_edge_length3: vec![true, true, false, false, false],
        has_height: vec![false; 5],
        ids: vec![uint::MAX; 5],
        names: vec![None; 5],
        rooted: true,
        root_node_index: 0,
    };

    assert!(tree_is_edge(&t, 1, 3));
    assert!(!tree_is_edge(&t, 3, 4));
    assert_eq!(tree_get_sibling(&t, 3), 4);
    assert_eq!(tree_get_first_neighbor(&t, 1, 0), 3);
    assert_eq!(tree_get_second_neighbor(&t, 1, 0), 4);
    assert_eq!(tree_get_neighbor_subscript(&t, 1, 4), 2);
    assert_eq!(tree_get_neighbor(&t, 1, 2), 4);
    assert!(tree_has_edge_length(&t, 1, 3));
    assert_eq!(tree_get_edge_length(&t, 1, 3), 1.3);

    tree_set_leaf_name(&mut t, 3, "leaf-c");
    tree_set_leaf_id(&mut t, 3, 77);
    tree_set_leaf_name(&mut t, 2, "leaf-b");
    tree_set_leaf_name(&mut t, 4, "leaf-d");
    assert_eq!(tree_get_leaf_name(&t, 3), Some("leaf-c"));
    assert_eq!(tree_get_leaf_id(&t, 3), 77);
    assert_eq!(leaf_indexes_to_ids(&t, &[3, 4]), vec![77, uint::MAX]);
    assert_eq!(tree_get_any_non_leaf_node(&t), 0);
    let tree_log = tree_log_me(&t);
    assert!(tree_log.starts_with("Tree::LogMe 5 nodes, rooted.\n"));
    assert!(tree_log.contains("[ROOT]"));
    assert!(tree_log.contains("    3   1.3000      4   1.4000"));
    assert!(tree_log.contains("   77  leaf-c"));
    let mut ser_t = t.clone();
    tree_set_edge_length(&mut ser_t, 2, 0, 1_234_567.0);
    let mut tree_file = TextFile::default();
    tree_to_file_l22(&ser_t, &mut tree_file);
    assert_eq!(
        String::from_utf8(tree_file.data.clone()).unwrap(),
        "(\n(\nleaf-c:0.3\n,\nleaf-d:0.4\n):0.1\n,\nleaf-b:1.23457e+06\n)\n;\n"
    );
    let tmp =
        std::env::temp_dir().join(format!("muscle_rs_tree_to_file_{}.nwk", std::process::id()));
    tree_to_file_l13(&ser_t, tmp.to_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(&tmp).unwrap(),
        "(\n(\nleaf-c:0.3\n,\nleaf-d:0.4\n):0.1\n,\nleaf-b:1.23457e+06\n)\n;\n"
    );
    std::fs::remove_file(&tmp).unwrap();
    assert_eq!(tree_leaf_index_to_node_index(&t, 0), 2);
    assert_eq!(tree_leaf_index_to_node_index(&t, 1), 3);
    assert_eq!(tree_get_node_index_l1199(&t, "leaf-c"), 3);
    assert_eq!(tree_get_node_index_l1204(&t, "leaf-c"), 3);

    assert_eq!(tree_first_depth_first_node(&t), 3);
    assert_eq!(tree_next_depth_first_node(&t, 3), 4);
    assert_eq!(tree_next_depth_first_node(&t, 4), 1);
    assert_eq!(tree_next_depth_first_node(&t, 1), 2);
    assert_eq!(tree_next_depth_first_node(&t, 2), 0);
    assert_eq!(tree_next_depth_first_node(&t, 0), NULL_NEIGHBOR);
    assert_eq!(tree_first_depth_first_node_r(&t), 2);
    assert_eq!(tree_next_depth_first_node_r(&t, 2), 4);
    assert_eq!(tree_next_depth_first_node_r(&t, 4), 3);
    assert_eq!(tree_next_depth_first_node_r(&t, 3), 1);

    assert_eq!(tree_get_leaf_parent(&t, 3), 1);
    assert_eq!(tree_get_path_to_root(&t, 3), vec![3, 1, 0]);
    assert_eq!(tree_get_lca(&t, 3, 4), 1);
    assert!((tree_get_distance(&t, 3, 0) - 0.4).abs() < 1e-12);
    assert_eq!(tree_get_distance(&t, 0, uint::MAX), 0.0);
    assert_eq!(tree_get_subtree_leaf_nodes(&t, 1), vec![3, 4]);
    assert_eq!(
        tree_get_subtree_leaf_labels(&t, 1),
        vec!["leaf-c".to_string(), "leaf-d".to_string()]
    );
    assert_eq!(tree_get_subtree_leaf_count(&t, 1), 2);
    assert_eq!(tree_get_subtree_leaf_count(&t, uint::MAX), 0);
    assert_eq!(
        tree_get_leaf_labels(&t),
        vec![
            "leaf-b".to_string(),
            "leaf-c".to_string(),
            "leaf-d".to_string()
        ]
    );
    let mut leaves = Vec::new();
    tree_append_leaves(&t, 0, &mut leaves);
    assert_eq!(leaves, vec![3, 4, 2]);
    assert!((tree_get_node_height(&mut t, 1) - 1.35).abs() < 1e-12);
    assert!((tree_get_node_height(&mut t, 0) - 1.825).abs() < 1e-12);

    tree_set_edge_length(&mut t, 3, 1, 2.5);
    assert_eq!(tree_get_edge_length(&t, 3, 1), 2.5);
    assert_eq!(tree_get_edge_length(&t, 1, 3), 2.5);

    let mut ladder = t.clone();
    assert_eq!(tree_ladderize(&mut ladder, true), 1);
    assert_eq!(ladder.neighbor2[0], 2);
    assert_eq!(ladder.neighbor3[0], 1);
    assert_eq!(ladder.edge_length2[0], 1.1);
    assert_eq!(ladder.edge_length3[0], 1.2);

    let mut unrooted_from_root = t.clone();
    tree_unroot_by_deleting_root(&mut unrooted_from_root);
    assert!(!unrooted_from_root.rooted);
    assert_eq!(unrooted_from_root.node_count, 4);
    assert_eq!(unrooted_from_root.neighbor1[0], 1);
    assert_eq!(unrooted_from_root.neighbor2[0], 2);
    assert_eq!(unrooted_from_root.neighbor3[0], 3);
    assert_eq!(unrooted_from_root.neighbor1[1], 0);
    assert!((unrooted_from_root.edge_length1[0] - 2.3).abs() < 1e-12);
    assert!((unrooted_from_root.edge_length1[1] - 2.3).abs() < 1e-12);

    let mut u = Tree {
        node_count: 6,
        cache_count: 6,
        neighbor1: vec![1, 4, 0, 0, 1, 1],
        neighbor2: vec![
            2,
            0,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
        ],
        neighbor3: vec![
            3,
            5,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
        ],
        edge_length1: vec![0.7, 1.4, 1.1, 1.2, 1.4, 1.5],
        edge_length2: vec![1.1, 0.7, 0.0, 0.0, 0.0, 0.0],
        edge_length3: vec![1.2, 1.5, 0.0, 0.0, 0.0, 0.0],
        height: vec![0.0; 6],
        has_edge_length1: vec![true; 6],
        has_edge_length2: vec![true, true, false, false, false, false],
        has_edge_length3: vec![true, true, false, false, false, false],
        has_height: vec![false; 6],
        ids: vec![uint::MAX; 6],
        names: vec![None; 6],
        rooted: false,
        root_node_index: uint::MAX,
    };
    let (leaf_count, total_distance) = tree_get_leaf_count_unrooted(&u, 0, 1);
    assert_eq!(leaf_count, 2);
    assert!((total_distance - 2.9).abs() < 1e-12);

    tree_orient_parent(&mut u, 1, 0);
    assert_eq!(u.neighbor1[1], 0);
    assert_eq!(u.neighbor2[1], 4);
    assert_eq!(u.neighbor3[1], 5);
    assert_eq!(u.edge_length1[1], 0.7);
    assert_eq!(u.edge_length2[1], 1.4);

    let mut min_edge_tree = Tree {
        node_count: 4,
        cache_count: 4,
        neighbor1: vec![1, 0, 1, 1],
        neighbor2: vec![NULL_NEIGHBOR, 2, NULL_NEIGHBOR, NULL_NEIGHBOR],
        neighbor3: vec![NULL_NEIGHBOR, 3, NULL_NEIGHBOR, NULL_NEIGHBOR],
        edge_length1: vec![0.2, 0.2, 1.7, 0.4],
        edge_length2: vec![0.0, 1.7, 0.0, 0.0],
        edge_length3: vec![0.0, 0.4, 0.0, 0.0],
        height: vec![0.0; 4],
        has_edge_length1: vec![true; 4],
        has_edge_length2: vec![false, true, false, false],
        has_edge_length3: vec![false, true, false, false],
        has_height: vec![false; 4],
        ids: vec![uint::MAX; 4],
        names: vec![None; 4],
        rooted: false,
        root_node_index: uint::MAX,
    };
    apply_min_edge_length(&mut min_edge_tree, 0.9);
    assert_eq!(tree_get_edge_length(&min_edge_tree, 0, 1), 0.9);
    assert_eq!(tree_get_edge_length(&min_edge_tree, 1, 0), 0.9);
    assert_eq!(tree_get_edge_length(&min_edge_tree, 1, 2), 1.7);
    assert_eq!(tree_get_edge_length(&min_edge_tree, 1, 3), 0.9);
    assert_eq!(tree_get_edge_length(&min_edge_tree, 3, 1), 0.9);

    let labels = vec![
        "".to_string(),
        "".to_string(),
        "leaf-b".to_string(),
        "leaf-c".to_string(),
        "leaf-d".to_string(),
    ];
    let parents = vec![uint::MAX, 0, 0, 1, 1];
    let lengths = vec![0.0, 1.1, 1.2, 1.3, 1.4];
    let mut from_vectors = Tree::default();
    tree_from_vectors(&mut from_vectors, &labels, &parents, &lengths);
    assert_eq!(from_vectors.node_count, 5);
    assert!(from_vectors.rooted);
    assert_eq!(from_vectors.root_node_index, 3);
    assert_eq!(from_vectors.names[0].as_deref(), Some("leaf-b"));
    assert_eq!(from_vectors.names[1].as_deref(), Some("leaf-c"));
    assert_eq!(from_vectors.names[2].as_deref(), Some("leaf-d"));
    assert_eq!(from_vectors.neighbor2[3], 4);
    assert_eq!(from_vectors.neighbor3[3], 0);
    assert_eq!(from_vectors.neighbor2[4], 1);
    assert_eq!(from_vectors.neighbor3[4], 2);

    let (out_labels, out_parents, out_lengths) = tree_to_vectors(&from_vectors);
    assert_eq!(
        out_labels,
        vec![
            "leaf-b".to_string(),
            "leaf-c".to_string(),
            "leaf-d".to_string(),
            "".to_string(),
            "".to_string()
        ]
    );
    assert_eq!(out_parents, vec![3, 4, 4, uint::MAX, 3]);
    assert_eq!(out_lengths, vec![1.2, 1.3, 1.4, 0.0, 1.1]);

    let nodes_file = std::env::temp_dir().join(format!(
        "muscle_rs_tree_subset_nodes_{}.tsv",
        std::process::id()
    ));
    std::fs::write(&nodes_file, b"0\tB2\n1\tC2\n2\tD2\n").unwrap();
    let (subset_nodes, subset_labels) = ints_from_file(nodes_file.to_str().unwrap());
    std::fs::remove_file(&nodes_file).unwrap();
    assert_eq!(subset_nodes, vec![0, 1, 2]);
    assert_eq!(
        subset_labels,
        vec!["B2".to_string(), "C2".to_string(), "D2".to_string()]
    );

    let subset_tree = make_subset_nodes(&from_vectors, &subset_nodes, &subset_labels);
    let (subset_out_labels, subset_out_parents, subset_out_lengths) = tree_to_vectors(&subset_tree);
    assert_eq!(subset_out_labels, vec!["B2", "C2", "D2", "", ""]);
    assert_eq!(subset_out_parents, vec![3, 4, 4, uint::MAX, 3]);
    assert_eq!(subset_out_lengths, vec![1.2, 1.3, 1.4, 0.0, 1.1]);

    let tree_file = std::env::temp_dir().join(format!(
        "muscle_rs_tree_subset_in_{}.nwk",
        std::process::id()
    ));
    let nodes_file = std::env::temp_dir().join(format!(
        "muscle_rs_tree_subset_nodes_cmd_{}.tsv",
        std::process::id()
    ));
    let out_file = std::env::temp_dir().join(format!(
        "muscle_rs_tree_subset_out_{}.nwk",
        std::process::id()
    ));
    tree_to_file_l13(&from_vectors, tree_file.to_str().unwrap());
    let mut parsed_input = Tree::default();
    tree_from_file_l143(&mut parsed_input, tree_file.to_str().unwrap());
    let split_prefix = std::env::temp_dir()
        .join(format!("muscle_rs_split_tree_{}_", std::process::id()))
        .to_string_lossy()
        .to_string();
    let split_out = std::env::temp_dir().join(format!(
        "muscle_rs_split_tree_out_{}.nwk",
        std::process::id()
    ));
    let split_files = cmd_split_tree(
        tree_file.to_str().unwrap(),
        2,
        &split_prefix,
        Some(split_out.to_str().unwrap()),
    )
    .unwrap();
    assert_eq!(split_files.len(), 2);
    let split_texts = split_files
        .iter()
        .map(|file_name| std::fs::read_to_string(file_name).unwrap())
        .collect::<Vec<_>>();
    assert!(split_texts.iter().any(|text| text.contains("leaf-b")));
    assert!(
        split_texts
            .iter()
            .any(|text| text.contains("leaf-c") || text.contains("leaf-d"))
    );
    assert!(
        std::fs::read_to_string(&split_out)
            .unwrap()
            .contains("split")
    );
    for file_name in split_files {
        std::fs::remove_file(file_name).unwrap();
    }
    std::fs::remove_file(&split_out).unwrap();

    let cmd_nodes: Vec<uint> = (0..parsed_input.node_count)
        .filter(|&node| {
            let i = node as usize;
            let neighbor_count = (parsed_input.neighbor1[i] != NULL_NEIGHBOR) as uint
                + (parsed_input.neighbor2[i] != NULL_NEIGHBOR) as uint
                + (parsed_input.neighbor3[i] != NULL_NEIGHBOR) as uint;
            parsed_input.node_count == 1 || neighbor_count == 1
        })
        .take(3)
        .collect();
    assert_eq!(cmd_nodes.len(), 3);
    let cmd_labels = vec!["X0".to_string(), "X1".to_string(), "X2".to_string()];
    std::fs::write(
        &nodes_file,
        format!(
            "{}\t{}\n{}\t{}\n{}\t{}\n",
            cmd_nodes[0], cmd_labels[0], cmd_nodes[1], cmd_labels[1], cmd_nodes[2], cmd_labels[2]
        ),
    )
    .unwrap();
    cmd_tree_subset_nodes(
        tree_file.to_str().unwrap(),
        nodes_file.to_str().unwrap(),
        out_file.to_str().unwrap(),
        false,
    );
    let expected_cmd_subset = make_subset_nodes(&parsed_input, &cmd_nodes, &cmd_labels);
    let direct_out_file = std::env::temp_dir().join(format!(
        "muscle_rs_tree_subset_direct_{}.nwk",
        std::process::id()
    ));
    tree_to_file_l13(&expected_cmd_subset, direct_out_file.to_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(&out_file).unwrap(),
        std::fs::read_to_string(&direct_out_file).unwrap()
    );

    let subtree_out = std::env::temp_dir().join(format!(
        "muscle_rs_divide_subtree_{}.nwk",
        std::process::id()
    ));
    let supertree_out = std::env::temp_dir().join(format!(
        "muscle_rs_divide_supertree_{}.nwk",
        std::process::id()
    ));
    let divide_tree_file =
        std::env::temp_dir().join(format!("muscle_rs_divide_input_{}.nwk", std::process::id()));
    let divide_labels = vec![
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];
    let divide_parents = vec![uint::MAX, 0, 0, 1, 1, 2, 2];
    let divide_lengths = vec![0.0; 7];
    let mut divide_input = Tree::default();
    tree_from_vectors(
        &mut divide_input,
        &divide_labels,
        &divide_parents,
        &divide_lengths,
    );
    tree_to_file_l13(&divide_input, divide_tree_file.to_str().unwrap());
    let mut parsed_divide_input = Tree::default();
    tree_from_file_l143(&mut parsed_divide_input, divide_tree_file.to_str().unwrap());
    let (cmd_subtree, cmd_supertree) = cmd_divide_tree(
        divide_tree_file.to_str().unwrap(),
        "C",
        "D",
        subtree_out.to_str().unwrap(),
        supertree_out.to_str().unwrap(),
    );
    let mut expected_subtree = Tree::default();
    let mut expected_supertree = Tree::default();
    let divide_node = tree_get_lca(
        &parsed_divide_input,
        tree_get_node_index_l1199(&parsed_divide_input, "C"),
        tree_get_node_index_l1199(&parsed_divide_input, "D"),
    );
    divide_tree(
        &parsed_divide_input,
        divide_node,
        &mut expected_subtree,
        &mut expected_supertree,
    );
    assert_eq!(
        tree_to_vectors(&cmd_subtree),
        tree_to_vectors(&expected_subtree)
    );
    assert_eq!(
        tree_to_vectors(&cmd_supertree),
        tree_to_vectors(&expected_supertree)
    );
    let expected_subtree_file = std::env::temp_dir().join(format!(
        "muscle_rs_divide_subtree_direct_{}.nwk",
        std::process::id()
    ));
    let expected_supertree_file = std::env::temp_dir().join(format!(
        "muscle_rs_divide_supertree_direct_{}.nwk",
        std::process::id()
    ));
    tree_to_file_l13(&expected_subtree, expected_subtree_file.to_str().unwrap());
    tree_to_file_l13(
        &expected_supertree,
        expected_supertree_file.to_str().unwrap(),
    );
    assert_eq!(
        std::fs::read_to_string(&subtree_out).unwrap(),
        std::fs::read_to_string(&expected_subtree_file).unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(&supertree_out).unwrap(),
        std::fs::read_to_string(&expected_supertree_file).unwrap()
    );
    std::fs::remove_file(&tree_file).unwrap();
    std::fs::remove_file(&nodes_file).unwrap();
    std::fs::remove_file(&out_file).unwrap();
    std::fs::remove_file(&direct_out_file).unwrap();
    std::fs::remove_file(&divide_tree_file).unwrap();
    std::fs::remove_file(&subtree_out).unwrap();
    std::fs::remove_file(&supertree_out).unwrap();
    std::fs::remove_file(&expected_subtree_file).unwrap();
    std::fs::remove_file(&expected_supertree_file).unwrap();

    let splitter_labels = vec![
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];
    let splitter_parents = vec![uint::MAX, 0, 0, 1, 1, 2, 2];
    let splitter_lengths = vec![0.0; 7];
    let mut splitter_tree = Tree::default();
    tree_from_vectors(
        &mut splitter_tree,
        &splitter_labels,
        &splitter_parents,
        &splitter_lengths,
    );
    let mut splitter = TreeSplitter::default();
    tree_splitter_run(&mut splitter, &splitter_tree, 3);
    assert_eq!(splitter.subtree_nodes.len(), 3);
    assert_eq!(
        tree_splitter_get_size_order(&splitter)
            .iter()
            .map(|&i| splitter.subtree_nodes[i as usize])
            .collect::<Vec<_>>(),
        vec![6, 0, 1]
    );
    assert_eq!(tree_splitter_get_biggest_node(&splitter), 6);
    assert_eq!(
        tree_splitter_get_labels_vec(&splitter),
        vec![
            vec!["C".to_string(), "D".to_string()],
            vec!["A".to_string()],
            vec!["B".to_string()]
        ]
    );
    let splitter_log = tree_splitter_log_state(&splitter);
    assert!(splitter_log.contains("_______________ Split 2 ______________"));
    assert!(splitter_log.contains("Total 4\n"));
    let (splitter_subtree, splitter_sub_labels) = tree_splitter_get_subtree(&splitter);
    assert_eq!(splitter_sub_labels, vec!["split0", "split1", "split2"]);
    assert_eq!(
        tree_get_leaf_labels(&splitter_subtree),
        vec![
            "split0".to_string(),
            "split1".to_string(),
            "split2".to_string()
        ]
    );
    let split_prefix =
        std::env::temp_dir().join(format!("muscle_rs_splitter_labels_{}_", std::process::id()));
    let split_files = tree_splitter_write_labels(&splitter, split_prefix.to_str().unwrap());
    assert_eq!(split_files.len(), 3);
    assert_eq!(std::fs::read_to_string(&split_files[0]).unwrap(), "C\nD\n");
    assert_eq!(std::fs::read_to_string(&split_files[1]).unwrap(), "A\n");
    assert_eq!(std::fs::read_to_string(&split_files[2]).unwrap(), "B\n");
    for file in split_files {
        std::fs::remove_file(file).unwrap();
    }

    let divide_labels = vec![
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "leaf-a".to_string(),
        "leaf-b".to_string(),
        "leaf-c".to_string(),
        "".to_string(),
        "leaf-d".to_string(),
        "leaf-e".to_string(),
    ];
    let divide_parents = vec![uint::MAX, 0, 0, 1, 1, 2, 2, 6, 6];
    let divide_lengths = vec![0.0, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7];
    let mut divide_source = Tree::default();
    tree_from_vectors(
        &mut divide_source,
        &divide_labels,
        &divide_parents,
        &divide_lengths,
    );
    let mut divided_subtree = Tree::default();
    let mut divided_supertree = Tree::default();
    divide_tree(
        &divide_source,
        6,
        &mut divided_subtree,
        &mut divided_supertree,
    );
    assert_eq!(
        tree_get_leaf_labels(&divided_subtree),
        vec!["leaf-a".to_string(), "leaf-b".to_string()]
    );
    assert_eq!(
        tree_get_leaf_labels(&divided_supertree),
        vec![
            "leaf-c".to_string(),
            "leaf-d".to_string(),
            "leaf-e".to_string()
        ]
    );

    let mut fraction_tree1 = Tree::default();
    let mut fraction_tree2 = Tree::default();
    divide_tree_fraction(
        &divide_source,
        0.40,
        &mut fraction_tree1,
        &mut fraction_tree2,
    );
    assert_eq!(
        tree_get_leaf_labels(&fraction_tree1),
        vec!["leaf-a".to_string(), "leaf-b".to_string()]
    );
    assert_eq!(
        tree_get_leaf_labels(&fraction_tree2),
        vec![
            "leaf-c".to_string(),
            "leaf-d".to_string(),
            "leaf-e".to_string()
        ]
    );
    let mut joined_tree = Tree::default();
    join_trees(&divided_subtree, &divided_supertree, &mut joined_tree, 0.25);
    assert_eq!(
        tree_get_leaf_labels(&joined_tree),
        vec![
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string(),
            "leaf-d".to_string(),
            "leaf-e".to_string()
        ]
    );
    let (joined_labels, joined_parents, joined_lengths) = tree_to_vectors(&joined_tree);
    assert_eq!(
        joined_labels,
        vec![
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string(),
            "leaf-d".to_string(),
            "leaf-e".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string()
        ]
    );
    assert_eq!(
        joined_parents[joined_tree.root_node_index as usize],
        uint::MAX
    );
    let root_child_lengths: Vec<f32> = joined_parents
        .iter()
        .enumerate()
        .filter_map(|(i, parent)| {
            if *parent == joined_tree.root_node_index {
                Some(joined_lengths[i])
            } else {
                None
            }
        })
        .collect();
    assert_eq!(root_child_lengths, vec![0.25, 0.25]);

    let mut perm_source = Tree::default();
    tree_create(
        &mut perm_source,
        6,
        4,
        &[0, 2, 4, 6, 9],
        &[1, 3, 5, 7, 8],
        &[0.1, 0.3, 0.5, 0.7, 0.9],
        &[0.2, 0.4, 0.6, 0.8, 1.0],
        &[0, 1, 2, 3, 4, 5],
        &[
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
        ],
    );
    let mut tree_abc = Tree::default();
    let mut tree_acb = Tree::default();
    let mut tree_bca = Tree::default();
    let mut labels_a = Vec::new();
    let mut labels_b = Vec::new();
    let mut labels_c = Vec::new();
    permute_tree(
        &perm_source,
        &mut tree_abc,
        &mut tree_acb,
        &mut tree_bca,
        &mut labels_a,
        &mut labels_b,
        &mut labels_c,
    );
    assert_eq!(labels_a, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(labels_b, vec!["c".to_string(), "d".to_string()]);
    assert_eq!(labels_c, vec!["e".to_string(), "f".to_string()]);
    assert_eq!(
        tree_get_leaf_labels(&tree_abc),
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string()
        ]
    );
    assert_eq!(
        tree_get_leaf_labels(&tree_acb),
        vec![
            "a".to_string(),
            "b".to_string(),
            "e".to_string(),
            "f".to_string(),
            "c".to_string(),
            "d".to_string()
        ]
    );
    assert_eq!(
        tree_get_leaf_labels(&tree_bca),
        vec![
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
            "a".to_string(),
            "b".to_string()
        ]
    );
    let perm_tree_file = std::env::temp_dir().join(format!(
        "muscle_rs_permute_tree_in_{}.nwk",
        std::process::id()
    ));
    let perm_prefix = std::env::temp_dir()
        .join(format!("muscle_rs_permute_tree_{}_", std::process::id()))
        .to_string_lossy()
        .to_string();
    tree_to_file_l13(&perm_source, perm_tree_file.to_str().unwrap());
    let mut parsed_perm_source = Tree::default();
    tree_from_file_l143(&mut parsed_perm_source, perm_tree_file.to_str().unwrap());
    let mut expected_cmd_abc = Tree::default();
    let mut expected_cmd_acb = Tree::default();
    let mut expected_cmd_bca = Tree::default();
    let mut expected_cmd_labels_a = Vec::new();
    let mut expected_cmd_labels_b = Vec::new();
    let mut expected_cmd_labels_c = Vec::new();
    permute_tree(
        &parsed_perm_source,
        &mut expected_cmd_abc,
        &mut expected_cmd_acb,
        &mut expected_cmd_bca,
        &mut expected_cmd_labels_a,
        &mut expected_cmd_labels_b,
        &mut expected_cmd_labels_c,
    );
    let (cmd_abc, cmd_acb, cmd_bca, cmd_labels_a, cmd_labels_b, cmd_labels_c) =
        cmd_permute_tree(perm_tree_file.to_str().unwrap(), Some(&perm_prefix));
    assert_eq!(
        tree_to_vectors(&cmd_abc),
        tree_to_vectors(&expected_cmd_abc)
    );
    assert_eq!(
        tree_to_vectors(&cmd_acb),
        tree_to_vectors(&expected_cmd_acb)
    );
    assert_eq!(
        tree_to_vectors(&cmd_bca),
        tree_to_vectors(&expected_cmd_bca)
    );
    assert_eq!(cmd_labels_a, expected_cmd_labels_a);
    assert_eq!(cmd_labels_b, expected_cmd_labels_b);
    assert_eq!(cmd_labels_c, expected_cmd_labels_c);
    let abc_file = format!("{perm_prefix}ABC.newick");
    let acb_file = format!("{perm_prefix}ACB.newick");
    let bca_file = format!("{perm_prefix}BCA.newick");
    let direct_abc_file = std::env::temp_dir().join(format!(
        "muscle_rs_permute_direct_abc_{}.nwk",
        std::process::id()
    ));
    tree_to_file_l13(&expected_cmd_abc, direct_abc_file.to_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(&abc_file).unwrap(),
        std::fs::read_to_string(&direct_abc_file).unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(format!("{perm_prefix}labelsA.txt")).unwrap(),
        expected_cmd_labels_a
            .iter()
            .map(|label| format!("{label}\n"))
            .collect::<String>()
    );
    std::fs::remove_file(&perm_tree_file).unwrap();
    std::fs::remove_file(&abc_file).unwrap();
    std::fs::remove_file(&acb_file).unwrap();
    std::fs::remove_file(&bca_file).unwrap();
    std::fs::remove_file(&direct_abc_file).unwrap();
    std::fs::remove_file(format!("{perm_prefix}labelsA.txt")).unwrap();
    std::fs::remove_file(format!("{perm_prefix}labelsB.txt")).unwrap();
    std::fs::remove_file(format!("{perm_prefix}labelsC.txt")).unwrap();
    let mut perm_noop = perm_source.clone();
    perm_tree(&mut perm_noop, TREEPERM::TP_ACB);
    assert_eq!(tree_to_vectors(&perm_noop), tree_to_vectors(&perm_source));

    let mut copied = Tree::default();
    tree_copy(&mut copied, &from_vectors);
    assert_eq!(copied.node_count, from_vectors.node_count);
    assert_eq!(copied.neighbor1, from_vectors.neighbor1);
    assert_eq!(copied.neighbor2, from_vectors.neighbor2);
    assert_eq!(copied.neighbor3, from_vectors.neighbor3);
    assert_eq!(copied.names, from_vectors.names);

    let mut singleton = Tree::default();
    tree_create_rooted(&mut singleton);
    assert_eq!(singleton.node_count, 1);
    assert_eq!(singleton.cache_count, 100);
    assert!(singleton.rooted);
    assert_eq!(singleton.root_node_index, 0);
    assert_eq!(singleton.neighbor1[0], NULL_NEIGHBOR);
    let left = tree_append_branch(&mut singleton, 0);
    assert_eq!(left, 1);
    assert_eq!(singleton.node_count, 3);
    assert_eq!(singleton.neighbor2[0], 1);
    assert_eq!(singleton.neighbor3[0], 2);
    assert_eq!(singleton.neighbor1[1], 0);
    assert_eq!(singleton.neighbor1[2], 0);

    let mut edge = Tree::default();
    tree_create_unrooted(&mut edge, 0.75);
    assert_eq!(edge.node_count, 2);
    assert!(!edge.rooted);
    assert_eq!(edge.neighbor1[0], 1);
    assert_eq!(edge.neighbor1[1], 0);
    assert_eq!(tree_get_edge_length(&edge, 0, 1), 0.75);

    let mut created = Tree::default();
    tree_create(
        &mut created,
        3,
        1,
        &[0, 3],
        &[1, 2],
        &[0.4, 0.6],
        &[0.5, 0.7],
        &[10, 11, 12],
        &[
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string(),
        ],
    );
    assert_eq!(created.node_count, 5);
    assert!(created.rooted);
    assert_eq!(created.root_node_index, 4);
    assert_eq!(created.ids[..3], [10, 11, 12]);
    assert_eq!(created.neighbor2[3], 0);
    assert_eq!(created.neighbor3[3], 1);
    assert_eq!(created.neighbor2[4], 3);
    assert_eq!(created.neighbor3[4], 2);
    assert!((tree_get_edge_length(&created, 4, 3) - 0.6).abs() < 1e-6);
    assert_eq!(tree_get_subtree_sizes(&created), vec![1, 1, 1, 2, 3]);
    assert_eq!(get_shrubs(&created, 1), vec![0, 1, 2]);
    assert_eq!(get_shrubs(&created, 2), vec![2, 3]);
    assert_eq!(get_shrubs(&created, 3), vec![4]);
    let shrub_tree_file =
        std::env::temp_dir().join(format!("muscle_rs_cmd_shrub_{}.nwk", std::process::id()));
    tree_to_file_l13(&created, shrub_tree_file.to_str().unwrap());
    let (cmd_shrub_lcas, cmd_pruned, cmd_shrub_log) =
        cmd_shrub(shrub_tree_file.to_str().unwrap(), Some(2));
    assert_eq!(cmd_shrub_lcas.len(), 2);
    assert_eq!(cmd_pruned.node_count, 3);
    assert!(cmd_shrub_log.contains("leaf-a"));
    assert!(cmd_shrub_log.contains("leaf-b"));
    assert!(cmd_shrub_log.contains("leaf-c"));
    assert!(cmd_shrub_log.contains("[leaf-a+leaf-b] [+leaf-c]"));
    std::fs::remove_file(&shrub_tree_file).unwrap();
    let mut input_seqs = MultiSequence::default();
    multi_sequence_from_strings(
        &mut input_seqs,
        &[
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string(),
        ],
        &["AC".to_string(), "AG".to_string(), "AT".to_string()],
    );
    let mut s7 = Super7 {
        input_seqs: Some(input_seqs),
        guide_tree: Some(created.clone()),
        ..Default::default()
    };
    super7_map_labels(&mut s7);
    assert_eq!(s7.node_to_seq_index, vec![0, 1, 2, uint::MAX, uint::MAX]);
    assert_eq!(s7.seq_index_to_node, vec![0, 1, 2]);
    super7_set_shrubs(&mut s7, 2);
    assert_eq!(s7.shrub_lcas, vec![2, 3]);
    super7_set_shrub_tree(&mut s7);
    assert_eq!(s7.shrub_labels, vec!["Shrub_0", "Shrub_1"]);
    assert_eq!(tree_get_leaf_labels(&s7.shrub_tree), s7.shrub_labels);
    let mut shrub_input = MultiSequence::default();
    super7_make_shrub_input(&s7, 3, &mut shrub_input);
    assert_eq!(
        shrub_input
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["leaf-a", "leaf-b"]
    );
    assert_eq!(shrub_input.owners, vec![false, false]);
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.labels = vec![
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string(),
        ];
        mega.label_to_idx.clear();
        mega.label_to_idx.insert("leaf-a".to_string(), 0);
        mega.label_to_idx.insert("leaf-b".to_string(), 1);
        mega.label_to_idx.insert("leaf-c".to_string(), 2);
        mega.profiles = vec![vec![vec![1, 2]], vec![vec![3, 4]], vec![vec![5, 6]]];
    }
    let s7m = Super7Mega { base: s7.clone() };
    assert_eq!(
        super7_mega_get_shrub_profiles(&s7m, 3),
        vec![vec![vec![1, 2]], vec![vec![3, 4]]]
    );
    let mut intra_s7m = s7m.clone();
    intra_s7m.base.mpc = Some(MPCFlat::default());
    let mut mega_intra_labels = Vec::new();
    super7_mega_intra_align_shrub(&mut intra_s7m, 1, |mpc, shrub_input| {
        assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
        mega_intra_labels = shrub_input
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>();
        shrub_input.clone()
    });
    assert_eq!(
        mega_intra_labels,
        vec!["leaf-a".to_string(), "leaf-b".to_string()]
    );
    assert_eq!(intra_s7m.base.shrub_msas.len(), 1);

    let mut single_s7 = Super7::default();
    let mut single_mpc_labels = Vec::new();
    super7_run(
        &mut single_s7,
        s7.input_seqs.as_ref().unwrap(),
        &created,
        3,
        |mpc, shrub_input| {
            assert!(mpc.d.disable);
            single_mpc_labels.push(
                shrub_input
                    .seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>(),
            );
            shrub_input.clone()
        },
        |_pp, _tree| panic!("single shrub Super7::Run should not call PProg"),
    );
    assert_eq!(
        single_mpc_labels,
        vec![vec![
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string()
        ]]
    );
    assert_eq!(
        single_s7
            .final_msa
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec![
            "leaf-a".to_string(),
            "leaf-b".to_string(),
            "leaf-c".to_string()
        ]
    );

    let mut multi_s7 = Super7::default();
    let mut shrub_mpc_labels = Vec::new();
    let multi_log = super7_run(
        &mut multi_s7,
        s7.input_seqs.as_ref().unwrap(),
        &created,
        2,
        |mpc, shrub_input| {
            assert_eq!(mpc.tree_perm, TREEPERM::TP_None);
            shrub_mpc_labels.push(
                shrub_input
                    .seqs
                    .iter()
                    .map(|seq| seq.label.clone())
                    .collect::<Vec<_>>(),
            );
            shrub_input.clone()
        },
        |pp, shrub_tree| {
            assert_eq!(pp.input_msa_count, 2);
            assert_eq!(pp.target_pair_count, 2000);
            assert_eq!(tree_get_leaf_labels(shrub_tree), vec!["Shrub_0", "Shrub_1"]);
            let mut final_msa = MultiSequence::default();
            multi_sequence_from_strings(
                &mut final_msa,
                &["final".to_string()],
                &["ACGT".to_string()],
            );
            final_msa
        },
    );
    assert_eq!(multi_log, "Aligning shrub 1 / 2\nAligning shrub 2 / 2\n");
    assert_eq!(
        shrub_mpc_labels,
        vec![
            vec!["leaf-c".to_string()],
            vec!["leaf-a".to_string(), "leaf-b".to_string()],
        ]
    );
    assert_eq!(multi_s7.shrub_labels, vec!["Shrub_0", "Shrub_1"]);
    assert_eq!(multi_s7.pp.input_msa_count, 2);
    assert_eq!(multi_s7.final_msa.seqs[0].label, "final");

    let cmd_dir = std::env::temp_dir().join(format!("muscle_rs_cmd_super7_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_dir).unwrap();
    let cmd_input = cmd_dir.join("input.fa");
    let cmd_tree = cmd_dir.join("guide.nwk");
    let cmd_output = cmd_dir.join("out.fa");
    std::fs::write(&cmd_input, b">leaf-a\nA-A\n>leaf-b\nA-G\n>leaf-c\nA-T\n").unwrap();
    tree_to_file_l13(&created, cmd_tree.to_str().unwrap());
    let mut cmd_seen = Vec::new();
    let (cmd_s7, cmd_guide_tree, cmd_log) = cmd_super7(
        cmd_input.to_str().unwrap(),
        cmd_output.to_str().unwrap(),
        Some(4),
        Some(cmd_tree.to_str().unwrap()),
        None,
        false,
        |_u, _tree| panic!("guidetreein cmd_super7 should not run UPGMA"),
        |_input, _tree| panic!("guidetreein cmd_super7 should not calculate guide tree"),
        |s7, input_seqs, guide_tree, shrub_size| {
            assert_eq!(shrub_size, 4);
            assert_eq!(guide_tree.node_count, 5);
            assert!(s7.mpc.is_some());
            cmd_seen = input_seqs
                .seqs
                .iter()
                .map(|seq| (seq.label.clone(), sequence_get_seq_as_string(seq)))
                .collect::<Vec<_>>();
            multi_sequence_from_strings(
                &mut s7.final_msa,
                &[
                    "leaf-a".to_string(),
                    "leaf-b".to_string(),
                    "leaf-c".to_string(),
                ],
                &["AA".to_string(), "AG".to_string(), "AT".to_string()],
            );
            "cmd-super7\n".to_string()
        },
    );
    assert_eq!(cmd_guide_tree.node_count, 5);
    assert_eq!(
        cmd_seen,
        vec![
            ("leaf-a".to_string(), "AA".to_string()),
            ("leaf-b".to_string(), "AG".to_string()),
            ("leaf-c".to_string(), "AT".to_string()),
        ]
    );
    assert_eq!(cmd_log, "cmd-super7\nDone.\n");
    assert_eq!(cmd_s7.final_msa.seqs.len(), 3);
    assert_eq!(
        std::fs::read_to_string(&cmd_output).unwrap(),
        ">leaf-a\nAA\n>leaf-b\nAG\n>leaf-c\nAT\n"
    );
    std::fs::remove_dir_all(&cmd_dir).unwrap();

    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.profiles = vec![
            vec![vec![1], vec![2]],
            vec![vec![3], vec![4]],
            vec![vec![5], vec![6]],
        ];
    }
    let mut single_s7m = Super7Mega::default();
    let mut mega_single_called = false;
    super7_mega_run(
        &mut single_s7m,
        s7.input_seqs.as_ref().unwrap(),
        &created,
        3,
        |_mpc, shrub_input| {
            mega_single_called = true;
            shrub_input.clone()
        },
        |_pp, _tree| panic!("single shrub Super7_mega::Run should not call PProg"),
    );
    assert!(mega_single_called);
    assert_eq!(single_s7m.base.final_msa.seqs.len(), 3);

    let mut multi_s7m = Super7Mega::default();
    let mega_multi_log = super7_mega_run(
        &mut multi_s7m,
        s7.input_seqs.as_ref().unwrap(),
        &created,
        2,
        |_mpc, shrub_input| shrub_input.clone(),
        |pp, _shrub_tree| {
            assert_eq!(pp.input_msa_count, 2);
            let mut final_msa = MultiSequence::default();
            multi_sequence_from_strings(
                &mut final_msa,
                &["mega_final".to_string()],
                &["AA".to_string()],
            );
            final_msa
        },
    );
    assert_eq!(
        mega_multi_log,
        "Aligning shrub 1 / 2\nAligning shrub 2 / 2\n"
    );
    assert_eq!(multi_s7m.base.final_msa.seqs[0].label, "mega_final");

    tree_assert_are_neighbors(&created, 4, 3);
    tree_validate_node(&created, 4);
    tree_validate(&created);
    let mut asym_tree = Tree::default();
    tree_create_unrooted(&mut asym_tree, 1.0);
    asym_tree.edge_length1[0] = 0.00012345;
    asym_tree.edge_length1[1] = 1.2345;
    let asym_panic = std::panic::catch_unwind(|| tree_assert_are_neighbors(&asym_tree, 0, 1));
    let asym_panic = asym_panic.unwrap_err();
    let asym_panic = asym_panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| asym_panic.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(
        asym_panic,
        "Tree::AssertAreNeighbors, Edge length disagrees 0-1=0.000123, 1-0=1.23"
    );

    let mut label_to_index = std::collections::HashMap::new();
    label_to_index.insert("leaf-a".to_string(), 0);
    label_to_index.insert("leaf-b".to_string(), 1);
    label_to_index.insert("leaf-c".to_string(), 2);
    assert_eq!(get_label(&label_to_index, 1), "leaf-b");
    let (indexes1, indexes2) = get_guide_tree_join_order(&created, &label_to_index);
    assert_eq!(indexes1, vec![0, 3]);
    assert_eq!(indexes2, vec![1, 2]);
    validate_join_order(&indexes1, &indexes2);
    let join_log = log_guide_tree_join_order(&created, &label_to_index, &indexes1, &indexes2);
    assert_eq!(
        join_log,
        "  Join  Index1  Index2\n     0       0       1   'leaf-a' + 'leaf-b'\n     1       3       2   Join0 + 'leaf-c'\n"
    );
    let mut from_join_order = Tree::default();
    make_guide_tree_from_join_order(&indexes1, &indexes2, &label_to_index, &mut from_join_order);
    assert_eq!(from_join_order.node_count, 5);
    assert_eq!(from_join_order.root_node_index, 4);
    assert_eq!(from_join_order.names[0].as_deref(), Some("leaf-a"));
    assert_eq!(from_join_order.names[1].as_deref(), Some("leaf-b"));
    assert_eq!(from_join_order.names[2].as_deref(), Some("leaf-c"));
    assert_eq!(from_join_order.neighbor2[3], 0);
    assert_eq!(from_join_order.neighbor3[3], 1);
    assert_eq!(from_join_order.neighbor2[4], 3);
    assert_eq!(from_join_order.neighbor3[4], 2);

    let join_tree_file = std::env::temp_dir().join(format!(
        "muscle_rs_guide_join_order_tree_{}.nwk",
        std::process::id()
    ));
    let join_table_file = std::env::temp_dir().join(format!(
        "muscle_rs_guide_join_order_table_{}.tsv",
        std::process::id()
    ));
    tree_to_file_l13(&created, join_tree_file.to_str().unwrap());
    let cmd_log = cmd_guide_tree_join_order(
        join_tree_file.to_str().unwrap(),
        Some(join_table_file.to_str().unwrap()),
    );
    assert_eq!(
        cmd_log,
        "  Join  Index1  Index2\n     0       1       2   'leaf-a' + 'leaf-b'\n     1       3       0   Join0 + 'leaf-c'\n"
    );
    assert_eq!(
        std::fs::read_to_string(&join_table_file).unwrap(),
        "0\t1\t2\tleaf\tleaf-a\tleaf\tleaf-b\n1\t3\t0\tjoin\t0\tleaf\tleaf-c\n"
    );
    std::fs::remove_file(&join_tree_file).unwrap();
    std::fs::remove_file(&join_table_file).unwrap();

    let mut es = PhyEnumEdgeState {
        init: false,
        node_index1: NULL_NEIGHBOR,
        node_index2: NULL_NEIGHBOR,
    };
    let mut edges = Vec::new();
    while phy_enum_edges(&created, &mut es) {
        edges.push((es.node_index1, es.node_index2));
    }
    assert_eq!(edges, vec![(0, 3), (1, 3), (3, 4), (2, 4)]);

    let mut es_r = PhyEnumEdgeState {
        init: false,
        node_index1: NULL_NEIGHBOR,
        node_index2: NULL_NEIGHBOR,
    };
    let mut edges_r = Vec::new();
    while phy_enum_edges_r(&created, &mut es_r) {
        edges_r.push((es_r.node_index1, es_r.node_index2));
    }
    assert_eq!(edges_r, vec![(2, 4), (1, 3), (0, 3), (3, 4)]);

    let mut leaves = Vec::new();
    get_leaves_subtree(&created, 3, 4, &mut leaves);
    assert_eq!(leaves, vec![0, 1]);
    assert_eq!(phy_get_leaves(&created, 4, 3), vec![2]);

    let mut es_bp = PhyEnumEdgeState {
        init: false,
        node_index1: NULL_NEIGHBOR,
        node_index2: NULL_NEIGHBOR,
    };
    assert_eq!(
        phy_enum_bi_parts(&created, &mut es_bp),
        Some((vec![0], vec![2, 1]))
    );
    let bipart_log = test_bi_part(&created);
    assert!(bipart_log.starts_with("Tree::LogMe 5 nodes, rooted.\n"));
    assert!(bipart_log.contains("PEBP=1 ES.Init=1 ES.ni1=0 ES.ni2=3\n"));
    assert!(bipart_log.contains("Part1:  0(leaf-a)\nPart2:  2(leaf-c) 1(leaf-b)\n"));
    assert!(bipart_log.ends_with("PEBP=0 ES.Init=1 ES.ni1=2 ES.ni2=4\n"));
    assert_eq!(get_leaves_excluding(&created, 4, 3), vec![2]);

    let mut parsed = Tree::default();
    let mut newick = TextFile::default();
    text_file_init(
        &mut newick,
        b"((leaf-a:0.4,leaf-b:0.5):0.6,leaf-c:0.7);",
        "tree",
    );
    tree_from_file_l150(&mut parsed, &mut newick);
    assert_eq!(parsed.node_count, 5);
    assert!(parsed.rooted);
    assert_eq!(parsed.root_node_index, 0);
    assert_eq!(parsed.names[3].as_deref(), Some("leaf-a"));
    assert_eq!(parsed.names[4].as_deref(), Some("leaf-b"));
    assert_eq!(parsed.names[2].as_deref(), Some("leaf-c"));
    assert_eq!(parsed.neighbor2[0], 1);
    assert_eq!(parsed.neighbor3[0], 2);
    assert_eq!(parsed.neighbor2[1], 3);
    assert_eq!(parsed.neighbor3[1], 4);
    assert!((tree_get_edge_length(&parsed, 1, 3) - 0.4).abs() < 1e-12);
    assert!((tree_get_edge_length(&parsed, 1, 4) - 0.5).abs() < 1e-12);
    assert!((tree_get_edge_length(&parsed, 0, 1) - 0.6).abs() < 1e-12);
    assert!((tree_get_edge_length(&parsed, 0, 2) - 0.7).abs() < 1e-12);
    tree_validate(&parsed);

    let mut unrooted = Tree::default();
    let mut unrooted_newick = TextFile::default();
    text_file_init(
        &mut unrooted_newick,
        b"(left:1,right:2):3,third:4;",
        "unrooted",
    );
    tree_from_file_l150(&mut unrooted, &mut unrooted_newick);
    assert!(!unrooted.rooted);
    assert_eq!(unrooted.node_count, 4);
    assert_eq!(unrooted.neighbor1[0], 3);
    assert_eq!(unrooted.neighbor2[0], 1);
    assert_eq!(unrooted.neighbor3[0], 2);
    assert_eq!(unrooted.neighbor1[3], 0);
    assert_eq!(unrooted.names[1].as_deref(), Some("left"));
    assert_eq!(unrooted.names[2].as_deref(), Some("right"));
    assert_eq!(unrooted.names[3].as_deref(), Some("third"));
    assert!((tree_get_edge_length(&unrooted, 0, 1) - 1.0).abs() < 1e-12);
    assert!((tree_get_edge_length(&unrooted, 0, 2) - 2.0).abs() < 1e-12);
    assert!((tree_get_edge_length(&unrooted, 0, 3) - 4.0).abs() < 1e-12);
    tree_validate(&unrooted);

    let mut excl_leaves = Vec::new();
    get_leaves_subtree_excluding(&created, 4, 3, &mut excl_leaves);
    assert_eq!(excl_leaves, vec![2]);
    let mut created_for_heights = created.clone();
    assert_eq!(
        get_internal_nodes_in_height_order(&mut created_for_heights),
        vec![3, 4]
    );
    let mut created_for_height_clusters = created.clone();
    assert_eq!(
        cluster_by_height(&mut created_for_height_clusters, 0.5),
        vec![2, 3]
    );
    let mut created_for_leaf_height_clusters = created.clone();
    assert_eq!(
        cluster_by_height(&mut created_for_leaf_height_clusters, 0.0),
        vec![0, 1, 2]
    );
    assert_eq!(get_leaves(&created, 4), vec![0, 1, 2]);
    assert_eq!(get_leaves(&created, 3), vec![0, 1]);
    let mut created_for_subfams = created.clone();
    assert_eq!(
        cluster_by_subfam_count(&mut created_for_subfams, 1),
        vec![4]
    );
    let mut created_for_subfams = created.clone();
    assert_eq!(
        cluster_by_subfam_count(&mut created_for_subfams, 2),
        vec![3, 2]
    );
    let mut created_for_subfams = created.clone();
    assert_eq!(
        cluster_by_subfam_count(&mut created_for_subfams, 3),
        vec![0, 1, 2]
    );
    let mut subfams = vec![4, NULL_NEIGHBOR];
    let mut created_for_iteration = created.clone();
    cluster_by_subfam_count_iteration(&mut created_for_iteration, &mut subfams, 1);
    assert_eq!(subfams, vec![3, 2]);
    let mut pruned = Tree::default();
    let pruned_labels = tree_prune_tree(&mut pruned, &created, &[3, 2], "subfam");
    assert_eq!(
        pruned_labels,
        vec!["subfam0".to_string(), "subfam1".to_string()]
    );
    assert_eq!(pruned.node_count, 3);
    assert!(pruned.rooted);
    assert_eq!(pruned.root_node_index, 2);
    assert_eq!(pruned.names[0].as_deref(), Some("subfam0"));
    assert_eq!(pruned.names[1].as_deref(), Some("subfam1"));
    assert_eq!(pruned.neighbor2[2], 0);
    assert_eq!(pruned.neighbor3[2], 1);
    assert!((tree_get_edge_length(&pruned, 2, 0) - 0.6).abs() < 1e-6);
    assert!((tree_get_edge_length(&pruned, 2, 1) - 0.7).abs() < 1e-6);
    tree_validate(&pruned);
    let mut pruned = Tree::default();
    let pruned_labels = tree_prune_tree(&mut pruned, &created, &[0, 1, 2], "sf");
    assert_eq!(
        pruned_labels,
        vec!["sf0".to_string(), "sf1".to_string(), "sf2".to_string()]
    );
    assert_eq!(pruned.node_count, 5);
    assert_eq!(pruned.root_node_index, 4);
    assert_eq!(pruned.neighbor2[3], 0);
    assert_eq!(pruned.neighbor3[3], 1);
    assert_eq!(pruned.neighbor2[4], 3);
    assert_eq!(pruned.neighbor3[4], 2);
    tree_validate(&pruned);

    let _guard = RNG_TEST_LOCK.lock().unwrap();
    reset_rand(1);
    let labels = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];
    let mut chain = Tree::default();
    make_random_chain_tree(&labels, &mut chain);
    tree_validate(&chain);
    assert_eq!(chain.root_node_index, 6);
    assert_eq!(
        chain.names,
        vec![
            Some("B".to_string()),
            Some("C".to_string()),
            Some("A".to_string()),
            Some("D".to_string()),
            None,
            None,
            None,
        ]
    );
    assert_eq!(chain.neighbor1, vec![5, 4, 4, 6, 5, 6, NULL_NEIGHBOR]);
    assert_eq!(
        chain.neighbor2,
        vec![
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            1,
            0,
            3,
        ]
    );
    assert_eq!(
        chain.neighbor3,
        vec![
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            NULL_NEIGHBOR,
            2,
            4,
            5,
        ]
    );
    assert!(
        chain
            .edge_length1
            .iter()
            .take(6)
            .all(|length| (*length - 1.0).abs() < 1e-6)
    );

    reset_rand(1);
    let labels_file = std::env::temp_dir().join(format!(
        "muscle_rs_random_chain_labels_{}.txt",
        std::process::id()
    ));
    let tree_file = std::env::temp_dir().join(format!(
        "muscle_rs_random_chain_tree_{}.nwk",
        std::process::id()
    ));
    std::fs::write(&labels_file, b"A\nB\nC\nD\n").unwrap();
    let cmd_tree =
        cmd_labels2randomchaintree(labels_file.to_str().unwrap(), tree_file.to_str().unwrap());
    assert_eq!(cmd_tree.names, chain.names);
    assert_eq!(cmd_tree.neighbor1, chain.neighbor1);
    assert_eq!(cmd_tree.neighbor2, chain.neighbor2);
    assert_eq!(cmd_tree.neighbor3, chain.neighbor3);
    assert_eq!(
        std::fs::read_to_string(&tree_file).unwrap(),
        "(\nD:1\n,\n(\nB:1\n,\n(\nC:1\n,\nA:1\n):1\n):1\n)\n;\n"
    );

    reset_rand(1);
    let mut mpc = MPCFlat {
        labels,
        ..MPCFlat::default()
    };
    mpc_flat_calc_guide_tree_random_chain(&mut mpc);
    assert_eq!(mpc.guide_tree.names, chain.names);
    assert_eq!(mpc.guide_tree.neighbor1, chain.neighbor1);
    std::fs::remove_file(&labels_file).unwrap();
    std::fs::remove_file(&tree_file).unwrap();
}

#[test]
fn upgma5_label_lifecycle_matches_cpp_maps() {
    let mut upgma = UPGMA5::default();
    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let dist_mx = vec![
        vec![0.0, 1.0, 2.0],
        vec![1.0, 0.0, 3.0],
        vec![2.0, 3.0, 0.0],
    ];

    upgma5_init(&mut upgma, &labels, &dist_mx);
    assert_eq!(upgma.leaf_count, 3);
    assert_eq!(upgma.labels, labels);
    assert_eq!(upgma.dist_mx, dist_mx);
    assert_eq!(upgma5_get_label_index(&upgma, "a"), 0);
    assert_eq!(upgma5_get_label_index(&upgma, "c"), 2);
    let mut run_tree = Tree::default();
    upgma5_run_l75(&mut upgma, "avg", &mut run_tree);
    assert_eq!(run_tree.node_count, 5);
    assert_eq!(run_tree.root_node_index, 4);
    assert_eq!(run_tree.neighbor2[3], 0);
    assert_eq!(run_tree.neighbor3[3], 1);
    assert_eq!(run_tree.neighbor2[4], 2);
    assert_eq!(run_tree.neighbor3[4], 3);
    assert!((run_tree.edge_length2[4] - 1.25).abs() < 1e-6);
    assert!((run_tree.edge_length3[4] - 0.75).abs() < 1e-6);

    upgma5_add_label(&mut upgma, "b");
    assert_eq!(upgma.labels.len(), 3);
    upgma5_add_label(&mut upgma, "d");
    assert_eq!(upgma5_get_label_index(&upgma, "d"), 3);
    assert_eq!(upgma.labels[3], "d");

    let mut ea = UPGMA5::default();
    let ea_mx = vec![
        vec![0.0, 0.25, 0.75],
        vec![0.25, 0.0, 0.5],
        vec![0.75, 0.5, 0.0],
    ];
    upgma5_init(&mut ea, &labels, &ea_mx);
    upgma5_fix_ea_dist_mx(&mut ea);
    assert_eq!(ea.dist_mx[0][0], 0.0);
    assert_eq!(ea.dist_mx[1][1], 0.0);
    assert_eq!(ea.dist_mx[2][2], 0.0);
    assert_eq!(ea.dist_mx[1][0], 0.75);
    assert_eq!(ea.dist_mx[0][1], 0.75);
    assert_eq!(ea.dist_mx[2][0], 0.25);
    assert_eq!(ea.dist_mx[2][1], 0.5);

    let mut sim = UPGMA5::default();
    let sim_mx = vec![
        vec![0.0, 0.2, 0.8],
        vec![0.2, 0.0, 0.5],
        vec![0.8, 0.5, 0.0],
    ];
    upgma5_init(&mut sim, &labels, &sim_mx);
    upgma5_scale_dist_mx(&mut sim, true);
    assert!((sim.dist_mx[1][0] - 10.0).abs() < 1e-6);
    assert!((sim.dist_mx[2][0] - 0.0).abs() < 1e-6);
    assert!((sim.dist_mx[2][1] - 5.0).abs() < 1e-6);
    assert_eq!(sim.dist_mx[0][1], sim.dist_mx[1][0]);

    let mut dist = UPGMA5::default();
    upgma5_init(&mut dist, &labels, &sim_mx);
    upgma5_scale_dist_mx(&mut dist, false);
    assert!((dist.dist_mx[1][0] - 0.0).abs() < 1e-6);
    assert!((dist.dist_mx[2][0] - 10.0).abs() < 1e-6);
    assert!((dist.dist_mx[2][1] - 5.0).abs() < 1e-6);

    let dist_file =
        std::env::temp_dir().join(format!("muscle_rs_upgma5_dist_{}.tsv", std::process::id()));
    std::fs::write(&dist_file, "b\ta\t1.25\na\tc\t2.5\nb\tc\t3.75\n").unwrap();
    let mut from_file = UPGMA5::default();
    upgma5_read_dist_mx(&mut from_file, dist_file.to_str().unwrap());
    assert_eq!(from_file.labels, vec!["b", "a", "c"]);
    assert_eq!(upgma5_get_label_index(&from_file, "c"), 2);
    assert_eq!(from_file.dist_mx[0][1], 1.25);
    assert_eq!(from_file.dist_mx[1][0], 1.25);
    assert_eq!(from_file.dist_mx[1][2], 2.5);
    assert_eq!(from_file.dist_mx[0][0], f32::MAX);
    std::fs::remove_file(&dist_file).unwrap();

    let dist2_file =
        std::env::temp_dir().join(format!("muscle_rs_upgma5_dist2_{}.tsv", std::process::id()));
    std::fs::write(
        &dist2_file,
        "distmx\t3\n0\tA\n1\tB\n2\tC\n0\t1\t0.4\n0\t2\t0.8\n1\t2\t1.2\n",
    )
    .unwrap();
    let mut from_file2 = UPGMA5::default();
    upgma5_read_dist_mx2(&mut from_file2, dist2_file.to_str().unwrap());
    assert_eq!(from_file2.labels, vec!["A", "B", "C"]);
    assert_eq!(from_file2.leaf_count, 3);
    assert_eq!(from_file2.dist_mx[0][0], 0.0);
    assert_eq!(from_file2.dist_mx[1][0], 0.4);
    assert_eq!(from_file2.dist_mx[2][0], 0.8);
    assert_eq!(from_file2.dist_mx[2][1], 1.2);
    std::fs::remove_file(&dist2_file).unwrap();

    let cmd_dist_file =
        std::env::temp_dir().join(format!("muscle_rs_upgma5_cmd_{}.tsv", std::process::id()));
    let cmd_tree_file =
        std::env::temp_dir().join(format!("muscle_rs_upgma5_cmd_{}.nwk", std::process::id()));
    std::fs::write(&cmd_dist_file, "a\tb\t0.25\na\tc\t0.75\nb\tc\t0.5\n").unwrap();
    let mut cmd_tree = Tree::default();
    make_guide_tree_from_join_order(
        &[0, 3],
        &[1, 2],
        &std::collections::HashMap::from([
            ("a".to_string(), 0),
            ("b".to_string(), 1),
            ("c".to_string(), 2),
        ]),
        &mut cmd_tree,
    );
    let expected_cmd_tree = cmd_tree.clone();
    let (cmd_u, returned_tree, cmd_log) = cmd_upgma5(
        cmd_dist_file.to_str().unwrap(),
        cmd_tree_file.to_str().unwrap(),
        false,
        false,
        true,
        Some("max"),
        |u, linkage| {
            assert_eq!(linkage, "max");
            assert_eq!(u.labels, vec!["a", "b", "c"]);
            assert_eq!(u.dist_mx[1][0], 0.75);
            assert_eq!(u.dist_mx[2][0], 0.25);
            assert_eq!(u.dist_mx[2][1], 0.5);
            expected_cmd_tree.clone()
        },
    );
    assert_eq!(cmd_log, "UPGMA5(max)\nAll done.\n");
    assert_eq!(cmd_u.labels, vec!["a", "b", "c"]);
    assert_eq!(returned_tree.node_count, expected_cmd_tree.node_count);
    let mut expected_tree_file = TextFile::default();
    tree_to_file_l22(&expected_cmd_tree, &mut expected_tree_file);
    assert_eq!(
        std::fs::read_to_string(&cmd_tree_file).unwrap(),
        String::from_utf8(expected_tree_file.data).unwrap()
    );
    std::fs::remove_file(&cmd_dist_file).unwrap();
    std::fs::remove_file(&cmd_tree_file).unwrap();

    let cmd_msa_file =
        std::env::temp_dir().join(format!("muscle_rs_upgma5_msa_{}.fa", std::process::id()));
    let cmd_msa_tree_file =
        std::env::temp_dir().join(format!("muscle_rs_upgma5_msa_{}.nwk", std::process::id()));
    std::fs::write(&cmd_msa_file, b">a\nEFIL\n>b\nEFKL\n>c\nPQRS\n").unwrap();
    let mut msa_dist_calls = Vec::new();
    let expected_msa_tree = expected_cmd_tree.clone();
    let (cmd_msa_u, cmd_msa_tree, cmd_msa_log) = cmd_upgma5_msa(
        cmd_msa_file.to_str().unwrap(),
        cmd_msa_tree_file.to_str().unwrap(),
        Some("min"),
        |seqi, seqj, col_count| {
            msa_dist_calls.push((seqi.to_string(), seqj.to_string(), col_count));
            match (seqi, seqj) {
                ("EFKL", "EFIL") => 0.1,
                ("PQRS", "EFIL") => 0.8,
                ("PQRS", "EFKL") => 0.7,
                _ => panic!("unexpected UPGMA5 MSA pair {seqi} {seqj}"),
            }
        },
        |u, linkage| {
            assert_eq!(linkage, "min");
            assert_eq!(u.labels, vec!["a", "b", "c"]);
            assert_eq!(u.dist_mx[1][0], 0.1);
            assert_eq!(u.dist_mx[2][0], 0.8);
            assert_eq!(u.dist_mx[2][1], 0.7);
            expected_msa_tree.clone()
        },
    );
    assert_eq!(cmd_msa_log, "UPGMA5(min)\nAll done.\n");
    assert_eq!(
        msa_dist_calls,
        vec![
            ("EFKL".to_string(), "EFIL".to_string(), 4),
            ("PQRS".to_string(), "EFIL".to_string(), 4),
            ("PQRS".to_string(), "EFKL".to_string(), 4),
        ]
    );
    assert_eq!(cmd_msa_u.labels, vec!["a", "b", "c"]);
    assert_eq!(cmd_msa_tree.node_count, expected_msa_tree.node_count);
    let mut expected_msa_tree_file = TextFile::default();
    tree_to_file_l22(&expected_msa_tree, &mut expected_msa_tree_file);
    assert_eq!(
        std::fs::read_to_string(&cmd_msa_tree_file).unwrap(),
        String::from_utf8(expected_msa_tree_file.data).unwrap()
    );
    std::fs::remove_file(&cmd_msa_file).unwrap();
    std::fs::remove_file(&cmd_msa_tree_file).unwrap();

    assert_eq!(avg(2.0, 6.0), 4.0);
    let log_upgma = UPGMA5 {
        leaf_count: 3,
        internal_node_index: 0,
        dist: vec![1.0, 1234.0, 0.00125],
        min_dist: vec![1.0, 1.0, 2.0],
        nearest_neighbor: vec![1, 0, 0],
        node_index: vec![0, 1, 2],
        left: vec![0],
        right: vec![1],
        height: vec![0.00125],
        left_length: vec![1234.0],
        right_length: vec![0.5],
        labels: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        ..UPGMA5::default()
    };
    let log = upgma5_log_me(&log_upgma);
    assert!(log.starts_with("Dist matrix\n"));
    assert!(log.contains("  a\n"));
    assert!(log.contains("    1  1.2e+03"));
    assert!(log.contains("0.0012"));
    assert!(log.contains("    i   Node   NrNb      Dist\n"));
    assert!(log.contains("    0      0      1     1.000\n"));
    assert!(log.contains(" Node      L      R  Height  LLength  RLength\n"));
    assert!(log.contains("    0      0      1  0.0012  1.2e+03     0.5\n"));

    upgma5_clear(&mut upgma);
    assert!(upgma.labels.is_empty());
    assert!(upgma.dist_mx.is_empty());
    assert!(upgma.label_to_index.is_empty());

    let dup = vec!["x".to_string(), "x".to_string()];
    let dup_mx = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    assert!(
        std::panic::catch_unwind(|| {
            let mut u = UPGMA5::default();
            upgma5_init(&mut u, &dup, &dup_mx);
        })
        .is_err()
    );
}

#[test]
fn pprog_state_helpers_match_cpp_indexing() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let mut msa1 = MultiSequence::default();
    let mut msa2 = MultiSequence::default();
    let mut msa3 = MultiSequence::default();
    multi_sequence_from_strings(&mut msa1, &["a".to_string()], &["AA".to_string()]);
    multi_sequence_from_strings(&mut msa2, &["b".to_string()], &["CC".to_string()]);
    multi_sequence_from_strings(&mut msa3, &["c".to_string()], &["GG".to_string()]);
    let labels = vec!["msa1".to_string(), "msa2".to_string(), "msa3".to_string()];

    let mut pp = PProg::default();
    p_prog_set_ms_as(
        &mut pp,
        &[msa1.clone(), msa2.clone(), msa3.clone()],
        &labels,
    );
    assert_eq!(pp.input_msa_count, 3);
    assert_eq!(pp.msas.len(), 5);
    assert_eq!(pp.msa_labels.len(), 5);
    assert_eq!(pp.msa_label_to_index["msa2"], 1);
    assert_eq!(p_prog_get_msa_label(&pp, 0), "msa1");
    assert_eq!(p_prog_get_msa(&pp, 1).seqs[0].label, "b");

    let mut pprog3_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut pprog3_input,
        &["left".to_string(), "right".to_string()],
        &["AC".to_string(), "GT".to_string()],
    );
    let mut pprog3_tree = Tree::default();
    tree_create(
        &mut pprog3_tree,
        2,
        0,
        &[0],
        &[1],
        &[1.0],
        &[1.0],
        &[0, 1],
        &["left".to_string(), "right".to_string()],
    );
    let mut pp3 = PProg3 {
        input_seqs: Some(pprog3_input),
        input_seq_weights: vec![0.5, 0.5],
        guide_tree: Some(pprog3_tree),
        node_to_path: vec![String::new(), String::new(), "DMI".to_string()],
        ..PProg3::default()
    };
    assert_eq!(
        sequence_get_seq_as_string(&p_prog3_get_aligned_seq(&pp3, 0)),
        "AC-"
    );
    assert_eq!(
        sequence_get_seq_as_string(&p_prog3_get_aligned_seq(&pp3, 1)),
        "-GT"
    );
    p_prog3_build_msa(&mut pp3);
    assert_eq!(
        pp3.msa
            .seqs
            .iter()
            .map(|seq| (seq.label.as_str(), sequence_get_seq_as_string(seq)))
            .collect::<Vec<_>>(),
        vec![("left", "AC-".to_string()), ("right", "-GT".to_string())]
    );

    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut pp3_run = PProg3 {
        ap: Some(M3AlnParams {
            subst_mx_letter: [[0.0; 20]; 20],
            gap_open: -1.0,
            ready: true,
            ..M3AlnParams::default()
        }),
        ..PProg3::default()
    };
    let mut pprog3_nw_calls = Vec::new();
    p_prog3_run(
        &mut pp3_run,
        pp3.input_seqs.as_ref().unwrap(),
        &[0.4, 0.6],
        pp3.guide_tree.as_ref().unwrap(),
        |_cm, left, right| {
            pprog3_nw_calls.push((left.pps.len(), right.pps.len()));
            "DMI".to_string()
        },
        |_left, _left_weight, _right, _right_weight, _subst, _gap_open, _path| {
            panic!("root join should not build parent profile")
        },
    );
    assert_eq!(pprog3_nw_calls, vec![(2, 2)]);
    assert_eq!(
        pp3_run
            .msa
            .seqs
            .iter()
            .map(|seq| sequence_get_seq_as_string(seq))
            .collect::<Vec<_>>(),
        vec!["AC-".to_string(), "-GT".to_string()]
    );

    let load_dir =
        std::env::temp_dir().join(format!("muscle_rs_pprog_load_{}", std::process::id()));
    std::fs::create_dir_all(&load_dir).unwrap();
    let load_a = load_dir.join("alpha.fa");
    let load_b = load_dir.join("beta.msa");
    std::fs::write(&load_a, b">a1\nEFIL\n>a2\nPQRS\n").unwrap();
    std::fs::write(&load_b, b">b1\nVWHY\n>b2\nLMNP\n").unwrap();
    let file_names = vec![
        load_a.to_str().unwrap().to_string(),
        load_b.to_str().unwrap().to_string(),
    ];
    let mut pp_load = PProg::default();
    let load_is_nucleo = p_prog_load_ms_as(&mut pp_load, &file_names);
    assert!(!load_is_nucleo);
    assert_eq!(pp_load.input_msa_count, 2);
    assert_eq!(pp_load.join_count, 1);
    assert_eq!(pp_load.node_count, 3);
    assert_eq!(pp_load.msas.len(), 3);
    assert_eq!(pp_load.msa_labels, vec!["alpha", "beta", ""]);
    assert_eq!(pp_load.msa_label_to_index["alpha"], 0);
    assert_eq!(pp_load.msa_label_to_index["beta"], 1);
    assert_eq!(
        pp_load.msas[0]
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| (seq.label.as_str(), seq.char_vec.iter().collect::<String>()))
            .collect::<Vec<_>>(),
        vec![("a1", "EFIL".to_string()), ("a2", "PQRS".to_string())]
    );
    assert_eq!(
        pp_load.msas[1]
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["b1", "b2"]
    );
    assert_eq!(
        sequence_get_seq_as_string(&get_global_input_seq_by_label("a1")),
        "EFIL"
    );
    assert_eq!(
        sequence_get_seq_as_string(&get_global_input_seq_by_label("b2")),
        "LMNP"
    );
    std::fs::remove_file(&load_a).unwrap();
    std::fs::remove_file(&load_b).unwrap();
    std::fs::remove_dir(&load_dir).unwrap();

    let cmd_dir =
        std::env::temp_dir().join(format!("muscle_rs_eadistmx_msas_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_dir).unwrap();
    let cmd_a = cmd_dir.join("gamma.fa");
    let cmd_b = cmd_dir.join("delta.fa");
    let cmd_list = cmd_dir.join("list.txt");
    let cmd_out = cmd_dir.join("out.tsv");
    std::fs::write(&cmd_a, b">ga\nAA\n").unwrap();
    std::fs::write(&cmd_b, b">db\nCC\n").unwrap();
    std::fs::write(
        &cmd_list,
        format!("{}\n{}\n", cmd_a.display(), cmd_b.display()),
    )
    .unwrap();
    let mut cmd_calls = Vec::new();
    let pp_cmd = cmd_eadistmx_msas(
        cmd_list.to_str().unwrap(),
        cmd_out.to_str().unwrap(),
        Some(13),
        |label, msa1, msa2, pair_count, path| {
            cmd_calls.push((
                label.to_string(),
                msa1.seqs[0].label.clone(),
                msa2.seqs[0].label.clone(),
                pair_count,
            ));
            *path = "BB".to_string();
            0.625
        },
    );
    assert_eq!(
        cmd_calls,
        vec![(
            "gamma+delta".to_string(),
            "ga".to_string(),
            "db".to_string(),
            13
        )]
    );
    assert_eq!(pp_cmd.score_mx[0][1], 0.625);
    assert_eq!(
        std::fs::read_to_string(&cmd_out).unwrap(),
        "gamma\tdelta\t0.6250\n"
    );
    std::fs::remove_file(&cmd_a).unwrap();
    std::fs::remove_file(&cmd_b).unwrap();
    std::fs::remove_file(&cmd_list).unwrap();
    std::fs::remove_file(&cmd_out).unwrap();
    std::fs::remove_dir(&cmd_dir).unwrap();

    let pprog_dir =
        std::env::temp_dir().join(format!("muscle_rs_cmd_pprog_{}", std::process::id()));
    std::fs::create_dir_all(&pprog_dir).unwrap();
    let pprog_a = pprog_dir.join("left.fa");
    let pprog_b = pprog_dir.join("right.fa");
    let pprog_list = pprog_dir.join("list.txt");
    let pprog_out = pprog_dir.join("out.fa");
    let pprog_tree = pprog_dir.join("tree.nwk");
    std::fs::write(&pprog_a, b">la\nAA\n").unwrap();
    std::fs::write(&pprog_b, b">rb\nCC\n").unwrap();
    std::fs::write(
        &pprog_list,
        format!("{}\n{}\n", pprog_a.display(), pprog_b.display()),
    )
    .unwrap();
    let mut pprog_calls = Vec::new();
    let (pprog_cmd, guide_tree) = cmd_pprog(
        pprog_list.to_str().unwrap(),
        pprog_out.to_str().unwrap(),
        Some(pprog_tree.to_str().unwrap()),
        Some(17),
        |label, msa1, msa2, pair_count, path| {
            pprog_calls.push((
                label.to_string(),
                msa1.seqs[0].label.clone(),
                msa2.seqs[0].label.clone(),
                pair_count,
            ));
            *path = "BB".to_string();
            1.0
        },
    );
    assert_eq!(
        pprog_calls,
        vec![(
            "left+right".to_string(),
            "la".to_string(),
            "rb".to_string(),
            17
        )]
    );
    assert_eq!(pprog_cmd.pending, vec![2]);
    assert_eq!(guide_tree.unwrap().node_count, 3);
    assert_eq!(
        std::fs::read_to_string(&pprog_out).unwrap(),
        ">la\nAA\n>rb\nCC\n"
    );
    assert!(
        std::fs::read_to_string(&pprog_tree)
            .unwrap()
            .contains("left")
    );
    std::fs::remove_file(&pprog_a).unwrap();
    std::fs::remove_file(&pprog_b).unwrap();
    std::fs::remove_file(&pprog_list).unwrap();
    std::fs::remove_file(&pprog_out).unwrap();
    std::fs::remove_file(&pprog_tree).unwrap();
    std::fs::remove_dir(&pprog_dir).unwrap();

    let pprog2_dir =
        std::env::temp_dir().join(format!("muscle_rs_cmd_pprog2_{}", std::process::id()));
    std::fs::create_dir_all(&pprog2_dir).unwrap();
    let pprog2_a = pprog2_dir.join("left2.fa");
    let pprog2_b = pprog2_dir.join("right2.fa");
    let pprog2_list = pprog2_dir.join("list.txt");
    let pprog2_joins = pprog2_dir.join("joins.tsv");
    let pprog2_out = pprog2_dir.join("out.fa");
    std::fs::write(&pprog2_a, b">l2\nAA\n").unwrap();
    std::fs::write(&pprog2_b, b">r2\nCC\n").unwrap();
    std::fs::write(
        &pprog2_list,
        format!("{}\n{}\n", pprog2_a.display(), pprog2_b.display()),
    )
    .unwrap();
    std::fs::write(&pprog2_joins, b"0\t1\n").unwrap();
    let mut pprog2_calls = Vec::new();
    let pprog2_cmd = cmd_pprog2(
        pprog2_list.to_str().unwrap(),
        pprog2_joins.to_str().unwrap(),
        pprog2_out.to_str().unwrap(),
        Some(19),
        |label, msa1, msa2, pair_count, path| {
            pprog2_calls.push((
                label.to_string(),
                msa1.seqs[0].label.clone(),
                msa2.seqs[0].label.clone(),
                pair_count,
            ));
            *path = "BB".to_string();
            1.0
        },
    );
    assert_eq!(
        pprog2_calls,
        vec![(
            "Join 1 / 1".to_string(),
            "l2".to_string(),
            "r2".to_string(),
            19
        )]
    );
    assert_eq!(pprog2_cmd.join_index, 1);
    assert_eq!(
        std::fs::read_to_string(&pprog2_out).unwrap(),
        ">l2\nAA\n>r2\nCC\n"
    );
    std::fs::remove_file(&pprog2_a).unwrap();
    std::fs::remove_file(&pprog2_b).unwrap();
    std::fs::remove_file(&pprog2_list).unwrap();
    std::fs::remove_file(&pprog2_joins).unwrap();
    std::fs::remove_file(&pprog2_out).unwrap();
    std::fs::remove_dir(&pprog2_dir).unwrap();

    p_prog_set_msa_label(&mut pp, 3, "join1");
    assert_eq!(p_prog_get_msa_label(&pp, 3), "join1");
    assert_eq!(pp.msa_label_to_index["join1"], 3);
    p_prog_set_msa(&mut pp, 3, &msa1);
    assert_eq!(p_prog_get_msa(&pp, 3).seqs[0].label, "a");
    p_prog_set_msa(&mut pp, 4, &msa2);
    assert_eq!(p_prog_get_final_msa(&pp).seqs[0].label, "b");

    pp.node_count = 5;
    pp.pending = vec![0, 1, 2, 3];
    pp.score_mx = vec![vec![0.0; 5]; 5];
    pp.score_mx[0][1] = 0.1;
    pp.score_mx[0][2] = 0.7;
    pp.score_mx[1][3] = 1.5;
    pp.score_mx[2][3] = 1.4;
    assert_eq!(p_prog_find_best_pair(&pp), (1, 3));
    let pending_log = p_prog_log_pending(&pp, "before");
    assert!(pending_log.starts_with("\nLogPending(before) m_JoinIndex=0\n"));
    assert!(pending_log.contains("  [   0]  seqs=1,cols=2 msa1\n"));
    assert!(pending_log.contains("  [   3]  seqs=1,cols=2 join1\n"));

    p_prog_delete_indexes_from_pending(&mut pp, 1, 3);
    assert_eq!(pp.pending, vec![0, 2]);

    let mut run2_calls = Vec::new();
    p_prog_run2(&mut pp, &[0, 2], &[1, 3], |pp, index1, index2| {
        run2_calls.push((pp.join_index, index1, index2));
    });
    assert_eq!(pp.join_count, 2);
    assert_eq!(pp.node_count, 5);
    assert_eq!(pp.join_index, 2);
    assert_eq!(run2_calls, vec![(0, 0, 1), (1, 2, 3)]);

    let mut pp_join = PProg::default();
    p_prog_set_ms_as(
        &mut pp_join,
        &[msa1.clone(), msa2.clone(), msa3.clone()],
        &labels,
    );
    pp_join.target_pair_count = 7;
    pp_join.join_count = 2;
    pp_join.node_count = 5;
    let mut flat_calls = Vec::new();
    p_prog_align_all_input_pairs(&mut pp_join, |label, mfa1, mfa2, pair_count, path| {
        flat_calls.push((
            label.to_string(),
            mfa1.seqs[0].label.clone(),
            mfa2.seqs[0].label.clone(),
            pair_count,
        ));
        *path = "BB".to_string();
        label.len() as f32
    });
    assert_eq!(
        flat_calls,
        vec![
            ("msa1+msa2".to_string(), "a".to_string(), "b".to_string(), 7),
            ("msa1+msa3".to_string(), "a".to_string(), "c".to_string(), 7),
            ("msa2+msa3".to_string(), "b".to_string(), "c".to_string(), 7),
        ]
    );
    assert_eq!(pp_join.path_mx[0][1], "BB");
    assert_eq!(pp_join.path_mx[1][0], "BB");
    assert_eq!(pp_join.score_mx[0][2], "msa1+msa3".len() as f32);

    pp_join.pending = vec![0, 1, 2];
    pp_join.join_index = 0;
    p_prog_join_by_precomputed_path(&mut pp_join, 0, 1);
    assert_eq!(pp_join.join_msa_indexes1, vec![0]);
    assert_eq!(pp_join.join_msa_indexes2, vec![1]);
    assert_eq!(p_prog_get_msa_label(&pp_join, 3), "Join1");
    assert_eq!(pp_join.pending, vec![2, 3]);
    assert_eq!(
        pp_join.msas[3]
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );

    pp_join.join_index = 1;
    let mut new_pending_calls = Vec::new();
    p_prog_align_new_to_pending(&mut pp_join, |label, mfa1, mfa2, pair_count, path| {
        new_pending_calls.push((
            label.to_string(),
            mfa1.seqs[0].label.clone(),
            mfa2.seqs[0].label.clone(),
            pair_count,
        ));
        *path = "BB".to_string();
        4.25
    });
    assert_eq!(
        new_pending_calls,
        vec![(
            "Join1+msa1".to_string(),
            "a".to_string(),
            "c".to_string(),
            7
        )]
    );
    assert_eq!(pp_join.score_mx[3][2], 4.25);
    assert_eq!(pp_join.score_mx[2][3], 4.25);
    assert_eq!(pp_join.path_mx[3][2], "BB");
    assert_eq!(pp_join.path_mx[2][3], "BB");

    let mut pp_run = PProg::default();
    p_prog_set_ms_as(&mut pp_run, &[msa1, msa2, msa3], &labels);
    pp_run.target_pair_count = 9;
    let mut run_labels = Vec::new();
    p_prog_run(&mut pp_run, |label, _mfa1, _mfa2, pair_count, path| {
        run_labels.push((label.to_string(), pair_count));
        *path = "BB".to_string();
        match label {
            "msa1+msa2" => 10.0,
            "msa1+msa3" => 2.0,
            "msa2+msa3" => 3.0,
            "Join1+msa1" => 5.0,
            _ => 1.0,
        }
    });
    assert_eq!(
        run_labels,
        vec![
            ("msa1+msa2".to_string(), 9),
            ("msa1+msa3".to_string(), 9),
            ("msa2+msa3".to_string(), 9),
            ("Join1+msa1".to_string(), 9),
        ]
    );
    assert_eq!(pp_run.join_msa_indexes1, vec![0, 2]);
    assert_eq!(pp_run.join_msa_indexes2, vec![1, 3]);
    assert_eq!(pp_run.pending, vec![4]);
    assert_eq!(p_prog_get_msa_label(&pp_run, 4), "Join2");
    assert_eq!(
        p_prog_get_final_msa(&pp_run)
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["c", "a", "b"]
    );

    let mut pp_align = PProg::default();
    let mut msa_a = MultiSequence::default();
    let mut msa_b = MultiSequence::default();
    let mut msa_c = MultiSequence::default();
    multi_sequence_from_strings(&mut msa_a, &["aa".to_string()], &["AA".to_string()]);
    multi_sequence_from_strings(&mut msa_b, &["bb".to_string()], &["CC".to_string()]);
    multi_sequence_from_strings(&mut msa_c, &["cc".to_string()], &["GG".to_string()]);
    p_prog_set_ms_as(
        &mut pp_align,
        &[msa_a.clone(), msa_b, msa_c.clone()],
        &labels,
    );
    p_prog_set_msa(&mut pp_align, 3, &msa_a);
    pp_align.input_msa_count = 3;
    pp_align.join_count = 2;
    pp_align.join_index = 1;
    pp_align.target_pair_count = 11;
    let mut align_join_calls = Vec::new();
    p_prog_align_and_join(
        &mut pp_align,
        3,
        2,
        |label, mfa1, mfa2, pair_count, path| {
            align_join_calls.push((
                label.to_string(),
                mfa1.seqs[0].label.clone(),
                mfa2.seqs[0].label.clone(),
                pair_count,
            ));
            *path = "BB".to_string();
            9.0
        },
    );
    assert_eq!(
        align_join_calls,
        vec![(
            "Join 2 / 2".to_string(),
            "aa".to_string(),
            "cc".to_string(),
            11
        )]
    );
    assert!(pp_align.msas[3].is_none());
    assert_eq!(
        pp_align.msas[4]
            .as_ref()
            .unwrap()
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["aa", "cc"]
    );
    assert_eq!(pp_align.join_msa_indexes1, vec![3]);
    assert_eq!(pp_align.join_msa_indexes2, vec![2]);

    let mut pp_tree = PProg::default();
    p_prog_set_ms_as(&mut pp_tree, &[msa_a.clone(), msa_c.clone()], &labels[..2]);
    pp_tree.target_pair_count = 5;
    let mut simple_label_to_index = std::collections::HashMap::new();
    simple_label_to_index.insert("msa1".to_string(), 0);
    simple_label_to_index.insert("msa2".to_string(), 1);
    let mut simple_tree = Tree::default();
    make_guide_tree_from_join_order(&[0], &[1], &simple_label_to_index, &mut simple_tree);
    let mut run_guide_calls = Vec::new();
    p_prog_run_guide_tree(&mut pp_tree, &simple_tree, |pp, index1, index2| {
        run_guide_calls.push((index1, index2));
        p_prog_align_and_join(
            pp,
            index1,
            index2,
            |_label, _msa1, _msa2, _pair_count, path| {
                *path = "BB".to_string();
                1.0
            },
        );
    });
    assert_eq!(run_guide_calls, vec![(0, 1)]);
    assert_eq!(pp_tree.join_msa_indexes1, vec![0]);
    assert_eq!(pp_tree.join_msa_indexes2, vec![1]);
    assert_eq!(
        p_prog_get_final_msa(&pp_tree)
            .seqs
            .iter()
            .map(|seq| seq.label.as_str())
            .collect::<Vec<_>>(),
        vec!["aa", "cc"]
    );

    let cmd_tree_dir =
        std::env::temp_dir().join(format!("muscle_rs_pprog_tree_cmd_{}", std::process::id()));
    std::fs::create_dir_all(&cmd_tree_dir).unwrap();
    let cmd_tree_input = cmd_tree_dir.join("input.fa");
    let cmd_tree_tree = cmd_tree_dir.join("tree.nwk");
    let cmd_tree_out = cmd_tree_dir.join("out.fa");
    std::fs::write(&cmd_tree_input, b">aa\nA-A\n>cc\nGG-\n").unwrap();
    let mut cmd_tree_label_to_index = std::collections::HashMap::new();
    cmd_tree_label_to_index.insert("aa".to_string(), 0);
    cmd_tree_label_to_index.insert("cc".to_string(), 1);
    let mut cmd_tree_input_tree = Tree::default();
    make_guide_tree_from_join_order(
        &[0],
        &[1],
        &cmd_tree_label_to_index,
        &mut cmd_tree_input_tree,
    );
    tree_to_file_l13(&cmd_tree_input_tree, cmd_tree_tree.to_str().unwrap());
    let mut cmd_tree_calls = Vec::new();
    let cmd_tree_pp = cmd_pprog_tree(
        cmd_tree_input.to_str().unwrap(),
        cmd_tree_tree.to_str().unwrap(),
        cmd_tree_out.to_str().unwrap(),
        Some(6),
        |label, msa1, msa2, pair_count, path| {
            cmd_tree_calls.push((
                label.to_string(),
                msa1.seqs[0].label.clone(),
                msa2.seqs[0].label.clone(),
                pair_count,
            ));
            *path = "BB".to_string();
            1.0
        },
    );
    assert_eq!(
        cmd_tree_calls,
        vec![(
            "Join 1 / 1".to_string(),
            "aa".to_string(),
            "cc".to_string(),
            6
        )]
    );
    assert_eq!(cmd_tree_pp.target_pair_count, 6);
    assert_eq!(
        std::fs::read_to_string(&cmd_tree_out).unwrap(),
        ">aa\nAA\n>cc\nGG\n"
    );

    let cmd_t_list = cmd_tree_dir.join("list.txt");
    let cmd_t_a = cmd_tree_dir.join("alpha.fa");
    let cmd_t_b = cmd_tree_dir.join("beta.fa");
    let cmd_t_out = cmd_tree_dir.join("out_t.fa");
    let cmd_t_guide_out = cmd_tree_dir.join("guide_out.nwk");
    std::fs::write(&cmd_t_a, b">a\nAA\n").unwrap();
    std::fs::write(&cmd_t_b, b">b\nGG\n").unwrap();
    std::fs::write(
        &cmd_t_list,
        format!("{}\n{}\n", cmd_t_a.display(), cmd_t_b.display()),
    )
    .unwrap();
    let mut cmd_t_label_to_index = std::collections::HashMap::new();
    cmd_t_label_to_index.insert("alpha".to_string(), 0);
    cmd_t_label_to_index.insert("beta".to_string(), 1);
    let mut cmd_t_tree = Tree::default();
    make_guide_tree_from_join_order(&[0], &[1], &cmd_t_label_to_index, &mut cmd_t_tree);
    tree_to_file_l13(&cmd_t_tree, cmd_tree_tree.to_str().unwrap());
    let (cmd_t_pp, cmd_t_written_tree) = cmd_pprogt(
        cmd_t_list.to_str().unwrap(),
        cmd_tree_tree.to_str().unwrap(),
        cmd_t_out.to_str().unwrap(),
        Some(cmd_t_guide_out.to_str().unwrap()),
        Some(8),
        |_label, _msa1, _msa2, pair_count, path| {
            assert_eq!(pair_count, 8);
            *path = "BB".to_string();
            1.0
        },
    );
    assert_eq!(cmd_t_pp.join_msa_indexes1, vec![0]);
    assert_eq!(cmd_t_pp.join_msa_indexes2, vec![1]);
    assert!(cmd_t_written_tree.is_some());
    assert_eq!(
        std::fs::read_to_string(&cmd_t_out).unwrap(),
        ">a\nAA\n>b\nGG\n"
    );
    assert!(
        std::fs::read_to_string(&cmd_t_guide_out)
            .unwrap()
            .contains("alpha")
    );
    std::fs::remove_dir_all(&cmd_tree_dir).unwrap();

    pp.join_msa_indexes1 = vec![0, 3];
    pp.join_msa_indexes2 = vec![1, 2];
    let guide_file =
        std::env::temp_dir().join(format!("muscle_rs_pprog_guide_{}.nwk", std::process::id()));
    let guide = p_prog_write_guide_tree(&pp, guide_file.to_str().unwrap()).unwrap();
    assert_eq!(guide.node_count, 5);
    assert_eq!(guide.root_node_index, 4);
    assert_eq!(guide.names[0].as_deref(), Some("msa1"));
    assert_eq!(guide.names[1].as_deref(), Some("msa2"));
    assert_eq!(guide.names[2].as_deref(), Some("msa3"));
    assert!(
        std::fs::read_to_string(&guide_file)
            .unwrap()
            .contains("msa1")
    );
    std::fs::remove_file(&guide_file).unwrap();
    assert!(p_prog_write_guide_tree(&pp, "").is_none());
}

#[test]
fn usorter_word_helpers_match_alpha_tables() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut sorter = USorter::default();
    u_sorter_init(&mut sorter);
    assert_eq!(sorter.word_length, 3);
    assert_eq!(sorter.dict_size, 8000);
    assert_eq!(sorter.rows.len(), 8000);
    assert_eq!(
        u_sorter_chars_to_word_amino(&sorter, b"ACD"),
        0 * 20 * 20 + 1 * 20 + 2
    );
    assert_eq!(
        u_sorter_chars_to_word(&sorter, b"acd"),
        0 * 20 * 20 + 1 * 20 + 2
    );
    assert_eq!(u_sorter_chars_to_word_amino(&sorter, b"AXD"), uint::MAX);

    set_alpha_l209(ALPHA::ALPHA_Nucleo);
    let mut sorter = USorter::default();
    u_sorter_init(&mut sorter);
    assert_eq!(sorter.word_length, 8);
    assert_eq!(sorter.dict_size, 65_536);
    assert_eq!(
        u_sorter_chars_to_word_nucleo(&sorter, b"ACGTACGT"),
        0 * 4u32.pow(7)
            + 1 * 4u32.pow(6)
            + 2 * 4u32.pow(5)
            + 3 * 4u32.pow(4)
            + 0 * 4u32.pow(3)
            + 1 * 4u32.pow(2)
            + 2 * 4
            + 3
    );
    assert_eq!(u_sorter_chars_to_word(&sorter, b"ACNTACGT"), uint::MAX);
}

#[test]
fn usorter_index_and_search_match_cpp_counts() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut sorter = USorter::default();
    u_sorter_init(&mut sorter);
    u_sorter_add_seq(&mut sorter, b"ACDEFGH", 10);
    u_sorter_add_seq(&mut sorter, b"ACDACD", 20);
    u_sorter_add_seq(&mut sorter, b"XXXX", 30);
    assert_eq!(sorter.index_seq_indexes, vec![10, 20, 30]);
    assert_eq!(
        sorter.rows[u_sorter_chars_to_word(&sorter, b"ACD") as usize],
        vec![0, 1, 1]
    );

    let (seq_indexes, word_counts) = u_sorter_search_seq(&sorter, b"ACDE");
    assert_eq!(seq_indexes, vec![10, 20]);
    assert_eq!(word_counts, vec![2, 2]);

    let (seq_indexes, word_counts) = u_sorter_search_seq(&sorter, b"ZZZ");
    assert!(seq_indexes.is_empty());
    assert!(word_counts.is_empty());

    let query_file =
        std::env::temp_dir().join(format!("muscle_rs_usorter_query_{}.fa", std::process::id()));
    let db_file =
        std::env::temp_dir().join(format!("muscle_rs_usorter_db_{}.fa", std::process::id()));
    std::fs::write(&query_file, b">q1\nACDE\n>q2\nZZZ\n").unwrap();
    std::fs::write(&db_file, b">d1\nACDEFGH\n>d2\nACDACD\n>d3\nXXXX\n").unwrap();
    assert_eq!(
        cmd_usorter(query_file.to_str().unwrap(), db_file.to_str().unwrap()),
        "\nQ>q1, 2 hits\n  [   2]  d1\n  [   2]  d2\n\nQ>q2, 0 hits\n"
    );
    std::fs::remove_file(&query_file).unwrap();
    std::fs::remove_file(&db_file).unwrap();
}

#[test]
fn kmer33_helpers_match_amino_mapping_and_counts() {
    assert_eq!(kmer_dist33_seq_to_kmer(b"ACD"), 0 + 1 * 20 + 2 * 20 * 20);
    assert_eq!(kmer_dist33_seq_to_kmer(b"acd"), 0 + 1 * 20 + 2 * 20 * 20);
    assert!(kmer_dist33_seq_to_kmer(b"ACX") as usize >= DICT_SIZE_33);

    let counts = kmer_dist33_count_kmers(b"ACDEFGH");
    assert_eq!(counts[kmer_dist33_seq_to_kmer(b"ACD") as usize], 1);
    assert_eq!(counts[kmer_dist33_seq_to_kmer(b"CDE") as usize], 1);
    assert_eq!(counts.iter().map(|x| uint::from(*x)).sum::<uint>(), 2);

    let counts2 = kmer_dist33_count_kmers(b"ACDACDH");
    assert_eq!(kmer_dist33_get_common_kmer_count(&counts, &counts2), 1);

    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let seqs = vec![
        "ACDEFGH".to_string(),
        "ACDACDH".to_string(),
        "ACDEFGH".to_string(),
    ];
    let mut ms = MultiSequence::default();
    multi_sequence_from_strings(&mut ms, &labels, &seqs);
    let dist_mx = kmer_dist33_get_dist_mx(&ms);
    assert_eq!(dist_mx[0][0], 0.0);
    assert_eq!(dist_mx[0][2], 0.0);
    assert!((dist_mx[0][1] - 1.5).abs() < 1e-6);
    assert_eq!(dist_mx[1][0], dist_mx[0][1]);
}

#[test]
fn kmer66_helpers_match_six_group_mapping_and_counts() {
    assert_eq!(
        kmer_dist66_seq_to_kmer(b"ACDEFG"),
        5 * 6u32.pow(4) + 2 * 6u32.pow(3) + 2 * 6u32.pow(2) + 4 * 6u32
    );
    assert_eq!(
        kmer_dist66_seq_to_kmer(b"acdefg"),
        5 * 6u32.pow(4) + 2 * 6u32.pow(3) + 2 * 6u32.pow(2) + 4 * 6u32
    );
    assert_eq!(
        kmer_dist66_seq_to_kmer(b"ILMVRH"),
        6u32.pow(5) + 6u32.pow(4) + 6u32.pow(3) + 6u32.pow(2) + 3 * 6u32 + 3
    );

    let counts = kmer_dist66_count_kmers(b"ACDEFGH");
    assert_eq!(counts[kmer_dist66_seq_to_kmer(b"ACDEFG") as usize], 1);
    assert_eq!(counts[kmer_dist66_seq_to_kmer(b"CDEFGH") as usize], 1);
    assert_eq!(counts.iter().map(|x| uint::from(*x)).sum::<uint>(), 2);

    let counts2 = kmer_dist66_count_kmers(b"ACDEFGACDEFG");
    assert_eq!(counts2[kmer_dist66_seq_to_kmer(b"ACDEFG") as usize], 2);
    assert_eq!(kmer_dist66_get_common_kmer_count(&counts, &counts2), 1);

    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let seqs = vec![
        "ACDEFGH".to_string(),
        "ACDEFGACDEFG".to_string(),
        "ACDEFGH".to_string(),
    ];
    let mut ms = MultiSequence::default();
    multi_sequence_from_strings(&mut ms, &labels, &seqs);
    let dist_mx = kmer_dist66_get_dist_mx(&ms);
    assert_eq!(dist_mx[0][0], 0.0);
    assert_eq!(dist_mx[0][2], 0.0);
    assert!((dist_mx[0][1] - 1.5).abs() < 1e-6);
    assert_eq!(dist_mx[1][0], dist_mx[0][1]);
}

#[test]
fn colscore_bin_matches_cpp_boundaries() {
    assert_eq!(get_bin(0.05), 0);
    assert_eq!(get_bin(0.1), 1);
    assert_eq!(get_bin(0.999), 9);
    assert_eq!(get_bin(1.0), 10);
}

#[test]
fn confidence_chars_and_bit_traceback_match_cpp_tables() {
    assert_eq!(conf_to_char1(0.0), '0');
    assert_eq!(conf_to_char1(0.999), '9');
    assert_eq!(conf_to_char1(1.0), '+');
    assert_eq!(conf_to_char2(0.987), '8');
    assert_eq!(conf_to_char2(1.0), '+');
    assert_eq!(conf_to_char_1(0.0), '_');
    assert_eq!(conf_to_char_1(0.3), '.');
    assert_eq!(conf_to_char_1(1.0), '^');
    assert_eq!(do1(&[0.0, 0.34, 0.99, 1.0], "_conf_", 1), ">_conf_\n039+\n");
    assert_eq!(
        do1(&[0.0, 0.34, 0.99, 1.0], "_conf_2", 2),
        ">_conf_2\n049+\n"
    );
    assert_eq!(
        do1(&[0.0, 0.34, 0.99, 1.0], "_conf_", -1),
        ">_conf_\n_.*^\n"
    );

    assert_eq!(x_char(BIT_DM, b'M'), b'D');
    assert_eq!(x_char(BIT_MD, b'D'), b'M');
    assert_eq!(x_char(BIT_MI, b'I'), b'M');

    let mut trace_back = vec![vec![0; 3]; 3];
    trace_back[2][2] = BIT_MI;
    trace_back[2][1] = BIT_DM;
    assert_eq!(bit_trace_back(&trace_back, 2, 2, b'I'), "DMI");

    let mut tb = vec![vec![0xff; 2]; 2];
    set_bit_tbm(&mut tb, 1, 1, b'D');
    assert_eq!(tb[1][1] & BIT_xM, BIT_DM);
    assert_eq!(tb[1][1] & BIT_xD, BIT_MD);
    assert_eq!(tb[1][1] & BIT_xI, BIT_MI);
    set_bit_tbm(&mut tb, 1, 1, b'I');
    assert_eq!(tb[1][1] & BIT_xM, BIT_IM);
    set_bit_tbd(&mut tb, 1, 1, b'D');
    assert_eq!(tb[1][1] & BIT_xD, BIT_DD);
    assert_eq!(tb[1][1] & BIT_xM, BIT_IM);
    set_bit_tbi(&mut tb, 1, 1, b'I');
    assert_eq!(tb[1][1] & BIT_xI, BIT_II);
    assert_eq!(tb[1][1] & BIT_xM, BIT_IM);

    let smx = Mx {
        name: "toy".to_string(),
        row_count: 3,
        col_count: 3,
        data: vec![
            vec![-2.0, -2.0, -2.0],
            vec![-2.0, 5.0, -2.0],
            vec![-2.0, -2.0, -2.0],
        ],
    };
    let mut mem = XDPMem::default();
    let (sw_score, sw_lo_i, sw_lo_j, sw_len_i, sw_len_j, sw_path) =
        sw_fast_s_mx(&mut mem, &smx, -3.0, -1.0);
    assert_eq!(sw_score, 5.0);
    assert_eq!((sw_lo_i, sw_lo_j, sw_len_i, sw_len_j), (1, 1, 1, 1));
    assert_eq!(sw_path, "M");
    assert_eq!(mem.tb_bit[1][1] & 0x10, 0x10);

    let mut mem = XDPMem::default();
    let (
        fast_str_score,
        fast_str_lo_i,
        fast_str_lo_j,
        fast_str_len_i,
        fast_str_len_j,
        fast_str_path,
    ) = sw_fast_strings_blosum62(&mut mem, "CAC", "DAD", -3.0, -1.0);
    assert_eq!(fast_str_score, get_blosum_score_chars(b'A', b'A'));
    assert_eq!(
        (fast_str_lo_i, fast_str_lo_j, fast_str_len_i, fast_str_len_j),
        (1, 1, 1, 1)
    );
    assert_eq!(fast_str_path, "M");

    let mut seq_a = Sequence::default();
    let mut seq_b = Sequence::default();
    sequence_from_string(&mut seq_a, "a", "CAC");
    sequence_from_string(&mut seq_b, "b", "DAD");
    let mut mem = XDPMem::default();
    let (
        fast_seq_score,
        fast_seq_lo_i,
        fast_seq_lo_j,
        fast_seq_len_i,
        fast_seq_len_j,
        fast_seq_path,
    ) = sw_fast_seqs_blosum62(&mut mem, &seq_a, &seq_b, -3.0, -1.0);
    assert_eq!(fast_seq_score, fast_str_score);
    assert_eq!(
        (fast_seq_lo_i, fast_seq_lo_j, fast_seq_len_i, fast_seq_len_j),
        (1, 1, 1, 1)
    );
    assert_eq!(fast_seq_path, "M");

    let mut mem = XDPMem::default();
    let mut pi = PathInfo::default();
    let vscore = viterbi_fast_mem(&mut mem, b"AC", 2, b"AC", 2, &mut pi);
    assert_eq!(pi.path, "MM");
    assert!(
        (vscore - get_blosum_score_chars(b'A', b'A') - get_blosum_score_chars(b'C', b'C')).abs()
            < 1e-6
    );
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.feature_count = 1;
        mega.weights = vec![1.0];
        mega.log_odds_mx_vec = vec![vec![vec![4.0, -2.0], vec![-2.0, 5.0]]];
    }
    {
        let mut gaps = VITERBI_MEGA_GAPS.lock().unwrap();
        gaps.l_open_a = 0.0;
        gaps.l_open_b = 0.0;
        gaps.l_ext_a = -0.5;
        gaps.l_ext_b = -0.5;
        gaps.r_open_a = 0.0;
        gaps.r_open_b = 0.0;
        gaps.r_ext_a = -0.5;
        gaps.r_ext_b = -0.5;
        gaps.open_a = -3.0;
        gaps.open_b = -3.0;
        gaps.ext_a = -0.5;
        gaps.ext_b = -0.5;
    }
    let mut mem = XDPMem::default();
    let mut pi = PathInfo::default();
    let mega_score = viterbi_mega(&mut mem, &[vec![0], vec![1]], &[vec![0], vec![1]], &mut pi);
    assert_eq!(pi.path, "MM");
    assert!((mega_score - 9.0).abs() < 1e-6);
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.labels = vec!["mega-a".to_string(), "mega-b".to_string()];
        mega.seqs = vec!["AC".to_string(), "AC".to_string()];
        mega.profiles = vec![vec![vec![0], vec![1]], vec![vec![0], vec![1]]];
    }
    let mega2_out =
        std::env::temp_dir().join(format!("muscle_rs_align_mega2_{}.fa", std::process::id()));
    let (mega2_score, mega2_pi, mega2_text) = align_mega2(
        mega2_out.to_str().unwrap(),
        Some(3.0),
        Some(0.5),
        Some(0.0),
        Some(0.5),
    );
    assert_eq!(mega2_pi.path, "MM");
    assert!((mega2_score - 9.0).abs() < 1e-6);
    assert_eq!(mega2_text.as_deref(), Some(">mega-a\nAC\n>mega-b\nAC\n"));
    assert_eq!(
        std::fs::read_to_string(&mega2_out).unwrap(),
        ">mega-a\nAC\n>mega-b\nAC\n"
    );
    std::fs::remove_file(&mega2_out).unwrap();

    let sw_cmd_in =
        std::env::temp_dir().join(format!("muscle_rs_cmd_sw_{}.fa", std::process::id()));
    std::fs::write(&sw_cmd_in, b">a\nCAC\n>b\nDAD\n").unwrap();
    let sw_cmd_log = cmd_sw(sw_cmd_in.to_str().unwrap());
    assert!(sw_cmd_log.contains("a"));
    assert!(sw_cmd_log.contains("b"));
    assert!(sw_cmd_log.ends_with("a b 1.96\n"));
    std::fs::remove_file(&sw_cmd_in).unwrap();

    let swdist_in =
        std::env::temp_dir().join(format!("muscle_rs_cmd_swdistmx_{}.fa", std::process::id()));
    let swdist_tree =
        std::env::temp_dir().join(format!("muscle_rs_cmd_swdistmx_{}.nwk", std::process::id()));
    std::fs::write(
        &swdist_in,
        b">a\nACDEFGHIKLMNPQRSTVWY\n>b\nACDEFGHIKLMNPQRSAVWY\n>c\nGGGGGGGGGGGGGGGGGGGG\n",
    )
    .unwrap();
    let (swdist_t, swdist_mx) =
        cmd_swdistmx(swdist_in.to_str().unwrap(), swdist_tree.to_str().unwrap());
    assert_eq!((swdist_t.node_count + 1) / 2, 3);
    assert_eq!(swdist_mx.len(), 3);
    assert_eq!(swdist_mx[0][1], swdist_mx[1][0]);
    assert!(swdist_mx[0][1].is_finite());
    assert!(std::fs::read_to_string(&swdist_tree).unwrap().contains(';'));
    std::fs::remove_file(&swdist_in).unwrap();
    std::fs::remove_file(&swdist_tree).unwrap();

    let mut rdrp_all = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut rdrp_all, "muscle/test_data/rdrp/rdrp.fa", true);
    let mut rdrp_subset = MultiSequence::default();
    for seq_index in 0..3 {
        rdrp_subset.seqs.push(rdrp_all.seqs[seq_index].clone());
        rdrp_subset.owners.push(true);
    }
    let rdrp_subset_in =
        std::env::temp_dir().join(format!("muscle_rs_rdrp_swdistmx_{}.fa", std::process::id()));
    let rdrp_subset_tree = std::env::temp_dir().join(format!(
        "muscle_rs_rdrp_swdistmx_{}.nwk",
        std::process::id()
    ));
    let mut rdrp_subset_text = String::new();
    for seq in &rdrp_subset.seqs {
        rdrp_subset_text.push_str(&seq_to_fasta_l2561(
            &sequence_get_seq_as_string(seq),
            &seq.label,
        ));
    }
    std::fs::write(&rdrp_subset_in, rdrp_subset_text).unwrap();
    let (rdrp_swdist_t, rdrp_swdist_mx) = cmd_swdistmx(
        rdrp_subset_in.to_str().unwrap(),
        rdrp_subset_tree.to_str().unwrap(),
    );
    assert_eq!((rdrp_swdist_t.node_count + 1) / 2, 3);
    assert_eq!(rdrp_swdist_mx.len(), 3);
    assert!(rdrp_swdist_mx[0][1].is_finite());
    assert!(
        std::fs::read_to_string(&rdrp_subset_tree)
            .unwrap()
            .contains(';')
    );
    std::fs::remove_file(&rdrp_subset_in).unwrap();
    std::fs::remove_file(&rdrp_subset_tree).unwrap();
}

#[test]
fn cmp_msa_colors_and_pfam_line_parser_match_cpp_logic() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();

    assert_eq!(hsv_to_rgb(0.0, 0.5, 0.95), (243, 121, 121));
    assert_eq!(hsv_to_rgb(1.0 / 3.0, 0.5, 0.95), (121, 243, 121));
    assert_eq!(darken(0x123456, 0.5), 0x09234e);
    let color = get_color(0);
    assert_eq!(color.len(), 7);
    assert!(color.starts_with('#'));
    assert!(color[1..].chars().all(|c| c.is_ascii_hexdigit()));

    let cmp_dir =
        std::env::temp_dir().join(format!("muscle_rs_cmd_cmp_msa_{}", std::process::id()));
    std::fs::create_dir_all(&cmp_dir).unwrap();
    let cmp_test = cmp_dir.join("test.fa");
    let cmp_ref = cmp_dir.join("ref.fa");
    let cmp_out = cmp_dir.join("cmp.html");
    std::fs::write(&cmp_test, b">seq1\nAC-G\n>seq2\nAT-G\n").unwrap();
    std::fs::write(&cmp_ref, b">seq1\nAC-G\n>seq2\nAT-G\n").unwrap();
    let cmp_qs = cmd_cmp_msa(
        cmp_test.to_str().unwrap(),
        cmp_ref.to_str().unwrap(),
        cmp_out.to_str().unwrap(),
    );
    assert_eq!(cmp_qs.labels, vec!["seq1".to_string(), "seq2".to_string()]);
    assert_eq!(cmp_qs.q, 1.0);
    assert_eq!(cmp_qs.tc, 1.0);
    let cmp_html = std::fs::read_to_string(&cmp_out).unwrap();
    assert!(cmp_html.starts_with("<html>\n<body>\n<span style=\"font-size:16px\"><pre>"));
    assert!(cmp_html.contains("<span style=\"color:gray\">-</span>"));
    assert!(cmp_html.contains("<span style=\"color:black\">seq1   </span>"));
    assert!(cmp_html.ends_with("</pre></span></body></html>\n"));
    let cmp_qs_no_output = cmd_cmp_msa(cmp_test.to_str().unwrap(), cmp_ref.to_str().unwrap(), "");
    assert_eq!(cmp_qs_no_output.q, 1.0);
    assert_eq!(cmp_qs_no_output.tc, 1.0);
    let (cmp_ref_qs, cmp_ref_log) =
        cmd_cmp_ref_msas(cmp_test.to_str().unwrap(), cmp_ref.to_str().unwrap(), 1.0);
    assert_eq!(cmp_ref_qs.ref_msas_compared_col_count, 3);
    assert_eq!(cmp_ref_qs.ref_msas_q, 1.0);
    assert!(cmp_ref_log.contains("@CMP_REF_MSAs test="));
    assert!(cmp_ref_log.contains("cols=3 Q=1.0000"));
    assert!(cmp_ref_log.contains("AA  AA  1.0000"));
    assert!(cmp_ref_log.contains("ACG  >seq1 (=seq1)"));
    assert!(cmp_ref_log.contains("|||"));
    assert!(cmp_ref_log.contains("ATG  >seq2"));
    std::fs::remove_dir_all(&cmp_dir).unwrap();

    let (dom_str, pfs) = parse_line("label\tgroup\tx\t2\tPF00001\tlo\thi\tPF00002\tlo\thi");
    assert_eq!(dom_str, "PF00001_PF00002");
    assert_eq!(pfs, vec!["PF00001", "PF00002"]);
    assert!(contains_pf_exactly_once("PF00001_PF00002", "PF00001"));
    assert!(!contains_pf_exactly_once("PF00001_PF00001", "PF00001"));
    assert!(dom_str_has_done_pf(
        "PF00001_PF00002",
        &["PF99999".to_string(), "PF00002".to_string()]
    ));
    assert!(!dom_str_has_done_pf(
        "PF00001_PF00002",
        &["PF99999".to_string()]
    ));
    let mut pfam_seqs = MultiSequence::default();
    let labels = (0..10).map(|i| format!("UP{i}")).collect::<Vec<_>>();
    let rows = (0..10)
        .map(|i| format!("ACGT{i}").replace('0', "A"))
        .collect::<Vec<_>>();
    multi_sequence_from_strings(&mut pfam_seqs, &labels, &rows);
    let mut up_to_seq_index = std::collections::BTreeMap::new();
    for (i, label) in labels.iter().enumerate() {
        up_to_seq_index.insert(label.clone(), i as uint);
    }
    let mut dom_str_to_lines = std::collections::BTreeMap::new();
    dom_str_to_lines.insert(
        "PF00001_PF00002".to_string(),
        (0..5)
            .map(|i| format!("UP{i}\tsp\tL\t2\tPF00001\t1\t2\tPF00002\t3\t4"))
            .collect::<Vec<_>>(),
    );
    dom_str_to_lines.insert(
        "PF00001_PF00003".to_string(),
        (5..10)
            .map(|i| format!("UP{i}\tsp\tL\t2\tPF00001\t1\t2\tPF00003\t3\t4"))
            .collect::<Vec<_>>(),
    );
    let mut done_labels = std::collections::BTreeSet::new();
    let (fa, tsv) = output_dom_str(
        &dom_str_to_lines,
        &up_to_seq_index,
        &pfam_seqs,
        "PF00001_PF00002",
        &mut done_labels,
    );
    assert!(fa.starts_with(">UP0\nACGTA\n"));
    assert_eq!(done_labels.len(), 5);
    assert_eq!(tsv.lines().count(), 5);
    assert!(
        do_dom_str(
            "PF00001_PF00002",
            6,
            &dom_str_to_lines,
            &up_to_seq_index,
            &pfam_seqs
        )
        .is_none()
    );
    let (fa_one, tsv_one) = do_dom_str(
        "PF00001_PF00002",
        5,
        &dom_str_to_lines,
        &up_to_seq_index,
        &pfam_seqs,
    )
    .unwrap();
    assert_eq!(fa_one.matches('>').count(), 5);
    assert_eq!(tsv_one.lines().count(), 5);
    let arg_dom_strs = dom_str_to_lines.keys().cloned().collect();
    let mut done_pfs = std::collections::BTreeSet::new();
    let (name, fa_local, tsv_local) = do_dom_strs(
        "PF00001",
        &arg_dom_strs,
        &dom_str_to_lines,
        &up_to_seq_index,
        &pfam_seqs,
        &mut done_pfs,
    )
    .unwrap();
    assert_eq!(name, "PF00001_local");
    assert_eq!(fa_local.matches('>').count(), 10);
    assert_eq!(tsv_local.lines().count(), 10);
    assert!(done_pfs.contains("PF00001"));
    assert!(done_pfs.contains("PF00002"));
    assert!(done_pfs.contains("PF00003"));
    let pfam_cmd_dir =
        std::env::temp_dir().join(format!("muscle_rs_pfamgroups_{}", std::process::id()));
    let pfam_cmd_fa = pfam_cmd_dir.join("seqs.fa");
    let pfam_cmd_tsv = pfam_cmd_dir.join("select.tsv");
    let pfam_cmd_out_fa = pfam_cmd_dir.join("fa");
    let pfam_cmd_out_tsv = pfam_cmd_dir.join("tsv");
    std::fs::create_dir_all(&pfam_cmd_dir).unwrap();
    let mut seq_file = String::new();
    let mut tsv_file = String::new();
    for i in 0..10 {
        seq_file.push_str(&format!(">UP{i}\nACGTAA\n"));
        let pf2 = if i < 5 { "PF00002" } else { "PF00003" };
        tsv_file.push_str(&format!("UP{i}\tsp\t6\t2\tPF00001\t1\t2\t{pf2}\t3\t4\n"));
    }
    std::fs::write(&pfam_cmd_fa, seq_file).unwrap();
    std::fs::write(&pfam_cmd_tsv, tsv_file).unwrap();
    let pfam_outputs = cmd_newbench_pfamgroups(
        pfam_cmd_tsv.to_str().unwrap(),
        pfam_cmd_fa.to_str().unwrap(),
        pfam_cmd_out_fa.to_str().unwrap(),
        pfam_cmd_out_tsv.to_str().unwrap(),
        10,
    );
    assert_eq!(pfam_outputs.len(), 1);
    let (pfam_fa, pfam_tsv) = pfam_outputs.get("PF00001_local").unwrap();
    assert_eq!(pfam_fa.matches('>').count(), 10);
    assert_eq!(pfam_tsv.lines().count(), 10);
    assert!(pfam_cmd_out_fa.join("PF00001_local").exists());
    assert!(pfam_cmd_out_tsv.join("PF00001_local").exists());
    std::fs::remove_dir_all(&pfam_cmd_dir).unwrap();
    let mut bench_ref1 = MultiSequence::default();
    multi_sequence_from_strings(&mut bench_ref1, &["b1".to_string()], &["AC".to_string()]);
    let mut bench_ref2 = MultiSequence::default();
    multi_sequence_from_strings(&mut bench_ref2, &["b2".to_string()], &["AG".to_string()]);
    let source_bench = Bench {
        ref_names: vec!["ref1.fa".to_string(), "ref2.fa".to_string()],
        refs: vec![bench_ref1.clone(), bench_ref2.clone()],
        inputs: vec![bench_ref2.clone(), bench_ref1.clone()],
        tcs: vec![0.25, 0.5],
        ..Bench::default()
    };
    let mut copied_bench = Bench::default();
    bench_copy(&mut copied_bench, &source_bench);
    assert_eq!(copied_bench.ref_names, source_bench.ref_names);
    assert_eq!(copied_bench.refs, source_bench.refs);
    assert_eq!(copied_bench.inputs, source_bench.inputs);
    let mut sampled_bench = Bench::default();
    bench_from_sample(&mut sampled_bench, &source_bench, 50);
    assert_eq!(sampled_bench.ref_names.len(), 1);
    assert!(source_bench.ref_names.contains(&sampled_bench.ref_names[0]));
    let bench_tc_file =
        std::env::temp_dir().join(format!("muscle_rs_bench_tcs_{}.tsv", std::process::id()));
    let tcs = bench_t_cs_to_file(&source_bench, bench_tc_file.to_str().unwrap()).unwrap();
    assert_eq!(tcs, "ref1.fa\t0.2500\nref2.fa\t0.5000\n");
    assert_eq!(std::fs::read_to_string(&bench_tc_file).unwrap(), tcs);
    std::fs::remove_file(&bench_tc_file).unwrap();
    let bench_dir =
        std::env::temp_dir().join(format!("muscle_rs_bench_load_{}", std::process::id()));
    let ref_dir = bench_dir.join("refs");
    let fa_dir = bench_dir.join("fa");
    std::fs::create_dir_all(&ref_dir).unwrap();
    std::fs::create_dir_all(&fa_dir).unwrap();
    let names_file = bench_dir.join("names.txt");
    std::fs::write(&names_file, b"one.fa\ntwo.fa\n").unwrap();
    std::fs::write(ref_dir.join("one.fa"), b">a\nA-C\n>b\nATC\n").unwrap();
    std::fs::write(ref_dir.join("two.fa"), b">c\nGG-\n").unwrap();
    std::fs::write(fa_dir.join("one.fa"), b">a\nA-C\n>b\nATC\n").unwrap();
    std::fs::write(fa_dir.join("two.fa"), b">c\nGG-\n").unwrap();

    let mut loaded_bench = Bench::default();
    let mut ref_dir_s = ref_dir.to_str().unwrap().to_string();
    dirize(&mut ref_dir_s);
    bench_load(&mut loaded_bench, names_file.to_str().unwrap(), &ref_dir_s);
    assert_eq!(loaded_bench.ref_names, vec!["one.fa", "two.fa"]);
    assert_eq!(
        sequence_get_seq_as_string(&loaded_bench.refs[0].seqs[0]),
        "A-C"
    );
    assert_eq!(
        sequence_get_seq_as_string(&loaded_bench.inputs[0].seqs[0]),
        "AC"
    );

    let mut loaded_q2_bench = Bench::default();
    let mut fa_dir_s = fa_dir.to_str().unwrap().to_string();
    dirize(&mut fa_dir_s);
    bench_load_q2(
        &mut loaded_q2_bench,
        names_file.to_str().unwrap(),
        &fa_dir_s,
        &ref_dir_s,
    );
    assert!(loaded_q2_bench.refs[0].dupe_labels_ok);
    assert_eq!(
        sequence_get_seq_as_string(&loaded_q2_bench.inputs[1].seqs[0]),
        "GG"
    );
    let mut alloc_bench = Bench {
        ap: Some(M3AlnParams {
            ready: true,
            ..M3AlnParams::default()
        }),
        thread_count: 3,
        ..Bench::default()
    };
    bench_alloc_threads(&mut alloc_bench, false);
    assert_eq!(alloc_bench.m3s.len(), 3);
    assert_eq!(alloc_bench.qss.len(), 3);
    assert!(alloc_bench.qs2s.is_empty());
    bench_alloc_threads(&mut alloc_bench, false);
    assert_eq!(alloc_bench.m3s.len(), 3);

    let mut alloc_q2_bench = Bench {
        ap: Some(M3AlnParams {
            ready: true,
            ..M3AlnParams::default()
        }),
        thread_count: 2,
        ..Bench::default()
    };
    bench_alloc_threads(&mut alloc_q2_bench, true);
    assert_eq!(alloc_q2_bench.m3s.len(), 2);
    assert!(alloc_q2_bench.qss.is_empty());
    assert_eq!(alloc_q2_bench.qs2s.len(), 2);
    let mut run_bench = Bench {
        ref_names: source_bench.ref_names.clone(),
        refs: source_bench.refs.clone(),
        inputs: source_bench.inputs.clone(),
        thread_count: 2,
        ..Bench::default()
    };
    let run_ap = M3AlnParams {
        ready: true,
        ..M3AlnParams::default()
    };
    let mut bench_calls = Vec::new();
    let final_score = bench_run(
        &mut run_bench,
        &run_ap,
        false,
        |_ap, ref_name, input, ref_msa, q2| {
            bench_calls.push((
                ref_name.to_string(),
                sequence_get_seq_as_string(&input.seqs[0]),
                sequence_get_seq_as_string(&ref_msa.seqs[0]),
                q2,
            ));
            if ref_name == "ref1.fa" {
                (0.25, 0.5)
            } else {
                (0.75, 1.0)
            }
        },
    );
    assert_eq!(
        bench_calls,
        vec![
            (
                "ref1.fa".to_string(),
                "AG".to_string(),
                "AC".to_string(),
                false
            ),
            (
                "ref2.fa".to_string(),
                "AC".to_string(),
                "AG".to_string(),
                false
            ),
        ]
    );
    assert_eq!(run_bench.tcs, vec![0.5, 1.0]);
    assert_eq!(run_bench.mean_q, 0.5);
    assert_eq!(run_bench.mean_tc, 0.75);
    assert_eq!(final_score, 1.25);
    assert_eq!(run_bench.final_score, 1.25);
    assert_eq!(run_bench.m3s.len(), 2);
    assert_eq!(run_bench.qss.len(), 2);

    let batch_root =
        std::env::temp_dir().join(format!("muscle_rs_cmd_batch_{}", std::process::id()));
    let batch_in = batch_root.join("in");
    let batch_out = batch_root.join("out");
    std::fs::create_dir_all(&batch_in).unwrap();
    std::fs::create_dir_all(&batch_out).unwrap();
    let batch_list = batch_root.join("batch.txt");
    std::fs::write(&batch_list, b"case1.fa\ncase2.fa\n").unwrap();
    std::fs::write(batch_in.join("case1.fa"), b">s1\nA-C\n>s2\nAG-\n").unwrap();
    std::fs::write(batch_in.join("case2.fa"), b">t1\n-PQ\n>t2\nR-S\n").unwrap();

    let mut batch_calls = Vec::new();
    let batch_ap = M3AlnParams {
        ready: true,
        linkage: "avg".to_string(),
        ..M3AlnParams::default()
    };
    let batch_outputs = cmd_batch(
        batch_list.to_str().unwrap(),
        batch_in.to_str().unwrap(),
        batch_out.to_str().unwrap(),
        &batch_ap,
        |_m3, ap, ms| {
            batch_calls.push((
                ap.linkage.clone(),
                sequence_get_seq_as_string(&ms.seqs[0]),
                sequence_get_seq_as_string(&ms.seqs[1]),
            ));
            ms.clone()
        },
    );
    assert_eq!(
        batch_calls,
        vec![
            ("avg".to_string(), "AC".to_string(), "AG".to_string()),
            ("avg".to_string(), "PQ".to_string(), "RS".to_string()),
        ]
    );
    assert_eq!(batch_outputs.len(), 2);
    assert_eq!(
        std::fs::read_to_string(batch_out.join("case1.fa")).unwrap(),
        ">s1\nAC\n>s2\nAG\n"
    );
    assert_eq!(
        std::fs::read_to_string(batch_out.join("case2.fa")).unwrap(),
        ">t1\nPQ\n>t2\nRS\n"
    );
    std::fs::remove_dir_all(&batch_root).unwrap();

    let cmd_bench_tsv = bench_dir.join("cmd_bench.tsv");
    let mut cmd_bench_calls = Vec::new();
    let (cmd_bench_state, cmd_bench_log) = cmd_bench(
        names_file.to_str().unwrap(),
        ref_dir.to_str().unwrap(),
        cmd_bench_tsv.to_str().unwrap(),
        &run_ap,
        |_ap, ref_name, input, ref_msa, q2| {
            cmd_bench_calls.push((
                ref_name.to_string(),
                sequence_get_seq_as_string(&input.seqs[0]),
                sequence_get_seq_as_string(&ref_msa.seqs[0]),
                q2,
            ));
            if ref_name == "one.fa" {
                (0.4, 0.6)
            } else {
                (0.8, 1.0)
            }
        },
    );
    assert_eq!(
        cmd_bench_calls,
        vec![
            (
                "one.fa".to_string(),
                "AC".to_string(),
                "A-C".to_string(),
                false,
            ),
            (
                "two.fa".to_string(),
                "GG".to_string(),
                "GG-".to_string(),
                false,
            ),
        ]
    );
    assert_eq!(cmd_bench_state.tcs, vec![0.6, 1.0]);
    assert_eq!(cmd_bench_log, "AvgQ=0.600 AvgTC=0.800 N=2\n");
    assert_eq!(
        std::fs::read_to_string(&cmd_bench_tsv).unwrap(),
        "one.fa\t0.6000\ntwo.fa\t1.0000\n"
    );

    let cmd_blosums_tsv = bench_dir.join("cmd_bench_blosums.tsv");
    let (_blosums_bench, blosums_log, blosums_tsv) = cmd_bench_blosums(
        names_file.to_str().unwrap(),
        ref_dir.to_str().unwrap(),
        cmd_blosums_tsv.to_str().unwrap(),
        |_ap, _ref_name, _input, _ref_msa, _q2| (0.1, 0.2),
    );
    assert!(blosums_log.contains("BLOSUM90:0"));
    assert!(blosums_log.contains("BLOSUM62:3"));
    let blosums_tsv = blosums_tsv.unwrap();
    assert!(blosums_tsv.starts_with("BLOSUM\tParamSet\tQ\tTC\tPerturbSeed\tDelta\n"));
    assert_eq!(
        std::fs::read_to_string(&cmd_blosums_tsv).unwrap(),
        blosums_tsv
    );

    let muscle3_in = bench_dir.join("muscle3.fa");
    let muscle3_out = bench_dir.join("muscle3_out.fa");
    std::fs::write(&muscle3_in, b">m1\nACGT\n>m2\nA-GT\n").unwrap();
    let mut muscle3_calls = Vec::new();
    let (muscle3_msa, muscle3_tree) = cmd_muscle3(
        muscle3_in.to_str().unwrap(),
        muscle3_out.to_str().unwrap(),
        None,
        false,
        &run_ap,
        |_ap, input| {
            muscle3_calls.push((
                sequence_get_seq_as_string(&input.seqs[0]),
                sequence_get_seq_as_string(&input.seqs[1]),
            ));
            (input.clone(), Tree::default())
        },
        |_ap, _input| panic!("RunRO should not be called"),
    );
    assert_eq!(
        muscle3_calls,
        vec![("ACGT".to_string(), "A-GT".to_string())]
    );
    assert!(muscle3_tree.is_some());
    assert_eq!(multi_sequence_get_col_count(&muscle3_msa), 4);
    assert_eq!(
        std::fs::read_to_string(&muscle3_out).unwrap(),
        ">m1\nACGT\n>m2\nA-GT\n"
    );

    let muscle3_ro_out = bench_dir.join("muscle3_ro_out.fa");
    let (muscle3_ro_msa, muscle3_ro_tree) = cmd_muscle3(
        muscle3_in.to_str().unwrap(),
        muscle3_ro_out.to_str().unwrap(),
        None,
        true,
        &run_ap,
        |_ap, _input| panic!("Run should not be called"),
        |_ap, input| input.clone(),
    );
    assert!(muscle3_ro_tree.is_none());
    assert_eq!(multi_sequence_get_col_count(&muscle3_ro_msa), 4);
    assert_eq!(
        std::fs::read_to_string(&muscle3_ro_out).unwrap(),
        ">m1\nACGT\n>m2\nA-GT\n"
    );

    let hmmdump_dir = bench_dir.join("hmmdump");
    std::fs::create_dir_all(&hmmdump_dir).unwrap();
    let hmmdump_files = cmd_hmmdump(hmmdump_dir.to_str().unwrap(), false, |_hp| {});
    assert_eq!(hmmdump_files.len(), 6);
    for file_name in &hmmdump_files {
        assert!(std::path::Path::new(file_name).exists());
    }
    assert!(
        std::fs::read_to_string(hmmdump_dir.join("params_report.txt"))
            .unwrap()
            .contains("const float InitProb_IM")
    );
    assert!(
        std::fs::read_to_string(hmmdump_dir.join("hmm3.tsv"))
            .unwrap()
            .starts_with("HMM\taa\n")
    );
    assert!(
        std::fs::read_to_string(hmmdump_dir.join("sa.hmm"))
            .unwrap()
            .starts_with("HMM\taa\n")
    );
    std::fs::remove_dir_all(&bench_dir).unwrap();

    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        *state = NewbenchSelectPfamsState::default();
    }
    assert_eq!(get_p_fix("PF00001", true), 0);
    assert_eq!(get_p_fix("PF00002", true), 1);
    assert_eq!(get_p_fix("PF00001", false), 0);
    assert_eq!(get_upix("UP1", true), 0);
    assert_eq!(get_upix("UP1", false), 0);

    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        *state = NewbenchSelectPfamsState::default();
    }
    std::fs::create_dir_all(".tmp").unwrap();
    let pfam_tsv = ".tmp/pfam_regions_unit.tsv";
    std::fs::write(
        pfam_tsv,
        "UP1\tSP1\tPF00001\t10\t20\t100\nUP1\tSP1\tPF00002\t30\t40\t100\nUP2\tSP2\tPF00001\t5\t10\t80\nUP2\tSP2\tPF00001\t20\t25\t80\n",
    )
    .unwrap();
    read_pfam_regions(pfam_tsv);
    assert_eq!(get_p_fix("PF00002", false), 1);
    assert_eq!(get_upix("UP2", false), 1);

    let mut aln = MultiSequence::default();
    multi_sequence_from_strings(
        &mut aln,
        &["UP1".to_string(), "UP2".to_string()],
        &["ACGT".to_string(), "TGCA".to_string()],
    );
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        state.primary_pf = "PF00001".to_string();
        state.aln_regionix_vec = state.upix_to_regionixs.clone();
    }
    assert!(has_primary_pf_repeat(&aln));
    assert_eq!(get_mean_primary_domain_length(&aln), 11.0);

    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        *state = NewbenchSelectPfamsState::default();
    }
    let primary_pfix = get_p_fix("PF00001", true);
    let before_pfix = get_p_fix("PF00002", true);
    let after_pfix = get_p_fix("PF00003", true);
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        state.primary_pf = "PF00001".to_string();
        state.primary_pfix = primary_pfix;
        state.pf_to_pfix.insert("PF00001".to_string(), primary_pfix);
        state.pf_to_pfix.insert("PF00002".to_string(), before_pfix);
        state.pf_to_pfix.insert("PF00003".to_string(), after_pfix);
        state.aln_pfix_to_char.insert(uint::MAX, '.');
        state.aln_pfix_to_char.insert(primary_pfix, '@');
        state.aln_pfix_to_char.insert(before_pfix, 'A');
        state.aln_pfix_to_char.insert(after_pfix, 'B');
        state.unique_ups = vec!["UP1".to_string(), "UP2".to_string()];
        state.up_to_upix.insert("UP1".to_string(), 0);
        state.up_to_upix.insert("UP2".to_string(), 1);
        state.upix_to_regionixs = vec![vec![0, 1, 2], vec![3, 4, 5]];
        state.up_to_sp.insert("UP1".to_string(), "SP1".to_string());
        state.up_to_sp.insert("UP2".to_string(), "SP2".to_string());
        state.ups = vec![
            "UP1".to_string(),
            "UP1".to_string(),
            "UP1".to_string(),
            "UP2".to_string(),
            "UP2".to_string(),
            "UP2".to_string(),
        ];
        state.sps = vec![
            "SP1".to_string(),
            "SP1".to_string(),
            "SP1".to_string(),
            "SP2".to_string(),
            "SP2".to_string(),
            "SP2".to_string(),
        ];
        state.pfs = vec![
            "PF00002".to_string(),
            "PF00001".to_string(),
            "PF00003".to_string(),
            "PF00002".to_string(),
            "PF00001".to_string(),
            "PF00003".to_string(),
        ];
        state.los = vec![1, 3, 5, 1, 3, 5];
        state.his = vec![2, 4, 6, 2, 4, 6];
        state.ls = vec![6, 6, 6, 6, 6, 6];
        state.aln_regionix_vec = vec![vec![0, 1, 2], vec![3, 4, 5]];
    }
    assert_eq!(get_annot_coverage(&[0, 1, 2]), 0.0);
    assert_eq!(
        select_up(0).as_deref(),
        Some("UP1\tSP1\t6\t3\tPF00002\t1\t2\tPF00001\t3\t4\tPF00003\t5\t6\n")
    );
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        *state = NewbenchSelectPfamsState::default();
    }
    let selectpfams_tsv = ".tmp/pfam_regions_select_cmd.tsv";
    let selectpfams_out = ".tmp/pfam_regions_select_cmd.out";
    std::fs::write(
        selectpfams_tsv,
        "UPA\tSPA\tPF1\t1\t45\t100\nUPA\tSPA\tPF2\t46\t95\t100\nUPB\tSPB\tPF1\t1\t20\t100\n",
    )
    .unwrap();
    let selected = cmd_newbench_selectpfams(selectpfams_tsv, selectpfams_out);
    assert_eq!(selected, "UPA\tSPA\t100\t2\tPF1\t1\t45\tPF2\t46\t95\n");
    assert_eq!(std::fs::read_to_string(selectpfams_out).unwrap(), selected);
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        *state = NewbenchSelectPfamsState::default();
    }
    assert_eq!(cmd_newbench_selectpfams(selectpfams_tsv, ""), selected);
    std::fs::remove_file(selectpfams_tsv).unwrap();
    std::fs::remove_file(selectpfams_out).unwrap();
    {
        let mut state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        *state = NewbenchSelectPfamsState::default();
        state.primary_pf = "PF00001".to_string();
        state.primary_pfix = primary_pfix;
        state.pf_to_pfix.insert("PF00001".to_string(), primary_pfix);
        state.pf_to_pfix.insert("PF00002".to_string(), before_pfix);
        state.pf_to_pfix.insert("PF00003".to_string(), after_pfix);
        state.aln_pfix_to_char.insert(uint::MAX, '.');
        state.aln_pfix_to_char.insert(primary_pfix, '@');
        state.aln_pfix_to_char.insert(before_pfix, 'A');
        state.aln_pfix_to_char.insert(after_pfix, 'B');
        state.unique_ups = vec!["UP1".to_string(), "UP2".to_string()];
        state.up_to_upix.insert("UP1".to_string(), 0);
        state.up_to_upix.insert("UP2".to_string(), 1);
        state.upix_to_regionixs = vec![vec![0, 1, 2], vec![3, 4, 5]];
        state.up_to_sp.insert("UP1".to_string(), "SP1".to_string());
        state.up_to_sp.insert("UP2".to_string(), "SP2".to_string());
        state.ups = vec![
            "UP1".to_string(),
            "UP1".to_string(),
            "UP1".to_string(),
            "UP2".to_string(),
            "UP2".to_string(),
            "UP2".to_string(),
        ];
        state.sps = vec![
            "SP1".to_string(),
            "SP1".to_string(),
            "SP1".to_string(),
            "SP2".to_string(),
            "SP2".to_string(),
            "SP2".to_string(),
        ];
        state.pfs = vec![
            "PF00002".to_string(),
            "PF00001".to_string(),
            "PF00003".to_string(),
            "PF00002".to_string(),
            "PF00001".to_string(),
            "PF00003".to_string(),
        ];
        state.los = vec![1, 3, 5, 1, 3, 5];
        state.his = vec![2, 4, 6, 2, 4, 6];
        state.ls = vec![6, 6, 6, 6, 6, 6];
        state.aln_regionix_vec = vec![vec![0, 1, 2], vec![3, 4, 5]];
    }
    let mut aln2 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut aln2,
        &["UP1.pdb".to_string(), "UP2.pdb".to_string()],
        &["aa-ccdd".to_string(), "aagccd-".to_string()],
    );
    set_aln_labels(&mut aln2);
    assert_eq!(aln2.seqs[0].label, "UP1");
    assert_eq!(aln2.seqs[1].label, "UP2");
    set_ungapped_seqs(&aln2);
    set_aln_regions(&aln2);
    {
        let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        assert_eq!(
            state.dom_strs,
            vec![
                "PF00002+PF00001+PF00003".to_string(),
                "PF00002+PF00001+PF00003".to_string()
            ]
        );
        assert_eq!(state.dom_coverages, vec![1.0, 1.0]);
        assert_eq!(
            state.dom_str_to_count.get("PF00002+PF00001+PF00003"),
            Some(&2)
        );
        assert_eq!(state.globalness, 1.0);
        assert_eq!(state.aln_pfix_to_char.get(&uint::MAX), Some(&'.'));
        assert_eq!(state.aln_pfix_to_char.get(&primary_pfix), Some(&'@'));
        assert_eq!(state.aln_pfix_to_char.get(&before_pfix), Some(&'A'));
        assert_eq!(state.aln_pfix_to_char.get(&after_pfix), Some(&'C'));
    }
    assert_eq!(
        get_before_after_p_fs(&aln2, 0),
        ("PF00002".to_string(), "PF00003".to_string())
    );
    assert_eq!(set_before_after_p_fs(&aln2), 1.0);
    set_ungapped_seqs(&aln2);
    set_pos_to_col_vec(&aln2);
    assert_eq!(set_pos_to_p_fix_vec(&aln2), 0);
    assert_eq!(set_col_to_p_fix_vec(&aln2), vec!["AA.@@CC", "AA@@CC."]);
    assert_eq!(get_consensus_pf(&aln2, 0), before_pfix);
    assert_eq!(get_consensus_pf(&aln2, 2), uint::MAX);
    set_col_annots(&aln2);
    {
        let state = NEWBENCH_SELECT_PFAMS_STATE.lock().unwrap();
        assert_eq!(state.annot_row, "::~^~:~");
        assert_eq!(state.core_count, 4);
        assert_eq!(state.core_primary_count, 1);
        assert_eq!(state.core_other_count, 3);
    }
    assert_eq!(get_primary_col_lo_hi(&aln2, 0), (3, 4));
    assert_eq!(get_primary_col_lo_hi(&aln2, 1), (2, 3));
    set_aln_case(&mut aln2);
    assert_eq!(sequence_get_seq_as_string(&aln2.seqs[0]), "AA-CcDd");
    assert_eq!(sequence_get_seq_as_string(&aln2.seqs[1]), "AAgCcD-");
    let trimmed = trim(&aln2);
    assert_eq!(sequence_get_seq_as_string(&trimmed.seqs[0]), ".Cc");
    assert_eq!(sequence_get_seq_as_string(&trimmed.seqs[1]), "gC.");
}

#[test]
fn sweep_specs_and_gap_stripping_match_cpp_logic() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    assert_eq!(q_scorer2_strip_gaps("A-C.G"), "ACG");
    assert_eq!(q_scorer2_get_pos_to_col("A-C.G"), vec![0, 2, 4]);
    assert!((q_scorer2_get_q("A-CG", "AT-G", "ACG", "ATG") - (2.0 / 3.0)).abs() < 1e-12);

    let mut test = MultiSequence::default();
    multi_sequence_from_strings(
        &mut test,
        &["seq1".to_string(), "seq2".to_string()],
        &["A-CG".to_string(), "AT-G".to_string()],
    );
    let mut ref_ms = MultiSequence::default();
    multi_sequence_from_strings(
        &mut ref_ms,
        &["seq1".to_string(), "seq2".to_string()],
        &["ACG".to_string(), "ATG".to_string()],
    );
    assert!((q_scorer2_run_l100(&test, &ref_ms) - (2.0 / 3.0)).abs() < 1e-12);
    assert!((q_scorer2_run_l91(&test, &ref_ms) - (2.0 / 3.0)).abs() < 1e-12);
    let qscore2_test =
        std::env::temp_dir().join(format!("muscle_rs_qscore2_test_{}.fa", std::process::id()));
    let qscore2_ref =
        std::env::temp_dir().join(format!("muscle_rs_qscore2_ref_{}.fa", std::process::id()));
    std::fs::write(&qscore2_test, b">seq1\nA-CG\n>seq2\nAT-G\n").unwrap();
    std::fs::write(&qscore2_ref, b">seq1\nACG\n>seq2\nATG\n").unwrap();
    assert_eq!(
        cmd_qscore2(
            qscore2_test.to_str().unwrap(),
            qscore2_ref.to_str().unwrap(),
            1.0,
        ),
        format!(
            "{}: Q=0.6667, TC=0.6667\n",
            get_base_name(qscore2_test.to_str().unwrap())
        )
    );
    let qscore2_cli_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-qscore2",
            qscore2_test.to_str().unwrap(),
            "-ref",
            qscore2_ref.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(qscore2_cli_cmd.status.success());
    assert_eq!(
        String::from_utf8(qscore2_cli_cmd.stdout).unwrap(),
        format!(
            "{}: Q=0.6667, TC=0.6667\n",
            get_base_name(qscore2_test.to_str().unwrap())
        )
    );
    let qscoredir_root =
        std::env::temp_dir().join(format!("muscle_rs_qscoredir_{}", std::process::id()));
    let qscoredir_test_dir = qscoredir_root.join("test");
    let qscoredir_ref_dir = qscoredir_root.join("ref");
    std::fs::create_dir_all(&qscoredir_test_dir).unwrap();
    std::fs::create_dir_all(&qscoredir_ref_dir).unwrap();
    let qscoredir_names = qscoredir_root.join("names.txt");
    let qscoredir_out = qscoredir_root.join("out.txt");
    std::fs::write(&qscoredir_names, b"ok.fa\nmissing.fa\n").unwrap();
    std::fs::write(
        qscoredir_test_dir.join("ok.fa"),
        b">seq1\nA-CG\n>seq2\nAT-G\n",
    )
    .unwrap();
    std::fs::write(qscoredir_ref_dir.join("ok.fa"), b">seq1\nACG\n>seq2\nATG\n").unwrap();
    let qscoredir_expected = format!(
        "set=ok.fa\tq=0.6667\ttc=0.6667\ntestdir={}/\tn=2\tN=1\tM=1\tavgq=0.6667\tavgtc=0.6667\n",
        qscoredir_test_dir.to_string_lossy()
    );
    assert_eq!(
        cmd_qscoredir(
            qscoredir_names.to_str().unwrap(),
            qscoredir_test_dir.to_str().unwrap(),
            qscoredir_ref_dir.to_str().unwrap(),
            qscoredir_out.to_str().unwrap(),
            1.0,
        ),
        qscoredir_expected
    );
    assert_eq!(
        std::fs::read_to_string(&qscoredir_out).unwrap(),
        qscoredir_expected
    );
    assert_eq!(
        cmd_qscoredir(
            qscoredir_names.to_str().unwrap(),
            qscoredir_test_dir.to_str().unwrap(),
            qscoredir_ref_dir.to_str().unwrap(),
            "",
            1.0,
        ),
        qscoredir_expected
    );
    let mut qs = QScorer::default();
    qs.max_gap_fract = 1.0;
    qs.name = "unit".to_string();
    qs.test = Some(test.clone());
    qs.ref_msa = Some(ref_ms.clone());
    q_scorer_init_ref_labels(&mut qs);
    assert_eq!(qs.ref_labels, vec!["seq1".to_string(), "seq2".to_string()]);
    assert_eq!(qs.ref_label_to_seq_index.get("seq2"), Some(&1));
    q_scorer_init_ref_to_test(&mut qs);
    assert_eq!(qs.labels, vec!["seq1".to_string(), "seq2".to_string()]);
    assert_eq!(qs.ref_seq_indexes, vec![0, 1]);
    assert_eq!(qs.test_seq_indexes, vec![0, 1]);
    q_scorer_init_col_pos_vecs(&mut qs);
    assert_eq!(qs.pos_to_test_col_vec[0], vec![0, 2, 3]);
    assert_eq!(qs.pos_to_ref_col_vec[0], vec![0, 1, 2]);
    assert_eq!(qs.ref_col_to_test_col_vec[0], vec![0, 2, 3]);
    q_scorer_init_ref_cols(&mut qs);
    assert_eq!(qs.ref_cols, vec![0, 1, 2]);
    q_scorer_init_ref_ungapped_counts(&mut qs);
    assert_eq!(qs.ref_ungapped_counts, vec![2, 2, 2]);
    q_scorer_do_ref_cols(&mut qs);
    assert_eq!(qs.best_test_cols, vec![0, uint::MAX, 3]);
    assert_eq!(qs.correct_pairs, 2);
    assert_eq!(qs.total_pairs, 3);
    assert_eq!(qs.correct_cols, 2);
    q_scorer_set_test_col_to_best_ref_col(&mut qs);
    assert_eq!(
        qs.test_col_to_best_ref_col,
        vec![0, uint::MAX, uint::MAX, 2]
    );
    let mut counts = Vec::new();
    q_scorer_update_ref_letter_counts(&qs, &mut counts);
    assert_eq!(counts, vec![vec![1, 0, 1], vec![1, 0, 1]]);
    q_scorer_set_test_col_is_aligned(&mut qs);
    q_scorer_set_ref_col_is_aligned(&mut qs);
    assert_eq!(qs.test_col_is_aligned, vec![true, false, false, true]);
    assert_eq!(qs.ref_col_is_aligned, vec![true, true, true]);
    let mut qs_run = QScorer::default();
    qs_run.max_gap_fract = 1.0;
    assert!(q_scorer_run_l346(
        &mut qs_run,
        "unit",
        &test,
        &ref_ms,
        false
    ));
    assert!((qs_run.q - (2.0_f32 / 3.0)).abs() < 1e-6);
    assert!((qs_run.tc - (2.0_f32 / 3.0)).abs() < 1e-6);
    q_scorer_cmp_ref_ms_as(&mut qs_run, "unit", &test, &ref_ms, false);
    assert_eq!(qs_run.ref_msas_compared_col_count, 2);
    assert_eq!(qs_run.ref_msas_test_cols, vec![0, 3]);
    assert_eq!(qs_run.ref_msas_ref_cols, vec![0, 2]);
    assert_eq!(qs_run.ref_msas_col_qs, vec![1.0, 1.0]);
    assert_eq!(qs_run.ref_msas_q, 1.0);
    let mut transq_test1 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut transq_test1,
        &["seq1".to_string(), "seq2".to_string()],
        &["ABC".to_string(), "ABC".to_string()],
    );
    let mut transq_test2 = MultiSequence::default();
    multi_sequence_from_strings(
        &mut transq_test2,
        &["seq1".to_string(), "seq2".to_string()],
        &["A-BC".to_string(), "AB-C".to_string()],
    );
    let mut transq_ref = MultiSequence::default();
    multi_sequence_from_strings(
        &mut transq_ref,
        &["seq1".to_string(), "seq2".to_string()],
        &["ABC".to_string(), "ABC".to_string()],
    );
    let mut transq_qs1 = QScorer::default();
    transq_qs1.max_gap_fract = 1.0;
    assert!(q_scorer_run_l346(
        &mut transq_qs1,
        "transq1",
        &transq_test1,
        &transq_ref,
        false
    ));
    let mut transq_qs2 = QScorer::default();
    transq_qs2.max_gap_fract = 1.0;
    assert!(q_scorer_run_l346(
        &mut transq_qs2,
        "transq2",
        &transq_test2,
        &transq_ref,
        false
    ));
    let mut qs3 = QScorer3 {
        test1: transq_test1,
        test2: transq_test2,
        ref_msa: transq_ref,
        qs1: transq_qs1.clone(),
        qs2: transq_qs2,
        indexes2: vec![0, 1],
        ref_cols: transq_qs1.ref_cols.clone(),
        ..QScorer3::default()
    };
    q_scorer3_trans_q(&mut qs3);
    assert_eq!(qs3.pairs, vec![(0, 1)]);
    assert_eq!(qs3.pair_index_to_q1, vec![1.0]);
    assert!((qs3.pair_index_to_q2[0] - (2.0 / 3.0)).abs() < 1e-6);
    assert!((qs3.pair_index_to_pwc[0] - (2.0 / 3.0)).abs() < 1e-6);
    q_scorer_clear(&mut qs);
    assert!(qs.test.is_none());
    assert!(qs.ref_msa.is_none());
    assert!(qs.labels.is_empty());

    let mut qs_byseq = QScorer {
        test: Some(test.clone()),
        ref_msa: Some(ref_ms.clone()),
        ..QScorer::default()
    };
    q_scorer_init_ref_labels_bysequence(&mut qs_byseq);
    assert_eq!(qs_byseq.ref_seq_to_seq_index.get("ACG"), Some(&0));
    q_scorer_init_ref_to_test_bysequence(&mut qs_byseq);
    assert_eq!(qs_byseq.ref_seq_index_to_test_seq_index, vec![0, 1]);
    std::fs::remove_file(&qscore2_test).unwrap();
    std::fs::remove_file(&qscore2_ref).unwrap();
    std::fs::remove_file(qscoredir_test_dir.join("ok.fa")).unwrap();
    std::fs::remove_file(qscoredir_ref_dir.join("ok.fa")).unwrap();
    std::fs::remove_file(&qscoredir_names).unwrap();
    std::fs::remove_file(&qscoredir_out).unwrap();
    std::fs::remove_dir(&qscoredir_test_dir).unwrap();
    std::fs::remove_dir(&qscoredir_ref_dir).unwrap();
    std::fs::remove_dir(&qscoredir_root).unwrap();
    assert_eq!(strip_gaps(".A-C."), "AC");
    assert_eq!(
        get_insert_lo_his(&[false, false, true, false, true, false]),
        (vec![0, 3, 5], vec![1, 3, 5])
    );
    let mut insert_msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut insert_msa,
        &["s1".to_string(), "s2".to_string()],
        &["AAxxBBcc".to_string(), "AA-yBBc-".to_string()],
    );
    assert_eq!(
        get_msa_col_aligned_vec(&insert_msa),
        vec![true, true, false, false, true, true, false, false]
    );
    let squeezed = squeeze_inserts(&insert_msa);
    assert_eq!(sequence_get_seq_as_string(&squeezed.seqs[0]), "AAxxBBcc");
    assert_eq!(sequence_get_seq_as_string(&squeezed.seqs[1]), "AA.yBBc.");
    let squeeze_in =
        std::env::temp_dir().join(format!("muscle_rs_squeeze_in_{}.fa", std::process::id()));
    let squeeze_out =
        std::env::temp_dir().join(format!("muscle_rs_squeeze_out_{}.fa", std::process::id()));
    std::fs::write(&squeeze_in, b">s1\nAAxxBBcc\n>s2\nAA-yBBc-\n").unwrap();
    let squeezed_cmd =
        cmd_squeeze_inserts(squeeze_in.to_str().unwrap(), squeeze_out.to_str().unwrap());
    assert_eq!(
        sequence_get_seq_as_string(&squeezed_cmd.seqs[1]),
        "AA.yBBc."
    );
    assert_eq!(
        std::fs::read_to_string(&squeeze_out).unwrap(),
        ">s1\nAAxxBBcc\n>s2\nAA.yBBc.\n"
    );
    std::fs::remove_file(&squeeze_in).unwrap();
    std::fs::remove_file(&squeeze_out).unwrap();
    let mustang_in = std::env::temp_dir().join(format!(
        "muscle_rs_mustang_core_in_{}.fa",
        std::process::id()
    ));
    let mustang_out = std::env::temp_dir().join(format!(
        "muscle_rs_mustang_core_out_{}.fa",
        std::process::id()
    ));
    std::fs::write(
        &mustang_in,
        b">model1.pdb\nACDEFGHIKLm-n\n>model2\nACDEFGHIKLnp-\n",
    )
    .unwrap();
    let mustang_text =
        cmd_mustang_core(mustang_in.to_str().unwrap(), mustang_out.to_str().unwrap());
    assert_eq!(
        mustang_text,
        ">model1\nACDEFGHIKLMn\n>model2\nACDEFGHIKLNp\n"
    );
    assert_eq!(std::fs::read_to_string(&mustang_out).unwrap(), mustang_text);
    std::fs::write(&mustang_in, b">short.pdb\nA-C\n>other\nABC\n").unwrap();
    std::fs::remove_file(&mustang_out).unwrap();
    let mustang_warning =
        cmd_mustang_core(mustang_in.to_str().unwrap(), mustang_out.to_str().unwrap());
    assert_eq!(mustang_warning, "WARNING: 2 aligned cols < 10\n");
    assert!(!mustang_out.exists());
    std::fs::remove_file(&mustang_in).unwrap();

    let (names, deltas) = parse_spatter_spec("gapopen,0.25/center,-1.5");
    assert_eq!(names, vec!["gapopen", "center"]);
    assert_eq!(deltas, vec![0.25, -1.5]);

    let (names, goods, los, his, sizes) = parse_grid_spec("gapopen,1.5,5,1,9/center,0.2,-1,1,3");
    assert_eq!(names, vec!["gapopen", "center"]);
    assert_eq!(goods, vec![1.5, 0.2]);
    assert_eq!(los, vec![1.0, -1.0]);
    assert_eq!(his, vec![5.0, 1.0]);
    assert_eq!(sizes, vec![9, 3]);

    let (names, goods, los, his, sizes) =
        parse_grid_spec("gapopen,-,0.1,0.9,5/center,ignored,-2,2,4");
    assert_eq!(names, vec!["gapopen", "center"]);
    assert!(goods.is_empty());
    assert_eq!(los, vec![0.1, -2.0]);
    assert_eq!(his, vec![0.9, 2.0]);
    assert_eq!(sizes, vec![5, 4]);

    let mut score_s = Sweeper {
        grid_counter: uint::MAX,
        spatter_iter: uint::MAX,
        ..Sweeper::default()
    };
    sweeper_set_param_names(&mut score_s, &["gapopen".to_string(), "center".to_string()]);
    let subst_mx = get_subst_mx_letter_blosum(62);
    let mut top_score = 0.0_f64;
    let mut top_q = 0.0_f64;
    let mut top_tc = 0.0_f64;
    let (ap, q, tc, log) = sweeper_get_score_l12(
        &score_s,
        &[-7.0, 0.3],
        &subst_mx,
        &mut top_score,
        &mut top_q,
        &mut top_tc,
        Some(3),
        |ap| {
            assert_eq!(ap.gap_open, -7.0);
            assert_eq!(ap.center, 0.3);
            (0.25, 0.75)
        },
    );
    assert_eq!(ap.tree_iters, 3);
    assert_eq!((q, tc), (0.25, 0.75));
    assert_eq!(top_score, 0.75);
    assert!(log.starts_with("gapopen=      -7 center=     0.3"));
    assert!(log.contains("<<"));

    let mut spatter_ap = M3AlnParams::default();
    m3_aln_params_set_from_cmd_line(
        &mut spatter_ap,
        false,
        false,
        None,
        None,
        None,
        Some(62),
        Some(0),
        None,
        None,
        None,
        None,
    );
    let (q, tc, log) = sweeper_get_score_l20(
        &score_s,
        &[-8.0, 0.4],
        &subst_mx,
        &mut spatter_ap,
        &mut top_score,
        &mut top_q,
        &mut top_tc,
        |ap| {
            assert_eq!(ap.gap_open, -8.0);
            assert_eq!(ap.center, 0.4);
            (0.5, 0.6)
        },
    );
    assert_eq!((q, tc), (0.5, 0.6));
    assert!(log.starts_with("gapopen=      -8 center=     0.4"));
    assert!(log.ends_with('\r'));

    let sweep_root =
        std::env::temp_dir().join(format!("muscle_rs_cmd_sweep_{}", std::process::id()));
    let sweep_ref_dir = sweep_root.join("ref");
    std::fs::create_dir_all(&sweep_ref_dir).unwrap();
    let sweep_names = sweep_root.join("names.txt");
    let sweep_fev = sweep_root.join("sweep.fev");
    std::fs::write(&sweep_names, b"case.fa\n").unwrap();
    std::fs::write(sweep_ref_dir.join("case.fa"), b">s1\nA-C\n>s2\nAG-\n").unwrap();
    let mut sweep_calls = Vec::new();
    let (sweep_s, sweep_bench, sweep_log, sweep_fev_text) = cmd_sweep(
        sweep_names.to_str().unwrap(),
        sweep_ref_dir.to_str().unwrap(),
        "gapopen,-7,-8,-6,2/center,0.3,0.2,0.4,2",
        sweep_fev.to_str().unwrap(),
        Some(62),
        None,
        Some(2),
        |ap, ref_name, input, ref_msa, q2| {
            sweep_calls.push((
                ref_name.to_string(),
                sequence_get_seq_as_string(&input.seqs[0]),
                sequence_get_seq_as_string(&ref_msa.seqs[0]),
                q2,
                ap.gap_open,
                ap.center,
                ap.tree_iters,
            ));
            (
                0.1 + f64::from(ap.center),
                0.2 + f64::from(-ap.gap_open) / 100.0,
            )
        },
    );
    assert_eq!(sweep_s.param_names, vec!["gapopen", "center"]);
    assert_eq!(sweep_s.scores.len(), 5);
    assert_eq!(sweep_bench.ref_names, vec!["case.fa"]);
    assert_eq!(sweep_calls[0].0, "case.fa");
    assert_eq!(sweep_calls[0].1, "AC");
    assert_eq!(sweep_calls[0].2, "A-C");
    assert!(!sweep_calls[0].3);
    assert_eq!(sweep_calls[0].6, 2);
    assert!(sweep_log.contains("Top params:"));
    assert_eq!(std::fs::read_to_string(&sweep_fev).unwrap(), sweep_fev_text);
    std::fs::remove_dir_all(&sweep_root).unwrap();

    let spatter_root =
        std::env::temp_dir().join(format!("muscle_rs_cmd_spatter_{}", std::process::id()));
    let spatter_ref_dir = spatter_root.join("ref");
    std::fs::create_dir_all(&spatter_ref_dir).unwrap();
    let spatter_names = spatter_root.join("names.txt");
    let spatter_output1 = spatter_root.join("warmup.fev");
    let spatter_fev = spatter_root.join("spatter.fev");
    std::fs::write(&spatter_names, b"case.fa\n").unwrap();
    std::fs::write(spatter_ref_dir.join("case.fa"), b">s1\nA-C\n>s2\nAG-\n").unwrap();
    reset_rand(1);
    let mut spatter_calls = 0;
    let (spatter_s1, spatter_s2, _bench, _warmup_bench, spatter_log, output1_text, fev_text) =
        cmd_spatter(
            spatter_names.to_str().unwrap(),
            spatter_ref_dir.to_str().unwrap(),
            100,
            "gapopen,-7,-8,-6,2/center,0.3,0.2,0.4,2",
            "gapopen,0.2/center,0.1",
            spatter_output1.to_str().unwrap(),
            spatter_fev.to_str().unwrap(),
            Some(62),
            None,
            1,
            1,
            1,
            0.5,
            |ap, _ref_name, _input, _ref_msa, _q2| {
                spatter_calls += 1;
                (
                    0.1 + f64::from(ap.center),
                    0.2 + f64::from(-ap.gap_open) / 100.0,
                )
            },
        );
    assert!(spatter_calls >= 5);
    assert_eq!(spatter_s1.param_names, vec!["gapopen", "center"]);
    assert_eq!(spatter_s2.param_names, vec!["gapopen", "center"]);
    assert!(spatter_log.contains("Warmup done"));
    assert!(spatter_log.contains("Top params:"));
    assert_eq!(
        std::fs::read_to_string(&spatter_output1).unwrap(),
        output1_text
    );
    assert_eq!(std::fs::read_to_string(&spatter_fev).unwrap(), fev_text);
    std::fs::remove_dir_all(&spatter_root).unwrap();
}

#[test]
fn balibase_fixture_commands_exercise_real_data_paths() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    let families = [
        ("BB11001", 4_usize, 4_usize),
        ("BB11002", 8, 8),
        ("BB11004", 4, 4),
        ("BB11005", 14, 9),
        ("BB11006", 8, 5),
        ("BB11007", 9, 9),
        ("BB11009", 4, 4),
    ];

    for (name, seq_count, profile_count) in families {
        let ref_file = format!("muscle/test_data/ref_alns/{name}");
        let stats = cmd_msastats(&ref_file, Some(0.5));
        assert!(stats.contains(&format!("{seq_count:10}  Sequences\n")));
        assert!(stats.contains("Mean seq length"));
        assert!(stats.contains("Mean col gap pct"));
        let stats_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args(["-msastats", &ref_file, "-max_gap_fract", "0.5", "-quiet"])
            .output()
            .unwrap();
        assert!(
            stats_cli.status.success(),
            "{name} stdout={} stderr={}",
            String::from_utf8_lossy(&stats_cli.stdout),
            String::from_utf8_lossy(&stats_cli.stderr)
        );
        assert_eq!(String::from_utf8(stats_cli.stdout).unwrap(), stats);

        let (q, tc) = cmd_qscore(&ref_file, &ref_file, false);
        assert!((q - 1.0).abs() < 1e-12, "{name} q={q}");
        assert!((tc - 1.0).abs() < 1e-12, "{name} tc={tc}");
        let qscore2 = cmd_qscore2(&ref_file, &ref_file, 1.0);
        assert_eq!(qscore2, format!("{name}: Q=1.0000, TC=1.0000\n"));
        let qscore2_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args([
                "-qscore2",
                &ref_file,
                "-ref",
                &ref_file,
                "-max_gap_fract",
                "1.0",
                "-quiet",
            ])
            .output()
            .unwrap();
        assert!(
            qscore2_cli.status.success(),
            "{name} stdout={} stderr={}",
            String::from_utf8_lossy(&qscore2_cli.stdout),
            String::from_utf8_lossy(&qscore2_cli.stderr)
        );
        assert_eq!(String::from_utf8(qscore2_cli.stdout).unwrap(), qscore2);

        let mut fa = MultiSequence::default();
        let fa_file = format!("muscle/test_data/fa/{name}");
        multi_sequence_load_mfa_l8(&mut fa, &fa_file, true);
        assert_eq!(fa.seqs.len(), seq_count);

        *MEGA_STATE.lock().unwrap() = MegaState::default();
        mega_from_file(&format!("muscle/test_data/mega/{name}.mega"));
        {
            let mega = MEGA_STATE.lock().unwrap();
            assert!(mega.loaded);
            assert_eq!(mega.feature_count, 8);
            assert_eq!(mega.profiles.len(), profile_count);
            assert_eq!(mega.labels.len(), profile_count);
            assert_eq!(mega.seqs.len(), profile_count);
            assert!(mega.log_odds_mx_vec.iter().all(|mx| !mx.is_empty()));
            let fa_by_seq = fa
                .seqs
                .iter()
                .map(|seq| (sequence_get_seq_as_string(seq), seq.char_vec.len()))
                .collect::<std::collections::BTreeMap<_, _>>();
            for ((label, seq), profile) in mega.labels.iter().zip(&mega.seqs).zip(&mega.profiles) {
                assert_eq!(Some(&profile.len()), fa_by_seq.get(seq), "{name} {label}");
            }
        }
    }

    let qscoredir = cmd_qscoredir(
        "muscle/test_data/info/BB.accs",
        "muscle/test_data/ref_alns",
        "muscle/test_data/ref_alns",
        "",
        1.0,
    );
    assert_eq!(qscoredir.matches("\tq=1.0000\ttc=1.0000\n").count(), 7);
    assert!(qscoredir.contains("set=BB11005\tq=1.0000\ttc=1.0000\n"));
    assert!(qscoredir.ends_with(
        "testdir=muscle/test_data/ref_alns/\tn=7\tN=7\tM=7\tavgq=1.0000\tavgtc=1.0000\n"
    ));
    let qscoredir_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-qscoredir",
            "muscle/test_data/info/BB.accs",
            "-testdir",
            "muscle/test_data/ref_alns",
            "-refdir",
            "muscle/test_data/ref_alns",
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        qscoredir_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&qscoredir_cli.stdout),
        String::from_utf8_lossy(&qscoredir_cli.stderr)
    );
    assert_eq!(String::from_utf8(qscoredir_cli.stdout).unwrap(), qscoredir);

    let efa_root = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_real_efa_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&efa_root).unwrap();
    let efa_paths = efa_root.join("paths.txt");
    let efa_file = efa_root.join("bb11001.efa");
    std::fs::write(
        &efa_paths,
        b"muscle/test_data/ref_alns/BB11001\nmuscle/test_data/ref_alns/BB11001\n",
    )
    .unwrap();
    let efa = cmd_fa2efa(
        efa_paths.to_str().unwrap(),
        efa_file.to_str().unwrap(),
        true,
        true,
    );
    assert_eq!(efa.msas.len(), 2);
    let qscore_efa = cmd_qscore_efa(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        1.0,
    );
    assert_eq!(qscore_efa.matches("Q=1.0000 TC=1.0000\n").count(), 2);
    let efastats = cmd_efastats(
        efa_file.to_str().unwrap(),
        1.0,
        Some("muscle/test_data/ref_alns/BB11001"),
    );
    assert!(efastats.contains("4 seqs, 2 MSAs"));
    assert!(efastats.contains("E_LP 0.0000, E_Cols 0.0000"));
    let disperse = cmd_disperse(efa_file.to_str().unwrap(), 1.0);
    assert!(disperse.contains("D_LP=0 D_Cols=0"));
    let efastats_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-efastats",
            efa_file.to_str().unwrap(),
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(efastats_cli.status.success());
    assert!(
        String::from_utf8(efastats_cli.stdout)
            .unwrap()
            .contains("E_LP 0.0000, E_Cols 0.0000")
    );
    let disperse_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-disperse",
            efa_file.to_str().unwrap(),
            "-max_gap_fract",
            "1.0",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(disperse_cli.status.success());
    assert!(
        String::from_utf8(disperse_cli.stdout)
            .unwrap()
            .contains("D_LP=0 D_Cols=0")
    );
    let colscore_file = efa_root.join("bb11001.colscore.tsv");
    let colscore = cmd_colscore_efa(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        colscore_file.to_str().unwrap(),
        1.0,
    );
    assert!(colscore.starts_with("meantc\t1.0000\n"));
    assert_eq!(std::fs::read_to_string(&colscore_file).unwrap(), colscore);
    let bestconf_file = efa_root.join("bb11001.bestconf.fa");
    let (bestconf, best_total, best_median) =
        cmd_efa_bestconf(efa_file.to_str().unwrap(), bestconf_file.to_str().unwrap());
    assert_eq!((best_total, best_median), (0, 0));
    assert!(bestconf.contains("4 seqs, 2 MSAs"));
    assert!(
        std::fs::read_to_string(&bestconf_file)
            .unwrap()
            .starts_with(">1aab_")
    );
    let bestcols_file = efa_root.join("bb11001.bestcols.fa");
    let bestcols = cmd_efa_bestcols(
        efa_file.to_str().unwrap(),
        bestcols_file.to_str().unwrap(),
        1.0,
        1.0,
        8,
    );
    assert_eq!(multi_sequence_get_col_count(&bestcols), 8);
    assert!(
        std::fs::read_to_string(&bestcols_file)
            .unwrap()
            .starts_with(">1aab_")
    );
    let addconf_file = efa_root.join("bb11001.addconf.efa");
    let addconf = cmd_addconfseq(
        efa_file.to_str().unwrap(),
        addconf_file.to_str().unwrap(),
        Some("muscle/test_data/ref_alns/BB11001"),
        Some("conf"),
        true,
    );
    assert_eq!(addconf.matches(">conf\n").count(), 2);
    assert_eq!(std::fs::read_to_string(&addconf_file).unwrap(), addconf);
    let trim_efa_file = efa_root.join("bb11001.trim.efa");
    let trimmed_efa = cmd_trimtoref_efa(
        efa_file.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        trim_efa_file.to_str().unwrap(),
    );
    assert_eq!(trimmed_efa.msas.len(), 2);
    assert_eq!(trimmed_efa.msas[0].seqs.len(), efa.msas[0].seqs.len());
    assert!(multi_sequence_get_col_count(&trimmed_efa.msas[0]) > 0);
    assert!(
        multi_sequence_get_col_count(&trimmed_efa.msas[0])
            <= multi_sequence_get_col_count(&efa.msas[0])
    );
    assert!(
        std::fs::read_to_string(&trim_efa_file)
            .unwrap()
            .contains("<BB11001")
    );
    let explode_prefix = efa_root.join("explode_").to_string_lossy().to_string();
    let exploded = cmd_efa_explode(
        efa_file.to_str().unwrap(),
        Some(&explode_prefix),
        Some(".fa"),
    );
    assert_eq!(exploded.len(), 2);
    for file_name in exploded {
        assert!(
            std::fs::read_to_string(&file_name)
                .unwrap()
                .starts_with(">1aab_")
        );
    }
    std::fs::remove_dir_all(&efa_root).unwrap();

    let consseq_file = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_consseq_{}.fa",
        std::process::id()
    ));
    cmd_consseq(
        "muscle/test_data/ref_alns/BB11001",
        consseq_file.to_str().unwrap(),
        Some("BB11001_cons"),
    );
    let consseq = std::fs::read_to_string(&consseq_file).unwrap();
    assert!(consseq.starts_with(">BB11001_cons\n"));
    assert!(consseq.lines().nth(1).unwrap().len() > 50);
    std::fs::remove_file(&consseq_file).unwrap();

    let squeeze_file = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_squeeze_{}.fa",
        std::process::id()
    ));
    let squeezed = cmd_squeeze_inserts(
        "muscle/test_data/ref_alns/BB11001",
        squeeze_file.to_str().unwrap(),
    );
    assert_eq!(squeezed.seqs.len(), 4);
    assert!(multi_sequence_get_col_count(&squeezed) <= multi_sequence_get_col_count(&efa.msas[0]));
    assert!(
        std::fs::read_to_string(&squeeze_file)
            .unwrap()
            .starts_with(">1aab_")
    );
    std::fs::remove_file(&squeeze_file).unwrap();

    let core_file = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_core_{}.txt",
        std::process::id()
    ));
    let core = cmd_core_blocks(
        "muscle/test_data/ref_alns/BB11001",
        core_file.to_str().unwrap(),
        4,
        4,
    );
    assert!(core.starts_with("core_blocks\t"));
    assert_eq!(std::fs::read_to_string(&core_file).unwrap(), core);
    std::fs::remove_file(&core_file).unwrap();

    *MEGA_STATE.lock().unwrap() = MegaState::default();
    mega_from_file("muscle/test_data/mega/BB11001.mega");
    let mega_msa_in = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_mega_msas_{}.fa",
        std::process::id()
    ));
    let mega_prefix = std::env::temp_dir()
        .join(format!(
            "muscle_rs_balibase_mega_msas_{}_",
            std::process::id()
        ))
        .to_string_lossy()
        .to_string();
    let mut mega_fasta = String::new();
    {
        let mega = MEGA_STATE.lock().unwrap();
        let max_len = mega.seqs.iter().map(String::len).max().unwrap();
        for (label, seq) in mega.labels.iter().zip(&mega.seqs) {
            let mut row = seq.clone();
            row.extend(std::iter::repeat_n('-', max_len - row.len()));
            mega_fasta.push_str(&seq_to_fasta_l2561(&row, label));
        }
    }
    std::fs::write(&mega_msa_in, mega_fasta).unwrap();
    let mega_outputs = cmd_mega_msas(mega_msa_in.to_str().unwrap(), &mega_prefix);
    assert_eq!(mega_outputs.len(), 8);
    assert_eq!(mega_outputs[0].0, format!("{mega_prefix}AA"));
    assert!(mega_outputs[0].1.starts_with(b">1aab__A\n"));
    assert!(
        std::fs::read(format!("{mega_prefix}AA"))
            .unwrap()
            .starts_with(b">1aab__A\n")
    );
    for (file_name, _bytes) in mega_outputs {
        std::fs::remove_file(file_name).unwrap();
    }
    std::fs::remove_file(&mega_msa_in).unwrap();

    let sw_tree = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_swdistmx_{}.nwk",
        std::process::id()
    ));
    let (tree, dist_mx) = cmd_swdistmx("muscle/test_data/fa/BB11001", sw_tree.to_str().unwrap());
    assert_eq!((tree.node_count + 1) / 2, 4);
    assert_eq!(dist_mx.len(), 4);
    for i in 0..4 {
        assert_eq!(dist_mx[i][i], f32::MAX);
        for j in 0..4 {
            assert_eq!(dist_mx[i][j], dist_mx[j][i]);
            if i != j {
                assert!(dist_mx[i][j].is_finite());
            }
        }
    }
    assert!(std::fs::read_to_string(&sw_tree).unwrap().contains(';'));
    let sw_tree_cli = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_swdistmx_cli_{}.nwk",
        std::process::id()
    ));
    let swdistmx_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-swdistmx",
            "muscle/test_data/fa/BB11001",
            "-output",
            sw_tree_cli.to_str().unwrap(),
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        swdistmx_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&swdistmx_cli.stdout),
        String::from_utf8_lossy(&swdistmx_cli.stderr)
    );
    assert!(swdistmx_cli.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(&sw_tree_cli).unwrap(),
        std::fs::read_to_string(&sw_tree).unwrap()
    );
    std::fs::remove_file(&sw_tree).unwrap();
    std::fs::remove_file(&sw_tree_cli).unwrap();

    let strip_direct = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_strip_gappy_direct_{}.fa",
        std::process::id()
    ));
    let strip_cli = std::env::temp_dir().join(format!(
        "muscle_rs_balibase_strip_gappy_cli_{}.fa",
        std::process::id()
    ));
    let (_discard_col_count, _discard_row_count) = cmd_strip_gappy(
        "muscle/test_data/ref_alns/BB11005",
        strip_direct.to_str().unwrap(),
        0.5,
        0.5,
    );
    let strip_gappy_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-strip_gappy",
            "muscle/test_data/ref_alns/BB11005",
            "-output",
            strip_cli.to_str().unwrap(),
            "-max_gap_fract",
            "0.5",
            "-max_gap_fract_row",
            "0.5",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        strip_gappy_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&strip_gappy_cli.stdout),
        String::from_utf8_lossy(&strip_gappy_cli.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&strip_cli).unwrap(),
        std::fs::read_to_string(&strip_direct).unwrap()
    );
    std::fs::remove_file(&strip_direct).unwrap();
    std::fs::remove_file(&strip_cli).unwrap();

    let qscore_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-qscore",
            "muscle/test_data/ref_alns/BB11001",
            "-ref",
            "muscle/test_data/ref_alns/BB11001",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(qscore_cli.status.success());
    assert_eq!(
        String::from_utf8(qscore_cli.stdout).unwrap(),
        "TC=1.0000 SP=1.0000\n"
    );
}

#[test]
fn high_risk_real_data_alignment_and_mega_paths() {
    let _global_guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    let root =
        std::env::temp_dir().join(format!("muscle_rs_high_risk_real_{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();

    for (cmd, extra_args, min_tc, min_sp) in [
        ("-super4", Vec::<&str>::new(), 0.99, 0.99),
        ("-super5", Vec::<&str>::new(), 0.98, 0.98),
        ("-super6", Vec::<&str>::new(), 0.99, 0.99),
        ("-super7", vec!["-shrub_size", "4"], 0.99, 0.99),
    ] {
        let output = root.join(format!("{}.fa", cmd.trim_start_matches('-')));
        let mut args = vec![
            cmd,
            "muscle/test_data/fa/BB11001",
            "-output",
            output.to_str().unwrap(),
            "-quiet",
        ];
        args.extend(extra_args);
        let run = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{cmd} stdout={} stderr={}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let out_msa = msa_from_fasta_file_l95(output.to_str().unwrap());
        assert_eq!(out_msa.seqs.len(), 4, "{cmd}");
        assert!(multi_sequence_get_col_count(&out_msa) > 0, "{cmd}");
        let (tc, sp) = cmd_qscore(
            output.to_str().unwrap(),
            "muscle/test_data/ref_alns/BB11001",
            false,
        );
        assert!(tc >= min_tc, "{cmd} tc={tc}");
        assert!(sp >= min_sp, "{cmd} sp={sp}");
    }

    let m3ensemble_out = root.join("m3ensemble.efa");
    let m3ensemble = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-m3ensemble",
            "muscle/test_data/fa/BB11001",
            "-output",
            m3ensemble_out.to_str().unwrap(),
            "-replicates",
            "1",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        m3ensemble.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&m3ensemble.stdout),
        String::from_utf8_lossy(&m3ensemble.stderr)
    );
    assert!(m3ensemble.stderr.is_empty());
    let m3ensemble_text = std::fs::read_to_string(&m3ensemble_out).unwrap();
    assert!(m3ensemble_text.starts_with("<blosum90:0.perturb0.delta0.1\n"));
    assert_eq!(m3ensemble_text.matches(">1aab_").count(), 1);
    assert!(m3ensemble_text.matches('\n').count() > 4);
    let m3ensemble_direct_out = root.join("m3ensemble_direct.efa");
    let m3ensemble_direct = cmd_m3ensemble(
        "muscle/test_data/fa/BB11001",
        m3ensemble_direct_out.to_str().unwrap(),
        Some(1),
    );
    assert_eq!(
        std::fs::read_to_string(&m3ensemble_direct_out).unwrap(),
        m3ensemble_direct
    );
    assert_eq!(m3ensemble_text, m3ensemble_direct);
    assert!(m3ensemble_direct.starts_with("<blosum90:0.perturb0.delta0.1\n"));

    let m3ensemble4_direct_out = root.join("m3ensemble_replicates4_direct.efa");
    let m3ensemble4_direct = cmd_m3ensemble(
        "muscle/test_data/fa/BB11001",
        m3ensemble4_direct_out.to_str().unwrap(),
        Some(4),
    );
    assert_eq!(
        std::fs::read_to_string(&m3ensemble4_direct_out).unwrap(),
        m3ensemble4_direct
    );
    let m3ensemble4_cli_out = root.join("m3ensemble_replicates4_cli.efa");
    let m3ensemble4_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-m3ensemble",
            "muscle/test_data/fa/BB11001",
            "-output",
            m3ensemble4_cli_out.to_str().unwrap(),
            "-replicates",
            "4",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        m3ensemble4_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&m3ensemble4_cli.stdout),
        String::from_utf8_lossy(&m3ensemble4_cli.stderr)
    );
    assert!(m3ensemble4_cli.stderr.is_empty());
    assert_eq!(
        std::fs::read_to_string(&m3ensemble4_cli_out).unwrap(),
        m3ensemble4_direct
    );
    let mut m3ensemble4 = Ensemble::default();
    ensemble_from_efa(&mut m3ensemble4, m3ensemble4_cli_out.to_str().unwrap());
    assert_eq!(
        m3ensemble4.msa_names,
        vec![
            "blosum90:0.perturb0.delta0.1",
            "blosum80:0.perturb0.delta0.1",
            "blosum70:0.perturb0.delta0.1",
            "blosum62:0.perturb0.delta0.1",
        ]
    );
    for (msa_name, msa) in m3ensemble4.msa_names.iter().zip(&m3ensemble4.msas) {
        assert_eq!(msa.seqs.len(), 4, "{msa_name}");
        assert!(multi_sequence_get_col_count(msa) > 0, "{msa_name}");
    }
    let m3ensemble4_qscore = cmd_qscore_efa(
        m3ensemble4_cli_out.to_str().unwrap(),
        "muscle/test_data/ref_alns/BB11001",
        1.0,
    );
    assert_eq!(m3ensemble4_qscore.matches("Q=").count(), 4);
    assert!(m3ensemble4_qscore.contains("blosum90:0.perturb0.delta0.1"));
    assert!(m3ensemble4_qscore.contains("blosum80:0.perturb0.delta0.1"));
    assert!(m3ensemble4_qscore.contains("blosum70:0.perturb0.delta0.1"));
    assert!(m3ensemble4_qscore.contains("blosum62:0.perturb0.delta0.1"));

    let super7_mega_out = root.join("super7_mega.fa");
    reset_rand(1);
    let (_s7m, _mega_guide_tree, super7_mega_log) = cmd_super7_mega(
        "muscle/test_data/mega/BB11001.mega",
        super7_mega_out.to_str().unwrap(),
        Some(4),
        None,
        None,
    );
    assert_eq!(super7_mega_log, "Done.\n");
    let super7_mega_msa = msa_from_fasta_file_l95(super7_mega_out.to_str().unwrap());
    assert_eq!(super7_mega_msa.seqs.len(), 4);
    assert!(multi_sequence_get_col_count(&super7_mega_msa) > 0);
    let super7_mega_cli = std::process::Command::new(env!("CARGO_BIN_EXE_muscle_rs"))
        .args([
            "-super7",
            "muscle/test_data/mega/BB11001.mega",
            "-mega",
            "-output",
            root.join("super7_mega_cli.fa").to_str().unwrap(),
            "-shrub_size",
            "4",
            "-quiet",
        ])
        .output()
        .unwrap();
    assert!(
        super7_mega_cli.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&super7_mega_cli.stdout),
        String::from_utf8_lossy(&super7_mega_cli.stderr)
    );
    assert_eq!(
        String::from_utf8(super7_mega_cli.stdout).unwrap(),
        "Done.\n"
    );
    let super7_mega_cli_msa =
        msa_from_fasta_file_l95(root.join("super7_mega_cli.fa").to_str().unwrap());
    assert_eq!(super7_mega_cli_msa.seqs.len(), super7_mega_msa.seqs.len());
    assert!(multi_sequence_get_col_count(&super7_mega_cli_msa) > 0);
    for (cli_seq, direct_seq) in super7_mega_cli_msa.seqs.iter().zip(&super7_mega_msa.seqs) {
        assert_eq!(cli_seq.label, direct_seq.label);
        assert_eq!(
            sequence_get_seq_as_string(cli_seq).replace('-', ""),
            sequence_get_seq_as_string(direct_seq).replace('-', "")
        );
    }

    let source_mega = std::fs::read_to_string("muscle/test_data/mega/BB11001.mega").unwrap();
    let two_chain_mega = root.join("BB11001.first_two.mega");
    let mut sliced = String::new();
    let mut keep_chain = true;
    for line in source_mega.lines() {
        if let Some(rest) = line.strip_prefix("mega\t") {
            let mut fields = rest.split('\t').collect::<Vec<_>>();
            assert!(fields.len() >= 3);
            fields[1] = "2";
            sliced.push_str("mega\t");
            sliced.push_str(&fields.join("\t"));
            sliced.push('\n');
            continue;
        }
        if let Some(rest) = line.strip_prefix("chain\t") {
            let chain_index = rest.split('\t').next().unwrap().parse::<uint>().unwrap();
            keep_chain = chain_index < 2;
        }
        if keep_chain {
            sliced.push_str(line);
            sliced.push('\n');
        }
    }
    std::fs::write(&two_chain_mega, sliced).unwrap();
    let mega2_out = root.join("mega2.fa");
    *MEGA_STATE.lock().unwrap() = MegaState::default();
    let (score, pi, text) = cmd_mega2(
        two_chain_mega.to_str().unwrap(),
        mega2_out.to_str().unwrap(),
        None,
        None,
        None,
        None,
    );
    assert!(score.is_finite());
    assert!(!pi.path.is_empty());
    let (label_a, label_b, seq_a, seq_b) = {
        let mega = MEGA_STATE.lock().unwrap();
        assert_eq!(mega.profiles.len(), 2);
        assert_eq!(mega.labels.len(), 2);
        (
            mega.labels[0].clone(),
            mega.labels[1].clone(),
            mega.seqs[0].clone(),
            mega.seqs[1].clone(),
        )
    };
    assert_eq!(get_na(&pi.path), seq_a.len() as uint);
    assert_eq!(get_nb(&pi.path), seq_b.len() as uint);
    let mega2_text = text.unwrap();
    assert_eq!(std::fs::read_to_string(&mega2_out).unwrap(), mega2_text);
    let lines = mega2_text.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], format!(">{label_a}"));
    assert_eq!(lines[2], format!(">{label_b}"));
    assert_eq!(lines[1].len(), lines[3].len());
    assert_eq!(lines[1].replace('-', ""), seq_a);
    assert_eq!(lines[3].replace('-', ""), seq_b);

    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn refine_kimura_and_conf_range_helpers_match_cpp_logic() {
    let _guard = RNG_TEST_LOCK.lock().unwrap();

    let mut weights = vec![2.0_f32, 3.0, 5.0];
    normalize_weights(&mut weights);
    assert_eq!(weights, vec![0.2, 0.3, 0.5]);

    reset_rand(1);
    assert_eq!(
        split_indexes3(10),
        vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7, 8], vec![9]]
    );

    assert!((get_kimura_dist(1.0) - 0.0).abs() < 1e-6);
    let p = 0.25_f32;
    let expected = -(1.0 - p - (p * p) / 5.0).ln();
    assert!((get_kimura_dist(0.75) - expected).abs() < 1e-6);
    assert_eq!(get_kimura_dist(0.25), 1.95);
    assert_eq!(get_kimura_dist(0.06), 10.0);

    let labels = vec!["s1".to_string(), "s2".to_string(), "s3".to_string()];
    let seqs = vec!["AAAA".to_string(), "AAAT".to_string(), "----".to_string()];
    let mut ms = MultiSequence::default();
    multi_sequence_from_strings(&mut ms, &labels, &seqs);
    let dist_mx = get_kimura_dist_mx(&ms);
    assert_eq!(dist_mx.len(), 3);
    assert_eq!(dist_mx[0][0], 0.0);
    assert_eq!(dist_mx[1][1], 0.0);
    assert_eq!(dist_mx[2][2], 0.0);
    assert_eq!(dist_mx[0][1], dist_mx[1][0]);
    assert!((dist_mx[0][1] - get_kimura_dist(0.75)).abs() < 1e-6);
    assert_eq!(dist_mx[0][2], dist_mx[2][0]);
    assert!((dist_mx[0][2] - get_kimura_dist(0.0)).abs() < 1e-6);

    let mut seen_pairs = Vec::new();
    let dist_mx_viterbi = get_kimura_dist_mx_viterbi(&ms, |seqi, li, seqj, lj| {
        seen_pairs.push((
            std::str::from_utf8(&seqi[..li as usize])
                .unwrap()
                .to_string(),
            std::str::from_utf8(&seqj[..lj as usize])
                .unwrap()
                .to_string(),
        ));
        PathInfo {
            path: "MMMM".to_string(),
            ..PathInfo::default()
        }
    });
    assert_eq!(
        seen_pairs,
        vec![
            ("AAAT".to_string(), "AAAA".to_string()),
            ("----".to_string(), "AAAA".to_string()),
            ("----".to_string(), "AAAT".to_string()),
        ]
    );
    assert_eq!(dist_mx_viterbi[0][0], 0.0);
    assert_eq!(dist_mx_viterbi[1][1], 0.0);
    assert_eq!(dist_mx_viterbi[2][2], 0.0);
    assert_eq!(dist_mx_viterbi[0][1], dist_mx_viterbi[1][0]);
    assert!((dist_mx_viterbi[0][1] - get_kimura_dist(0.75)).abs() < 1e-6);
    assert_eq!(dist_mx_viterbi[0][2], dist_mx_viterbi[2][0]);
    assert!((dist_mx_viterbi[0][2] - get_kimura_dist(0.0)).abs() < 1e-6);

    let mut prot_dist_calls = Vec::new();
    let (prot_dist_mx, prot_labels) = make_dist_mx(&ms, |seqi, seqj, col_count| {
        prot_dist_calls.push((seqi.to_string(), seqj.to_string(), col_count));
        (seqi.bytes().filter(|&c| c == b'A').count() + seqj.bytes().filter(|&c| c == b'T').count())
            as f64
    });
    assert_eq!(prot_labels, labels);
    assert_eq!(
        prot_dist_calls,
        vec![
            ("AAAT".to_string(), "AAAA".to_string(), 4),
            ("----".to_string(), "AAAA".to_string(), 4),
            ("----".to_string(), "AAAT".to_string(), 4),
        ]
    );
    assert_eq!(prot_dist_mx[0][0], 0.0);
    assert_eq!(prot_dist_mx[1][1], 0.0);
    assert_eq!(prot_dist_mx[2][2], 0.0);
    assert_eq!(prot_dist_mx[1][0], 3.0);
    assert_eq!(prot_dist_mx[0][1], 3.0);
    assert_eq!(prot_dist_mx[2][0], 0.0);
    assert_eq!(prot_dist_mx[0][2], 0.0);
    assert_eq!(prot_dist_mx[2][1], 1.0);
    assert_eq!(prot_dist_mx[1][2], 1.0);

    let (confs, los, his) = get_conf_ranges_ungapped(b"AA--BBB.C", 9);
    assert_eq!(confs, b"A-B.C".to_vec());
    assert_eq!(los, vec![0, 2, 2, 5, 5]);
    assert_eq!(his, vec![1, 1, 4, 4, 5]);

    let (confs, los, his) = get_conf_ranges_gapped(b"999--001", 8);
    assert_eq!(confs, b"9-01".to_vec());
    assert_eq!(los, vec![0, 3, 5, 7]);
    assert_eq!(his, vec![2, 4, 6, 7]);

    let head = html_head();
    assert!(head.starts_with("<!DOCTYPE html>\n<html lang=\"en\">\n"));
    assert!(head.contains(".Style0 {background-color: #ff6464;}"));
    assert!(head.contains(".Style9 {background-color: #98e8f9;}"));
    let foot = html_foot();
    assert!(foot.contains("Confidence high"));
    assert!(foot.contains("<span class=\"Style9\">9</span><span class=\"Style8\">8</span>"));

    let html_file =
        std::env::temp_dir().join(format!("muscle_rs_letterconf_{}.html", std::process::id()));
    let mut ref_msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut ref_msa,
        &["s1".to_string(), "longer".to_string()],
        &["AC-G".to_string(), "TTAA".to_string()],
    );
    let mut conf_msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut conf_msa,
        &["s1".to_string(), "longer".to_string()],
        &["99-0".to_string(), "8888".to_string()],
    );
    write_letter_conf_html(html_file.to_str().unwrap(), &ref_msa, &conf_msa);
    let html = std::fs::read_to_string(&html_file).unwrap();
    assert!(html.contains("<span class=\"Style9\">AC</span>"));
    assert!(html.contains("<span class=\"StyleG\">-</span>"));
    assert!(html.contains("<span class=\"Style0\">G</span>"));
    assert!(html.contains("<span class=\"Style8\">TTAA</span>"));
    std::fs::remove_file(&html_file).unwrap();
}

#[test]
fn greedy_rects_matches_scan_and_consumption_order() {
    assert_eq!(get_overlap(1, 5, 3, 7), 3);
    assert_eq!(get_overlap(1, 2, 3, 4), 0);
    assert_eq!(get_overlap(3, 3, 3, 3), 1);

    assert!(in_bounds(0, 0, 2, 3));
    assert!(!in_bounds(-1, 0, 2, 3));
    assert!(!in_bounds(2, 0, 2, 3));
    assert!(!in_bounds(0, 3, 2, 3));

    let mat = vec![
        vec![true, true, false, true],
        vec![true, true, false, true],
        vec![false, false, true, true],
    ];
    assert_eq!(
        greedy_rects(&mat, 1, 1),
        vec![
            Rect {
                top: 0,
                left: 0,
                width: 2,
                height: 2,
            },
            Rect {
                top: 0,
                left: 3,
                width: 1,
                height: 3,
            },
            Rect {
                top: 2,
                left: 2,
                width: 1,
                height: 1,
            },
        ]
    );
    assert_eq!(
        greedy_rects(&mat, 2, 2),
        vec![Rect {
            top: 0,
            left: 0,
            width: 2,
            height: 2,
        }]
    );
    assert!(greedy_rects(&[vec![true], vec![true, false]], 1, 1).is_empty());

    let core_in = std::env::temp_dir().join(format!(
        "muscle_rs_core_blocks_in_{}.fa",
        std::process::id()
    ));
    let core_out = std::env::temp_dir().join(format!(
        "muscle_rs_core_blocks_out_{}.txt",
        std::process::id()
    ));
    std::fs::write(&core_in, b">s1\nAB--\n>s2\nAB-D\n>s3\nABCD\n").unwrap();
    assert_eq!(
        cmd_core_blocks(core_in.to_str().unwrap(), core_out.to_str().unwrap(), 2, 2),
        "core_blocks\t1\nblock\t0\t2\t3\nAB\t0\ts1\nAB\t0\ts2\nAB\t0\ts3\n"
    );
    assert_eq!(
        std::fs::read_to_string(&core_out).unwrap(),
        "core_blocks\t1\nblock\t0\t2\t3\nAB\t0\ts1\nAB\t0\ts2\nAB\t0\ts3\n"
    );
    assert_eq!(
        cmd_core_blocks(core_in.to_str().unwrap(), "", 2, 2),
        "core_blocks\t1\nblock\t0\t2\t3\nAB\t0\ts1\nAB\t0\ts2\nAB\t0\ts3\n"
    );
    std::fs::remove_file(&core_in).unwrap();
    std::fs::remove_file(&core_out).unwrap();
}

#[test]
fn read_first_char_and_masm_col_methods_match_cpp_logic() {
    let _guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let tmp = std::env::temp_dir().join(format!(
        "muscle_rs_read_first_char_{}.efa",
        std::process::id()
    ));
    std::fs::write(&tmp, b"<real\n>seq\nACD\n").unwrap();
    assert_eq!(read_first_char(tmp.to_str().unwrap()), '<');
    std::fs::remove_file(&tmp).unwrap();

    set_alpha_l209(ALPHA::ALPHA_Amino);
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.feature_names = vec!["aa".to_string(), "ss".to_string()];
        mega.alpha_sizes = vec![20, 3];
        mega.log_odds_mx_vec = vec![
            (0..20)
                .map(|i| {
                    (0..20)
                        .map(|j| if i == j { 2.0_f32 } else { -1.0_f32 })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            vec![
                vec![1.0, 0.0, -1.0],
                vec![0.5, 1.5, -0.5],
                vec![-1.0, 0.0, 2.0],
            ],
        ];
    }
    assert_eq!(mega_get_feature_name(1), "ss");
    assert_eq!(mega_get_alpha_size(0), 20);

    let masm = MASM {
        feature_count: 2,
        alpha_sizes: vec![20, 3],
        aa_feature_idx: 0,
        ..MASM::default()
    };
    let mut col = MASMCol {
        masm: Some(Box::new(masm.clone())),
        col_index: 7,
        freqs_vec: vec![
            vec![
                0.6, 0.1, 0.05, 0.05, 0.05, 0.05, 0.02, 0.01, 0.01, 0.01, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            vec![0.25, 0.5, 0.25],
        ],
        ..MASMCol::default()
    };
    assert_eq!(masm_col_get_consensus_aa_char(&col), 'A');
    col.freqs_vec[0][0] = 0.4;
    col.freqs_vec[0][1] = 0.3;
    col.freqs_vec[0][2] = 0.2;
    assert_eq!(masm_col_get_consensus_aa_char(&col), 'a');
    for freq in &mut col.freqs_vec[0] {
        *freq = 0.0;
    }
    col.freqs_vec[0][3] = 0.49;
    assert_eq!(masm_col_get_consensus_aa_char(&col), '-');

    col.freqs_vec[0] = vec![
        0.6, 0.1, 0.05, 0.05, 0.05, 0.05, 0.02, 0.01, 0.01, 0.01, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0,
    ];
    masm_col_set_score_vec(&mut col);
    assert_eq!(col.scores_vec.len(), 2);
    assert!((col.scores_vec[0][0] - 0.85).abs() < 1e-6);
    assert!((col.scores_vec[0][1] + 0.65).abs() < 1e-6);
    assert!((col.scores_vec[1][1] - 0.75).abs() < 1e-6);
    assert_eq!(masm_col_get_aa_scores(&col), &col.scores_vec[0]);
    assert!((masm_col_get_match_score_mega_profile_pos(&col, &[0, 1]) - 1.6).abs() < 1e-6);

    let mut out = TextFile::default();
    masm_col_to_file(&col, &mut out);
    text_file_rewind(&mut out);
    let mut parsed = MASMCol {
        masm: Some(Box::new(masm)),
        ..MASMCol::default()
    };
    masm_col_from_file(&mut parsed, &mut out);
    assert_eq!(parsed.col_index, 7);
    assert_eq!(parsed.freqs_vec.len(), 2);
    assert_eq!(parsed.scores_vec.len(), 2);
    assert_eq!(parsed.freqs_vec[1], vec![0.25, 0.5, 0.25]);
    assert!((parsed.scores_vec[1][2] - 0.25).abs() < 1e-6);
    let col_log = masm_col_log_me(&col);
    assert!(col_log.contains("MSAMCol[7]"));
    assert!(col_log.contains(" A=0.85"));
    assert!(!col_log.contains(" A=0.850"));
}

#[test]
fn blosum_and_mega_from_msa_aa_only_match_cpp_logic() {
    let _guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    let _rng_guard = RNG_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    assert!((get_blosum_score_letters(0, 0) - 1.9646).abs() < 1e-6);
    assert!((get_blosum_score_letters(18, 19) - 1.0771).abs() < 1e-6);
    assert_eq!(get_blosum_score_letters(20, 0), 0.0);
    assert!((get_blosum_score_chars(b'A', b'C') + 0.2043).abs() < 1e-6);
    assert!((get_blosum_score_chars(b'w', b'Y') - 1.0771).abs() < 1e-6);
    assert_eq!(get_blosum_score_chars(b'X', b'A'), 0.0);
    let subst = get_subst_mx_letter_blosum(62);
    assert!((subst[0][0] - 1.9646).abs() < 1e-6);
    assert!((subst[18][19] - 1.0771).abs() < 1e-6);
    let subst90 = get_subst_mx_letter_blosum(90);
    let subst80 = get_subst_mx_letter_blosum(80);
    let subst70 = get_subst_mx_letter_blosum(70);
    assert!((subst90[0][0] - 4.9179).abs() < 1e-6);
    assert!((subst80[0][0] - 4.5099).abs() < 1e-6);
    assert!((subst70[0][0] - 4.2364).abs() < 1e-6);
    assert_ne!(subst90, subst);
    assert_ne!(subst80, subst);
    assert_ne!(subst70, subst);
    let (gap_open, center) = get_gap_params_blosum(62, 2);
    assert!((gap_open + 6.6825562).abs() < 1e-6);
    assert!((center - 0.59377569).abs() < 1e-6);
    let (gap_open, center) = get_gap_params_blosum(90, 3);
    assert!((gap_open + 7.0647068).abs() < 1e-6);
    assert!((center - 1.2546233).abs() < 1e-6);
    let blosum = get_blosum62_log_odds_letter_mx();
    assert_eq!(blosum.len(), 20);
    assert_eq!(blosum[0].len(), 20);
    assert!((blosum[19][19] - 3.2975).abs() < 1e-6);
    let mut ap = M3AlnParams {
        gap_open: f32::MAX,
        center: f32::MAX,
        nuc_match_score: f32::MAX,
        nuc_mismatch_score: f32::MAX,
        term_gap_open: f32::MAX,
        term_gap_ext: f32::MAX,
        linkage: "min".to_string(),
        tree_iters: 1,
        kmer_dist: "66".to_string(),
        min_std_rand: 1,
        ..M3AlnParams::default()
    };
    m3_aln_params_set_blosum(&mut ap, 62, 0, f32::MAX, f32::MAX, 0, 0.0, 0.0, 0.0);
    assert!(ap.ready);
    assert!(ap.center_added);
    assert!((ap.gap_open + 6.0).abs() < 1e-6);
    assert!((ap.center - 0.79999995).abs() < 1e-6);
    assert!((ap.subst_mx_letter[0][0] - (subst[0][0] + ap.center)).abs() < 1e-6);
    let printed = m3_aln_params_print(&ap);
    assert!(printed.contains("m_GapOpen=-6 m_Center=0.8"));
    assert!(printed.contains("kmerdist=66"));
    let print_probe = M3AlnParams {
        gap_open: 1234.0,
        center: 0.00125,
        linkage: "min".to_string(),
        tree_iters: 1,
        kmer_dist: "66".to_string(),
        perturb_seed: 7,
        perturb_subst_mx_delta: 1234.0,
        perturb_gap_params_delta: 0.00125,
        perturb_dist_mx_delta: 1.0,
        ..M3AlnParams::default()
    };
    let print_probe_text = m3_aln_params_print(&print_probe);
    assert!(
        print_probe_text.contains(
            "m_GapOpen=1234 m_Center=0.00125 linkage=min treeiters=1 kmerdist=66\n perturb(7) substmx=1.23e+03, gapparams=0.00125, distmx=1"
        ),
        "{print_probe_text:?}"
    );

    let mut mx = [[0.0f32; 20]; 20];
    mx[0][0] = 1.0;
    mx[0][1] = 2.0;
    let mut ap2 = M3AlnParams {
        linkage: "min".to_string(),
        tree_iters: 1,
        kmer_dist: "66".to_string(),
        min_std_rand: 1,
        ..M3AlnParams::default()
    };
    m3_aln_params_update_mx(&mut ap2, &mx, -3.0, 0.5);
    assert_eq!(ap2.subst_mx_letter[0][0], 1.5);
    assert_eq!(ap2.subst_mx_letter[0][1], 2.5);
    assert!(ap2.ready);
    m3_aln_params_init_perturb(&mut ap2, 5);
    assert_eq!(m3_aln_params_get_rand(&mut ap2), 241355);
    let mut x = 10.0;
    m3_aln_params_perturb1_l17(&mut ap2, &mut x, 0.5);
    assert_ne!(x, 10.0);
    let mut ap3 = M3AlnParams {
        perturb_seed: 11,
        perturb_subst_mx_delta: 0.1,
        perturb_gap_params_delta: 0.1,
        perturb_dist_mx_delta: 0.1,
        gap_open: -4.0,
        center: 0.2,
        subst_mx_letter: subst,
        min_std_rand: 1,
        ..M3AlnParams::default()
    };
    m3_aln_params_perturb_my_params(&mut ap3);
    assert!(ap3.perturb_gap_params_done);
    assert!(ap3.perturb_subst_mx_done);
    assert_ne!(ap3.gap_open, -4.0);
    let mut dist_mx = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    m3_aln_params_init_perturb(&mut ap3, 11);
    m3_aln_params_perturb_dist_mx(&mut ap3, &mut dist_mx);
    assert_eq!(dist_mx[0][1], dist_mx[1][0]);
    assert_ne!(dist_mx[0][1], 1.0);

    let mut cmd_ap = M3AlnParams {
        min_std_rand: 1,
        ..M3AlnParams::default()
    };
    let cmd_log = m3_aln_params_set_from_cmd_line(
        &mut cmd_ap,
        false,
        true,
        None,
        Some(-5.5),
        Some(0.25),
        Some(62),
        Some(0),
        Some(0),
        Some("avg"),
        Some("33"),
        Some(3),
    )
    .unwrap();
    assert!(cmd_ap.ready);
    assert!(cmd_ap.center_added);
    assert_eq!(cmd_ap.gap_open, -5.5);
    assert_eq!(cmd_ap.center, 0.25);
    assert_eq!(cmd_ap.linkage, "avg");
    assert_eq!(cmd_ap.kmer_dist, "33");
    assert_eq!(cmd_ap.tree_iters, 3);
    assert_eq!(cmd_ap.perturb_seed, 0);
    assert!(cmd_log.contains("m_GapOpen=-5.5 m_Center=0.25 linkage=avg"));

    let subst_file =
        std::env::temp_dir().join(format!("muscle_rs_m3_subst_{}.tsv", std::process::id()));
    let aa = "ACDEFGHIKLMNPQRSTVWY"
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>();
    let mut subst_text = String::new();
    subst_text.push('\t');
    subst_text.push_str(&aa.join("\t"));
    subst_text.push('\n');
    for (i, row_aa) in aa.iter().enumerate() {
        subst_text.push_str(row_aa);
        for j in 0..20 {
            subst_text.push('\t');
            subst_text.push_str(&(100 + i * 20 + j).to_string());
        }
        subst_text.push('\n');
    }
    std::fs::write(&subst_file, subst_text).unwrap();
    let mut file_ap = M3AlnParams {
        min_std_rand: 1,
        ..M3AlnParams::default()
    };
    assert!(
        m3_aln_params_set_from_cmd_line(
            &mut file_ap,
            false,
            false,
            Some(subst_file.to_str().unwrap()),
            Some(-2.0),
            Some(0.5),
            None,
            None,
            Some(7),
            None,
            None,
            None,
        )
        .is_none()
    );
    assert!(file_ap.ready);
    assert_eq!(file_ap.linkage, "biased");
    assert_eq!(file_ap.kmer_dist, "66");
    assert_eq!(file_ap.tree_iters, 1);
    assert_ne!(file_ap.gap_open, -2.0);
    assert_ne!(file_ap.center, 0.5);
    assert_ne!(file_ap.gap_open, -2.0 + 0.5);
    assert!(file_ap.perturb_gap_params_done);
    assert!(file_ap.perturb_subst_mx_done);
    assert_ne!(file_ap.subst_mx_letter[0][0], 100.5);
    std::fs::remove_file(&subst_file).unwrap();

    let mut refine_msa = MultiSequence::default();
    multi_sequence_from_strings(
        &mut refine_msa,
        &["r0".to_string(), "r1".to_string(), "r2".to_string()],
        &["AC".to_string(), "AD".to_string(), "AE".to_string()],
    );
    let mut refined = MultiSequence::default();
    reset_rand(1);
    let mut nw_calls = 0;
    let refine_log = m3_refine(
        &refine_msa,
        &ap,
        &[0.2, 0.3, 0.5],
        &mut refined,
        |_cm, p1, p2| {
            nw_calls += 1;
            format!("{}x{}", p1.pps.len(), p2.pps.len())
        },
    );
    assert_eq!(nw_calls, 96);
    assert!(refined.seqs.is_empty());
    assert_eq!(refine_log.matches("Path01=").count(), 32);
    assert!(refine_log.contains("\nPath01=2x2\nPath02=2x2\nPath12=2x2\n"));

    let m3refine_in =
        std::env::temp_dir().join(format!("muscle_rs_cmd_m3refine_{}.fa", std::process::id()));
    std::fs::write(&m3refine_in, b">r0\nAC\n>r1\nAD\n>r2\nAE\n").unwrap();
    let (cmd_msa, cmd_labels, cmd_weights, cmd_tree, cmd_ap, cmd_refined, cmd_log) = cmd_m3refine(
        m3refine_in.to_str().unwrap(),
        |u, tree| {
            assert_eq!(
                u.labels,
                vec!["r0".to_string(), "r1".to_string(), "r2".to_string()]
            );
            tree_create(
                tree,
                3,
                1,
                &[0, 3],
                &[1, 2],
                &[0.2, 0.3],
                &[0.2, 0.3],
                &[0, 1, 2],
                &["r0".to_string(), "r1".to_string(), "r2".to_string()],
            );
        },
        |ap| {
            ap.ready = true;
            ap.gap_open = -6.0;
            ap.center = 0.8;
            ap.subst_mx_letter = subst;
        },
        |msa, ap, weights, refined_msa| {
            assert_eq!(msa.seqs.len(), 3);
            assert!(ap.ready);
            assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-6);
            multi_sequence_copy(refined_msa, msa);
            "cmd-m3refine\n".to_string()
        },
    );
    assert_eq!(cmd_labels, vec!["r0", "r1", "r2"]);
    assert_eq!(cmd_msa.seqs.len(), 3);
    assert_eq!(cmd_tree.node_count, 5);
    assert!(cmd_ap.ready);
    assert_eq!(cmd_refined.seqs.len(), 3);
    assert_eq!(cmd_log, "cmd-m3refine\n");
    assert!((cmd_weights.iter().sum::<f32>() - 1.0).abs() < 1e-6);
    std::fs::remove_file(&m3refine_in).unwrap();

    let mut muscle3_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut muscle3_input,
        &["m0".to_string(), "m1".to_string(), "m2".to_string()],
        &[
            "ACDEFGH".to_string(),
            "ACDACDH".to_string(),
            "GGGGGGG".to_string(),
        ],
    );
    set_global_input_ms(&muscle3_input);
    let mut muscle3_ap = M3AlnParams {
        subst_mx_letter: subst,
        gap_open: -6.0,
        ready: true,
        linkage: "biased".to_string(),
        tree_iters: 1,
        kmer_dist: "33".to_string(),
        ..M3AlnParams::default()
    };
    let mut m3 = Muscle3::default();
    let mut m3_upgma_calls = Vec::new();
    let mut m3_pp_calls = Vec::new();
    let final_msa = muscle3_run(
        &mut m3,
        &muscle3_ap,
        &muscle3_input,
        |u, linkage, tree| {
            m3_upgma_calls.push((linkage.to_string(), u.dist_mx.clone()));
            tree_create(
                tree,
                3,
                1,
                &[0, 3],
                &[1, 2],
                &[0.2, 0.3],
                &[0.2, 0.3],
                &[0, 1, 2],
                &["m0".to_string(), "m1".to_string(), "m2".to_string()],
            );
        },
        |pp3, input_seqs, weights, guide_tree| {
            assert!(pp3.ap.as_ref().unwrap().ready);
            assert_eq!(guide_tree.node_count, 5);
            assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-6);
            m3_pp_calls.push(weights.to_vec());
            input_seqs.clone()
        },
    );
    assert_eq!(m3_upgma_calls.len(), 2);
    assert_eq!(m3_pp_calls.len(), 2);
    assert_eq!(m3.labels, vec!["m0", "m1", "m2"]);
    assert_eq!(m3.final_msa.as_ref().unwrap(), &final_msa);
    assert_eq!(
        final_msa
            .seqs
            .iter()
            .map(|seq| seq.label.clone())
            .collect::<Vec<_>>(),
        vec!["m0".to_string(), "m1".to_string(), "m2".to_string()]
    );
    assert_eq!(m3_upgma_calls[0].0, "biased");
    assert!((m3_upgma_calls[0].1[0][1] - 1.5).abs() < 1e-6);
    assert!((m3_upgma_calls[1].1[0][1] - get_kimura_dist(4.0 / 7.0)).abs() < 1e-6);
    muscle3_ap.kmer_dist = "bad".to_string();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut bad_m3 = Muscle3::default();
            muscle3_run(
                &mut bad_m3,
                &muscle3_ap,
                &muscle3_input,
                |_u, _linkage, _tree| {},
                |_pp3, input_seqs, _weights, _tree| input_seqs.clone(),
            );
        }))
        .is_err()
    );

    let mut run_ro_input = MultiSequence::default();
    multi_sequence_from_strings(
        &mut run_ro_input,
        &["ro0".to_string(), "ro1".to_string(), "ro2".to_string()],
        &["A".to_string(), "C".to_string(), "D".to_string()],
    );
    let mut run_ro_m3 = Muscle3 {
        ap: Some(M3AlnParams {
            subst_mx_letter: subst,
            gap_open: -6.0,
            ready: true,
            ..M3AlnParams::default()
        }),
        ..Muscle3::default()
    };
    let mut run_ro_paths = Vec::new();
    let mut run_ro_weights = Vec::new();
    reset_rand(1);
    let run_ro_msa = muscle3_run_ro(
        &mut run_ro_m3,
        &M3AlnParams::default(),
        &run_ro_input,
        |_cm, prof_k, accumulated_prof| {
            assert_eq!(prof_k.pps.len(), 1);
            assert_eq!(accumulated_prof.pps.len(), 1);
            run_ro_paths.push("M".to_string());
            "M".to_string()
        },
        |prof_k, w1, accumulated_prof, wn, _subst, _gap_open, path, combined_prof| {
            assert_eq!(path, "M");
            assert_eq!(prof_k.pps.len(), 1);
            assert_eq!(accumulated_prof.pps.len(), 1);
            run_ro_weights.push((w1, wn));
            *combined_prof = accumulated_prof.clone();
        },
    );
    assert_eq!(run_ro_paths, vec!["M".to_string(), "M".to_string()]);
    assert!((run_ro_weights[0].0 - 0.5).abs() < 1e-6);
    assert!((run_ro_weights[0].1 - 0.5).abs() < 1e-6);
    assert!((run_ro_weights[1].0 - 0.2).abs() < 1e-6);
    assert!((run_ro_weights[1].1 - 0.8).abs() < 1e-6);
    assert_eq!(run_ro_m3.final_msa.as_ref().unwrap(), &run_ro_msa);
    assert!(multi_sequence_is_aligned(&run_ro_msa));
    let mut run_ro_labels = run_ro_msa
        .seqs
        .iter()
        .map(|seq| seq.label.clone())
        .collect::<Vec<_>>();
    run_ro_labels.sort();
    assert_eq!(
        run_ro_labels,
        vec!["ro0".to_string(), "ro1".to_string(), "ro2".to_string()]
    );
    muscle3_write_msa(&Muscle3::default(), "");
    assert!(
        std::panic::catch_unwind(|| muscle3_write_msa(&Muscle3::default(), ".tmp/no-final.fa"))
            .is_err()
    );
    let muscle3_out = ".tmp/muscle3-write.fa";
    std::fs::create_dir_all(".tmp").unwrap();
    muscle3_write_msa(&run_ro_m3, muscle3_out);
    assert_eq!(
        std::fs::read_to_string(muscle3_out)
            .unwrap()
            .matches('>')
            .count(),
        3
    );
    std::fs::remove_file(muscle3_out).unwrap();

    let m3select_in =
        std::env::temp_dir().join(format!("muscle_rs_cmd_m3select_{}.fa", std::process::id()));
    let m3select_out =
        std::env::temp_dir().join(format!("muscle_rs_cmd_m3select_{}.out", std::process::id()));
    std::fs::write(
        &m3select_in,
        b">a\nACDEFGHIKLMNPQRSTVWY\n>b\nACDEFGHIKLMNPQRSAVWY\n>c\nGGGGGGGGGGGGGGGGGGGG\n",
    )
    .unwrap();
    let selected = cmd_m3select(
        m3select_in.to_str().unwrap(),
        m3select_out.to_str().unwrap(),
        Some(1),
    );
    assert_eq!(selected.matches('>').count(), 3);
    assert!(selected.contains(">a\n"));
    assert_eq!(std::fs::read_to_string(&m3select_out).unwrap(), selected);
    std::fs::remove_file(&m3select_in).unwrap();
    std::fs::remove_file(&m3select_out).unwrap();

    let mut seq_a = Sequence::default();
    let mut seq_b = Sequence::default();
    sequence_from_string(&mut seq_a, "a", "ACX");
    sequence_from_string(&mut seq_b, "b", "cYW");
    let seq_mx = make_blosum62_s_mx_l30(&seq_a, &seq_b);
    assert_eq!(seq_mx.name, "BlosumS");
    assert_eq!(seq_mx.row_count, 3);
    assert_eq!(seq_mx.col_count, 3);
    assert!((seq_mx.data[0][0] + 0.2043).abs() < 1e-6);
    assert!((seq_mx.data[1][1] + 1.2036).abs() < 1e-6);
    assert_eq!(seq_mx.data[2][2], 0.0);
    let str_mx = make_blosum62_s_mx_l54("AwX", "CY");
    assert_eq!(str_mx.row_count, 3);
    assert_eq!(str_mx.col_count, 2);
    assert!((str_mx.data[0][0] + 0.2043).abs() < 1e-6);
    assert!((str_mx.data[1][1] - 1.0771).abs() < 1e-6);
    assert_eq!(str_mx.data[2][0], 0.0);
    let mut log_odds_mx = vec![vec![0.0; 20]; 20];
    log_odds_mx[0][1] = 2.0;
    log_odds_mx[1][1] = 5.0;
    log_odds_mx[0][19] = 19.0;
    let los_mx = make_log_odds_s_mx(&seq_a, &seq_b, &log_odds_mx);
    assert_eq!(los_mx.name, "LOS");
    assert_eq!(los_mx.row_count, 3);
    assert_eq!(los_mx.col_count, 3);
    assert_eq!(los_mx.data[0][0], 2.0);
    assert_eq!(los_mx.data[1][0], 5.0);
    assert_eq!(los_mx.data[0][1], 19.0);
    assert_eq!(los_mx.data[2][2], 0.0);
    let mut mem = XDPMem::default();
    let (lo_score, lo_i, lo_j, len_i, len_j, lo_path) =
        sw_fast_seqs_lo(&mut mem, &log_odds_mx, &seq_a, &seq_b, -3.0, -1.0);
    assert_eq!(lo_score, 19.0);
    assert_eq!((lo_i, lo_j, len_i, len_j), (0, 1, 1, 1));
    assert_eq!(lo_path, "M");
    reset_rand(1);
    assert_eq!(get_rand_seq_l130(), "LWK");
    reset_rand(1);
    assert_eq!(get_rand_seq_l208(), "LWK");
    reset_rand(1);
    assert_eq!(get_random_seq(3, 8), "LWK");
    reset_rand(1);
    let rows = get_rand_rows();
    assert!((1..=5).contains(&rows.len()));
    let row_len = rows[0].len();
    assert!((3..=7).contains(&row_len));
    assert!(rows.iter().all(|row| row.len() == row_len));
    for col in 0..row_len {
        assert!(rows.iter().any(|row| row.as_bytes()[col] != b'-'));
    }

    let mut aln = MultiSequence::default();
    multi_sequence_from_strings(
        &mut aln,
        &[
            "real_hbb_human".to_string(),
            "real_hba_human".to_string(),
            "wildcards".to_string(),
        ],
        &[
            "MVLSPADKTNVKAAW".to_string(),
            "M-VL.SPADKTNVK".to_string(),
            "XBZ.-ac".to_string(),
        ],
    );

    {
        *MEGA_STATE.lock().unwrap() = MegaState::default();
        let mega_file = std::env::temp_dir().join(format!(
            "muscle_rs_mega_from_file_{}.mega",
            std::process::id()
        ));
        std::fs::write(
            &mega_file,
            concat!(
                "mega\t1\t2\t-4.5\t-0.6\n",
                "0\tAA\t3\t2.0\n",
                "freqs\t0.5\t0\t0.5\n",
                "0\t1\n",
                "1\t0.25\t0.75\n",
                "2\t0\t0.2\t0.8\n",
                "logoddsmx\n",
                "0\tA\t1.0\n",
                "1\tC\t2.0\t3.0\n",
                "2\tD\t4.0\t5.0\t6.0\n",
                "chain\t0\tp0\t2\n",
                "0\t0\tA\n",
                "0\t1\tC\n",
                "chain\t1\tp1\t2\n",
                "1\t0\tA\n",
                "1\t1\tC\n",
            ),
        )
        .unwrap();
        mega_from_file(mega_file.to_str().unwrap());
        let mega = MEGA_STATE.lock().unwrap();
        assert!(mega.loaded);
        assert!(mega.lines.is_empty());
        assert_eq!(mega.feature_count, 1);
        assert_eq!(mega.feature_names, vec!["AA"]);
        assert_eq!(mega.alpha_sizes, vec![3]);
        assert_eq!(mega.weights, vec![2.0]);
        assert_eq!(mega.gap_open, -4.5);
        assert_eq!(mega.gap_ext, -0.6);
        assert_eq!(mega.labels, vec!["p0", "p1"]);
        assert_eq!(mega.seqs, vec!["AC", "AC"]);
        assert_eq!(mega.label_to_idx["p1"], 1);
        assert_eq!(mega.seq_to_idx["AC"], 1);
        assert_eq!(mega.profiles[0], vec![vec![0], vec![1]]);
        assert!((mega.log_probs_vec[0][1] - 1e-6_f32.ln()).abs() < 1e-6);
        assert!((mega.log_prob_mx_vec[0][2][0] - 1e-6_f32.ln()).abs() < 1e-6);
        assert_eq!(mega.log_odds_mx_vec[0][2][1], 5.0);
        drop(mega);
        std::fs::remove_file(&mega_file).unwrap();
    }

    mega_from_msa_aa_only(&aln, -4.5, -0.6);
    let mega = MEGA_STATE.lock().unwrap();
    assert_eq!(mega.file_name, "FromMSA_AAOnly()");
    assert!(mega.lines.is_empty());
    assert_eq!(mega.feature_names, vec!["AA"]);
    assert_eq!(mega.weights, vec![1.0]);
    assert_eq!(mega.alpha_sizes, vec![20]);
    assert_eq!(mega.feature_count, 1);
    assert!(mega.loaded);
    assert_eq!(mega.gap_open, -4.5);
    assert_eq!(mega.gap_ext, -0.6);
    assert_eq!(
        mega.labels,
        vec!["real_hbb_human", "real_hba_human", "wildcards"]
    );
    assert_eq!(mega.seqs, vec!["MVLSPADKTNVKAAW", "MVLSPADKTNVK", "XBZac"]);
    assert_eq!(mega.label_to_idx["real_hba_human"], 1);
    assert_eq!(mega.seq_to_idx["XBZac"], 2);
    assert_eq!(mega.profiles[0][0], vec![10]);
    assert_eq!(mega.profiles[0][14], vec![18]);
    assert_eq!(
        mega.profiles[2],
        vec![vec![0], vec![0], vec![0], vec![0], vec![1]]
    );
    assert!(mega.log_probs_vec.is_empty());
    assert!(mega.log_prob_mx_vec.is_empty());
    assert_eq!(mega.log_odds_mx_vec.len(), 1);
    assert!((mega.log_odds_mx_vec[0][18][19] - 1.0771).abs() < 1e-6);
    drop(mega);

    assert_eq!(
        make_mega_profile("ACXy"),
        vec![vec![0], vec![1], vec![0], vec![19]]
    );
    let mut mega_fwd_wrap = alloc_fb(2, 2);
    let mut mega_bwd_wrap = alloc_fb(2, 2);
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        *mega = MegaState::default();
        mega.feature_count = 1;
        mega.weights = vec![1.0];
        mega.labels = vec!["x".to_string(), "y".to_string()];
        mega.profiles = vec![vec![vec![0], vec![1]], vec![vec![1], vec![0]]];
        mega.label_to_idx.insert("x".to_string(), 0);
        mega.label_to_idx.insert("y".to_string(), 1);
        mega.log_probs_vec = vec![vec![0.2, 0.3]];
        mega.log_prob_mx_vec = vec![vec![vec![0.1, 0.2], vec![0.2, 0.4]]];
    }
    mpc_flat_mega_calc_fwd_flat_mpc_flat(0, 2, 1, 2, &mut mega_fwd_wrap);
    mpc_flat_mega_calc_bwd_flat_mpc_flat(0, 2, 1, 2, &mut mega_bwd_wrap);
    assert!(mega_fwd_wrap.iter().any(|x| *x != 0.0));
    assert!(mega_bwd_wrap.iter().any(|x| *x != 0.0));
    let mut pprog_fwd_wrap = alloc_fb(2, 2);
    let mut pprog_bwd_wrap = alloc_fb(2, 2);
    p_prog_mega_calc_fwd_flat_p_prog(0, 2, 1, 2, &mut pprog_fwd_wrap);
    p_prog_mega_calc_bwd_flat_p_prog(0, 2, 1, 2, &mut pprog_bwd_wrap);
    assert_eq!(pprog_fwd_wrap, mega_fwd_wrap);
    assert_eq!(pprog_bwd_wrap, mega_bwd_wrap);
    assert_eq!(make_mega_profile_aa("mZ"), vec![vec![10], vec![0]]);
    assert_eq!(s_wer_get_na("MDIIMD"), 4);
    assert_eq!(s_wer_get_nb("MDIIMD"), 4);
    let mut swer = SWer::default();
    let mut lo_a = uint::MAX;
    let mut lo_b = uint::MAX;
    let mut path = String::new();
    let score = s_wer_run(
        &mut swer,
        "ACD|EFG",
        "WXYZ",
        &mut lo_a,
        &mut lo_b,
        &mut path,
        |s, lo_a, lo_b, path| {
            assert_eq!(s.rows_a, vec!["ACD".to_string(), "EFG".to_string()]);
            assert_eq!(s.la, 3);
            assert_eq!(s.lb, 4);
            *lo_a = 1;
            *lo_b = 2;
            *path = "MI".to_string();
            3.5
        },
    );
    assert_eq!(score, 3.5);
    assert_eq!(lo_a, 1);
    assert_eq!(lo_b, 2);
    assert_eq!(path, "MI");
    assert_eq!(swer.a, "ACD|EFG");
    assert_eq!(swer.b, "WXYZ");
    let zero_score = s_wer_run(
        &mut swer,
        "AA",
        "BB",
        &mut lo_a,
        &mut lo_b,
        &mut path,
        |_s, _lo_a, _lo_b, path| {
            *path = "MMMM".to_string();
            -1.0
        },
    );
    assert_eq!(zero_score, 0.0);
    cmp_fwd_m(&[vec![1.0, f32::MAX]], &[vec![1.0, f32::MAX]]);
    let mx_log = log_m("S", &[vec![1.25, f32::MAX]]);
    assert!(mx_log.contains("LogM(S)"));
    assert!(mx_log.contains("*"));

    let single = make_masm_seq("ACD", -1.0, -0.4);
    assert_eq!(single.label, "MSA");
    assert_eq!(single.seq_count, 1);
    assert_eq!(single.col_count, 3);
    assert_eq!(single.feature_names, vec!["AA"]);

    let rows = make_masm_rows(&["A-C".to_string(), "AGC".to_string()], -1.0, -0.4);
    assert_eq!(rows.label, "Rows");
    assert_eq!(rows.seq_count, 2);
    assert_eq!(rows.col_count, 3);
    assert_eq!(rows.ungapped_seqs, vec!["AC", "AGC"]);
    let aa_rows = make_masm_a_as(&["A-C".to_string(), "AGC".to_string()]);
    assert_eq!(aa_rows.label, "MSA");
    assert_eq!(aa_rows.seq_count, 2);
    assert_eq!(aa_rows.col_count, 3);
    assert_eq!(aa_rows.gap_open, 3.0);
    assert_eq!(aa_rows.gap_ext, 1.0);

    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.feature_names = vec!["AA".to_string(), "SS".to_string()];
        mega.weights = vec![1.0, 1.0];
        mega.alpha_sizes = vec![20, 3];
        mega.feature_count = 2;
        mega.labels = vec!["p1".to_string(), "p2".to_string()];
        mega.label_to_idx.clear();
        mega.label_to_idx.insert("p1".to_string(), 0);
        mega.label_to_idx.insert("p2".to_string(), 1);
        mega.profiles = vec![vec![vec![0, 0], vec![1, 1]], vec![vec![2, 2], vec![3, 1]]];
        mega.loaded = true;
    }
    let mega_msa_in =
        std::env::temp_dir().join(format!("muscle_rs_mega_msas_in_{}.fa", std::process::id()));
    let mega_msa_prefix = std::env::temp_dir()
        .join(format!("muscle_rs_mega_msas_{}_", std::process::id()))
        .to_string_lossy()
        .to_string();
    std::fs::write(&mega_msa_in, b">p1\nA-C\n>p2\nDE.\n").unwrap();
    let mega_outputs = cmd_mega_msas(mega_msa_in.to_str().unwrap(), &mega_msa_prefix);
    assert_eq!(mega_outputs.len(), 2);
    assert_eq!(mega_outputs[0].1, b">p1\n\xff-\xff\n>p2\n\xff\xff.\n");
    assert_eq!(mega_outputs[1].1, b">p1\nA-B\n>p2\nCB.\n");
    assert_eq!(
        std::fs::read(format!("{mega_msa_prefix}AA")).unwrap(),
        b">p1\n\xff-\xff\n>p2\n\xff\xff.\n"
    );
    assert_eq!(
        std::fs::read(format!("{mega_msa_prefix}SS")).unwrap(),
        b">p1\nA-B\n>p2\nCB.\n"
    );
    std::fs::remove_file(&mega_msa_in).unwrap();
    std::fs::remove_file(format!("{mega_msa_prefix}AA")).unwrap();
    std::fs::remove_file(format!("{mega_msa_prefix}SS")).unwrap();
}

#[test]
fn mega_accessors_and_masm_profile_calculations_match_cpp_logic() {
    let _guard = GLOBAL_STATE_TEST_LOCK.lock().unwrap();
    set_alpha_l209(ALPHA::ALPHA_Amino);

    {
        let mut mega = MEGA_STATE.lock().unwrap();
        *mega = MegaState::default();
        mega.feature_count = 2;
        mega.feature_names = vec!["AA".to_string(), "SS".to_string()];
        mega.weights = vec![0.7, 0.3];
        mega.alpha_sizes = vec![4, 3];
        mega.labels = vec!["p0".to_string(), "p1".to_string()];
        mega.profiles = vec![vec![vec![0, 1], vec![2, 2]], vec![vec![3, 0]]];
        mega.seqs = vec!["AC".to_string(), "D".to_string()];
        mega.lines = vec!["a\tb".to_string(), "x\ty\tz".to_string()];
        mega.log_probs_vec = vec![vec![0.1, 0.2, 0.3, 1.0], vec![0.4, 0.6, 0.8]];
        mega.log_odds_mx_vec = vec![
            vec![
                vec![1.0, 2.0, 3.0, 4.0],
                vec![5.0, 6.0, 7.0, 8.0],
                vec![9.0, 10.0, 11.0, 12.0],
                vec![13.0, 14.0, 15.0, 16.0],
            ],
            vec![
                vec![-1.0, -2.0, -3.0],
                vec![-4.0, -5.0, -6.0],
                vec![-7.0, -8.0, -9.0],
            ],
        ];
        mega.log_prob_mx_vec = vec![
            vec![
                vec![0.1, 0.2, 0.3, 0.4],
                vec![0.5, 0.6, 0.7, 0.8],
                vec![0.9, 1.0, 1.1, 1.2],
                vec![1.3, 1.4, 1.5, 1.6],
            ],
            vec![
                vec![2.0, 3.0, 4.0],
                vec![5.0, 6.0, 7.0],
                vec![8.0, 9.0, 10.0],
            ],
        ];
        mega.label_to_idx.insert("p0".to_string(), 0);
        mega.label_to_idx.insert("p1".to_string(), 1);
        mega.seq_to_idx.insert("AC".to_string(), 0);
        mega.seq_to_idx.insert("D".to_string(), 1);
    }

    assert_eq!(mega_get_aa_feature_idx(), 0);
    assert_eq!(mega_get_gsi_by_label("p1"), 1);
    assert_eq!(mega_get_label_by_gsi(0), "p0");
    assert_eq!(mega_get_label(1), "p1");
    assert_eq!(mega_get_weight(0), 0.7);
    assert_eq!(mega_get_profile_by_gsi(0), vec![vec![0, 1], vec![2, 2]]);
    assert_eq!(mega_get_profile_by_label("p1"), vec![vec![3, 0]]);
    assert_eq!(
        mega_get_profile_by_seq("AC", true),
        Some(vec![vec![0, 1], vec![2, 2]])
    );
    assert_eq!(mega_get_profile_by_seq("missing", false), None);
    assert_eq!(mega_get_next_fields(2), vec!["a", "b"]);
    assert_eq!(mega_get_next_fields(uint::MAX), vec!["x", "y", "z"]);

    let marginals = mega_calc_marginal_freqs(&[vec![0.1, 0.2], vec![0.2, 0.5]]);
    assert!((marginals[0] - 0.3).abs() < 1e-6);
    assert!((marginals[1] - 0.7).abs() < 1e-6);
    assert_eq!(
        mega_log_mx("Mx", &[vec![1.25, -2.0], vec![3.5, 4.0]]),
        "\nMx/2\n            0       1\n[ 0]     1.25   -2.00\n[ 1]     3.50    4.00\n"
    );
    assert_eq!(
        mega_log_vec("V", &[1.25, -2.0, 3.5]),
        "\nV/3\n  [ 0]=1.25 [ 1]=-2.00 [ 2]=3.50\n"
    );
    let feature_log = mega_log_feature_params(0);
    assert!(feature_log.starts_with("\nFeature AA, weight 0.7\n\nAA/4\n"));
    assert!(feature_log.contains("[ 3]=1.00"));
    assert!(feature_log.contains("\nAA/4\n"));
    assert!((mega_get_ins_score(&[vec![3, 1]], 0) - 0.88).abs() < 1e-6);
    assert!((mega_get_match_score_log_odds(&[vec![0, 1]], 0, &[vec![2, 0]], 0) - 0.9).abs() < 1e-6);
    assert!((mega_get_match_score(&[vec![3, 2]], 0, &[vec![1, 0]], 0) - 3.38).abs() < 1e-6);
    {
        let mut start = PAIR_HMM_START_SCORE.lock().unwrap();
        *start = [
            0.6_f32.ln(),
            0.2_f32.ln(),
            0.2_f32.ln(),
            0.1_f32.ln(),
            0.1_f32.ln(),
        ];
        let mut trans = PAIR_HMM_TRANS_SCORE.lock().unwrap();
        *trans = [[LOG_ZERO; 5]; 5];
        trans[HMMSTATE_M as usize][HMMSTATE_M as usize] = 0.5_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_IX as usize] = 0.1_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_IY as usize] = 0.1_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_JX as usize] = 0.1_f32.ln();
        trans[HMMSTATE_M as usize][HMMSTATE_JY as usize] = 0.1_f32.ln();
        trans[HMMSTATE_IX as usize][HMMSTATE_IX as usize] = 0.3_f32.ln();
        trans[HMMSTATE_IY as usize][HMMSTATE_IY as usize] = 0.3_f32.ln();
        trans[HMMSTATE_JX as usize][HMMSTATE_JX as usize] = 0.4_f32.ln();
        trans[HMMSTATE_JY as usize][HMMSTATE_JY as usize] = 0.4_f32.ln();
        trans[HMMSTATE_IX as usize][HMMSTATE_M as usize] = 0.7_f32.ln();
        trans[HMMSTATE_IY as usize][HMMSTATE_M as usize] = 0.7_f32.ln();
        trans[HMMSTATE_JX as usize][HMMSTATE_M as usize] = 0.6_f32.ln();
        trans[HMMSTATE_JY as usize][HMMSTATE_M as usize] = 0.6_f32.ln();
    }
    let prof_x = vec![vec![3, 1]];
    let prof_y = vec![vec![1, 0]];
    let mut mega_fwd = alloc_fb(1, 1);
    let mut mega_bwd = alloc_fb(1, 1);
    mega_calc_fwd_flat_mega(&prof_x, &prof_y, &mut mega_fwd);
    mega_calc_bwd_flat_mega(&prof_x, &prof_y, &mut mega_bwd);
    let m11 = HMMSTATE_COUNT as usize * (1 * (1 + 1) + 1) + HMMSTATE_M as usize;
    assert!((mega_fwd[m11] - (0.6_f32.ln() + 2.48)).abs() < 1e-6);
    assert!((mega_bwd[m11] - 0.6_f32.ln()).abs() < 1e-6);
    let ix10 = HMMSTATE_COUNT as usize * (1 * (1 + 1)) + HMMSTATE_IX as usize;
    assert!((mega_fwd[ix10] - (0.2_f32.ln() + 0.88)).abs() < 1e-6);

    let mut ms = MultiSequence::default();
    multi_sequence_from_strings(
        &mut ms,
        &["s0".to_string(), "s1".to_string(), "s2".to_string()],
        &["A-C.".to_string(), "AB--".to_string(), ".BCD".to_string()],
    );
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.profiles = vec![
            vec![vec![0, 1], vec![1, 2]],
            vec![vec![0, 2], vec![2, 1]],
            vec![vec![1, 0], vec![2, 1], vec![3, 2]],
        ];
        mega.seq_to_idx.clear();
        mega.seq_to_idx.insert("AC".to_string(), 0);
        mega.seq_to_idx.insert("AB".to_string(), 1);
        mega.seq_to_idx.insert("BCD".to_string(), 2);
    }
    let mut setup_masm = MASM {
        aln: Some(Box::new(ms.clone())),
        col_count: 4,
        seq_count: 3,
        feature_count: 2,
        ..MASM::default()
    };
    masm_set_ungapped_seqs(&mut setup_masm);
    assert_eq!(setup_masm.ungapped_seqs, vec!["AC", "AB", "BCD"]);
    masm_set_feature_aln_vec(&mut setup_masm);
    assert_eq!(
        setup_masm.feature_aln_vec,
        vec![
            vec![
                vec![0, u8::MAX, 1, u8::MAX],
                vec![0, 2, u8::MAX, u8::MAX],
                vec![u8::MAX, 1, 2, 3],
            ],
            vec![
                vec![1, u8::MAX, 2, u8::MAX],
                vec![2, 1, u8::MAX, u8::MAX],
                vec![u8::MAX, 0, 1, 2],
            ],
        ]
    );
    let mut from_msa = MASM::default();
    masm_from_msa(&mut from_msa, &ms, "profile-label", 6.0, 2.0);
    assert_eq!(from_msa.label, "profile-label");
    assert_eq!(from_msa.col_count, 4);
    assert_eq!(from_msa.seq_count, 3);
    assert_eq!(from_msa.feature_count, 2);
    assert_eq!(from_msa.feature_names, vec!["AA", "SS"]);
    assert_eq!(from_msa.alpha_sizes, vec![4, 3]);
    assert_eq!(from_msa.ungapped_seqs, vec!["AC", "AB", "BCD"]);
    assert_eq!(from_msa.feature_aln_vec, setup_masm.feature_aln_vec);
    assert_eq!(from_msa.cols.len(), 4);
    assert!((from_msa.cols[0].letter_freq - 2.0 / 3.0).abs() < 1e-6);
    assert!((from_msa.cols[0].gap_close_freq - 1.0 / 3.0).abs() < 1e-6);
    assert_eq!(from_msa.cols[0].gap_open_freq, 0.0);
    assert_eq!(from_msa.cols[0].gap_ext_freq, 0.0);
    assert!((from_msa.cols[0].gap_open - 3.0).abs() < 1e-6);
    assert!((from_msa.cols[0].gap_close - 2.0).abs() < 1e-6);
    assert!((from_msa.cols[0].gap_ext - 2.0).abs() < 1e-6);
    assert_eq!(
        from_msa.cols[0].freqs_vec[0],
        vec![2.0 / 3.0, 0.0, 0.0, 0.0]
    );
    assert!((from_msa.cols[0].scores_vec[0][0] - 2.0 / 3.0).abs() < 1e-6);
    assert!((from_msa.cols[0].scores_vec[0][1] - 10.0 / 3.0).abs() < 1e-6);
    let log = masm_log_me(&from_msa);
    assert!(log.contains("MASM 3 seqs, 4 cols, 2 features, open 6, ext 2, label profile-label"));
    assert!(log.contains("Feature 0  AS 4  AA"));

    let tmp = std::env::temp_dir().join(format!(
        "muscle_rs_masm_roundtrip_{}.txt",
        std::process::id()
    ));
    masm_to_file_l150(&from_msa, tmp.to_str().unwrap());
    let mut parsed_masm = MASM::default();
    masm_from_file(&mut parsed_masm, tmp.to_str().unwrap());
    assert_eq!(
        cmd_masm_stats(tmp.to_str().unwrap()),
        "         3  Sequences\n         4  Columns\n         2  Features  AA/4 SS/3\n"
    );
    std::fs::remove_file(&tmp).unwrap();
    let train_aln = std::env::temp_dir().join(format!(
        "muscle_rs_masm_train_aln_{}.fa",
        std::process::id()
    ));
    let train_out = std::env::temp_dir().join(format!(
        "muscle_rs_masm_train_out_{}.masm",
        std::process::id()
    ));
    std::fs::write(&train_aln, b">ta\nAC-\n>tb\nA-C\n").unwrap();
    let mut train_loads = Vec::new();
    let trained = cmd_masm_train(
        train_aln.to_str().unwrap(),
        "model.mega",
        train_out.to_str().unwrap(),
        None,
        |mega_file_name| {
            train_loads.push(mega_file_name.to_string());
            let mut aln = MultiSequence::default();
            multi_sequence_load_mfa_l8(&mut aln, train_aln.to_str().unwrap(), false);
            mega_from_msa_aa_only(&aln, 6.0, 2.0);
        },
    );
    assert_eq!(train_loads, vec!["model.mega"]);
    assert_eq!(trained.label, base_name(train_aln.to_str().unwrap()));
    assert_eq!(trained.seq_count, 2);
    assert_eq!(trained.col_count, 3);
    assert_eq!(
        cmd_masm_stats(train_out.to_str().unwrap()),
        "         2  Sequences\n         3  Columns\n         1  Features  AA/20\n"
    );
    let trained_labeled = cmd_masm_train(
        train_aln.to_str().unwrap(),
        "model2.mega",
        "",
        Some("manual-label"),
        |mega_file_name| {
            assert_eq!(mega_file_name, "model2.mega");
            let mut aln = MultiSequence::default();
            multi_sequence_load_mfa_l8(&mut aln, train_aln.to_str().unwrap(), false);
            mega_from_msa_aa_only(&aln, 6.0, 2.0);
        },
    );
    assert_eq!(trained_labeled.label, "manual-label");
    std::fs::remove_file(&train_aln).unwrap();
    std::fs::remove_file(&train_out).unwrap();
    {
        let mut mega = MEGA_STATE.lock().unwrap();
        mega.feature_names = vec!["AA".to_string(), "SS".to_string()];
        mega.alpha_sizes = vec![4, 3];
        mega.feature_count = 2;
    }
    assert_eq!(parsed_masm.seq_count, from_msa.seq_count);
    assert_eq!(parsed_masm.col_count, from_msa.col_count);
    assert_eq!(parsed_masm.feature_count, from_msa.feature_count);
    assert_eq!(parsed_masm.feature_names, from_msa.feature_names);
    assert_eq!(parsed_masm.alpha_sizes, from_msa.alpha_sizes);
    assert_eq!(parsed_masm.aa_feature_idx, 0);
    assert_eq!(parsed_masm.cols.len(), from_msa.cols.len());
    assert_eq!(parsed_masm.cols[0].col_index, 0);
    assert!((parsed_masm.cols[0].freqs_vec[0][0] - from_msa.cols[0].freqs_vec[0][0]).abs() < 1e-3);
    assert!(
        (parsed_masm.cols[0].scores_vec[0][1] - from_msa.cols[0].scores_vec[0][1]).abs() < 0.05
    );
    let masm = MASM {
        aln: Some(Box::new(ms)),
        col_count: 4,
        seq_count: 3,
        feature_count: 2,
        alpha_sizes: vec![4, 3],
        feature_aln_vec: vec![
            vec![
                vec![0, u8::MAX, 1, u8::MAX],
                vec![0, 2, u8::MAX, u8::MAX],
                vec![u8::MAX, 1, 2, 3],
            ],
            vec![
                vec![1, u8::MAX, 2, u8::MAX],
                vec![2, 1, u8::MAX, u8::MAX],
                vec![u8::MAX, 0, 1, 2],
            ],
        ],
        ..MASM::default()
    };
    assert_eq!(masm_get_counts(&masm, 0), (2, 0, 0, 1));
    assert_eq!(masm_get_counts(&masm, 1), (2, 0, 0, 1));
    assert_eq!(masm_get_counts(&masm, 2), (2, 1, 0, 0));
    assert_eq!(masm_get_counts(&masm, 3), (1, 0, 1, 1));

    assert_eq!(
        masm_get_freqs(&masm, 1, 0),
        vec![0.0, 1.0 / 3.0, 1.0 / 3.0, 0.0]
    );
    let freqs_vec = masm_get_freqs_vec(&masm, 2);
    assert_eq!(freqs_vec[0], vec![0.0, 1.0 / 3.0, 1.0 / 3.0, 0.0]);
    assert_eq!(freqs_vec[1], vec![0.0, 1.0 / 3.0, 1.0 / 3.0]);

    let aa_masm = MASM {
        feature_count: 2,
        aa_feature_idx: 0,
        ..MASM::default()
    };
    let mut col0 = MASMCol {
        masm: Some(Box::new(aa_masm.clone())),
        scores_vec: vec![vec![1.0, 2.0, 3.0, 4.0], vec![0.5, 1.5, 2.5]],
        freqs_vec: vec![
            vec![
                0.7, 0.1, 0.1, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
            ],
            vec![0.3, 0.4, 0.3],
        ],
        ..MASMCol::default()
    };
    let col1 = MASMCol {
        masm: Some(Box::new(aa_masm.clone())),
        scores_vec: vec![
            vec![
                -1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0, -11.0, -12.0, -13.0,
                -14.0, -15.0, -16.0, -17.0, -18.0, -19.0, -20.0,
            ],
            vec![10.0, 20.0, 30.0],
        ],
        freqs_vec: vec![
            vec![
                0.2, 0.3, 0.4, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
            ],
            vec![0.3, 0.4, 0.3],
        ],
        ..MASMCol::default()
    };
    assert_eq!(score_pp(&col0, &[2, u8::MAX]), 3.0);
    assert_eq!(score_pp(&col0, &[1, 2]), 4.5);
    col0.col_index = 0;
    let score_masm = MASM {
        col_count: 2,
        cols: vec![col0, col1],
        ..MASM::default()
    };
    let smx = masm_make_s_mx(&score_masm, &[vec![0, 0], vec![3, 1], vec![u8::MAX, 2]]);
    assert_eq!(smx.row_count, 2);
    assert_eq!(smx.col_count, 3);
    assert_eq!(smx.data[0], vec![1.5, 5.5, 2.5]);
    assert_eq!(smx.data[1], vec![9.0, 16.0, 30.0]);

    let mut query = Sequence::default();
    sequence_from_string(&mut query, "q", "ACX");
    let seq_smx = masm_make_s_mx_sequence(&score_masm, &query);
    assert_eq!(seq_smx.data[0], vec![1.0, 2.0, 0.0]);
    assert_eq!(seq_smx.data[1], vec![-1.0, -2.0, 0.0]);
    assert_eq!(masm_get_consensus_seq(&score_masm), "Ad");
    let aa_profile = vec![vec![0, 1], vec![2, 2], vec![3, 0]];
    assert_eq!(get_mega_profile_aa_seq(&aa_profile), "ADE");
    assert_eq!(
        write_local_aln_masm("masm", &score_masm, "prof", &aa_profile, 0, 0, "MDII"),
        "\n    1 Ad-- 2  masm\n      |   \n    1 A-DE 3  prof\n"
    );

    let mut scorer_masm = score_masm.clone();
    scorer_masm.gap_open = 10.0;
    scorer_masm.gap_ext = 2.0;
    scorer_masm.cols[0].gap_open = 5.0;
    scorer_masm.cols[0].gap_close = 7.0;
    scorer_masm.cols[0].gap_ext = 3.0;
    let mut ps_mm = PathScorerMASMMega::default();
    path_scorer_masm_mega_init(&mut ps_mm, &scorer_masm, &aa_profile);
    assert_eq!(ps_mm.base.la, 2);
    assert_eq!(ps_mm.base.lb, 3);
    assert_eq!(path_scorer_masm_mega_get_match_score(&ps_mm, 0, 1), 5.5);
    assert_eq!(path_scorer_masm_mega_get_score_mm(&ps_mm, 0, 0), 0.0);
    assert_eq!(path_scorer_masm_mega_get_score_md(&ps_mm, 0, 0), -5.0);
    assert_eq!(path_scorer_masm_mega_get_score_mi(&ps_mm, 0, 0), -5.0);
    assert_eq!(path_scorer_masm_mega_get_score_dm(&ps_mm, 0, 0), -7.0);
    assert_eq!(path_scorer_masm_mega_get_score_dd(&ps_mm, 0, 0), -3.0);
    assert_eq!(path_scorer_masm_mega_get_score_im(&ps_mm, 0, 0), -5.0);
    assert_eq!(path_scorer_masm_mega_get_score_ii(&ps_mm, 0, 0), -2.0);

    let ps_aa = PathScorerAABLOSUM62 {
        gap_open: -6.0,
        gap_ext: -1.0,
        seq_a: "ACG".to_string(),
        seq_b: "AD".to_string(),
        ..PathScorerAABLOSUM62::default()
    };
    assert_eq!(
        path_scorer_aa_blosum62_get_match_score(&ps_aa, 0, 0),
        get_blosum_score_chars(b'A', b'A')
    );
    assert_eq!(path_scorer_aa_blosum62_get_score_mm(&ps_aa, 0, 0), 0.0);
    assert_eq!(path_scorer_aa_blosum62_get_score_md(&ps_aa, 0, 0), -6.0);
    assert_eq!(path_scorer_aa_blosum62_get_score_mi(&ps_aa, 0, 0), -6.0);
    assert_eq!(path_scorer_aa_blosum62_get_score_dm(&ps_aa, 0, 0), 0.0);
    assert_eq!(path_scorer_aa_blosum62_get_score_dd(&ps_aa, 0, 0), -1.0);
    assert_eq!(path_scorer_aa_blosum62_get_score_im(&ps_aa, 0, 0), 0.0);
    assert_eq!(path_scorer_aa_blosum62_get_score_ii(&ps_aa, 0, 0), -1.0);
    let aa_masm = make_masm_aa("ACD");
    assert_eq!(aa_masm.label, "MSA");
    assert_eq!(aa_masm.seq_count, 1);
    assert_eq!(aa_masm.col_count, 3);
    assert_eq!(aa_masm.gap_open, 3.0);
    assert_eq!(aa_masm.gap_ext, 1.0);
    let sw_ma = make_masm_aa("CAC");
    let sw_pb = make_mega_profile_aa("DAD");
    let sw_smx = masm_make_s_mx(&sw_ma, &sw_pb);
    let mut sw_mem = XDPMem::default();
    let (mm_sw_score, mm_sw_lo_i, mm_sw_lo_j, mm_sw_len_i, mm_sw_len_j, mm_sw_path) =
        sw_fast_masm_mega_prof(&mut sw_mem, &sw_ma, &sw_pb, -3.0, -1.0);
    assert!(sw_smx.data[1][1] > 0.0);
    assert!((mm_sw_score - sw_smx.data[1][1]).abs() < 1e-6);
    assert_eq!(
        (mm_sw_lo_i, mm_sw_lo_j, mm_sw_len_i, mm_sw_len_j),
        (1, 1, 1, 1)
    );
    assert_eq!(mm_sw_path, "M");
    let mut sw_mem = XDPMem::default();
    let (
        fast_masm_score,
        fast_masm_lo_i,
        fast_masm_lo_j,
        fast_masm_len_i,
        fast_masm_len_j,
        fast_masm_path,
    ) = sw_fast_masm(&mut sw_mem, &sw_ma, &sw_pb);
    assert!((fast_masm_score - mm_sw_score).abs() < 1e-6);
    assert_eq!(
        (
            fast_masm_lo_i,
            fast_masm_lo_j,
            fast_masm_len_i,
            fast_masm_len_j
        ),
        (1, 1, 1, 1)
    );
    assert_eq!(fast_masm_path, "M");
    let mut sw_query = Sequence::default();
    sequence_from_string(&mut sw_query, "q", "DAD");
    let mut sw_mem = XDPMem::default();
    let (
        fast_seq_score,
        fast_seq_lo_i,
        fast_seq_lo_j,
        fast_seq_len_i,
        fast_seq_len_j,
        fast_seq_path,
    ) = sw_fast_masm_seq(&mut sw_mem, &sw_ma, &sw_query, 3.0, 1.0);
    assert!((fast_seq_score - masm_make_s_mx_sequence(&sw_ma, &sw_query).data[1][1]).abs() < 1e-6);
    assert_eq!(
        (fast_seq_lo_i, fast_seq_lo_j, fast_seq_len_i, fast_seq_len_j),
        (1, 1, 1, 1)
    );
    assert_eq!(fast_seq_path, "M");
    let swmasm_masm =
        std::env::temp_dir().join(format!("muscle_rs_cmd_swmasm_{}.masm", std::process::id()));
    let swmasm_out =
        std::env::temp_dir().join(format!("muscle_rs_cmd_swmasm_{}.tsv", std::process::id()));
    let mut sw_cmd_ma = sw_ma.clone();
    for col in &mut sw_cmd_ma.cols {
        for scores in &mut col.scores_vec {
            scores.fill(-10.0);
        }
    }
    sw_cmd_ma.cols[1].scores_vec[0][0] = 1.25;
    masm_to_file_l150(&sw_cmd_ma, swmasm_masm.to_str().unwrap());
    let cmd_swmasm_log = cmd_swmasm(
        swmasm_masm.to_str().unwrap(),
        "query.mega",
        swmasm_out.to_str().unwrap(),
        |mega_file_name| {
            assert_eq!(mega_file_name, "query.mega");
            let mut query_msa = MultiSequence::default();
            multi_sequence_from_strings(&mut query_msa, &["q1".to_string()], &["DAD".to_string()]);
            mega_from_msa_aa_only(&query_msa, -3.0, -1.0);
        },
    );
    assert!(cmd_swmasm_log.contains("Score = 1.25"));
    assert!(cmd_swmasm_log.contains("q1"));
    assert_eq!(
        std::fs::read_to_string(&swmasm_out).unwrap(),
        "MSA\tq1\t1.25\n"
    );
    std::fs::remove_file(&swmasm_masm).unwrap();
    std::fs::remove_file(&swmasm_out).unwrap();

    let swmasm_seq_aln = std::env::temp_dir().join(format!(
        "muscle_rs_cmd_swmasm_seq_aln_{}.fa",
        std::process::id()
    ));
    let swmasm_seq_query = std::env::temp_dir().join(format!(
        "muscle_rs_cmd_swmasm_seq_query_{}.fa",
        std::process::id()
    ));
    let swmasm_seq_out = std::env::temp_dir().join(format!(
        "muscle_rs_cmd_swmasm_seq_{}.masm",
        std::process::id()
    ));
    std::fs::write(&swmasm_seq_aln, b">a\nCAC\n").unwrap();
    std::fs::write(&swmasm_seq_query, b">query_label\nDAD\n").unwrap();
    let cmd_swmasm_seq_log = cmd_swmasm_seq(
        swmasm_seq_aln.to_str().unwrap(),
        "seqmodel.mega",
        swmasm_seq_query.to_str().unwrap(),
        swmasm_seq_out.to_str().unwrap(),
        |mega_file_name| {
            assert_eq!(mega_file_name, "seqmodel.mega");
            let mut train_msa = MultiSequence::default();
            multi_sequence_load_mfa_l8(&mut train_msa, swmasm_seq_aln.to_str().unwrap(), false);
            mega_from_msa_aa_only(&train_msa, -4.0, -0.5);
        },
    );
    assert!(cmd_swmasm_seq_log.contains("query_label"));
    assert_eq!(
        cmd_swmasm_seq_log,
        "      1.96       query_label        1        1  M\n\n"
    );
    assert!(cmd_swmasm_seq_log.contains("M"));
    assert!(
        std::fs::read_to_string(&swmasm_seq_out)
            .unwrap()
            .starts_with("MASM\t1\t3\t1\t4\t0.5\tFomMSA\n")
    );
    let mut parsed_cmd_masm = MASM::default();
    masm_from_file(&mut parsed_cmd_masm, swmasm_seq_out.to_str().unwrap());
    assert_eq!(parsed_cmd_masm.label, "FomMSA");
    std::fs::remove_file(&swmasm_seq_aln).unwrap();
    std::fs::remove_file(&swmasm_seq_query).unwrap();
    std::fs::remove_file(&swmasm_seq_out).unwrap();
    let mut swer = SWer::default();
    let mut swer_lo_a = uint::MAX;
    let mut swer_lo_b = uint::MAX;
    let mut swer_path = String::new();
    let swer_seq_score = s_wer_run(
        &mut swer,
        "CAC",
        "DAD",
        &mut swer_lo_a,
        &mut swer_lo_b,
        &mut swer_path,
        |s, lo_a, lo_b, path| s_wer_masm_mega_seqs_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert!((swer_seq_score - mm_sw_score).abs() < 1e-6);
    assert_eq!((swer_lo_a, swer_lo_b, swer_path.as_str()), (1, 1, "M"));

    let mut swer = SWer::default();
    let mut fast_lo_a = uint::MAX;
    let mut fast_lo_b = uint::MAX;
    let mut fast_path = String::new();
    let fast_score = s_wer_run(
        &mut swer,
        "CAC",
        "DAD",
        &mut fast_lo_a,
        &mut fast_lo_b,
        &mut fast_path,
        |s, lo_a, lo_b, path| s_wer_masm_mega_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert!((fast_score - mm_sw_score).abs() < 1e-6);
    assert_eq!((fast_lo_a, fast_lo_b, fast_path.as_str()), (1, 1, "M"));

    let mut swer = SWer::default();
    let mut enum_lo_a = uint::MAX;
    let mut enum_lo_b = uint::MAX;
    let mut enum_path = String::new();
    let enum_score = s_wer_run(
        &mut swer,
        "CAC",
        "DAD",
        &mut enum_lo_a,
        &mut enum_lo_b,
        &mut enum_path,
        |s, lo_a, lo_b, path| s_wer_enum_masm_mega_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert!((enum_score - mm_sw_score).abs() < 1e-6);
    assert_eq!((enum_lo_a, enum_lo_b, enum_path.as_str()), (1, 1, "M"));

    let mut swer = SWer::default();
    let mut simple_lo_a = uint::MAX;
    let mut simple_lo_b = uint::MAX;
    let mut simple_path = String::new();
    let simple_score = s_wer_run(
        &mut swer,
        "CAC",
        "DAD",
        &mut simple_lo_a,
        &mut simple_lo_b,
        &mut simple_path,
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert!((simple_score - enum_score).abs() < 1e-3);
    assert_eq!(
        (simple_lo_a, simple_lo_b, simple_path.as_str()),
        (enum_lo_a, enum_lo_b, enum_path.as_str())
    );
    let masm_mega_log = test_masm_mega("ACD", "AD", |ma, pb, open, ext| {
        assert_eq!(ma.label, "MSA");
        assert_eq!(ma.col_count, 3);
        assert_eq!(get_mega_profile_aa_seq(pb), "AD");
        assert_eq!(open, -3.0);
        assert_eq!(ext, -1.0);
        (1_234.0, 1, 2, 3, 4, "MDM".to_string())
    });
    assert_eq!(masm_mega_log, "Test_MASM_Mega 1.23e+03 (1, 2) MDM\n");
    let test_path_log = test_path(
        "ACG",
        "AD",
        0,
        1,
        "MDM",
        |la, lb, a, b, pos_a, pos_b, path| {
            assert_eq!(la, 3);
            assert_eq!(lb, 2);
            assert_eq!(a, "ACG");
            assert_eq!(b, "AD");
            assert_eq!(pos_a, 0);
            assert_eq!(pos_b, 1);
            assert_eq!(path, "MDM");
            0.00125
        },
    );
    assert_eq!(test_path_log, "TestPath 0.00125 (0, 1) MDM\n");
    let aa_test_log = test_l89(
        "A",
        "C",
        |_pos_a, _pos_b, path| {
            assert_eq!(path, "M");
            1_234.0
        },
        |a, b, open, ext| {
            assert_eq!(a, "A");
            assert_eq!(b, "C");
            assert_eq!(open, -3.0);
            assert_eq!(ext, -1.0);
            (0.00125, 0, 1, 1, 1, "M".to_string())
        },
        |ma, pb, open, ext| {
            assert_eq!(ma.col_count, 1);
            assert_eq!(get_mega_profile_aa_seq(pb), "C");
            assert_eq!(open, -3.0);
            assert_eq!(ext, -1.0);
            (1.0, 1, 0, 1, 1, "M".to_string())
        },
    );
    assert_eq!(
        aa_test_log,
        "\nA=A(1)  B=C(1)\n  1.23e+03  M  (0, 0)  Brute\n  0.00125  M  (0, 1)  SW\n        1  M  (1, 0)  MASM_Mega \n"
    );
    let _rng_guard = RNG_TEST_LOCK.lock().unwrap();
    reset_rand(1);
    let mut aa_stats = TestSwStats::default();
    let aa_random_log = test_random_l144(
        &mut aa_stats,
        |_a, _b, _pos_a, _pos_b, path| if path == "M" { 7.0 } else { 1.0 },
        |_a, _b, _open, _ext| (7.0, 0, 0, 1, 1, "M".to_string()),
        |_ma, _pb, _open, _ext| (7.0, 0, 0, 1, 1, "M".to_string()),
    );
    assert_eq!(aa_random_log, "");
    assert_eq!(
        (
            aa_stats.n,
            aa_stats.n_all_ok,
            aa_stats.n_path_diff,
            aa_stats.n_score_diff
        ),
        (1, 1, 0, 0)
    );
    reset_rand(1);
    let cmd_aa_log = cmd_test_sw_aa(
        2,
        |_a, _b, _pos_a, _pos_b, path| if path == "M" { 7.0 } else { 1.0 },
        |_a, _b, _open, _ext| (7.0, 0, 0, 1, 1, "M".to_string()),
        |_ma, _pb, _open, _ext| (7.0, 0, 0, 1, 1, "M".to_string()),
    );
    assert_eq!(cmd_aa_log, "N 2, allok 2, pathdiff 0, scorediff 0\n");
    let mm_rows = vec!["A".to_string(), "C".to_string()];
    let (mm_test_log, mm_n, mm_ok, mm_path_diff, mm_score_diff) = test_l96(
        &mm_rows,
        "D",
        true,
        |_pos_a, _pos_b, path| {
            assert_eq!(path, "M");
            1_234.0
        },
        |ma, pb, open, ext| {
            assert_eq!(ma.seq_count, 2);
            assert_eq!(get_mega_profile_aa_seq(pb), "D");
            assert_eq!(open, -3.0);
            assert_eq!(ext, -1.0);
            (0.00125, 0, 0, 1, 1, "D".to_string())
        },
    );
    assert_eq!(mm_n, 1);
    assert_eq!(mm_ok, 0);
    assert_eq!(mm_path_diff, 1);
    assert_eq!(mm_score_diff, 1);
    assert_eq!(
        mm_test_log,
        "@PATHDIFF\n@SCOREDIFF\n\nA  >A0\nC  >A1\nD  >B\n  1.23e+03  M  (0, 0)  Brute\n  0.00125  D  (0, 0)  MASM_Mega \n"
    );
    let (mm_string_log, mm_string_n, mm_string_ok, mm_string_path_diff, mm_string_score_diff) =
        test_l157(
            "A|C",
            "D",
            |_pos_a, _pos_b, _path| 4.0,
            |ma, _pb, _open, _ext| {
                assert_eq!(ma.seq_count, 2);
                (4.0, 0, 0, 1, 1, "M".to_string())
            },
        );
    assert_eq!(mm_string_log, "");
    assert_eq!(
        (
            mm_string_n,
            mm_string_ok,
            mm_string_path_diff,
            mm_string_score_diff
        ),
        (1, 1, 0, 0)
    );
    reset_rand(1);
    let mut mm_stats = TestSwStats::default();
    let mm_random_log = test_random_l253(
        &mut mm_stats,
        |_pos_a, _pos_b, path| if path == "M" { 6.0 } else { 1.0 },
        |_ma, _pb, _open, _ext| (6.0, 0, 0, 1, 1, "M".to_string()),
    );
    assert_eq!(mm_random_log, "");
    assert_eq!(
        (
            mm_stats.n,
            mm_stats.n_all_ok,
            mm_stats.n_path_diff,
            mm_stats.n_score_diff
        ),
        (1, 1, 0, 0)
    );
    let trace_log = log_path(
        "A|C",
        "D",
        0,
        0,
        "M",
        |ma, pb, la, lb, pos_a, pos_b, path| {
            assert_eq!(ma.seq_count, 2);
            assert_eq!(get_mega_profile_aa_seq(pb), "D");
            assert_eq!(la, 1);
            assert_eq!(lb, 1);
            assert_eq!(pos_a, 0);
            assert_eq!(pos_b, 0);
            assert_eq!(path, "M");
            "TRACE\n".to_string()
        },
    );
    assert!(trace_log.starts_with("___________________________________\nMASM 2 seqs, 1 cols"));
    assert!(trace_log.contains("\nA  >A0\nC  >A1\nD  >B\nPath M\nTRACE\n"));
    let cmd_mm_log = cmd_test_sw_mm(
        |_pos_a, _pos_b, path| if path == "M" { 5.0 } else { 1.0 },
        |_ma, _pb, _open, _ext| (5.0, 0, 0, 1, 1, "M".to_string()),
        |_ma, _pb, _la, _lb, _pos_a, _pos_b, path| {
            assert_eq!(path, "MDDM");
            "CMDTRACE\n".to_string()
        },
    );
    assert!(cmd_mm_log.starts_with("___________________________________\nMASM 1 seqs, 7 cols"));
    assert!(cmd_mm_log.ends_with("Path MDDM\nCMDTRACE\n"));

    let score_md = path_scorer_get_score(
        'M',
        'D',
        1,
        0,
        |pos_a, pos_b| path_scorer_aa_blosum62_get_match_score(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_mm(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_md(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_mi(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_dm(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_dd(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_im(&ps_aa, pos_a, pos_b),
        |pos_a, pos_b| path_scorer_aa_blosum62_get_score_ii(&ps_aa, pos_a, pos_b),
    );
    assert_eq!(score_md, -6.0);

    let local =
        path_scorer_get_local_score(3, 2, 0, 0, "MDM", |from_state, to_state, pos_a, pos_b| {
            path_scorer_get_score(
                from_state,
                to_state,
                pos_a,
                pos_b,
                |pos_a, pos_b| path_scorer_aa_blosum62_get_match_score(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_mm(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_md(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_mi(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_dm(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_dd(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_im(&ps_aa, pos_a, pos_b),
                |pos_a, pos_b| path_scorer_aa_blosum62_get_score_ii(&ps_aa, pos_a, pos_b),
            )
        });
    assert_eq!(
        local,
        get_blosum_score_chars(b'A', b'A') - 6.0 + get_blosum_score_chars(b'G', b'D')
    );

    let mut aa_brute = TestSwMmBruteState {
        log_all_paths: true,
        ..TestSwMmBruteState::default()
    };
    assert_eq!(
        on_path_l32(&mut aa_brute, 1, 4, "MM", |pos_a, pos_b, path| {
            10.0 + pos_a as f32 + pos_b as f32 + path.len() as f32
        }),
        Some("        17      1      4  MM\n".to_string())
    );
    assert_eq!(aa_brute.best_score, 17.0);
    assert_eq!(aa_brute.best_path, "MM");
    clear_brute_l24(&mut aa_brute);
    assert_eq!(aa_brute.best_score, 0.0);
    assert!(aa_brute.best_path.is_empty());
    assert_eq!(aa_brute.best_pos_a, uint::MAX);
    assert_eq!(aa_brute.best_pos_b, uint::MAX);
    assert!(aa_brute.log_all_paths);

    let mut brute = TestSwMmBruteState {
        log_all_paths: true,
        ..TestSwMmBruteState::default()
    };
    assert_eq!(
        on_path_l44(&mut brute, 2, 3, "MDM", |pos_a, pos_b, path| {
            pos_a as f32 + pos_b as f32 + path.len() as f32
        }),
        Some("         8      2      3  MDM\n".to_string())
    );
    assert_eq!(brute.best_score, 8.0);
    assert_eq!(brute.best_path, "MDM");
    assert_eq!(brute.best_pos_a, 2);
    assert_eq!(brute.best_pos_b, 3);
    on_path_l158(&mut brute, 5, 6, "MM", |_pos_a, _pos_b, _path| 9.0);
    assert_eq!(brute.best_score, 9.0);
    assert_eq!(brute.best_path, "MM");
    assert_eq!(brute.best_pos_a, 5);
    assert_eq!(brute.best_pos_b, 6);
    assert_eq!(
        on_path_l44(&mut brute, 0, 0, "M", |_pos_a, _pos_b, _path| 1.0),
        Some("         1      0      0  M\n".to_string())
    );
    assert_eq!(brute.best_path, "MM");
    clear_brute_l36(&mut brute);
    assert_eq!(brute.best_score, 0.0);
    assert!(brute.best_path.is_empty());
    assert_eq!(brute.best_pos_a, uint::MAX);
    assert_eq!(brute.best_pos_b, uint::MAX);
    assert!(brute.log_all_paths);

    let ps_enum = PathScorerAABLOSUM62 {
        gap_open: -5.0,
        gap_ext: -1.0,
        seq_a: "AA".to_string(),
        seq_b: "AA".to_string(),
        ..PathScorerAABLOSUM62::default()
    };
    let mut fwd_m = Vec::new();
    let (score, lo_a, lo_b, path) = sw_enum_dp_fwd_m(2, 2, &mut fwd_m, |pos_a, pos_b, path| {
        path_scorer_get_local_score(2, 2, pos_a, pos_b, path, |from_state, to_state, pa, pb| {
            path_scorer_get_score(
                from_state,
                to_state,
                pa,
                pb,
                |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_enum, pa, pb),
            )
        })
    });
    let aa = get_blosum_score_chars(b'A', b'A');
    assert!((score - aa * 2.0).abs() < 1e-6);
    assert_eq!(lo_a, 0);
    assert_eq!(lo_b, 0);
    assert_eq!(path, "MM");
    assert_eq!(fwd_m.len(), 3);
    assert_eq!(fwd_m[0], vec![0.0, 0.0, 0.0]);
    assert!((fwd_m[1][1] - aa).abs() < 1e-6);
    assert!((fwd_m[2][2] - aa * 2.0).abs() < 1e-6);
    let (score2, lo_a2, lo_b2, path2) = sw_enum_dp(2, 2, |pos_a, pos_b, path| {
        path_scorer_get_local_score(2, 2, pos_a, pos_b, path, |from_state, to_state, pa, pb| {
            path_scorer_get_score(
                from_state,
                to_state,
                pa,
                pb,
                |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_enum, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_enum, pa, pb),
            )
        })
    });
    assert!((score2 - score).abs() < 1e-6);
    assert_eq!(lo_a2, lo_a);
    assert_eq!(lo_b2, lo_b);
    assert_eq!(path2, path);

    let ps_simple = PathScorerAABLOSUM62 {
        gap_open: -1.0,
        gap_ext: -1.0,
        seq_a: "ACG".to_string(),
        seq_b: "AG".to_string(),
        ..PathScorerAABLOSUM62::default()
    };
    let mut simple_lo_a = uint::MAX;
    let mut simple_lo_b = uint::MAX;
    let mut simple_path = String::new();
    let simple_score = sw_simple(
        3,
        2,
        &mut simple_lo_a,
        &mut simple_lo_b,
        &mut simple_path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_simple, pa, pb),
    );
    assert!(simple_score > 0.0);
    assert_eq!(simple_lo_a, 0);
    assert_eq!(simple_lo_b, 0);
    assert_eq!(simple_path, "MDM");
    let simple_local_score = path_scorer_get_local_score(
        3,
        2,
        simple_lo_a,
        simple_lo_b,
        &simple_path,
        |from_state, to_state, pa, pb| {
            path_scorer_get_score(
                from_state,
                to_state,
                pa,
                pb,
                |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_simple, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_simple, pa, pb),
            )
        },
    );
    assert!((simple_local_score - simple_score).abs() < 1e-6);
    let ps_simple2 = PathScorerAABLOSUM62 {
        gap_open: -3.0,
        gap_ext: -1.0,
        seq_a: "CAC".to_string(),
        seq_b: "DAD".to_string(),
        ..PathScorerAABLOSUM62::default()
    };
    let mut simple2_mem = XDPMem::default();
    let mut simple2_lo_a = uint::MAX;
    let mut simple2_lo_b = uint::MAX;
    let mut simple2_path = String::new();
    let simple2_score = sw_simple2(
        &mut simple2_mem,
        3,
        3,
        &mut simple2_lo_a,
        &mut simple2_lo_b,
        &mut simple2_path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_simple2, pa, pb),
    );
    assert!(simple2_score > 0.0);
    assert_eq!(
        (simple2_lo_a, simple2_lo_b, simple2_path.as_str()),
        (1, 1, "M")
    );
    let mut swps_mem = XDPMem::default();
    let mut swps_lo_a = uint::MAX;
    let mut swps_lo_b = uint::MAX;
    let mut swps_path = String::new();
    let swps_score = swps(
        &mut swps_mem,
        3,
        3,
        &mut swps_lo_a,
        &mut swps_lo_b,
        &mut swps_path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_simple2, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_simple2, pa, pb),
    );
    assert!((swps_score - simple2_score).abs() < 1e-6);
    assert_eq!(
        (swps_lo_a, swps_lo_b, swps_path.as_str()),
        (simple2_lo_a, simple2_lo_b, simple2_path.as_str())
    );
    let mut swer_ps_state = SWer::default();
    let mut swer_ps = PathScorerAABLOSUM62 {
        gap_open: -3.0,
        gap_ext: -1.0,
        ..PathScorerAABLOSUM62::default()
    };
    let mut swer_ps_lo_a = uint::MAX;
    let mut swer_ps_lo_b = uint::MAX;
    let mut swer_ps_path = String::new();
    let swer_ps_score = s_wer_run(
        &mut swer_ps_state,
        "CAC",
        "DAD",
        &mut swer_ps_lo_a,
        &mut swer_ps_lo_b,
        &mut swer_ps_path,
        |s, lo_a, lo_b, path| s_wer_ps_sw(s, &mut swer_ps, lo_a, lo_b, path),
    );
    assert!((swer_ps_score - swps_score).abs() < 1e-6);
    assert_eq!(
        (swer_ps_lo_a, swer_ps_lo_b, swer_ps_path.as_str()),
        (swps_lo_a, swps_lo_b, swps_path.as_str())
    );
    let mut enum_gap_open = f32::MAX;
    let mut enum_gap_ext = f32::MAX;
    s_wer_enum_seqs_aa_blosum62_set_gaps(&mut enum_gap_open, &mut enum_gap_ext, -3.0, -1.0);
    assert_eq!((enum_gap_open, enum_gap_ext), (-3.0, -1.0));

    let mut aa_swer = SWer::default();
    let mut aa_fast_lo_a = uint::MAX;
    let mut aa_fast_lo_b = uint::MAX;
    let mut aa_fast_path = String::new();
    let aa_fast_score = s_wer_run(
        &mut aa_swer,
        "CAC",
        "DAD",
        &mut aa_fast_lo_a,
        &mut aa_fast_lo_b,
        &mut aa_fast_path,
        |s, lo_a, lo_b, path| s_wer_fast_seqs_aa_blosum62_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert_eq!(aa_fast_score, get_blosum_score_chars(b'A', b'A'));
    assert_eq!(
        (aa_fast_lo_a, aa_fast_lo_b, aa_fast_path.as_str()),
        (1, 1, "M")
    );

    let mut aa_swer = SWer::default();
    let mut aa_simple_lo_a = uint::MAX;
    let mut aa_simple_lo_b = uint::MAX;
    let mut aa_simple_path = String::new();
    let aa_simple_score = s_wer_run(
        &mut aa_swer,
        "CAC",
        "DAD",
        &mut aa_simple_lo_a,
        &mut aa_simple_lo_b,
        &mut aa_simple_path,
        |s, lo_a, lo_b, path| s_wer_simple_seqs_aa_blosum62_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert_eq!(aa_simple_score, aa_fast_score);
    assert_eq!(
        (aa_simple_lo_a, aa_simple_lo_b, aa_simple_path.as_str()),
        (aa_fast_lo_a, aa_fast_lo_b, aa_fast_path.as_str())
    );

    let mut aa_swer = SWer::default();
    let mut aa_enum_lo_a = uint::MAX;
    let mut aa_enum_lo_b = uint::MAX;
    let mut aa_enum_path = String::new();
    let aa_enum_score = s_wer_run(
        &mut aa_swer,
        "CAC",
        "DAD",
        &mut aa_enum_lo_a,
        &mut aa_enum_lo_b,
        &mut aa_enum_path,
        |s, lo_a, lo_b, path| s_wer_enum_seqs_aa_blosum62_sw(s, -3.0, -1.0, lo_a, lo_b, path),
    );
    assert_eq!(aa_enum_score, aa_fast_score);
    assert_eq!(
        (aa_enum_lo_a, aa_enum_lo_b, aa_enum_path.as_str()),
        (aa_fast_lo_a, aa_fast_lo_b, aa_fast_path.as_str())
    );
    let cmd_swsimple2_log = cmd_swsimple2();
    assert_eq!(cmd_swsimple2_log, "Score 9.81 path=MMMM\n");
    let cmp_mx_panic = std::panic::catch_unwind(|| {
        cmp_mx(
            'M',
            &[vec![0.0, 0.0], vec![0.0, 0.00012345]],
            &[vec![0.0, 0.0], vec![0.0, 1.2345]],
        )
    });
    let cmp_mx_panic = cmp_mx_panic.unwrap_err();
    let cmp_mx_panic = cmp_mx_panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| cmp_mx_panic.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(cmp_mx_panic, "M i=1 j=1 0.000123 1.23");

    let mut fwd_m = Vec::new();
    let mut fwd_d = Vec::new();
    let mut fwd_i = Vec::new();
    let mut tbm = Vec::new();
    let mut tbd = Vec::new();
    let mut tbi = Vec::new();
    let mut mdi_lo_a = uint::MAX;
    let mut mdi_lo_b = uint::MAX;
    let mut mdi_path = String::new();
    let mdi_score = sw_simple_fwd_mdi(
        3,
        2,
        &mut mdi_lo_a,
        &mut mdi_lo_b,
        &mut mdi_path,
        &mut fwd_m,
        &mut fwd_d,
        &mut fwd_i,
        &mut tbm,
        &mut tbd,
        &mut tbi,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_simple, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_simple, pa, pb),
    );
    assert_eq!((mdi_lo_a, mdi_lo_b, mdi_path), (0, 0, "MDM".to_string()));
    assert!((mdi_score - simple_score).abs() < 1e-6);
    assert_eq!(fwd_m.len(), 4);
    assert_eq!(fwd_m[0].len(), 3);
    assert_eq!(tbm[0][0], 'S');
    assert_eq!(tbd[1][1], 'M');
    assert_eq!(tbi[1][1], 'M');
}
