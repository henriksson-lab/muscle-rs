// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct GlobalInputState {
    pub(crate) ms: Option<MultiSequence>,
    pub(crate) seq_count: uint,
    pub(crate) mean_seq_length: f64,
    pub(crate) max_seq_length: uint,
    pub(crate) label_to_idx: std::collections::BTreeMap<String, uint>,
    pub(crate) label_to_seq: std::collections::BTreeMap<String, Sequence>,
}

pub(crate) static GLOBAL_INPUT_STATE: std::sync::Mutex<GlobalInputState> =
    std::sync::Mutex::new(GlobalInputState {
        ms: None,
        seq_count: 0,
        mean_seq_length: 0.0,
        max_seq_length: 0,
        label_to_idx: std::collections::BTreeMap::new(),
        label_to_seq: std::collections::BTreeMap::new(),
    });

/// Returns the global sequence index for `label`, panicking if the label is unknown.
#[track_caller]
pub fn get_gsi_by_label(label: &str) -> uint {
    let state = GLOBAL_INPUT_STATE.lock().unwrap();
    *state
        .label_to_idx
        .get(label)
        .unwrap_or_else(|| panic!("GetGSIByLabel({label})"))
}

/// Returns the FASTA label for the global sequence at index `gsi`.
#[track_caller]
pub fn get_label_by_gsi(gsi: uint) -> String {
    let state = GLOBAL_INPUT_STATE.lock().unwrap();
    let ms = state.ms.as_ref().expect("GetLabelByGSI, global MS not set");
    ms.seqs[gsi as usize].label.clone()
}

/// Returns the length of the global sequence at index `gsi`.
#[track_caller]
pub fn get_seq_length_by_gsi(gsi: uint) -> uint {
    let state = GLOBAL_INPUT_STATE.lock().unwrap();
    let ms = state
        .ms
        .as_ref()
        .expect("GetSeqLengthByGSI, global MS not set");
    ms.seqs[gsi as usize].char_vec.len() as uint
}

/// Returns the length of the global sequence with the given label.
#[track_caller]
pub fn get_seq_length_by_global_label(label: &str) -> uint {
    let seq = get_global_input_seq_by_label(label);
    seq.char_vec.len() as uint
}

/// Returns the global `Sequence` whose label matches `label`.
#[track_caller]
pub fn get_sequence_by_global_label(label: &str) -> Sequence {
    let gsi = get_gsi_by_label(label);
    get_sequence_by_gsi(gsi)
}

/// Returns a clone of the global `Sequence` at index `gsi`.
#[track_caller]
pub fn get_sequence_by_gsi(gsi: uint) -> Sequence {
    let state = GLOBAL_INPUT_STATE.lock().unwrap();
    let ms = state
        .ms
        .as_ref()
        .expect("GetSequenceByGSI, global MS not set");
    ms.seqs[gsi as usize].clone()
}

/// Returns the global sequence at `gsi` as a vector of raw bytes.
#[track_caller]
pub fn get_byte_seq_by_gsi(gsi: uint) -> Vec<byte> {
    let seq = get_sequence_by_gsi(gsi);
    seq.char_vec.iter().map(|&c| c as byte).collect()
}

/// Inserts a temporary sequence into the global label->sequence map.
#[track_caller]
pub fn add_global_tmp_seq(seq: &Sequence) {
    let mut state = GLOBAL_INPUT_STATE.lock().unwrap();
    state.label_to_seq.insert(seq.label.clone(), seq.clone());
}

/// Installs `ms` as the global input MS and rebuilds the label/index/length caches.
#[track_caller]
pub fn set_global_input_ms(ms: &MultiSequence) {
    let mut state = GLOBAL_INPUT_STATE.lock().unwrap();
    state.ms = Some(ms.clone());
    state.seq_count = ms.seqs.len() as uint;
    state.mean_seq_length = 0.0;
    state.max_seq_length = 0;
    state.label_to_idx.clear();
    state.label_to_seq.clear();
    let mut sum_seq_length = 0.0;
    for gsi in 0..state.seq_count {
        let seq = &ms.seqs[gsi as usize];
        let label = seq.label.clone();
        if state.label_to_idx.contains_key(&label) {
            panic!("Error duplicate label in input >{label}");
        }
        state.label_to_idx.insert(label.clone(), gsi);
        state.label_to_seq.insert(label, seq.clone());
        let l = seq.char_vec.len() as uint;
        state.max_seq_length = state.max_seq_length.max(l);
        sum_seq_length += f64::from(l);
    }
    if state.seq_count > 0 {
        state.mean_seq_length = sum_seq_length / f64::from(state.seq_count);
    }
}

/// Returns a clone of the currently installed global input MS.
#[track_caller]
pub fn get_global_input_ms() -> MultiSequence {
    GLOBAL_INPUT_STATE
        .lock()
        .unwrap()
        .ms
        .clone()
        .expect("GetGlobalInputMS, global MS not set")
}

/// Returns the number of sequences in the global input MS.
#[track_caller]
pub fn get_global_ms_seq_count() -> uint {
    GLOBAL_INPUT_STATE.lock().unwrap().seq_count
}

/// Returns the mean sequence length cached for the global input MS.
#[track_caller]
pub fn get_global_ms_mean_seq_length() -> f64 {
    GLOBAL_INPUT_STATE.lock().unwrap().mean_seq_length
}

/// Returns the number of global sequence indexes (alias for global MS seq count).
#[track_caller]
pub fn get_gsi_count() -> uint {
    get_global_ms_seq_count()
}

/// Returns a clone of the global input `Sequence` at the given index `gsi`.
#[track_caller]
pub fn get_global_input_seq_by_index(gsi: uint) -> Sequence {
    let state = GLOBAL_INPUT_STATE.lock().unwrap();
    assert!(gsi < state.seq_count);
    let ms = state
        .ms
        .as_ref()
        .expect("GetGlobalInputSeqByIndex, global MS not set");
    ms.seqs[gsi as usize].clone()
}

/// Returns the global input `Sequence` whose label matches `label`.
#[track_caller]
pub fn get_global_input_seq_by_label(label: &str) -> Sequence {
    let state = GLOBAL_INPUT_STATE.lock().unwrap();
    let seq = state
        .label_to_seq
        .get(label)
        .unwrap_or_else(|| panic!("GetGlobalInputSeqByLabel({label})"));
    assert!(seq.label == label);
    seq.clone()
}

/// Returns the global sequence's raw bytes for the sequence with the given label.
#[track_caller]
pub fn get_global_byte_seq_by_label(label: &str) -> Vec<byte> {
    let seq = get_global_input_seq_by_label(label);
    seq.char_vec.iter().map(|&c| c as byte).collect()
}
