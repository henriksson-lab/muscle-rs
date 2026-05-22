// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Profile-vs-profile substitution score using `PPA`'s sort order and `PPB`'s AA scores.
#[track_caller]
pub fn score_prof_pos2(ppa: &ProfPos3, ppb: &ProfPos3) -> f32 {
    let mut score = 0.0;
    for n in 0..20 {
        let letter = ppa.sort_order[n] as usize;
        let freq_a = ppa.freqs[letter];
        if freq_a == 0.0 {
            break;
        }
        score += freq_a * ppb.aa_scores[letter];
    }
    score
}

/// Records the traceback predecessor (`M`/`D`/`I`) for the M state at cell `(i, j)`.
#[track_caller]
pub fn set_bit_tbm(tb: &mut [Vec<byte>], i: uint, j: uint, c: byte) {
    let bit = match c {
        b'M' => BIT_MM,
        b'D' => BIT_DM,
        b'I' => BIT_IM,
        _ => panic!("SetBitTBM invalid char {}", c as char),
    };
    tb[i as usize][j as usize] &= !BIT_xM;
    tb[i as usize][j as usize] |= bit;
}

/// Records the traceback predecessor (`M`/`D`) for the D state at cell `(i, j)`.
#[track_caller]
pub fn set_bit_tbd(tb: &mut [Vec<byte>], i: uint, j: uint, c: byte) {
    let bit = match c {
        b'M' => BIT_MD,
        b'D' => BIT_DD,
        _ => panic!("SetBitTBD invalid char {}", c as char),
    };
    tb[i as usize][j as usize] &= !BIT_xD;
    tb[i as usize][j as usize] |= bit;
}

/// Records the traceback predecessor (`M`/`I`) for the I state at cell `(i, j)`.
#[track_caller]
pub fn set_bit_tbi(tb: &mut [Vec<byte>], i: uint, j: uint, c: byte) {
    let bit = match c {
        b'M' => BIT_MI,
        b'I' => BIT_II,
        _ => panic!("SetBitTBI invalid char {}", c as char),
    };
    tb[i as usize][j as usize] &= !BIT_xI;
    tb[i as usize][j as usize] |= bit;
}

/// Memory-efficient Needleman-Wunsch on two `Profile3`s; returns alignment score and path string.
#[track_caller]
pub fn nw_small3(cm: &mut CacheMem3, prof_a: &Profile3, prof_b: &Profile3) -> (f32, String) {
    let u_length_a = prof_a.pps.len() as uint;
    let u_length_b = prof_b.pps.len() as uint;
    let u_prefix_count_a = u_length_a + 1;
    let u_prefix_count_b = u_length_b + 1;
    assert!(u_length_a > 0);
    assert!(u_length_b > 0);

    let e = 0.0_f32;
    let mut m_curr = vec![MINUS_INFINITY; u_prefix_count_b as usize];
    let mut m_next = vec![MINUS_INFINITY; u_prefix_count_b as usize];
    let mut m_prev = vec![MINUS_INFINITY; u_prefix_count_b as usize];
    let mut d_row = vec![MINUS_INFINITY; u_prefix_count_b as usize];
    let mut tb = vec![vec![0_u8; u_prefix_count_b as usize]; u_prefix_count_a as usize];

    for j in 2..=u_length_b {
        set_bit_tbi(&mut tb, 0, j, b'I');
    }

    for j in 0..=u_length_b {
        d_row[j as usize] = MINUS_INFINITY;
        set_bit_tbd(&mut tb, 0, j, b'D');
    }

    m_prev[0] = 0.0;
    for j in 1..=u_length_b {
        m_prev[j as usize] = MINUS_INFINITY;
    }

    m_curr[0] = MINUS_INFINITY;
    m_curr[1] = score_prof_pos2(&prof_a.pps[0], &prof_b.pps[0]);
    set_bit_tbm(&mut tb, 1, 1, b'M');

    for j in 2..=u_length_b {
        m_curr[j as usize] = score_prof_pos2(&prof_a.pps[0], &prof_b.pps[j as usize - 1])
            + prof_b.pps[0].gap_open_score
            + (j - 2) as f32 * e
            + prof_b.pps[j as usize - 2].gap_close_score;
        set_bit_tbm(&mut tb, 1, j, b'I');
    }

    for i in 1..u_length_a {
        let mut iij = MINUS_INFINITY;
        d_row[0] = prof_a.pps[0].gap_open_score + (i - 1) as f32 * e;

        m_curr[0] = MINUS_INFINITY;
        if i == 1 {
            m_curr[1] = score_prof_pos2(&prof_a.pps[0], &prof_b.pps[0]);
            set_bit_tbm(&mut tb, i, 1, b'M');
        } else {
            m_curr[1] = score_prof_pos2(&prof_a.pps[i as usize - 1], &prof_b.pps[0])
                + prof_a.pps[0].gap_open_score
                + (i - 2) as f32 * e
                + prof_a.pps[i as usize - 2].gap_close_score;
            set_bit_tbm(&mut tb, i, 1, b'D');
        }

        for j in 1..u_length_b {
            m_next[j as usize + 1] =
                score_prof_pos2(&prof_a.pps[i as usize], &prof_b.pps[j as usize]);
        }

        for j in 1..u_length_b {
            let jj = j as usize;
            let dd = d_row[jj] + e;
            let md = m_prev[jj] + prof_a.pps[i as usize - 1].gap_open_score;
            if dd > md {
                d_row[jj] = dd;
            } else {
                d_row[jj] = md;
                set_bit_tbd(&mut tb, i, j, b'M');
            }

            iij += e;
            let mi = m_curr[jj - 1] + prof_b.pps[jj - 1].gap_open_score;
            if mi >= iij {
                iij = mi;
                set_bit_tbi(&mut tb, i, j, b'M');
            }

            let dm = d_row[jj] + prof_a.pps[i as usize - 1].gap_close_score;
            let im = iij + prof_b.pps[jj - 1].gap_close_score;
            let mm = m_curr[jj];
            tb[i as usize + 1][jj + 1] &= !BIT_xM;
            if mm >= dm && mm >= im {
                m_next[jj + 1] += mm;
                tb[i as usize + 1][jj + 1] |= BIT_MM;
            } else if dm >= mm && dm >= im {
                m_next[jj + 1] += dm;
                tb[i as usize + 1][jj + 1] |= BIT_DM;
            } else {
                assert!(im >= mm && im >= dm);
                m_next[jj + 1] += im;
                tb[i as usize + 1][jj + 1] |= BIT_IM;
            }
        }

        let jj = u_length_b as usize;
        let dd = d_row[jj] + e;
        let md = m_prev[jj] + prof_a.pps[i as usize - 1].gap_open_score;
        if dd > md {
            d_row[jj] = dd;
        } else {
            d_row[jj] = md;
            set_bit_tbd(&mut tb, i, u_length_b, b'M');
        }

        iij += e;
        let mi = m_curr[jj - 1] + prof_b.pps[jj - 1].gap_open_score;
        if mi >= iij {
            set_bit_tbi(&mut tb, i, u_length_b, b'M');
        }

        let old_prev = m_prev;
        m_prev = m_curr;
        m_curr = m_next;
        m_next = old_prev;
    }

    m_curr[0] = MINUS_INFINITY;
    if u_length_a > 1 {
        m_curr[1] = score_prof_pos2(&prof_a.pps[u_length_a as usize - 1], &prof_b.pps[0])
            + (u_length_a - 2) as f32 * e
            + prof_a.pps[0].gap_open_score
            + prof_a.pps[u_length_a as usize - 2].gap_close_score;
    } else {
        m_curr[1] = score_prof_pos2(&prof_a.pps[u_length_a as usize - 1], &prof_b.pps[0])
            + prof_a.pps[0].gap_open_score
            + prof_a.pps[0].gap_close_score;
    }
    set_bit_tbm(&mut tb, u_length_a, 1, b'D');

    d_row[0] = MINUS_INFINITY;
    for j in 1..=u_length_b {
        let jj = j as usize;
        let dd = d_row[jj] + e;
        let md = m_prev[jj] + prof_a.pps[u_length_a as usize - 1].gap_open_score;
        if dd > md {
            d_row[jj] = dd;
        } else {
            d_row[jj] = md;
            set_bit_tbd(&mut tb, u_length_a, j, b'M');
        }
    }

    let mut iij = MINUS_INFINITY;
    for j in 1..=u_length_b {
        let jj = j as usize;
        iij += e;
        let mi = m_curr[jj - 1] + prof_b.pps[jj - 1].gap_open_score;
        if mi >= iij {
            iij = mi;
            set_bit_tbi(&mut tb, u_length_a, j, b'M');
        }
    }

    let mab = m_curr[u_length_b as usize];
    let dab = d_row[u_length_b as usize];
    let iab = iij;
    let mut score = mab;
    let mut edge_type = b'M';
    if dab > score {
        score = dab;
        edge_type = b'D';
    }
    if iab > score {
        score = iab;
        edge_type = b'I';
    }

    let path = bit_trace_back(&tb, u_length_a, u_length_b, edge_type);
    cm.cache_m_curr = m_curr;
    cm.cache_m_next = m_next;
    cm.cache_m_prev = m_prev;
    cm.cache_d_row = d_row;
    cm.cache_tb = tb;
    (score, path)
}
