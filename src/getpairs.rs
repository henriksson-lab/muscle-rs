// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Enumerates all unordered pairs `(i, j)` with `i < j` over `0..count`.
#[track_caller]
pub fn get_all_pairs_l3(count: uint) -> (Vec<uint>, Vec<uint>) {
    let mut indexes1 = Vec::new();
    let mut indexes2 = Vec::new();
    for i in 0..count {
        for j in i + 1..count {
            indexes1.push(i);
            indexes2.push(j);
        }
    }
    (indexes1, indexes2)
}

/// Enumerates the full Cartesian product of indices from two sets of given sizes.
#[track_caller]
pub fn get_all_pairs_l18(count1: uint, count2: uint) -> (Vec<uint>, Vec<uint>) {
    let mut indexes1 = Vec::new();
    let mut indexes2 = Vec::new();
    for i in 0..count1 {
        for j in 0..count2 {
            indexes1.push(i);
            indexes2.push(j);
        }
    }
    (indexes1, indexes2)
}

/// Returns up to `target_pair_count` random (or all) pairs from two index ranges.
#[track_caller]
pub fn get_pairs(count1: uint, count2: uint, target_pair_count: uint) -> (Vec<uint>, Vec<uint>) {
    let all_pair_count = count1 * count2;
    if target_pair_count == uint::MAX || all_pair_count < target_pair_count * 3 / 2 {
        return get_all_pairs_l18(count1, count2);
    }

    let mut pair_set = std::collections::BTreeSet::new();
    let max_counter = target_pair_count * 10;
    let mut counter = 0;
    while counter < max_counter && (pair_set.len() as uint) < target_pair_count {
        counter += 1;
        let i = randu32() % count1;
        let j = randu32() % count2;
        if i == j {
            continue;
        }
        pair_set.insert((i, j));
    }

    let pair_count = pair_set.len() as uint;
    assert!(pair_count > target_pair_count / 2);
    pair_set.into_iter().unzip()
}
