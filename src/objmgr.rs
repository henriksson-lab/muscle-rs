// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub(crate) static OBJ_MGR_STATE: std::sync::LazyLock<std::sync::Mutex<ObjMgr>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(ObjMgr::default()));

pub(crate) static OBJ_GLOBAL_STATS: std::sync::Mutex<([uint; 2], [uint; 2], [uint; 2], [f32; 2])> =
    std::sync::Mutex::new(([0; 2], [0; 2], [0; 2], [0.0; 2]));

pub(crate) static OBJ_NEXT_ID: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(1);

#[derive(Clone, Debug)]
pub struct CppObj {
    pub type_: ObjType,
    pub ref_count: uint,
    pub fwd: Option<usize>,
    pub bwd: Option<usize>,
    pub mem_bytes: uint,
}

#[derive(Clone, Debug, Default)]
pub struct CppObjMgr {
    free: [Option<usize>; OBJ_TYPE_COUNT],
    busy: [Option<usize>; OBJ_TYPE_COUNT],
    objects: Vec<CppObj>,
    busy_counts: [uint; OBJ_TYPE_COUNT],
    alloc_call_counts: [uint; OBJ_TYPE_COUNT],
    free_call_counts: [uint; OBJ_TYPE_COUNT],
    get_call_counts: [uint; OBJ_TYPE_COUNT],
}

#[derive(Clone, Debug, Default)]
pub struct CppObjMgrRuntime {
    oms: Vec<Option<CppObjMgr>>,
}

const OBJ_TYPE_COUNT: usize = 2;

fn obj_type_index(type_: ObjType) -> usize {
    assert!(type_ < ObjType::OTCount);
    type_ as usize
}

impl CppObjMgrRuntime {
    #[track_caller]
    pub fn new() -> Self {
        Self::default()
    }

    #[track_caller]
    pub fn thread_count(&self) -> uint {
        self.oms.len() as uint
    }

    #[track_caller]
    pub fn manager_exists(&self, thread_index: uint) -> bool {
        self.oms
            .get(thread_index as usize)
            .and_then(|mgr| mgr.as_ref())
            .is_some()
    }

    #[track_caller]
    pub fn manager_mut(&mut self, thread_index: uint) -> Option<&mut CppObjMgr> {
        self.oms
            .get_mut(thread_index as usize)
            .and_then(|mgr| mgr.as_mut())
    }
}

/// C++-literal `ObjMgr::GetObjMgr`: grow the per-thread manager table by 32
/// slots and return the current thread's mutable manager.
#[track_caller]
pub fn obj_mgr_cpp_literal_get_obj_mgr(runtime: &mut CppObjMgrRuntime) -> &mut CppObjMgr {
    let thread_index = get_thread_index() as usize;
    if thread_index >= runtime.oms.len() {
        let new_thread_count = thread_index + 32;
        runtime.oms.resize_with(new_thread_count, || None);
    }
    if runtime.oms[thread_index].is_none() {
        runtime.oms[thread_index] = Some(CppObjMgr::default());
    }
    runtime.oms[thread_index].as_mut().unwrap()
}

/// C++-literal `ObjMgr::StaticGetObj`.
#[track_caller]
pub fn obj_mgr_cpp_literal_static_get_obj(runtime: &mut CppObjMgrRuntime, type_: ObjType) -> usize {
    obj_mgr_cpp_literal_get_obj_mgr(runtime).thread_get_obj(type_)
}

impl CppObjMgr {
    /// C++-literal `ObjMgr::AllocNew`: allocate a concrete object slot with
    /// zero ref-count and no list links.
    #[track_caller]
    pub fn alloc_new(&mut self, type_: ObjType) -> usize {
        let i = obj_type_index(type_);
        self.alloc_call_counts[i] += 1;
        let handle = self.objects.len();
        self.objects.push(CppObj {
            type_,
            ref_count: 0,
            fwd: None,
            bwd: None,
            mem_bytes: 0,
        });
        handle
    }

    /// C++-literal `ObjMgr::ThreadGetObj`: recycle from the free list or
    /// allocate new, then push at the head of the busy list.
    #[track_caller]
    pub fn thread_get_obj(&mut self, type_: ObjType) -> usize {
        let i = obj_type_index(type_);
        self.get_call_counts[i] += 1;
        let new_obj = match self.free[i] {
            None => self.alloc_new(type_),
            Some(handle) => {
                assert_eq!(self.objects[handle].ref_count, 0);
                self.free[i] = self.objects[handle].fwd;
                if let Some(next) = self.free[i] {
                    self.objects[next].bwd = None;
                }
                handle
            }
        };

        if let Some(head) = self.busy[i] {
            assert_eq!(self.objects[head].bwd, None);
            self.objects[head].bwd = Some(new_obj);
        }
        self.objects[new_obj].fwd = self.busy[i];
        self.objects[new_obj].bwd = None;
        self.busy[i] = Some(new_obj);
        self.objects[new_obj].ref_count = 1;
        self.busy_counts[i] += 1;
        new_obj
    }

    /// C++-literal `ObjMgr::FreeObj`, used by the compatibility sidecar tests.
    #[track_caller]
    pub fn free_obj(&mut self, handle: usize) {
        assert_eq!(self.objects[handle].ref_count, 0);
        let type_ = self.objects[handle].type_;
        let i = obj_type_index(type_);
        self.free_call_counts[i] += 1;
        if self.busy[i] == Some(handle) {
            self.busy[i] = self.objects[handle].fwd;
        }
        let prev = self.objects[handle].bwd;
        let next = self.objects[handle].fwd;
        if let Some(prev) = prev {
            self.objects[prev].fwd = next;
        }
        if let Some(next) = next {
            self.objects[next].bwd = prev;
        }
        if let Some(free_head) = self.free[i] {
            assert_eq!(self.objects[free_head].bwd, None);
            self.objects[free_head].bwd = Some(handle);
        }
        self.objects[handle].fwd = self.free[i];
        self.objects[handle].bwd = None;
        self.free[i] = Some(handle);
        self.busy_counts[i] -= 1;
    }

    #[track_caller]
    pub fn object(&self, handle: usize) -> &CppObj {
        &self.objects[handle]
    }

    #[track_caller]
    pub fn object_mut(&mut self, handle: usize) -> &mut CppObj {
        &mut self.objects[handle]
    }

    #[track_caller]
    pub fn free_head(&self, type_: ObjType) -> Option<usize> {
        self.free[obj_type_index(type_)]
    }

    #[track_caller]
    pub fn busy_head(&self, type_: ObjType) -> Option<usize> {
        self.busy[obj_type_index(type_)]
    }

    /// C++-literal `ObjMgr::GetTotalMem`: sum virtual payload bytes for busy
    /// objects only.
    #[track_caller]
    pub fn get_total_mem(&self, type_: ObjType) -> f32 {
        let i = obj_type_index(type_);
        let mut total = 0.0_f32;
        let mut obj = self.busy[i];
        while let Some(handle) = obj {
            total += self.objects[handle].mem_bytes as f32;
            assert_eq!(self.objects[handle].type_, type_);
            obj = self.objects[handle].fwd;
        }
        total
    }

    /// C++ DEBUG-literal `ObjMgr::ValidateType`.
    #[track_caller]
    pub fn validate_type(&self, type_: ObjType) {
        let i = obj_type_index(type_);
        let na = self.alloc_call_counts[i];
        let nf = self.free_call_counts[i];

        let mut nb = 0;
        let mut obj = self.busy[i];
        while let Some(handle) = obj {
            nb += 1;
            assert!(nb <= na);
            assert_eq!(self.objects[handle].type_, type_);
            assert!(self.objects[handle].ref_count > 0);
            if let Some(prev) = self.objects[handle].bwd {
                assert_eq!(self.objects[prev].fwd, Some(handle));
            }
            if let Some(next) = self.objects[handle].fwd {
                assert_eq!(self.objects[next].bwd, Some(handle));
            }
            obj = self.objects[handle].fwd;
        }

        let mut nf_seen = 0;
        let mut obj = self.free[i];
        while let Some(handle) = obj {
            nf_seen += 1;
            assert!(nf_seen <= nf);
            assert_eq!(self.objects[handle].ref_count, 0);
            assert_eq!(self.objects[handle].type_, type_);
            if let Some(prev) = self.objects[handle].bwd {
                assert_eq!(self.objects[prev].fwd, Some(handle));
            }
            if let Some(next) = self.objects[handle].fwd {
                assert_eq!(self.objects[next].bwd, Some(handle));
            }
            obj = self.objects[handle].fwd;
        }
        assert_eq!(nb + nf_seen, na);
        assert_eq!(nb, self.busy_counts[i]);
    }

    /// C++ DEBUG-literal `ObjMgr::Validate`: the translated row preserves the
    /// unconditional fatal diagnostic at the start of the body.
    #[track_caller]
    pub fn validate(&self) {
        die("Validate!");
    }
}

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
    {
        let mut mgr = OBJ_MGR_STATE.lock().unwrap();
        let busy = mgr.busy.entry(obj.type_).or_default();
        let pos = busy
            .iter()
            .position(|x| x.type_ == obj.type_ && x.id == obj.id)
            .unwrap();
        busy[pos].ref_count -= 1;
    }
    obj.ref_count -= 1;
    if obj.ref_count == 0 {
        obj_mgr_free_obj(obj.clone());
    }
}

/// Increments the ref-count of `obj` (caller must already hold a reference).
#[track_caller]
pub fn obj_mgr_up(obj: &mut Obj) {
    assert!(obj.ref_count != 0);
    {
        let mut mgr = OBJ_MGR_STATE.lock().unwrap();
        let busy = mgr.busy.entry(obj.type_).or_default();
        let pos = busy
            .iter()
            .position(|x| x.type_ == obj.type_ && x.id == obj.id)
            .unwrap();
        busy[pos].ref_count += 1;
    }
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
        id: OBJ_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
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
            id: OBJ_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
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
    assert!(obj.type_ < ObjType::OTCount);
    let mut mgr = OBJ_MGR_STATE.lock().unwrap();
    let busy = mgr.busy.entry(obj.type_).or_default();
    if let Some(pos) = busy
        .iter()
        .position(|x| x.type_ == obj.type_ && x.id == obj.id)
    {
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
    obj_mgr_get_total_mem_locked(&mgr, type_)
}

fn obj_mgr_get_total_mem_locked(mgr: &ObjMgr, type_: ObjType) -> f32 {
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
        let mem = obj_mgr_get_total_mem_locked(&mgr, type_);
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
        let busy_count = int_to_str(u64::from(stats.1[i]));
        let free_count = int_to_str(u64::from(stats.0[i]));
        out.push_str(&format!(
            "{:>16.16}  {:>10}  {:>10}  {:>10.10}  {:>10}\n",
            obj_type_to_str(type_),
            busy_count,
            free_count,
            mem_bytes_to_str(f64::from(stats.3[i])),
            stats.2[i]
        ));
    }
    log(&out);
    out
}
