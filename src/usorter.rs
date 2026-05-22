// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct USorter {
    pub rows: Vec<Vec<uint>>,
    pub index_seq_indexes: Vec<uint>,
    pub word_length: uint,
    pub dict_size: uint,
} // original: USorter (muscle/src/usorter.h)

/// Configure the sorter for the current alphabet (word length and dictionary size).
#[track_caller]
pub fn u_sorter_init(sorter: &mut USorter) {
    let state = ALPHA_STATE.lock().unwrap();
    assert!(state.alpha_size > 0);
    match state.alpha {
        ALPHA::ALPHA_Amino => {
            sorter.word_length = 3;
            sorter.dict_size = myipow(20, sorter.word_length);
        }
        ALPHA::ALPHA_Nucleo => {
            sorter.word_length = 8;
            sorter.dict_size = myipow(4, sorter.word_length);
        }
        _ => panic!("Invalid Alpha={:?}", state.alpha),
    }
    drop(state);
    sorter.rows.clear();
    sorter.rows.resize(sorter.dict_size as usize, Vec::new());
}

/// Encode a window of characters into a dictionary word, dispatching by alphabet.
#[track_caller]
pub fn u_sorter_chars_to_word(sorter: &USorter, chars: &[byte]) -> uint {
    let alpha = ALPHA_STATE.lock().unwrap().alpha;
    match alpha {
        ALPHA::ALPHA_Amino => u_sorter_chars_to_word_amino(sorter, chars),
        ALPHA::ALPHA_Nucleo => u_sorter_chars_to_word_nucleo(sorter, chars),
        alpha => panic!("Invalid Alpha={alpha:?}"),
    }
}

/// Encode an amino-acid window of length `word_length` as a base-20 word.
#[track_caller]
pub fn u_sorter_chars_to_word_amino(sorter: &USorter, chars: &[byte]) -> uint {
    let state = ALPHA_STATE.lock().unwrap();
    let mut word = 0;
    for i in 0..sorter.word_length as usize {
        let c = chars[i] as usize;
        let letter = state.char_to_letter[c];
        if letter >= 20 {
            return uint::MAX;
        }
        word = word * 20 + letter;
    }
    word
}

/// Encode a nucleotide window of length `word_length` as a base-4 word.
#[track_caller]
pub fn u_sorter_chars_to_word_nucleo(sorter: &USorter, chars: &[byte]) -> uint {
    let state = ALPHA_STATE.lock().unwrap();
    let mut word = 0;
    for i in 0..sorter.word_length as usize {
        let c = chars[i] as usize;
        let letter = state.char_to_letter[c];
        if letter >= 4 {
            return uint::MAX;
        }
        word = word * 4 + letter;
    }
    word
}

/// Index every word of `seq` so it can be retrieved by word lookup.
#[track_caller]
pub fn u_sorter_add_seq(sorter: &mut USorter, seq: &[byte], seq_index: uint) {
    assert!(ALPHA_STATE.lock().unwrap().alpha_size > 0);
    let index = sorter.index_seq_indexes.len() as uint;
    if seq.len() < sorter.word_length as usize {
        return;
    }
    let word_count = seq.len() + 1 - sorter.word_length as usize;
    for i in 0..word_count {
        let word = u_sorter_chars_to_word(sorter, &seq[i..]);
        if word < sorter.dict_size {
            sorter.rows[word as usize].push(index);
        }
    }
    sorter.index_seq_indexes.push(seq_index);
}

/// Rank indexed sequences by shared word count with `seq`; return top hits and their counts.
#[track_caller]
pub fn u_sorter_search_seq(sorter: &USorter, seq: &[byte]) -> (Vec<uint>, Vec<uint>) {
    let mut top_seq_indexes = Vec::new();
    let mut top_word_counts = Vec::new();
    let index_size = sorter.index_seq_indexes.len();
    if index_size == 0 {
        return (top_seq_indexes, top_word_counts);
    }
    if seq.len() < sorter.word_length as usize {
        return (top_seq_indexes, top_word_counts);
    }
    let word_count = seq.len() + 1 - sorter.word_length as usize;
    let mut u: Vec<uint> = vec![0; index_size];
    for i in 0..word_count {
        let word = u_sorter_chars_to_word(sorter, &seq[i..]);
        if word >= sorter.dict_size {
            continue;
        }
        for &target_seq_index in &sorter.rows[word as usize] {
            assert!((target_seq_index as usize) < index_size);
            u[target_seq_index as usize] += 1;
        }
    }
    let order_uint = quick_sort_order_desc_by(index_size, |a, b| u[a].cmp(&u[b]));
    let order: Vec<usize> = order_uint.iter().map(|&v| v as usize).collect();
    let top_seq_index = order[0];
    let top_word_count = u[top_seq_index];
    let mut min_u = (top_word_count / 2).wrapping_sub(1);
    if min_u == 0 {
        min_u = 1;
    }
    let mut last_word_count = top_word_count;
    for index in order {
        let word_count = u[index];
        assert!(word_count <= last_word_count);
        last_word_count = word_count;
        if word_count < min_u {
            break;
        }
        top_seq_indexes.push(sorter.index_seq_indexes[index]);
        top_word_counts.push(word_count);
    }
    (top_seq_indexes, top_word_counts)
}

/// CLI entry: index the DB FASTA, then report top USorter hits for each query sequence.
#[track_caller]
pub fn cmd_usorter(query_file_name: &str, db_file_name: &str) -> String {
    let mut query = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut query, query_file_name, false);

    let mut db = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut db, db_file_name, false);

    set_alpha_l209(ALPHA::ALPHA_Amino);

    let mut us = USorter::default();
    u_sorter_init(&mut us);
    let db_seq_count = db.seqs.len() as uint;
    for db_seq_index in 0..db_seq_count {
        let seq = &db.seqs[db_seq_index as usize];
        let seq_chars = sequence_get_seq_as_string(seq).into_bytes();
        u_sorter_add_seq(&mut us, &seq_chars, db_seq_index);
    }

    let mut out = String::new();
    let query_seq_count = query.seqs.len() as uint;
    for query_seq_index in 0..query_seq_count {
        let seq = &query.seqs[query_seq_index as usize];
        let seq_chars = sequence_get_seq_as_string(seq).into_bytes();
        let (top_seq_indexes, top_word_counts) = u_sorter_search_seq(&us, &seq_chars);

        let n = top_seq_indexes.len();
        assert_eq!(top_word_counts.len(), n);
        out.push('\n');
        out.push_str(&format!("Q>{}, {} hits\n", seq.label, n));
        for i in 0..n {
            let db_seq_index = top_seq_indexes[i];
            let count = top_word_counts[i];
            out.push_str(&format!(
                "  [{:4}]  {}\n",
                count, db.seqs[db_seq_index as usize].label
            ));
        }
    }
    out
}
