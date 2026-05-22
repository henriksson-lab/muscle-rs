// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const DICT_SIZE_33: usize = 20 * 20 * 20;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiSequence {
    pub seqs: Vec<Sequence>,
    pub owners: Vec<bool>,
    pub dupe_labels_ok: bool,
    pub id_count: uint,
    pub id_to_seq_index: Vec<uint>,
    pub seq_index_to_id: Vec<uint>,
} // original: MultiSequence (muscle/src/kmerdist33.h)

#[derive(Clone, Debug, Default)]
pub struct KmerDist33; // original: KmerDist33 (muscle/src/kmerdist33.h)

/// Encodes the first three amino-acid letters of `seq` as a base-20 kmer index.
#[track_caller]
pub fn kmer_dist33_seq_to_kmer(seq: &[byte]) -> uint {
    assert!(seq.len() >= 3);
    let letter = |c: byte| -> uint {
        match c {
            b'A' | b'a' => 0,
            b'C' | b'c' => 1,
            b'D' | b'd' => 2,
            b'E' | b'e' => 3,
            b'F' | b'f' => 4,
            b'G' | b'g' => 5,
            b'H' | b'h' => 6,
            b'I' | b'i' => 7,
            b'K' | b'k' => 8,
            b'L' | b'l' => 9,
            b'M' | b'm' => 10,
            b'N' | b'n' => 11,
            b'P' | b'p' => 12,
            b'Q' | b'q' => 13,
            b'R' | b'r' => 14,
            b'S' | b's' => 15,
            b'T' | b't' => 16,
            b'V' | b'v' => 17,
            b'W' | b'w' => 18,
            b'Y' | b'y' => 19,
            _ => 0xff,
        }
    };
    let u1 = letter(seq[0]);
    let u2 = letter(seq[1]);
    let u3 = letter(seq[2]);
    u1 + u2 * 20 + u3 * 20 * 20
}

/// Builds the 3-mer histogram for `seq` over the 20^3 amino-acid alphabet.
#[track_caller]
pub fn kmer_dist33_count_kmers(seq: &[byte]) -> Vec<byte> {
    let mut kmer_to_count = vec![0u8; DICT_SIZE_33];
    for i in 0..seq.len() {
        if i + 5 >= seq.len() {
            break;
        }
        let kmer = kmer_dist33_seq_to_kmer(&seq[i..]);
        if kmer as usize >= DICT_SIZE_33 {
            continue;
        }
        kmer_to_count[kmer as usize] = kmer_to_count[kmer as usize].wrapping_add(1);
    }
    kmer_to_count
}

/// Sums the per-kmer minimum across two histograms (number of shared 3-mers).
#[track_caller]
pub fn kmer_dist33_get_common_kmer_count(kmer_to_count1: &[byte], kmer_to_count2: &[byte]) -> uint {
    assert!(kmer_to_count1.len() >= DICT_SIZE_33);
    assert!(kmer_to_count2.len() >= DICT_SIZE_33);
    let mut sum = 0;
    for kmer in 0..DICT_SIZE_33 {
        sum += uint::from(std::cmp::min(kmer_to_count1[kmer], kmer_to_count2[kmer]));
    }
    sum
}

/// Builds the pairwise distance matrix from shared-3-mer fractions.
#[track_caller]
pub fn kmer_dist33_get_dist_mx(ms: &MultiSequence) -> Vec<Vec<f32>> {
    let seq_count = ms.seqs.len();
    let mut dist_mx = vec![vec![0.0f32; seq_count]; seq_count];
    for seq_indexi in 0..seq_count {
        let seqi: Vec<byte> = ms.seqs[seq_indexi]
            .char_vec
            .iter()
            .map(|c| *c as byte)
            .collect();
        let kmer_to_counti = kmer_dist33_count_kmers(&seqi);
        let common_countii =
            kmer_dist33_get_common_kmer_count(&kmer_to_counti, &kmer_to_counti) as f32;
        dist_mx[seq_indexi][seq_indexi] = 0.0;

        for seq_indexj in 0..seq_indexi {
            let seqj: Vec<byte> = ms.seqs[seq_indexj]
                .char_vec
                .iter()
                .map(|c| *c as byte)
                .collect();
            let kmer_to_countj = kmer_dist33_count_kmers(&seqj);
            let common_countjj =
                kmer_dist33_get_common_kmer_count(&kmer_to_countj, &kmer_to_countj) as f32;
            let common_countij =
                kmer_dist33_get_common_kmer_count(&kmer_to_counti, &kmer_to_countj) as f32;
            let d1 = 3.0 * (common_countii - common_countij) / common_countii;
            let d2 = 3.0 * (common_countjj - common_countij) / common_countjj;
            let d_min = d1.min(d2);
            dist_mx[seq_indexi][seq_indexj] = d_min;
            dist_mx[seq_indexj][seq_indexi] = d_min;
        }
    }
    dist_mx
}
