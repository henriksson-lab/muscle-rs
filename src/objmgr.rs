// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub(crate) static OBJ_MGR_STATE: std::sync::LazyLock<std::sync::Mutex<ObjMgr>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(ObjMgr::default()));

pub(crate) static OBJ_GLOBAL_STATS: std::sync::Mutex<([uint; 2], [uint; 2], [uint; 2], [f32; 2])> =
    std::sync::Mutex::new(([0; 2], [0; 2], [0; 2], [0.0; 2]));

/// Returns a snapshot copy of the global `ObjMgr` (subset of `OT_SeqInfo`/`OT_PathInfo`).
#[track_caller]
pub fn obj_mgr_get_obj_mgr() -> ObjMgr {
    let thread_index = get_thread_index();
    let mgr = OBJ_MGR_STATE.lock().unwrap();
    if thread_index >= 32 {
        assert!(thread_index < uint::MAX);
    }
    let mut copy = ObjMgr::default();
    for type_ in [ObjType::OT_SeqInfo, ObjType::OT_PathInfo] {
        if let Some(free) = mgr.free.get(&type_) {
            copy.free.insert(type_, free.clone());
        }
        if let Some(busy) = mgr.busy.get(&type_) {
            copy.busy.insert(type_, busy.clone());
        }
    }
    copy
}

/// Returns the textual name for an `ObjType` (e.g. `SeqInfo`).
#[track_caller]
pub fn obj_type_to_str(type_: ObjType) -> &'static str {
    match type_ {
        ObjType::OT_SeqInfo => "SeqInfo",
        ObjType::OT_PathInfo => "PathInfo",
        ObjType::OTCount => "OT_??",
    }
}

/// Returns a two-letter abbreviation for an `ObjType` (e.g. `SI`).
#[track_caller]
pub fn obj_type_to_str2(type_: ObjType) -> &'static str {
    match type_ {
        ObjType::OT_SeqInfo => "SI",
        _ => "??",
    }
}

/// Returns a freshly default-constructed `ObjMgr`.
#[track_caller]
pub fn obj_mgr_obj_mgr() -> ObjMgr {
    ObjMgr::default()
}

/// Decrements the ref-count of `obj`, freeing it when it reaches zero.
#[track_caller]
pub fn obj_mgr_down(obj: &mut Obj) {
    assert!(obj.ref_count > 0);
    obj.ref_count -= 1;
    if obj.ref_count == 0 {
        obj_mgr_free_obj(obj.clone());
    }
}

/// Increments the ref-count of `obj` (caller must already hold a reference).
#[track_caller]
pub fn obj_mgr_up(obj: &mut Obj) {
    assert!(obj.ref_count != 0);
    obj.ref_count += 1;
}

/// Allocates or recycles a managed `Obj` of `type_` from the static pool.
#[track_caller]
pub fn obj_mgr_static_get_obj(type_: ObjType) -> Obj {
    obj_mgr_thread_get_obj(type_)
}

/// Allocates a brand-new `Obj` of `type_` without consulting the free list.
#[track_caller]
pub fn obj_mgr_alloc_new(type_: ObjType) -> Obj {
    obj_mgr_validate_type(type_);
    Obj {
        type_,
        ref_count: 0,
    }
}

/// Pops a free `Obj` of `type_` (or creates one) and moves it to the busy list.
#[track_caller]
pub fn obj_mgr_thread_get_obj(type_: ObjType) -> Obj {
    obj_mgr_validate_type(type_);
    let mut mgr = OBJ_MGR_STATE.lock().unwrap();
    let mut new_obj = mgr
        .free
        .entry(type_)
        .or_default()
        .pop()
        .unwrap_or_else(|| Obj {
            type_,
            ref_count: 0,
        });
    assert_eq!(new_obj.ref_count, 0);
    new_obj.ref_count = 1;
    mgr.busy.entry(type_).or_default().push(new_obj.clone());
    new_obj
}

/// Returns `obj` from busy to free list, asserting ref-count is zero.
#[track_caller]
pub fn obj_mgr_free_obj(obj: Obj) {
    assert_eq!(obj.ref_count, 0);
    obj_mgr_validate_type(obj.type_);
    let mut mgr = OBJ_MGR_STATE.lock().unwrap();
    let busy = mgr.busy.entry(obj.type_).or_default();
    if let Some(pos) = busy.iter().position(|x| x.type_ == obj.type_) {
        busy.remove(pos);
    }
    mgr.free.entry(obj.type_).or_default().push(obj);
}

/// Returns the number of free `Obj`s of `type_`.
#[track_caller]
pub fn obj_mgr_get_free_count(type_: ObjType) -> uint {
    obj_mgr_validate_type(type_);
    OBJ_MGR_STATE
        .lock()
        .unwrap()
        .free
        .get(&type_)
        .map(|v| v.len() as uint)
        .unwrap_or(0)
}

/// Returns the number of busy `Obj`s of `type_`.
#[track_caller]
pub fn obj_mgr_get_busy_count(type_: ObjType) -> uint {
    obj_mgr_validate_type(type_);
    OBJ_MGR_STATE
        .lock()
        .unwrap()
        .busy
        .get(&type_)
        .map(|v| v.len() as uint)
        .unwrap_or(0)
}

/// Returns the maximum ref-count seen on any busy `Obj` of `type_`.
#[track_caller]
pub fn obj_mgr_get_max_ref_count(type_: ObjType) -> uint {
    obj_mgr_validate_type(type_);
    OBJ_MGR_STATE
        .lock()
        .unwrap()
        .busy
        .get(&type_)
        .and_then(|v| v.iter().map(|obj| obj.ref_count).max())
        .unwrap_or(0)
}

/// Returns total heap bytes held by busy `Obj`s of `type_`.
#[track_caller]
pub fn obj_mgr_get_total_mem(type_: ObjType) -> f32 {
    obj_mgr_validate_type(type_);
    let mgr = OBJ_MGR_STATE.lock().unwrap();
    let busy_count = mgr.busy.get(&type_).map(|v| v.len()).unwrap_or(0);
    (busy_count * std::mem::size_of::<Obj>()) as f32
}

/// Asserts that all `Obj`s of `type_` have consistent state.
#[track_caller]
pub fn obj_mgr_validate_type(type_: ObjType) {
    assert!(type_ < ObjType::OTCount);
    let mgr = OBJ_MGR_STATE.lock().unwrap();
    let alloc_count = mgr.free.get(&type_).map(|v| v.len()).unwrap_or(0)
        + mgr.busy.get(&type_).map(|v| v.len()).unwrap_or(0);

    let mut nb = 0_usize;
    if let Some(busy) = mgr.busy.get(&type_) {
        for obj in busy {
            nb += 1;
            assert!(nb <= alloc_count);
            assert_eq!(obj.type_, type_);
            assert!(obj.ref_count > 0);
        }
    }

    let mut nf = 0_usize;
    if let Some(free) = mgr.free.get(&type_) {
        for obj in free {
            nf += 1;
            assert!(nf <= alloc_count);
            assert_eq!(obj.ref_count, 0);
            assert_eq!(obj.type_, type_);
        }
    }
    assert_eq!(nb + nf, alloc_count);
}

/// Asserts internal consistency across all object types.
#[track_caller]
pub fn obj_mgr_validate() {
    let mgr = OBJ_MGR_STATE.lock().unwrap();
    for (type_, objs) in &mgr.free {
        assert!(*type_ < ObjType::OTCount);
        for obj in objs {
            assert_eq!(obj.type_, *type_);
            assert_eq!(obj.ref_count, 0);
        }
    }
    for (type_, objs) in &mgr.busy {
        assert!(*type_ < ObjType::OTCount);
        for obj in objs {
            assert_eq!(obj.type_, *type_);
            assert!(obj.ref_count > 0);
        }
    }
}

/// Folds all known thread-local managers into the global stats accumulator.
#[track_caller]
pub fn obj_mgr_update_global_stats() {
    obj_mgr_thread_update_global_stats();
}

/// Folds the per-thread pool counts into the global stats accumulator.
#[track_caller]
pub fn obj_mgr_thread_update_global_stats() {
    let mgr = OBJ_MGR_STATE.lock().unwrap();
    let mut stats = OBJ_GLOBAL_STATS.lock().unwrap();
    for type_ in [ObjType::OT_SeqInfo, ObjType::OT_PathInfo] {
        let i = type_ as usize;
        let free_count = mgr.free.get(&type_).map(|v| v.len() as uint).unwrap_or(0);
        let busy_count = mgr.busy.get(&type_).map(|v| v.len() as uint).unwrap_or(0);
        let max_ref_count = mgr
            .busy
            .get(&type_)
            .and_then(|v| v.iter().map(|obj| obj.ref_count).max())
            .unwrap_or(0);
        let mem = (busy_count as usize * std::mem::size_of::<Obj>()) as f32;
        stats.0[i] += free_count;
        stats.1[i] += busy_count;
        stats.2[i] = stats.2[i].max(max_ref_count);
        stats.3[i] += mem;
    }
}

/// Returns a C++-style global stats table accumulated by `UpdateGlobalStats`.
#[track_caller]
pub fn obj_mgr_log_global_stats() -> String {
    let stats = OBJ_GLOBAL_STATS.lock().unwrap();
    let mut out = String::new();
    out.push('\n');
    out.push_str(
        "            Type        Busy        Free         Mem   MaxRefCnt        Gets      Allocs       Frees\n",
    );
    out.push_str(
        "----------------  ----------  ----------  ----------  ----------  ----------  ----------  ----------\n",
    );
    for type_ in [ObjType::OT_SeqInfo, ObjType::OT_PathInfo] {
        let i = type_ as usize;
        out.push_str(&format!(
            "{:>16.16}  {:>10}  {:>10}  {:>10.0}  {:>10}\n",
            obj_type_to_str(type_),
            stats.1[i],
            stats.0[i],
            stats.3[i],
            stats.2[i]
        ));
    }
    out
}
