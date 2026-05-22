// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Advances a multi-dimensional counter `indexes` within `sizes`; returns false when exhausted.
#[track_caller]
pub fn get_next_enum_grid(sizes: &[uint], indexes: &mut [uint]) -> bool {
    assert_eq!(indexes.len(), sizes.len());
    for i in 0..sizes.len() {
        let index = indexes[i];
        let size = sizes[i];
        if index + 1 < size {
            indexes[i] += 1;
            return true;
        }
        indexes[i] = 0;
    }
    false
}
