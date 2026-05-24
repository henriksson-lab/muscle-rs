// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

fn format_searchpd_dist_g3(d: f64) -> String {
    if d == 0.0 {
        return "0".to_string();
    }
    if !d.is_finite() {
        return d.to_string();
    }
    let exp = d.abs().log10().floor() as i32;
    let mut s = if exp < -4 || exp >= 3 {
        let raw = format!("{d:.2e}");
        let (mantissa, exponent) = raw.split_once('e').unwrap();
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        let exp_value = exponent.parse::<i32>().unwrap();
        let sign = if exp_value >= 0 { '+' } else { '-' };
        format!("{mantissa}e{sign}{:02}", exp_value.abs())
    } else {
        let decimals = (2 - exp).max(0) as usize;
        format!("{d:.decimals$}")
    };
    if !s.contains('e') && !s.contains('E') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

fn searchpd_setup(
    input_file_name: &str,
    db_file_name: &str,
    max_pd: f64,
) -> (MultiSequence, MultiSequence) {
    let mut query = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut query, input_file_name, true);

    let mut db = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut db, db_file_name, true);

    let is_nucleo = multi_sequence_guess_is_nucleo(&query);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    let query_seq_count = query.seqs.len() as uint;
    let _ = progress_log(&format!(
        "{query_seq_count} query seqs, maxpd {max_pd:.2}\n"
    ));
    (query, db)
}

/// Run pair Viterbi alignment and return the protein distance between the aligned rows.
#[track_caller]
pub fn align_and_prot_dist<FViterbi, FDist>(
    seqi: &[byte],
    li: uint,
    seqj: &[byte],
    lj: uint,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> f64
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let pi = viterbi_fast_mem(seqi, li, seqj, lj);
    let (row_x, row_y) = make_aln_rows_l45(seqi, li, seqj, lj, &pi);
    let col_count = pi.path.len() as uint;
    assert_eq!(row_x.len() as uint, col_count);
    assert_eq!(row_y.len() as uint, col_count);
    get_prot_dist(&row_x, &row_y, col_count)
}

/// `searchpd` command: align each query against a database and report hits within MaxPD protein distance.
#[track_caller]
pub fn cmd_searchpd<FViterbi, FDist>(
    input_file_name: &str,
    db_file_name: &str,
    max_pd: f64,
    tsv_out_file_name: &str,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> String
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let (query, db) = searchpd_setup(input_file_name, db_file_name, max_pd);
    let query_seq_count = query.seqs.len() as uint;

    let mut out = String::new();
    for (counter, q) in query.seqs.iter().enumerate() {
        let _ = progress_step(counter as uint, query_seq_count, "Searching");
        let seq_q = sequence_get_seq_as_string(q).into_bytes();
        let lq = seq_q.len() as uint;
        for t in &db.seqs {
            let seq_t = sequence_get_seq_as_string(t).into_bytes();
            let lt = seq_t.len() as uint;
            let d = align_and_prot_dist(
                &seq_q,
                lq,
                &seq_t,
                lt,
                |seqi, li, seqj, lj| viterbi_fast_mem(seqi, li, seqj, lj),
                |row_x, row_y, col_count| get_prot_dist(row_x, row_y, col_count),
            );
            if d <= max_pd {
                out.push_str(&format!(
                    "{}\t{}\t{}\n",
                    q.label,
                    t.label,
                    format_searchpd_dist_g3(d)
                ));
            }
        }
    }
    if !tsv_out_file_name.is_empty() {
        std::fs::write(tsv_out_file_name, &out).expect("failed to write searchpd TSV");
    }
    out
}

/// Threaded CLI implementation for `searchpd`.
///
/// C++ writes each hit from OpenMP workers under a lock, so hit row order is
/// scheduler-dependent across queries. Rust keeps work parallel but merges
/// per-query rows in input order; database order remains stable within a query.
#[track_caller]
pub fn cmd_searchpd_threaded(
    input_file_name: &str,
    db_file_name: &str,
    max_pd: f64,
    tsv_out_file_name: &str,
    threads: Option<uint>,
) -> String {
    let (query, db) = searchpd_setup(input_file_name, db_file_name, max_pd);
    let query_seq_count = query.seqs.len() as uint;
    if query_seq_count == 0 {
        if !tsv_out_file_name.is_empty() {
            std::fs::write(tsv_out_file_name, "").expect("failed to write searchpd TSV");
        }
        return String::new();
    }

    let thread_count = threads
        .unwrap_or_else(get_requested_thread_count)
        .min(query_seq_count)
        .max(1);
    let progress_counter = std::sync::Mutex::new(0);
    let mut results = vec![String::new(); query_seq_count as usize];

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for thread_index in 0..thread_count {
            let start = (query_seq_count * thread_index) / thread_count;
            let end = (query_seq_count * (thread_index + 1)) / thread_count;
            let query = &query;
            let db = &db;
            let progress_counter = &progress_counter;
            handles.push(scope.spawn(move || {
                let mut thread_results = Vec::new();
                for query_seq_index in start..end {
                    {
                        let mut counter = progress_counter.lock().unwrap();
                        let _ = progress_step(*counter, query_seq_count, "Searching");
                        *counter += 1;
                    }
                    let q = &query.seqs[query_seq_index as usize];
                    let seq_q = sequence_get_seq_as_string(q).into_bytes();
                    let lq = seq_q.len() as uint;
                    let mut query_out = String::new();
                    for t in &db.seqs {
                        let seq_t = sequence_get_seq_as_string(t).into_bytes();
                        let lt = seq_t.len() as uint;
                        let d = align_and_prot_dist(
                            &seq_q,
                            lq,
                            &seq_t,
                            lt,
                            |seqi, li, seqj, lj| {
                                let mut mem = XDPMem::default();
                                let mut pi = PathInfo::default();
                                viterbi_fast_mem(&mut mem, seqi, li, seqj, lj, &mut pi);
                                pi
                            },
                            |row_x, row_y, col_count| {
                                get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
                            },
                        );
                        if d <= max_pd {
                            query_out.push_str(&format!(
                                "{}\t{}\t{}\n",
                                q.label,
                                t.label,
                                format_searchpd_dist_g3(d)
                            ));
                        }
                    }
                    thread_results.push((query_seq_index, query_out));
                }
                thread_results
            }));
        }
        for handle in handles {
            for (query_seq_index, query_out) in handle.join().unwrap() {
                results[query_seq_index as usize] = query_out;
            }
        }
    });

    let out = results.concat();
    if !tsv_out_file_name.is_empty() {
        std::fs::write(tsv_out_file_name, &out).expect("failed to write searchpd TSV");
    }
    out
}

/// C++-literal test sidecar for `cmd_searchpd`.
///
/// The translated public threaded command deliberately merges by query index.
/// This helper instead replays an explicit OpenMP-visible pair execution/write
/// order: the first pair for a query performs that query's `ProgressStep`, and
/// qualifying hits are appended immediately as the C++ locked `fprintf` would.
#[track_caller]
pub fn cmd_searchpd_cpp_literal_with_pair_schedule<FViterbi, FDist>(
    input_file_name: &str,
    db_file_name: &str,
    max_pd: f64,
    tsv_out_file_name: &str,
    pair_schedule: &[(uint, uint)],
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> String
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let (query, db) = searchpd_setup(input_file_name, db_file_name, max_pd);
    let query_seq_count = query.seqs.len() as uint;
    let db_seq_count = db.seqs.len() as uint;
    let mut pair_done = vec![false; (query_seq_count * db_seq_count) as usize];
    let mut query_started = vec![false; query_seq_count as usize];
    let mut counter = 0_u32;
    let mut out = String::new();

    let mut run_pair = |query_seq_index: uint,
                        db_seq_index: uint,
                        pair_done: &mut [bool],
                        query_started: &mut [bool],
                        counter: &mut uint,
                        out: &mut String| {
        if query_seq_index >= query_seq_count || db_seq_index >= db_seq_count {
            return;
        }
        let pair_slot = (query_seq_index * db_seq_count + db_seq_index) as usize;
        if pair_done[pair_slot] {
            return;
        }
        pair_done[pair_slot] = true;
        if !query_started[query_seq_index as usize] {
            let _ = progress_step(*counter, query_seq_count, "Searching");
            *counter += 1;
            query_started[query_seq_index as usize] = true;
        }

        let q = &query.seqs[query_seq_index as usize];
        let t = &db.seqs[db_seq_index as usize];
        let seq_q = sequence_get_seq_as_string(q).into_bytes();
        let seq_t = sequence_get_seq_as_string(t).into_bytes();
        let d = align_and_prot_dist(
            &seq_q,
            seq_q.len() as uint,
            &seq_t,
            seq_t.len() as uint,
            |seqi, li, seqj, lj| viterbi_fast_mem(seqi, li, seqj, lj),
            |row_x, row_y, col_count| get_prot_dist(row_x, row_y, col_count),
        );
        if d <= max_pd {
            out.push_str(&format!(
                "{}\t{}\t{}\n",
                q.label,
                t.label,
                format_searchpd_dist_g3(d)
            ));
        }
    };

    for &(query_seq_index, db_seq_index) in pair_schedule {
        run_pair(
            query_seq_index,
            db_seq_index,
            &mut pair_done,
            &mut query_started,
            &mut counter,
            &mut out,
        );
    }
    for query_seq_index in 0..query_seq_count {
        for db_seq_index in 0..db_seq_count {
            run_pair(
                query_seq_index,
                db_seq_index,
                &mut pair_done,
                &mut query_started,
                &mut counter,
                &mut out,
            );
        }
    }

    if !tsv_out_file_name.is_empty() {
        std::fs::write(tsv_out_file_name, &out).expect("failed to write searchpd TSV");
    }
    out
}
