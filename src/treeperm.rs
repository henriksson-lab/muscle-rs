// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum TREEPERM {
    #[default]
    TP_None = 0,
    TP_ABC = 1,
    TP_ACB = 2,
    TP_BCA = 3,
    TP_All = 4,
} // original: TREEPERM (muscle/src/treeperm.h)

/// Parse a `TREEPERM` enum value from its lowercase string form.
#[track_caller]
pub fn str_to_treeperm(s: &str) -> TREEPERM {
    match s {
        "none" => TREEPERM::TP_None,
        "abc" => TREEPERM::TP_ABC,
        "acb" => TREEPERM::TP_ACB,
        "bca" => TREEPERM::TP_BCA,
        "all" => TREEPERM::TP_All,
        _ => panic!("Invalid perm '{s}'"),
    }
}

/// Return the lowercase string name of a `TREEPERM` value.
#[track_caller]
pub fn treeperm_to_str(tp: TREEPERM) -> &'static str {
    match tp {
        TREEPERM::TP_None => "none",
        TREEPERM::TP_ABC => "abc",
        TREEPERM::TP_ACB => "acb",
        TREEPERM::TP_BCA => "bca",
        TREEPERM::TP_All => "all",
    }
}
