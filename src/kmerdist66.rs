// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const DICT_SIZE_66: usize = 6 * 6 * 6 * 6 * 6 * 6;

#[derive(Clone, Debug, Default)]
pub struct KmerDist66; // original: KmerDist66 (muscle/src/kmerdist66.h)

/// Encodes the first six residues of `seq` as a base-6 kmer index over reduced groups.
#[track_caller]
pub fn kmer_dist66_seq_to_kmer(seq: &[byte]) -> uint {
    assert!(seq.len() >= 6);
    let group = |c: byte| -> uint {
        match c {
            b'C' | b'c' => 5,
            b'D' | b'd' | b'E' | b'e' | b'N' | b'n' | b'Q' | b'q' => 2,
            b'F' | b'f' | b'W' | b'w' | b'Y' | b'y' => 4,
            b'H' | b'h' | b'K' | b'k' | b'R' | b'r' => 3,
            b'I' | b'i' | b'L' | b'l' | b'M' | b'm' | b'V' | b'v' => 1,
            _ => 0,
        }
    };
    let u1 = group(seq[0]);
    let u2 = group(seq[1]);
    let u3 = group(seq[2]);
    let u4 = group(seq[3]);
    let u5 = group(seq[4]);
    let u6 = group(seq[5]);
    u6 + u5 * 6 + u4 * 6 * 6 + u3 * 6 * 6 * 6 + u2 * 6 * 6 * 6 * 6 + u1 * 6 * 6 * 6 * 6 * 6
}

/// Builds the 6-mer histogram for `seq` over the 6^6 reduced alphabet.
#[track_caller]
pub fn kmer_dist66_count_kmers(seq: &[byte]) -> Vec<byte> {
    let mut kmer_to_count = vec![0u8; DICT_SIZE_66];
    for i in 0..seq.len() {
        if i + 5 >= seq.len() {
            break;
        }
        let kmer = kmer_dist66_seq_to_kmer(&seq[i..]);
        assert!((kmer as usize) < DICT_SIZE_66);
        kmer_to_count[kmer as usize] = kmer_to_count[kmer as usize].wrapping_add(1);
    }
    kmer_to_count
}

/// Sums the per-kmer minimum across two histograms (number of shared 6-mers).
#[track_caller]
pub fn kmer_dist66_get_common_kmer_count(kmer_to_count1: &[byte], kmer_to_count2: &[byte]) -> uint {
    assert!(kmer_to_count1.len() >= DICT_SIZE_66);
    assert!(kmer_to_count2.len() >= DICT_SIZE_66);
    let mut sum = 0;
    for kmer in 0..DICT_SIZE_66 {
        sum += uint::from(std::cmp::min(kmer_to_count1[kmer], kmer_to_count2[kmer]));
    }
    sum
}

/// Builds the pairwise distance matrix from shared 6/6 reduced-alphabet kmers.
#[track_caller]
pub fn kmer_dist66_get_dist_mx(ms: &MultiSequence) -> Vec<Vec<f32>> {
    let seq_count = ms.seqs.len();
    let mut dist_mx = vec![vec![0.0f32; seq_count]; seq_count];
    for seq_indexi in 0..seq_count {
        let seqi: Vec<byte> = ms.seqs[seq_indexi]
            .char_vec
            .iter()
            .map(|c| *c as byte)
            .collect();
        let kmer_to_counti = kmer_dist66_count_kmers(&seqi);
        let common_countii =
            kmer_dist66_get_common_kmer_count(&kmer_to_counti, &kmer_to_counti) as f32;
        dist_mx[seq_indexi][seq_indexi] = 0.0;

        for seq_indexj in 0..seq_indexi {
            let seqj: Vec<byte> = ms.seqs[seq_indexj]
                .char_vec
                .iter()
                .map(|c| *c as byte)
                .collect();
            let kmer_to_countj = kmer_dist66_count_kmers(&seqj);
            let common_countjj =
                kmer_dist66_get_common_kmer_count(&kmer_to_countj, &kmer_to_countj) as f32;
            let common_countij =
                kmer_dist66_get_common_kmer_count(&kmer_to_counti, &kmer_to_countj) as f32;
            let d1 = 3.0 * (common_countii - common_countij) / common_countii;
            let d2 = 3.0 * (common_countjj - common_countij) / common_countjj;
            let d_min = d1.min(d2);
            dist_mx[seq_indexi][seq_indexj] = d_min;
            dist_mx[seq_indexj][seq_indexi] = d_min;
        }
    }
    dist_mx
}
