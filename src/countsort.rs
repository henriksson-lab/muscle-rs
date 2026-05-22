// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct CountSortMem; // original: CountSortMem (muscle/src/countsort.h)

/// Faithful port of muscle/src/sort.h `QuickSortOrderRecurse<T, Desc>`.
/// Hoare partition over an indirection `order` array. Unstable — used to
/// reproduce C++ tie-breaking exactly.
fn quick_sort_order_recurse<F>(
    values_len: usize,
    left: i64,
    right: i64,
    order: &mut [uint],
    cmp: &F,
    desc: bool,
) where
    F: Fn(usize, usize) -> std::cmp::Ordering,
{
    let _ = values_len;
    let mut i = left;
    let mut j = right;
    let mid = (left + right) / 2;
    let pivot = order[mid as usize] as usize;

    while i <= j {
        if desc {
            while cmp(order[i as usize] as usize, pivot) == std::cmp::Ordering::Greater {
                i += 1;
            }
            while cmp(order[j as usize] as usize, pivot) == std::cmp::Ordering::Less {
                j -= 1;
            }
        } else {
            while cmp(order[i as usize] as usize, pivot) == std::cmp::Ordering::Less {
                i += 1;
            }
            while cmp(order[j as usize] as usize, pivot) == std::cmp::Ordering::Greater {
                j -= 1;
            }
        }

        if i <= j {
            order.swap(i as usize, j as usize);
            i += 1;
            j -= 1;
        }
    }

    if left < j {
        quick_sort_order_recurse(values_len, left, j, order, cmp, desc);
    }
    if i < right {
        quick_sort_order_recurse(values_len, i, right, order, cmp, desc);
    }
}

/// Faithful port of muscle/src/sort.h `QuickSortOrderDesc<T>`. Returns an
/// `order` permutation that sorts `0..n` by `cmp(a, b)` in descending order,
/// using the same unstable quicksort the C++ code uses (so ties resolve the
/// same way the C++ binary does).
pub fn quick_sort_order_desc_by<F>(n: usize, cmp: F) -> Vec<uint>
where
    F: Fn(usize, usize) -> std::cmp::Ordering,
{
    let mut order: Vec<uint> = (0..n as uint).collect();
    if n == 0 {
        return order;
    }
    quick_sort_order_recurse(n, 0, (n - 1) as i64, &mut order, &cmp, true);
    order
}

/// Ascending variant matching C++ `QuickSortOrder<T>`.
pub fn quick_sort_order_by<F>(n: usize, cmp: F) -> Vec<uint>
where
    F: Fn(usize, usize) -> std::cmp::Ordering,
{
    let mut order: Vec<uint> = (0..n as uint).collect();
    if n == 0 {
        return order;
    }
    quick_sort_order_recurse(n, 0, (n - 1) as i64, &mut order, &cmp, false);
    order
}
