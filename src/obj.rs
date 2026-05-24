// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct ObjMgr {
    pub free: std::collections::BTreeMap<ObjType, Vec<Obj>>,
    pub busy: std::collections::BTreeMap<ObjType, Vec<Obj>>,
} // original: ObjMgr (muscle/src/obj.h)

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Obj {
    pub type_: ObjType,
    pub ref_count: uint,
    pub id: usize,
} // original: Obj (muscle/src/obj.h)
