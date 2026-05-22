// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeqInfo {
    pub index: uint,
    pub label: String,
    pub seq: Vec<byte>,
    pub label_buffer: String,
    pub seq_buffer: Vec<byte>,
    pub l: uint,
    pub rev_comp: bool,
    pub label_bytes: uint,
    pub max_l: uint,
    pub max_label_bytes: uint,
    pub orf_nuc_seq: Option<Box<SeqInfo>>,
    pub is_orf: bool,
    pub orf_nuc_lo: uint,
    pub orf_nuc_hi: uint,
    pub orf_nuc_l: uint,
    pub orf_frame: i32,
} // original: SeqInfo (muscle/src/seqinfo.h)

/// Construct a fresh, empty SeqInfo with default sentinel values.
#[track_caller]
pub fn seq_info_seq_info() -> SeqInfo {
    SeqInfo {
        index: uint::MAX,
        label: String::new(),
        seq: Vec::new(),
        label_buffer: String::new(),
        seq_buffer: Vec::new(),
        l: 0,
        rev_comp: false,
        label_bytes: 0,
        max_l: 0,
        max_label_bytes: 0,
        orf_nuc_seq: None,
        is_orf: false,
        orf_nuc_lo: uint::MAX,
        orf_nuc_hi: uint::MAX,
        orf_nuc_l: uint::MAX,
        orf_frame: 0,
    }
}

/// Destructor equivalent: release all owned buffers and reset to default.
#[track_caller]
pub fn seq_info_destructor_seq_info(si: &mut SeqInfo) {
    *si = seq_info_seq_info();
}

/// ObjMgr callback: reset transient fields when the reference count drops to zero.
#[track_caller]
pub fn seq_info_on_zero_ref_count(si: &mut SeqInfo) {
    si.index = uint::MAX;
    si.seq.clear();
    si.label.clear();
    si.l = 0;
    si.rev_comp = false;
    si.is_orf = false;
}

/// Deep-copy `rhs` into `si`, including label, sequence and ORF metadata.
#[track_caller]
pub fn seq_info_copy(si: &mut SeqInfo, rhs: &SeqInfo) {
    seq_info_alloc_seq(si, rhs.l);

    si.index = rhs.index;
    si.l = rhs.l;

    let label_bytes = rhs.label.len() as uint + 1;
    seq_info_alloc_label(si, label_bytes);

    si.label_buffer = rhs.label.clone();
    si.seq_buffer.clear();
    si.seq_buffer.extend_from_slice(&rhs.seq[..rhs.l as usize]);

    si.seq = si.seq_buffer.clone();
    si.label = si.label_buffer.clone();

    si.is_orf = rhs.is_orf;
    si.orf_frame = rhs.orf_frame;
    si.orf_nuc_lo = rhs.orf_nuc_lo;
    si.orf_nuc_hi = rhs.orf_nuc_hi;
    si.orf_nuc_l = rhs.orf_nuc_l;
    si.orf_nuc_seq = rhs.orf_nuc_seq.clone();
}

/// Re-initialize `si` for reuse with the given index, retaining buffer allocations.
#[track_caller]
pub fn seq_info_init(si: &mut SeqInfo, index: uint) {
    si.index = index;
    si.l = 0;
    si.label_bytes = 0;
    si.seq = si.seq_buffer.clone();
    si.label = si.label_buffer.clone();
    si.is_orf = false;
    si.rev_comp = false;
}

/// Ensure the label buffer can hold at least `n` bytes, growing if needed.
#[track_caller]
pub fn seq_info_alloc_label(si: &mut SeqInfo, n: uint) {
    if n <= si.max_label_bytes {
        return;
    }

    let new_max_label_bytes = n + 128;
    si.label_buffer = String::with_capacity(new_max_label_bytes as usize);
    si.label = si.label_buffer.clone();
    si.max_label_bytes = new_max_label_bytes;
}

/// Ensure the sequence buffer can hold at least `n` bytes, using a tiered growth policy.
#[track_caller]
pub fn seq_info_alloc_seq(si: &mut SeqInfo, n: uint) {
    if n < si.max_l {
        si.seq = si.seq_buffer.clone();
        return;
    }

    let mut new_max_l = if n < 10000 {
        let x = 2 * n + 4096;
        x.div_ceil(4096) * 4096
    } else if n < 1000000 {
        let x = 2 * n + 65536;
        x.div_ceil(65536) * 65536
    } else {
        let x = (3 * n) / 2 + 1048576;
        x.div_ceil(1048576) * 1048576
    };
    if new_max_l < si.l || new_max_l < n {
        new_max_l = n + 4096;
    }
    let mut new_seq_buffer = Vec::with_capacity(new_max_l as usize);
    if si.l > 0 {
        new_seq_buffer.extend_from_slice(&si.seq[..si.l as usize]);
    }
    si.seq = new_seq_buffer.clone();
    si.seq_buffer = new_seq_buffer;
    si.max_l = new_max_l;
}

/// Pad the sequence up to length `l` with character `c`.
#[track_caller]
pub fn seq_info_pad(si: &mut SeqInfo, l: uint, c: char, _q: char) {
    if l <= si.l {
        return;
    }
    seq_info_alloc_seq(si, l);

    if si.seq_buffer.len() < l as usize {
        si.seq_buffer.resize(l as usize, 0);
    }
    for i in si.l as usize..l as usize {
        si.seq_buffer[i] = c as byte;
    }
    si.l = l;
    si.seq = si.seq_buffer[..si.l as usize].to_vec();
}

/// Set the SeqInfo label, growing the label buffer as needed.
#[track_caller]
pub fn seq_info_set_label(si: &mut SeqInfo, label: &str) {
    let n = label.len() as uint + 1;
    seq_info_alloc_label(si, n);
    si.label_bytes = n;
    si.label_buffer.clear();
    si.label_buffer.push_str(label);
    si.label = si.label_buffer.clone();
}

/// Set index/label/sequence by copying bytes into owned buffers.
#[track_caller]
pub fn seq_info_set_copy(si: &mut SeqInfo, index: uint, label: &str, seq: &[byte]) {
    si.index = index;
    seq_info_set_label(si, label);
    seq_info_alloc_seq(si, seq.len() as uint);
    si.seq_buffer.clear();
    si.seq_buffer.extend_from_slice(seq);
    si.seq = si.seq_buffer.clone();
    si.l = seq.len() as uint;
    si.is_orf = false;
}

/// Set index/label/sequence by reference-like (still copied) assignment.
#[track_caller]
pub fn seq_info_set_ptrs(si: &mut SeqInfo, index: uint, label: &str, seq: &[byte]) {
    si.index = index;
    si.label = label.to_string();
    si.seq = seq.to_vec();
    si.l = seq.len() as uint;
    si.is_orf = false;
}

/// Populate `rev_si` with the reverse of `si` (does not complement).
#[track_caller]
pub fn seq_info_get_reverse(si: &SeqInfo, rev_si: &mut SeqInfo) {
    seq_info_alloc_seq(rev_si, si.l);
    seq_info_set_label(rev_si, &si.label);
    rev_si.index = uint::MAX;

    rev_si.seq_buffer.clear();
    rev_si.seq_buffer.resize(si.l as usize, 0);
    for i in 0..si.l as usize {
        let c = si.seq[i];
        rev_si.seq_buffer[si.l as usize - i - 1] = c;
    }

    rev_si.seq = rev_si.seq_buffer.clone();
    rev_si.l = si.l;
    rev_si.rev_comp = !si.rev_comp;
    rev_si.index = si.index;
}

/// Render `si` as a multi-line log string with label and full sequence.
#[track_caller]
pub fn seq_info_log_me(si: &SeqInfo) -> String {
    format!(
        "SeqInfo L {}, MaxL {} >{}\n{}\n",
        si.l,
        si.max_l,
        si.label,
        String::from_utf8_lossy(&si.seq[..si.l as usize])
    )
}

/// Total bytes of label + sequence buffer allocated.
#[track_caller]
pub fn seq_info_get_mem_bytes(si: &SeqInfo) -> uint {
    si.max_label_bytes + si.max_l
}

/// "Input length": ORF nuc length when this is an ORF, else the regular length.
#[track_caller]
pub fn seq_info_get_il(si: &SeqInfo) -> uint {
    if si.is_orf {
        return si.orf_nuc_l;
    }
    si.l
}

/// Format the sequence as a FASTA record with the supplied label.
#[track_caller]
pub fn seq_info_to_fasta(si: &SeqInfo, label: &str) -> String {
    if si.l == 0 {
        return String::new();
    }

    seq_to_fasta_l2571(&si.seq[..si.l as usize], Some(label))
}

/// Format as FASTA/FASTQ (FASTX). This SeqInfo lacks quality, so behaves like ToFasta.
#[track_caller]
pub fn seq_info_to_fastx(si: &SeqInfo, label: &str) -> String {
    seq_info_to_fasta(si, label)
}

/// Truncate the sequence to length `l` if currently longer.
#[track_caller]
pub fn seq_info_truncate_length(si: &mut SeqInfo, l: uint) {
    if si.l >= l {
        si.l = l;
        si.seq.truncate(l as usize);
        si.seq_buffer.truncate(l as usize);
    }
}

/// Drop the last `n` characters of the sequence.
#[track_caller]
pub fn seq_info_strip_right(si: &mut SeqInfo, n: uint) {
    assert!(n < si.l);
    si.l -= n;
    si.seq.truncate(si.l as usize);
    si.seq_buffer.truncate(si.l as usize);
}

/// Drop the first `n` characters of the sequence.
#[track_caller]
pub fn seq_info_strip_left(si: &mut SeqInfo, n: uint) {
    assert!(n < si.l);
    si.l -= n;
    for i in 0..si.l as usize {
        si.seq_buffer[i] = si.seq_buffer[i + n as usize];
    }
    si.seq_buffer.truncate(si.l as usize);
    si.seq = si.seq_buffer.clone();
}

/// Remove gap characters ('-' and '.') from the sequence in place.
#[track_caller]
pub fn seq_info_strip_gaps(si: &mut SeqInfo) {
    seq_info_alloc_seq(si, si.l);
    let mut new_l = 0_usize;
    for i in 0..si.l as usize {
        let c = si.seq[i];
        if c == b'.' || c == b'-' {
            continue;
        }
        if si.seq_buffer.len() <= new_l {
            si.seq_buffer.push(c);
        } else {
            si.seq_buffer[new_l] = c;
        }
        new_l += 1;
    }
    si.l = new_l as uint;
    si.seq_buffer.truncate(new_l);
    si.seq = si.seq_buffer.clone();
}

/// Count characters that are wildcards (outside the 4-letter nucleo or 20-letter amino alphabet).
#[track_caller]
pub fn seq_info_get_wildcard_count(si: &SeqInfo, nucleo: bool) -> uint {
    let mut count = 0;
    for i in 0..si.l as usize {
        let c = si.seq[i];
        let letter = if nucleo {
            match (c as char).to_ascii_uppercase() {
                'A' => 0,
                'C' => 1,
                'G' => 2,
                'T' | 'U' => 3,
                _ => uint::MAX,
            }
        } else {
            match (c as char).to_ascii_uppercase() {
                'A' => 0,
                'C' => 1,
                'D' => 2,
                'E' => 3,
                'F' => 4,
                'G' => 5,
                'H' => 6,
                'I' => 7,
                'K' => 8,
                'L' => 9,
                'M' => 10,
                'N' => 11,
                'P' => 12,
                'Q' => 13,
                'R' => 14,
                'S' => 15,
                'T' => 16,
                'V' => 17,
                'W' => 18,
                'Y' => 19,
                _ => uint::MAX,
            }
        };
        let alpha_size = if nucleo { 4 } else { 20 };
        if letter >= alpha_size {
            count += 1;
        }
    }
    count
}

/// Count 'N'/'n' characters in the sequence.
#[track_caller]
pub fn seq_info_get_n_count(si: &SeqInfo) -> uint {
    let mut count = 0;
    for i in 0..si.l as usize {
        let c = si.seq[i];
        if c == b'N' || c == b'n' {
            count += 1;
        }
    }
    count
}
