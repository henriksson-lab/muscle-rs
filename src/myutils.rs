// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const LOG_ZERO: f32 = -2e20_f32;

pub const LOG_ONE: f32 = 0.0_f32;

pub const INVALID_LOG: f32 = f32::MAX;

pub const UNINIT_LOG: f32 = 9e9_f32;

pub const OUT_OF_BAND_LOG: f32 = 8e8_f32;

pub const EXP_UNDERFLOW_THRESHOLD: f32 = -4.6_f32;

pub const LOG_UNDERFLOW_THRESHOLD: f32 = 7.5_f32;

pub const MINUS_INFINITY: f32 = -9e9_f32;

pub const UNINIT: f32 = -8e8_f32;

#[derive(Clone, Debug)]
pub(crate) struct RandState {
    pub(crate) slcg_state: uint32,
    pub(crate) init_done: bool,
    pub(crate) x: [uint32; 5],
}

pub(crate) static RAND_STATE: std::sync::Mutex<RandState> = std::sync::Mutex::new(RandState {
    slcg_state: 1,
    init_done: false,
    x: [0; 5],
});
pub(crate) static PENDING_RAND_SEED: std::sync::Mutex<Option<uint>> = std::sync::Mutex::new(None);

pub(crate) static START_TIME: std::sync::OnceLock<std::time::SystemTime> =
    std::sync::OnceLock::new();

pub static G_ARGV: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
pub static G_ARG1: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

#[derive(Clone, Debug)]
enum CmdOptValue {
    Flag(bool),
    Uns(uint),
    Float(f64),
    Str(String),
}

#[derive(Clone, Debug, Default)]
struct CmdOptEntry {
    value: Option<CmdOptValue>,
    used: bool,
}

static CMD_OPT_STATE: std::sync::LazyLock<
    std::sync::Mutex<std::collections::BTreeMap<String, CmdOptEntry>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));

pub(crate) static G_LOG_FILE: std::sync::Mutex<Option<std::fs::File>> = std::sync::Mutex::new(None);

pub(crate) static QUIET_STATE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static DIE_PROCESS_EXIT_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static IN_DIE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PROGRESS_STEP_RANGE_WARNING_DONE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub(crate) static FILE_TO_FILE_NAME: std::sync::LazyLock<
    std::sync::Mutex<std::collections::BTreeMap<usize, String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));

pub(crate) static SUM_ALLOC: std::sync::Mutex<f64> = std::sync::Mutex::new(0.0);

pub(crate) static PEAK_MEM_USE_BYTES: std::sync::Mutex<f64> = std::sync::Mutex::new(0.0);

pub(crate) static TRACKED_ALLOCS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::BTreeMap<String, f64>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));

#[derive(Clone, Debug)]
pub(crate) struct ProgressState {
    pub(crate) desc: String,
    pub(crate) index: uint,
    pub(crate) count: uint,
    pub(crate) counts_interval: uint,
    pub(crate) step_calls: uint,
    pub(crate) time_last_output_step: u64,
    pub(crate) prefix_on: bool,
    pub(crate) curr_line_length: uint,
    pub(crate) last_line_length: uint,
    pub(crate) prog_file_size: f64,
    pub(crate) prog_file_tick: uint,
    pub(crate) prog_file_msg: String,
}

pub(crate) static PROGRESS_STATE: std::sync::Mutex<ProgressState> =
    std::sync::Mutex::new(ProgressState {
        desc: String::new(),
        index: 0,
        count: 0,
        counts_interval: 0,
        step_calls: 0,
        time_last_output_step: 0,
        prefix_on: true,
        curr_line_length: 0,
        last_line_length: 0,
        prog_file_size: 0.0,
        prog_file_tick: 0,
        prog_file_msg: String::new(),
    });
pub(crate) static PROGRESS_CALLBACK_STATE: std::sync::Mutex<fn() -> &'static str> =
    std::sync::Mutex::new(default_pcb);

#[derive(Clone, Debug, Default)]
pub struct Stat; // original: stat (muscle/src/myutils.cpp)

#[derive(Clone, Debug, Default)]
pub struct FinddataT; // original: _finddata_t (muscle/src/myutils.cpp)

#[derive(Clone, Debug, Default)]
pub struct Dirent; // original: dirent (muscle/src/myutils.cpp)

#[derive(Clone, Debug, Default)]
pub struct TaskBasicInfo; // original: task_basic_info (muscle/src/myutils.cpp)

#[derive(Clone, Debug, Default)]
pub struct Tm; // original: tm (muscle/src/myutils.cpp)

/// Returns the base-2 logarithm of `x`.
pub fn mylog2(x: f64) -> f64 {
    x.ln() / 2.0_f64.ln()
}

/// Returns the base-10 logarithm of `x`.
pub fn mylog10(x: f64) -> f64 {
    x.ln() / 10.0_f64.ln()
}

/// Returns the number of threads requested via `-threads`, clamped to the CPU core count.
#[track_caller]
pub fn get_requested_thread_count() -> uint {
    let max_n = get_cpu_core_count().max(1);
    let core_count = get_cpu_core_count();
    let mut n = core_count;
    {
        let argv = G_ARGV.lock().unwrap();
        for i in 0..argv.len() {
            if argv[i] == "-threads" && i + 1 < argv.len() {
                n = str_to_uint_l1278(&argv[i + 1], false);
                break;
            }
        }
    }
    if n > max_n {
        let _ = warning(&format!("Max OMP threads {max_n}"));
        n = max_n;
    }
    if n == 0 {
        n = 1;
    }
    let _ = progress(&format!(
        "CPU has {core_count} cores, running {n} threads\n"
    ));
    n
}

/// Returns a short string identifying the build platform (e.g. `linux64`, `osxarm64`).
#[track_caller]
pub fn get_platform() -> &'static str {
    #[cfg(all(target_pointer_width = "32", target_os = "windows"))]
    {
        "win32"
    }
    #[cfg(all(target_pointer_width = "32", target_os = "macos"))]
    {
        "osx32"
    }
    #[cfg(all(
        target_pointer_width = "32",
        not(target_os = "windows"),
        not(target_os = "macos")
    ))]
    {
        "linux32"
    }
    #[cfg(all(target_pointer_width = "64", target_os = "windows"))]
    {
        "win64"
    }
    #[cfg(all(
        target_pointer_width = "64",
        target_os = "macos",
        target_arch = "aarch64"
    ))]
    {
        "osxarm64"
    }
    #[cfg(all(
        target_pointer_width = "64",
        target_os = "macos",
        not(target_arch = "aarch64")
    ))]
    {
        "osx64"
    }
    #[cfg(all(
        target_pointer_width = "64",
        not(target_os = "windows"),
        not(target_os = "macos"),
    ))]
    {
        "linux64"
    }
}

/// Returns the final path component of `path_name` (after the last `/` or `\\`).
#[track_caller]
pub fn base_name(path_name: &str) -> &str {
    path_name
        .rfind(['/', '\\'])
        .map(|ix| &path_name[ix + 1..])
        .unwrap_or(path_name)
}

/// Allocates a 32 KB I/O scratch buffer.
#[track_caller]
pub fn alloc_buffer() -> Vec<u8> {
    vec![0; MY_IO_BUFSIZ]
}

const MY_IO_BUFSIZ: usize = 32_000;

#[cfg(unix)]
#[repr(C)]
pub struct CppFile {
    _private: [u8; 0],
}

#[cfg(unix)]
unsafe extern "C" {
    fn fileno(stream: *mut CppFile) -> i32;
    fn setvbuf(stream: *mut CppFile, buffer: *mut i8, mode: i32, size: usize) -> i32;
}

#[cfg(unix)]
static CPP_IO_BUFFERS: std::sync::OnceLock<std::sync::Mutex<Vec<Option<Vec<u8>>>>> =
    std::sync::OnceLock::new();

/// Mirrors C++ `AllocBuffer(FILE*)` for callers that own a C stdio stream.
///
/// # Safety
///
/// `file` must be either null or a valid live C `FILE *`. The installed
/// descriptor buffer is retained for process lifetime, matching the C++ static
/// `g_IOBuffers` table.
#[cfg(unix)]
#[track_caller]
pub unsafe fn alloc_buffer_cpp_literal(file: *mut CppFile) -> i32 {
    if file.is_null() {
        return 0;
    }
    let fd = unsafe { fileno(file) };
    if !(0..256).contains(&fd) {
        return 0;
    }
    let buffers = CPP_IO_BUFFERS.get_or_init(|| {
        let mut buffers = Vec::with_capacity(256);
        buffers.resize_with(256, || None);
        std::sync::Mutex::new(buffers)
    });
    let mut buffers = buffers.lock().unwrap();
    if buffers[fd as usize].is_none() {
        buffers[fd as usize] = Some(vec![0; MY_IO_BUFSIZ]);
    }
    let buffer = buffers[fd as usize].as_mut().unwrap();
    unsafe { setvbuf(file, buffer.as_mut_ptr().cast::<i8>(), 0, MY_IO_BUFSIZ) }
}

#[cfg(not(unix))]
#[repr(C)]
pub struct CppFile {
    _private: [u8; 0],
}

#[cfg(not(unix))]
#[track_caller]
pub unsafe fn alloc_buffer_cpp_literal(_file: *mut CppFile) -> i32 {
    0
}

/// Mirrors the compiled C++ `FreeBuffer(FILE*)`, whose body is disabled.
#[track_caller]
pub fn free_buffer(_buffer: &mut Vec<u8>) {}

/// Returns whole seconds elapsed since the first call (initializes the global start time on first use).
pub fn get_elapsed_secs() -> uint {
    let start = START_TIME.get_or_init(std::time::SystemTime::now);
    start.elapsed().map(|d| d.as_secs() as uint).unwrap_or(0)
}

/// Returns true if `file_name` refers to an existing filesystem entry.
#[track_caller]
pub fn stdio_file_exists(file_name: &str) -> bool {
    std::fs::metadata(file_name).is_ok()
}

/// Aborts with a formatted assertion-failure message.
#[track_caller]
pub fn myassertfail(exp: &str, file: &str, line: uint) -> ! {
    die(&format!("{file}({line}) assert failed: {exp}"))
}

/// Returns true if the given (0/1/2) standard file descriptor is attached to a TTY.
#[track_caller]
pub fn myisatty(fd: i32) -> bool {
    match fd {
        0 => std::io::IsTerminal::is_terminal(&std::io::stdin()),
        1 => std::io::IsTerminal::is_terminal(&std::io::stdout()),
        2 => std::io::IsTerminal::is_terminal(&std::io::stderr()),
        _ => false,
    }
}

/// Cross-platform `fseeko` wrapper using `SEEK_SET`/`SEEK_CUR`/`SEEK_END` semantics.
#[track_caller]
pub fn fseeko(file: &mut std::fs::File, offset: i64, whence: i32) -> i32 {
    use std::io::{Seek, SeekFrom};
    let seek_from = match whence {
        0 => SeekFrom::Start(offset as u64),
        1 => SeekFrom::Current(offset),
        2 => SeekFrom::End(offset),
        _ => return -1,
    };
    match file.seek(seek_from) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[cfg(unix)]
fn stdio_file_key(f: &std::fs::File) -> usize {
    use std::os::fd::AsRawFd;
    f.as_raw_fd() as usize
}

#[cfg(windows)]
fn stdio_file_key(f: &std::fs::File) -> usize {
    use std::os::windows::io::AsRawHandle;
    f.as_raw_handle() as usize
}

#[cfg(not(any(unix, windows)))]
fn stdio_file_key(f: &std::fs::File) -> usize {
    f as *const std::fs::File as usize
}

/// Returns a multi-line dump of position/error state for `f` (diagnostic).
#[track_caller]
pub fn log_stdio_file_state(f: &mut std::fs::File) -> String {
    let tell_pos = get_stdio_file_pos64(f);
    let fseek_pos = std::io::Seek::seek(f, std::io::SeekFrom::Current(0)).unwrap_or(tell_pos);
    let mut out = String::new();
    out.push_str(&format!("FILE *     {:p}\n", f));
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        out.push_str(&format!("fileno     {}\n", f.as_raw_fd()));
    }
    #[cfg(not(unix))]
    {
        out.push_str("fileno     0\n");
    }
    out.push_str("feof       0\n");
    out.push_str("ferror     0\n");
    out.push_str(&format!("ftell      {tell_pos}\n"));
    out.push_str(&format!("fseek      {fseek_pos}\n"));
    let file_to_file_name = FILE_TO_FILE_NAME.lock().unwrap();
    if let Some(name) = file_to_file_name.get(&stdio_file_key(f)) {
        out.push_str(&format!("Name       {name}\n"));
    } else {
        out.push_str("Not found in FileToFileName\n");
    }
    log(&out);
    out
}

/// Splits `file_name` into `(directory, base)` at the last `/`; directory defaults to `.`.
pub fn parse_file_name(file_name: &str) -> (String, String) {
    if let Some(n) = file_name.rfind('/') {
        (file_name[..n].to_string(), file_name[n + 1..].to_string())
    } else {
        (".".to_string(), file_name.to_string())
    }
}

/// Lists `dir_name` entries like the Windows `_findfirst` body.
#[track_caller]
pub fn read_dir_l261(dir_name: &str) -> Vec<String> {
    if dir_name.contains('?') || dir_name.contains('*') {
        die(&format!("Invalid directory name '{dir_name}'"));
    }

    let mut names = vec![".".to_string(), "..".to_string()];
    let entries = std::fs::read_dir(dir_name)
        .unwrap_or_else(|_| panic!("Directory not found '{}'", dir_name));
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        names.push(entry.file_name().to_string_lossy().to_string());
    }

    let mut file_names = Vec::new();
    for (ix, file_name) in names.into_iter().enumerate() {
        file_names.push(file_name.clone());
        if ix > 0 {
            file_names.push(file_name);
        }
    }
    file_names.sort();
    file_names
}

/// Variant of `read_dir_l261`; lists `dir_name` entries sorted with `.` and `..`.
#[track_caller]
pub fn read_dir_l291(dir_name: &str) -> Vec<String> {
    let mut file_names = Vec::new();
    file_names.push(".".to_string());
    file_names.push("..".to_string());
    let entries = std::fs::read_dir(dir_name)
        .unwrap_or_else(|_| panic!("Directory not found '{}'", dir_name));
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        file_names.push(entry.file_name().to_string_lossy().to_string());
    }
    file_names.sort();
    file_names
}

/// Opens `file_name` for reading or panics if it cannot be opened.
#[track_caller]
pub fn open_stdio_file(file_name: &str) -> std::fs::File {
    if file_name.is_empty() {
        panic!("Missing input file name");
    }
    let f = std::fs::File::open(file_name)
        .unwrap_or_else(|e| panic!("Cannot open {file_name}, errno={:?} {e}", e.raw_os_error()));
    FILE_TO_FILE_NAME
        .lock()
        .unwrap()
        .insert(stdio_file_key(&f), file_name.to_string());
    f
}

/// Creates/truncates `file_name` for read-write, or returns `None` for an empty name.
#[track_caller]
pub fn create_stdio_file(file_name: &str) -> Option<std::fs::File> {
    if file_name.is_empty() {
        return None;
    }
    let f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)
        .unwrap_or_else(|e| {
            panic!(
                "Cannot create {file_name}, errno={:?} {e}",
                e.raw_os_error()
            )
        });
    FILE_TO_FILE_NAME
        .lock()
        .unwrap()
        .insert(stdio_file_key(&f), file_name.to_string());
    Some(f)
}

/// Seeks `f` to absolute 32-bit offset `pos`, panicking on failure.
#[track_caller]
pub fn set_stdio_file_pos(f: &mut std::fs::File, pos: uint32) {
    let new_pos = std::io::Seek::seek(f, std::io::SeekFrom::Start(pos as u64))
        .expect("SetStdioFilePos failed");
    if new_pos != pos as u64 {
        panic!("SetStdioFilePos({pos}) failed, NewPos={new_pos}");
    }
}

/// Seeks `f` to absolute 64-bit offset `pos`, panicking on failure.
#[track_caller]
pub fn set_stdio_file_pos64(f: &mut std::fs::File, pos: uint64) {
    let new_pos =
        std::io::Seek::seek(f, std::io::SeekFrom::Start(pos)).expect("SetStdioFilePos64 failed");
    if new_pos != pos {
        panic!("SetStdioFilePos64({pos}) failed, NewPos={new_pos}");
    }
}

/// Reads up to `bytes` from `f`, returning whatever was read (may be short).
#[track_caller]
pub fn read_stdio_file_no_fail(f: &mut std::fs::File, bytes: uint32) -> Vec<byte> {
    let mut buffer = vec![0; bytes as usize];
    let bytes_read = std::io::Read::read(f, &mut buffer).expect("ReadStdioFile_NoFail failed");
    buffer.truncate(bytes_read);
    buffer
}

/// Seeks to `pos` and reads exactly `bytes` from `f`.
#[track_caller]
pub fn read_stdio_file_l395(f: &mut std::fs::File, pos: uint32, bytes: uint32) -> Vec<byte> {
    set_stdio_file_pos(f, pos);
    let mut buffer = vec![0; bytes as usize];
    std::io::Read::read_exact(f, &mut buffer)
        .unwrap_or_else(|e| panic!("ReadStdioFile failed, attempted {bytes} bytes, errno={e}"));
    buffer
}

/// 64-bit variant with the original C++ element-count comparison bug.
#[track_caller]
pub fn read_stdio_file64_l408(f: &mut std::fs::File, pos: uint64, bytes: uint64) -> Vec<byte> {
    let bytes32 = bytes as uint32;
    assert_eq!(bytes32 as uint64, bytes);
    set_stdio_file_pos64(f, pos);
    let mut buffer = vec![0; bytes as usize];
    std::io::Read::read_exact(f, &mut buffer)
        .unwrap_or_else(|e| panic!("ReadStdioFile64 failed, attempted {bytes} bytes, errno={e}"));
    if bytes > 1 {
        log_stdio_file_state(f);
        panic!("ReadStdioFile64 failed, attempted {bytes} bytes, errno=0");
    }
    buffer
}

/// Reads exactly `bytes` from `f` at the current position.
#[track_caller]
pub fn read_stdio_file_l423(f: &mut std::fs::File, bytes: uint32) -> Vec<byte> {
    let mut buffer = vec![0; bytes as usize];
    std::io::Read::read_exact(f, &mut buffer)
        .unwrap_or_else(|e| panic!("ReadStdioFile64 failed, attempted {bytes} bytes, errno={e}"));
    buffer
}

/// 64-bit variant: reads exactly `bytes` from `f` at the current position.
#[track_caller]
pub fn read_stdio_file64_l435(f: &mut std::fs::File, bytes: uint64) -> Vec<byte> {
    let mut buffer = vec![0; bytes as usize];
    std::io::Read::read_exact(f, &mut buffer)
        .unwrap_or_else(|e| panic!("ReadStdioFile64 failed, attempted {bytes} bytes, errno={e}"));
    buffer
}

/// Reads the entire file (32-bit size) into a new buffer.
#[track_caller]
pub fn read_all_stdio_file(f: &mut std::fs::File) -> Vec<byte> {
    let pos = get_stdio_file_pos64(f);
    let file_size64 = get_stdio_file_size64(f);
    let file_size = file_size64 as uint32;
    #[cfg(target_pointer_width = "32")]
    if file_size as uint64 != file_size64 {
        panic!("ReadAllStdioFile (32-bit): file too big");
    }
    set_stdio_file_pos(f, 0);
    let mut buffer = vec![0; file_size as usize];
    std::io::Read::read_exact(f, &mut buffer).expect("ReadAllStdioFile failed");
    set_stdio_file_pos64(f, pos);
    buffer
}

/// Opens `file_name` and reads its full contents (64-bit size).
#[track_caller]
pub fn read_all_stdio_file64_l463(file_name: &str) -> Vec<byte> {
    let mut f = open_stdio_file(file_name);
    read_all_stdio_file64_l476(&mut f)
}

/// Reads the entire (open) file, preserving the original C++ chunk-loop bugs.
#[track_caller]
pub fn read_all_stdio_file64_l476(f: &mut std::fs::File) -> Vec<byte> {
    let saved_pos = get_stdio_file_pos64(f);
    let file_size = get_stdio_file_size64(f);
    if file_size > uint::MAX as uint64 {
        panic!(
            "ReadAllStdioFile64, file too big {}",
            mem_bytes_to_str(file_size as f64)
        );
    }
    let mut buffer = vec![0; file_size as usize];
    let pos = 0_u64;
    let mut bytes_left = file_size;
    let chunk_size = 0x40000000_u64;
    while bytes_left != 0 {
        let bytes_to_read = bytes_left.min(chunk_size);
        let chunk = read_stdio_file64_l408(f, pos, bytes_to_read);
        buffer[pos as usize..(pos + bytes_to_read) as usize].copy_from_slice(&chunk);
        bytes_left -= bytes_to_read;
    }
    set_stdio_file_pos64(f, saved_pos);
    buffer
}

/// Opens `file_name`, truncates the probed size to 32 bits, and reads that many bytes.
#[track_caller]
pub fn read_all_stdio_file32(file_name: &str) -> Vec<byte> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::raw::{c_char, c_int, c_void};

        unsafe extern "C" {
            fn open(pathname: *const c_char, flags: c_int) -> c_int;
            fn lseek(fd: c_int, offset: i64, whence: c_int) -> i64;
            fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
            fn close(fd: c_int) -> c_int;
        }

        const O_RDONLY: c_int = 0;
        const SEEK_SET: c_int = 0;
        const SEEK_END: c_int = 2;

        let c_file_name = CString::new(file_name)
            .unwrap_or_else(|_| panic!("ReadAllStdioFile:Cannot open {file_name}"));
        let h = unsafe { open(c_file_name.as_ptr(), O_RDONLY) };
        if h < 0 {
            panic!("ReadAllStdioFile:Cannot open {file_name}");
        }
        let file_size64 = unsafe { lseek(h, 0, SEEK_END) };
        #[cfg(not(target_os = "macos"))]
        if file_size64 == -1 {
            panic!("ReadAllStdioFile:Error seeking {file_name}");
        }
        let file_size = file_size64 as uint32;
        let st_bytes = file_size as usize;
        if st_bytes as i64 != i64::from(file_size) {
            panic!("ReadAllStdioFile: off_t overflow");
        }
        let mut buffer = vec![0; st_bytes];
        unsafe {
            lseek(h, 0, SEEK_SET);
        }
        let n = unsafe { read(h, buffer.as_mut_ptr().cast::<c_void>(), st_bytes) };
        if n != isize::try_from(file_size).unwrap_or(isize::MAX) {
            panic!(
                "ReadAllStdioFile, Error reading {file_name}, attempted {} got {}",
                file_size, n
            );
        }
        unsafe {
            close(h);
        }
        return buffer;
    }

    #[cfg(not(unix))]
    {
        let mut f = std::fs::File::open(file_name)
            .unwrap_or_else(|_| panic!("ReadAllStdioFile:Cannot open {file_name}"));
        let file_size = std::io::Seek::seek(&mut f, std::io::SeekFrom::End(0))
            .unwrap_or_else(|_| panic!("ReadAllStdioFile:Error seeking {file_name}"))
            as uint32;
        set_stdio_file_pos(&mut f, 0);
        let mut buffer = vec![0; file_size as usize];
        std::io::Read::read_exact(&mut f, &mut buffer).unwrap_or_else(|_| {
            panic!(
                "ReadAllStdioFile, Error reading {file_name}, attempted {} got short read",
                file_size
            )
        });
        buffer
    }
}

/// Seeks to `pos` and writes `buffer` to `f`.
#[track_caller]
pub fn write_stdio_file_l558(f: &mut std::fs::File, pos: uint32, buffer: &[byte]) {
    set_stdio_file_pos(f, pos);
    std::io::Write::write_all(f, buffer).unwrap_or_else(|e| {
        panic!(
            "WriteStdioFile failed, attempted {} bytes, errno={e}",
            buffer.len()
        )
    });
}

/// Writes the bytes of `s` to `f`.
#[track_caller]
pub fn write_stdio_file_str(f: &mut std::fs::File, s: &str) {
    write_stdio_file_l578(f, s.as_bytes());
}

/// Writes `buffer` to `f`, panicking on partial writes.
#[track_caller]
pub fn write_stdio_file_l578(f: &mut std::fs::File, buffer: &[byte]) {
    std::io::Write::write_all(f, buffer).unwrap_or_else(|e| {
        panic!(
            "WriteStdioFile failed, attempted {} bytes, errno={e}",
            buffer.len()
        )
    });
}

/// 64-bit variant: writes `buffer` to `f`, panicking on partial writes.
#[track_caller]
pub fn write_stdio_file64(f: &mut std::fs::File, buffer: &[byte]) {
    std::io::Write::write_all(f, buffer).unwrap_or_else(|e| {
        panic!(
            "WriteStdioFile failed, attempted {} bytes, errno={e}",
            buffer.len()
        )
    });
}

/// Return false on EOF, true if line successfully read.
#[track_caller]
pub fn read_line_stdio_file_l605(f: &mut std::fs::File, bytes: uint32) -> Option<String> {
    if (bytes as i32) < 0 {
        panic!("ReadLineStdioFile: Bytes < 0");
    }
    let mut line = Vec::new();
    let mut buf = [0_u8; 1];
    loop {
        match std::io::Read::read(f, &mut buf) {
            Ok(0) => {
                if line.is_empty() {
                    return None;
                }
                panic!("ReadLineStdioFile: line too long or missing end-of-line");
            }
            Ok(_) => {
                line.push(buf[0]);
                if line.len() >= bytes as usize {
                    panic!("ReadLineStdioFile: line too long or missing end-of-line");
                }
                if buf[0] == b'\n' {
                    break;
                }
            }
            Err(e) => panic!("ReadLineStdioFile: errno={e}"),
        }
    }
    if line.is_empty() || *line.last().unwrap() != b'\n' {
        panic!("ReadLineStdioFile: line too long or missing end-of-line");
    }
    if line.last() == Some(&b'\n') || line.last() == Some(&b'\r') {
        line.pop();
    }
    if line.last() == Some(&b'\n') || line.last() == Some(&b'\r') {
        line.pop();
    }
    Some(String::from_utf8(line).expect("line is not UTF-8"))
}

/// Reads one tab-delimited line of `field_count` fields, or an empty vector on EOF.
#[track_caller]
pub fn read_tabbed_line(f: &mut std::fs::File, field_count: uint) -> Vec<String> {
    let line = read_line_stdio_file_l650(f).expect("Unxpected end-of-file in tabbed text");
    let fields = split(&line, '\t');
    let n = fields.len() as uint;
    if field_count != uint::MAX && n != field_count {
        panic!("Expected {field_count} tabbed fields, got {n}");
    }
    fields
}

/// Return false on EOF, true if line successfully read.
#[track_caller]
pub fn read_line_stdio_file_l650(f: &mut std::fs::File) -> Option<String> {
    let mut line = String::new();
    let mut buf = [0_u8; 1];
    loop {
        match std::io::Read::read(f, &mut buf) {
            Ok(0) => {
                if !line.is_empty() {
                    return Some(line);
                }
                return None;
            }
            Ok(_) => {
                let c = buf[0];
                if c == b'\r' {
                    continue;
                }
                if c == b'\n' {
                    return Some(line);
                }
                line.push(c as char);
            }
            Err(e) => panic!("ReadLineStdioFile, errno={e}"),
        }
    }
}

/// Renames `file_name_from` to `file_name_to`, panicking on failure.
#[track_caller]
pub fn rename_stdio_file(file_name_from: &str, file_name_to: &str) {
    std::fs::rename(file_name_from, file_name_to).unwrap_or_else(|e| {
        panic!("RenameStdioFile({file_name_from},{file_name_to}) failed, errno={e}")
    });
}

/// Flushes pending writes on `f`.
#[track_caller]
pub fn flush_stdio_file(f: &mut std::fs::File) {
    std::io::Write::flush(f).unwrap_or_else(|e| panic!("fflush failed: {e}"));
}

/// Drops `f` (if `Some`), closing the underlying file.
#[track_caller]
pub fn close_stdio_file(f: Option<std::fs::File>) {
    if let Some(file) = f.as_ref() {
        FILE_TO_FILE_NAME
            .lock()
            .unwrap()
            .remove(&stdio_file_key(file));
    }
    drop(f);
}

/// Returns the current 32-bit position of `f`.
#[track_caller]
pub fn get_stdio_file_pos32(f: &mut std::fs::File) -> uint32 {
    let file_pos = std::io::Seek::stream_position(f).unwrap_or_else(|e| panic!("ftello={e}"));
    if file_pos > uint32::MAX as u64 {
        panic!("File offset too big for 32-bit version");
    }
    file_pos as uint32
}

/// Returns the current 64-bit position of `f`.
#[track_caller]
pub fn get_stdio_file_pos64(f: &mut std::fs::File) -> uint64 {
    std::io::Seek::stream_position(f).unwrap_or_else(|e| panic!("ftello={e}"))
}

/// Returns the total size of `f` as a 32-bit value, matching C++ wrap behavior on 64-bit builds.
#[track_caller]
pub fn get_stdio_file_size32(f: &mut std::fs::File) -> uint32 {
    let current_pos = get_stdio_file_pos32(f);
    let length = std::io::Seek::seek(f, std::io::SeekFrom::End(0)).expect("fseek in GetFileSize");
    set_stdio_file_pos(f, current_pos);
    #[cfg(target_pointer_width = "32")]
    if length > uint32::MAX as u64 {
        panic!("File size too big for 32-bit version");
    }
    length as uint32
}

/// Returns the total size of `f` as a 64-bit value.
#[track_caller]
pub fn get_stdio_file_size64(f: &mut std::fs::File) -> uint64 {
    let current_pos = get_stdio_file_pos64(f);
    let length = std::io::Seek::seek(f, std::io::SeekFrom::End(0)).expect("fseek in GetFileSize64");
    set_stdio_file_pos64(f, current_pos);
    length
}

/// Renames `file_name1` to `file_name2` (move).
#[track_caller]
pub fn move_stdio_file(file_name1: &str, file_name2: &str) {
    if stdio_file_exists(file_name2) {
        delete_stdio_file(file_name2);
    }
    rename_stdio_file(file_name1, file_name2);
}

/// Deletes `file_name`.
#[track_caller]
pub fn delete_stdio_file(file_name: &str) {
    std::fs::remove_file(file_name)
        .unwrap_or_else(|e| panic!("remove({file_name}) failed, errno={e}"));
}

/// Returns the OS-reported usable physical-memory size in bytes.
#[track_caller]
pub fn get_usable_mem_bytes() -> f64 {
    let ram = get_phys_mem_bytes_l990();
    #[cfg(all(target_pointer_width = "32", target_os = "windows"))]
    {
        if ram > 2e9 {
            return 2e9;
        }
    }
    #[cfg(all(target_pointer_width = "32", not(target_os = "windows")))]
    {
        if ram > 4e9 {
            return 4e9;
        }
    }
    ram
}

/// Returns the current thread identity as a short string (used in log prefixes).
#[track_caller]
pub fn get_thread_str() -> String {
    let thread_index = get_thread_index();
    let mut s = String::with_capacity(64000 + 1);
    if thread_index > 0 {
        s.reserve(0);
    }
    s
}

/// Thin wrapper that returns its argument as an owned String (replaces C++ vsnprintf).
#[track_caller]
pub fn myvstrprintf(s: &str) -> String {
    let mut out = get_thread_str();
    out.push_str(s);
    if out.len() > 64000 - 1 {
        out.truncate(64000 - 1);
    }
    out
}

/// Prints `s` to stdout and also returns it.
#[track_caller]
pub fn pf(s: &str) -> String {
    s.to_string()
}

/// Writes `s` to caller-owned file sink, matching C++ `Pf(FILE*)` null handling.
#[track_caller]
pub fn pf_file(f: Option<&mut std::fs::File>, s: &str) {
    if let Some(f) = f {
        std::io::Write::write_all(f, s.as_bytes()).expect("Pf write failed");
    }
}

/// Appends `s` to `str_`.
#[track_caller]
pub fn ps(str_: &mut String, s: &str) {
    *str_ = myvstrprintf(s);
}

/// Appends `s` plus a space to `str_`.
#[track_caller]
pub fn psa(str_: &mut String, s: &str) {
    let tmp = myvstrprintf(s);
    str_.push_str(&tmp);
}

/// Appends `s` plus a comma-space to `str_` (handles leading comma).
#[track_caller]
pub fn psasc(str_: &mut String, s: &str) {
    let n = str_.len();
    if n > 0 && !str_.ends_with(';') {
        str_.push(';');
    }
    let tmp = myvstrprintf(s);
    str_.push_str(&tmp);
    let n = str_.len();
    if n > 0 && !str_.ends_with(';') {
        str_.push(';');
    }
}

/// Opens the given path as the global log file, creating/truncating it.
#[track_caller]
pub fn set_log_file_name(file_name: &str) {
    let mut log_file = G_LOG_FILE.lock().unwrap();
    *log_file = None;
    if file_name.is_empty() {
        return;
    }
    let file = std::fs::File::create(file_name)
        .unwrap_or_else(|err| panic!("CreateStdioFile({file_name}): {err}"));
    *log_file = Some(file);
}

/// Writes `s` to the global log file and to stderr.
#[track_caller]
pub fn log(s: &str) {
    use std::io::Write;
    let mut log_file = G_LOG_FILE.lock().unwrap();
    if let Some(file) = log_file.as_mut() {
        file.write_all(s.as_bytes()).unwrap();
        file.flush().unwrap();
    }
}

fn log_fatal_context(msg: &str) -> String {
    if IN_DIE.swap(true, std::sync::atomic::Ordering::SeqCst) {
        std::process::exit(1);
    }
    log("\n");
    log(&format!("{:?}\n", std::time::SystemTime::now()));
    let cmd_line = get_cmd_line();
    if !cmd_line.is_empty() {
        log(&cmd_line);
    }
    log("\n");
    log(&format!(
        "Elapsed time: {}\n",
        secs_to_hhmmss(get_elapsed_secs())
    ));
    let text = format!("\n---Fatal error---\n{msg}");
    log(&format!("{text}\n"));
    text
}

/// Selects whether translated `die()` exits like C++ or panics for reusable tests.
#[track_caller]
pub fn set_die_process_exit_enabled(enabled: bool) {
    DIE_PROCESS_EXIT_ENABLED.store(enabled, std::sync::atomic::Ordering::SeqCst);
}

/// Logs `msg` and aborts the process or panics, depending on entrypoint mode.
#[track_caller]
pub fn die(msg: &str) -> ! {
    let text = log_fatal_context(msg);
    if DIE_PROCESS_EXIT_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!("{text}");
        std::process::exit(1);
    }
    IN_DIE.store(false, std::sync::atomic::Ordering::SeqCst);
    panic!("{text}");
}

/// C++-literal `Die_` process helper for entrypoint/subprocess parity.
#[track_caller]
pub fn die_cpp_literal(msg: &str) -> ! {
    let text = log_fatal_context(msg);
    eprintln!("{text}");
    std::process::exit(1);
}

/// Logs `msg` as a warning and returns the formatted string.
#[track_caller]
pub fn warning(msg: &str) -> String {
    let mut text = String::new();
    text.push('\n');
    text.push_str("WARNING: ");
    text.push_str(msg);
    text.push_str("\n\n");
    eprint!("{text}");
    log(&format!("\nWARNING: {msg}\n"));
    text
}

/// Sleeps for `ms` milliseconds (variant 1).
#[track_caller]
pub fn mysleep_l952(ms: uint) {
    std::thread::sleep(std::time::Duration::from_millis(u64::from(ms)));
}

/// Sleeps for `ms` milliseconds (variant 2).
#[track_caller]
pub fn mysleep_l957(ms: uint) {
    std::thread::sleep(std::time::Duration::from_micros(u64::from(ms)));
}

/// Returns current process memory bytes on Windows via `GetProcessMemoryInfo`.
#[track_caller]
pub fn get_mem_use_bytes_l964() -> f64 {
    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        struct ProcessMemoryCountersEx {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
            private_usage: usize,
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
        }
        #[link(name = "psapi")]
        unsafe extern "system" {
            fn GetProcessMemoryInfo(
                process: *mut std::ffi::c_void,
                counters: *mut ProcessMemoryCountersEx,
                size: u32,
            ) -> i32;
        }

        let mut pmc = ProcessMemoryCountersEx {
            cb: std::mem::size_of::<ProcessMemoryCountersEx>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
            private_usage: 0,
        };
        let ok = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut pmc,
                std::mem::size_of::<ProcessMemoryCountersEx>() as u32,
            )
        };
        if ok == 0 {
            return 1_000_000.0;
        }
        update_peak_mem_use_bytes(pmc.private_usage as f64)
    }
    #[cfg(not(target_os = "windows"))]
    {
        get_mem_use_bytes_l1008()
    }
}

fn update_peak_mem_use_bytes(bytes: f64) -> f64 {
    let mut peak = PEAK_MEM_USE_BYTES.lock().unwrap();
    if bytes > *peak {
        *peak = bytes;
    }
    bytes
}

/// Returns physical RAM size in bytes (variant 1).
#[track_caller]
pub fn get_phys_mem_bytes_l979() -> f64 {
    get_phys_mem_bytes_l990()
}

/// Returns physical RAM size in bytes (variant 2; reads `/proc/meminfo`).
#[track_caller]
pub fn get_phys_mem_bytes_l990() -> f64 {
    let Ok(s) = std::fs::read_to_string("/proc/meminfo") else {
        return 0.0;
    };
    let Some(line) = s.lines().next() else {
        return 0.0;
    };
    let mut fields = line.split_whitespace();
    if fields.next() != Some("MemTotal:") {
        return 0.0;
    }
    match fields.next().and_then(|kb| kb.parse::<uint>().ok()) {
        Some(kb) => f64::from(kb) * 1000.0,
        None => 0.0,
    }
}

fn get_page_size_bytes() -> f64 {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        const SC_PAGESIZE: i32 = 30;
        unsafe extern "C" {
            fn sysconf(name: i32) -> isize;
        }
        let page_size = unsafe { sysconf(SC_PAGESIZE) };
        if page_size > 0 {
            return page_size as f64;
        }
    }
    4096.0
}

/// Returns current process memory bytes by reading `/proc/<pid>/statm`.
#[track_caller]
pub fn get_mem_use_bytes_l1008() -> f64 {
    let Ok(buffer) = std::fs::read_to_string("/proc/self/statm") else {
        return 0.0;
    };
    let Some(pages) = buffer
        .split_whitespace()
        .next()
        .and_then(|x| x.parse::<f64>().ok())
    else {
        return 0.0;
    };
    update_peak_mem_use_bytes(pages * get_page_size_bytes())
}

/// Returns RSS bytes on macOS via `task_info`.
#[track_caller]
pub fn get_mem_use_bytes_l1060() -> f64 {
    #[cfg(target_os = "macos")]
    {
        const KERN_SUCCESS: i32 = 0;
        const KERN_INVALID_ARGUMENT: i32 = 4;
        const TASK_BASIC_INFO: i32 = 20;
        const DEFAULT_MEM_USE: f64 = 0.0;

        type MachPortT = u32;
        type MachMsgTypeNumberT = u32;
        type KernReturnT = i32;

        #[repr(C)]
        struct TimeValue {
            seconds: i32,
            microseconds: i32,
        }

        #[repr(C)]
        struct TaskBasicInfo {
            virtual_size: usize,
            resident_size: usize,
            resident_size_max: usize,
            user_time: TimeValue,
            system_time: TimeValue,
            policy: i32,
            suspend_count: i32,
        }

        #[link(name = "System")]
        unsafe extern "C" {
            fn mach_task_self() -> MachPortT;
            fn task_info(
                target_task: MachPortT,
                flavor: i32,
                task_info_out: *mut i32,
                task_info_out_cnt: *mut MachMsgTypeNumberT,
            ) -> KernReturnT;
        }

        let mut ti = TaskBasicInfo {
            virtual_size: 0,
            resident_size: 0,
            resident_size_max: 0,
            user_time: TimeValue {
                seconds: 0,
                microseconds: 0,
            },
            system_time: TimeValue {
                seconds: 0,
                microseconds: 0,
            },
            policy: 0,
            suspend_count: 0,
        };
        let mut count = (std::mem::size_of::<TaskBasicInfo>() / std::mem::size_of::<i32>()) as u32;
        let ok = unsafe {
            task_info(
                mach_task_self(),
                TASK_BASIC_INFO,
                &mut ti as *mut TaskBasicInfo as *mut i32,
                &mut count,
            )
        };
        if ok == KERN_INVALID_ARGUMENT || ok != KERN_SUCCESS {
            return DEFAULT_MEM_USE;
        }
        update_peak_mem_use_bytes(ti.resident_size as f64)
    }
    #[cfg(not(target_os = "macos"))]
    {
        get_mem_use_bytes_l1008()
    }
}

/// Returns physical RAM size on macOS via `sysctl`.
#[track_caller]
pub fn get_phys_mem_bytes_l1079() -> f64 {
    get_phys_mem_bytes_l990()
}

/// Returns the unsupported-platform fallback memory byte count.
#[track_caller]
pub fn get_mem_use_bytes_l1089() -> f64 {
    update_peak_mem_use_bytes(0.0)
}

/// Returns directory entries in `dir_name`, matching the Windows body.
#[track_caller]
pub fn mylistdir_l1096(dir_name: &str) -> Vec<String> {
    let mut file_names = Vec::new();
    file_names.push(".".to_string());
    file_names.push("..".to_string());

    let Ok(entries) = std::fs::read_dir(dir_name) else {
        return Vec::new();
    };
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let file_name = entry.file_name().to_string_lossy().to_string();
        file_names.push(file_name);
    }

    file_names
}

/// Variant returning directory entries in `dir_name` (excluding `.`/`..`).
#[track_caller]
pub fn mylistdir_l1122(dir_name: &str) -> Vec<String> {
    let mut file_names = Vec::new();
    file_names.push(".".to_string());
    file_names.push("..".to_string());
    let entries =
        std::fs::read_dir(dir_name).unwrap_or_else(|_| panic!("Directory not found: {dir_name}"));
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        file_names.push(entry.file_name().to_string_lossy().to_string());
    }
    file_names
}

/// Returns the peak resident-set size in bytes.
#[track_caller]
pub fn get_peak_mem_use_bytes() -> f64 {
    *PEAK_MEM_USE_BYTES.lock().unwrap()
}

/// Formats whole seconds as `HH:MM:SS`.
#[track_caller]
pub fn secs_to_hhmmss(secs: uint) -> String {
    let hh = secs / 3600;
    let mm = (secs - hh * 3600) / 60;
    let ss = secs % 60;
    if hh == 0 {
        format!("{mm:02}:{ss:02}")
    } else {
        format!("{hh:02}:{mm:02}:{ss:02}")
    }
}

/// Formats seconds (with fractions) using compact unit suffixes.
#[track_caller]
pub fn secs_to_str(secs: f64) -> String {
    let format_g2 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 2 {
            let raw = format!("{d:.1e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (1 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    if secs >= 60.0 {
        return secs_to_hhmmss(secs as uint);
    }
    if secs < 1e-6 {
        format!("{}s", format_g2(secs))
    } else if secs < 1e-3 {
        format!("{:.2}ms", secs * 1e3)
    } else if secs < 1.0 {
        format!("{secs:.3}s")
    } else if secs < 10.0 {
        format!("{secs:.2}s")
    } else {
        format!("{secs:.1}s")
    }
}

/// Formats a byte count using `kb`/`Mb`/`Gb`/`Tb` suffixes.
#[track_caller]
pub fn mem_bytes_to_str(bytes: f64) -> String {
    if bytes < 1e4 {
        format!("{bytes:.1}b")
    } else if bytes < 1e6 {
        format!("{:.1}kb", bytes / 1e3)
    } else if bytes < 10e6 {
        format!("{:.1}Mb", bytes / 1e6)
    } else if bytes < 1e9 {
        format!("{:.0}Mb", bytes / 1e6)
    } else if bytes < 100e9 {
        format!("{:.1}Gb", bytes / 1e9)
    } else {
        format!("{:.0}Gb", bytes / 1e9)
    }
}

/// Returns true if `s` parses as a finite floating-point literal.
#[track_caller]
pub fn is_valid_float_str_l1191(s: &str) -> bool {
    !s.is_empty() && s.parse::<f64>().is_ok()
}

/// Variant of `is_valid_float_str_l1191`.
#[track_caller]
pub fn is_valid_float_str_l1199(s: &str) -> bool {
    is_valid_float_str_l1191(s)
}

/// Parses `s` as `f64`, optionally treating `*` as `f64::MAX`.
#[track_caller]
pub fn str_to_float_l1204(s: &str, star_is_dbl_max: bool) -> f64 {
    str_to_float_l1209(s, star_is_dbl_max)
}

/// Parses `s` as `f64`, with optional `*`-as-`f64::MAX` and stricter validation.
#[track_caller]
pub fn str_to_float_l1209(s: &str, star_is_dbl_max: bool) -> f64 {
    if star_is_dbl_max && s == "*" {
        return f64::MAX;
    }
    if !is_valid_float_str_l1191(s) {
        panic!("Invalid floating-point number '{s}'");
    }
    s.parse::<f64>().unwrap()
}

/// Parses a memory size string (e.g. `1.5G`, `200M`) to a byte count.
#[track_caller]
pub fn str_to_mem_bytes(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let d = str_to_float_l1209(s, false);
    match s.as_bytes()[s.len() - 1].to_ascii_uppercase() {
        b'0'..=b'9' => d,
        b'K' => 1000.0 * d,
        b'M' => 1e6 * d,
        b'G' => 1e9 * d,
        _ => panic!("Invalid amount of memory '{s}'"),
    }
}

/// Replaces every occurrence of `a` in `s` with `_b`; returns true on any change.
#[track_caller]
pub fn replace(s: &mut String, a: &str, _b: &str) -> bool {
    let Some(n) = s.find(a) else {
        return false;
    };
    let mut t = String::new();
    for c in s[..n].chars() {
        t.push(c);
    }
    *s = t;
    true
}

/// Returns true if `s` ends with `t`.
#[track_caller]
pub fn ends_with(s: &str, t: &str) -> bool {
    if s.len() < t.len() {
        return false;
    }
    s.as_bytes()[s.len() - t.len()..] == *t.as_bytes()
}

/// Returns true if `s` is a non-empty sequence of ASCII digits.
#[track_caller]
pub fn is_uint_str(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_digit())
}

/// Parses `s` as a `uint`, optionally treating `*` as `uint::MAX`.
#[track_caller]
pub fn str_to_uint_l1278(s: &str, star_is_uint_max: bool) -> uint {
    if star_is_uint_max && s == "*" {
        return uint::MAX;
    }
    if !is_uint_str(s) {
        panic!("Invalid integer '{s}'");
    }
    let mut n: uint = 0;
    for c in s.bytes() {
        if !c.is_ascii_digit() {
            return n;
        }
        n = n.wrapping_mul(10).wrapping_add(uint::from(c - b'0'));
    }
    n
}

/// Parses `s` as a `uint64`.
#[track_caller]
pub fn str_to_uint64_l1294(s: &str) -> uint64 {
    if !is_uint_str(s) {
        panic!("Invalid integer '{s}'");
    }
    let mut n: uint64 = 0;
    for c in s.bytes() {
        if !c.is_ascii_digit() {
            return n;
        }
        n = n.wrapping_mul(10).wrapping_add(uint64::from(c - b'0'));
    }
    n
}

/// Variant that parses `s` as a `uint64`.
#[track_caller]
pub fn str_to_uint64_l1308(s: &str) -> uint64 {
    str_to_uint64_l1294(s)
}

/// Variant that parses `s` as a `uint`, optionally treating `*` as `uint::MAX`.
#[track_caller]
pub fn str_to_uint_l1313(s: &str, star_is_uint_max: bool) -> uint {
    str_to_uint_l1278(s, star_is_uint_max)
}

/// Formats `i` as a decimal string (alternate width form).
#[track_caller]
pub fn int_to_str2(i: uint64) -> String {
    if i < 9999 {
        format!("{}", i as uint)
    } else if i < uint::MAX as uint64 {
        format!("{} ({})", i as uint, int_to_str(i))
    } else {
        int_to_str(i)
    }
}

/// Formats a percentage with adaptive precision.
#[track_caller]
pub fn pct_to_str(pct: f64) -> String {
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    if pct == 0.0 {
        "0%".to_string()
    } else if pct < 0.1 {
        format!("{}%", format_g3(pct))
    } else {
        format!("{pct:.2}%")
    }
}

/// Formats `i` as a comma-separated decimal string.
#[track_caller]
pub fn int_to_str(i: uint64) -> String {
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let d = i as f64;
    if i < 10000 {
        format!("{}", i as uint)
    } else if i < 1_000_000 {
        format!("{:.1}k", d / 1e3)
    } else if i < 100_000_000 {
        format!("{:.1}M", d / 1e6)
    } else if i < 1_000_000_000 {
        format!("{:.0}M", d / 1e6)
    } else if i < 10_000_000_000 {
        format!("{:.1}G", d / 1e9)
    } else if i < 100_000_000_000 {
        format!("{:.0}G", d / 1e9)
    } else {
        format_g3(d)
    }
}

/// Formats a 64-bit `i` as a comma-separated decimal string.
#[track_caller]
pub fn int64_to_str(i: uint64) -> String {
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let d = i as f64;
    if i < 10000 {
        format!("{}", i as uint)
    } else if i < 1_000_000 {
        format!("{:.1}k", d / 1e3)
    } else if i < 10_000_000 {
        format!("{:.1}M", d / 1e6)
    } else if i < 1_000_000_000 {
        format!("{:.0}M", d / 1e6)
    } else if i < 10_000_000_000 {
        format!("{:.1}G", d / 1e9)
    } else if i < 100_000_000_000 {
        format!("{:.0}G", d / 1e9)
    } else {
        format_g3(d)
    }
}

/// Formats `d` with `%g`-like compact notation.
#[track_caller]
pub fn float_to_str_l1385(d: f64) -> String {
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let a = d.abs();
    if a < 0.01 {
        format_g3(a)
    } else if a < 1.0 {
        format!("{a:.3}")
    } else if a <= 10.0 {
        if a.fract() < 0.05 {
            format!("{d:.0}")
        } else {
            format!("{d:.1}")
        }
    } else if a < 10000.0 {
        format!("{d:.1}")
    } else if a < 1e6 {
        format!("{:.1}k", d / 1e3)
    } else if a < 10e6 {
        format!("{:.1}M", d / 1e6)
    } else if a < 1e9 {
        format!("{:.1}M", d / 1e6)
    } else if a < 999e9 {
        format!("{:.1}G", d / 1e9)
    } else if a < 999e12 {
        format!("{:.1}T", d / 1e9)
    } else {
        format_g3(d)
    }
}

/// Formats an unsigned integer as a float string (legacy variant).
#[track_caller]
pub fn float_to_str_l1417(u: uint64) -> String {
    float_to_str_l1385(u as f64)
}

/// Formats `d` as an integer when whole, otherwise as a compact float string.
#[track_caller]
pub fn int_float_to_str(d: f64) -> String {
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let a = d.abs();
    if a < 1.0 {
        format_g3(a)
    } else if a <= 10.0 {
        format!("{d:.0}")
    } else if a < 10000.0 {
        format!("{d:.0}")
    } else if a < 1e6 {
        format!("{:.1}k", d / 1e3)
    } else if a < 10e6 {
        format!("{:.1}M", d / 1e6)
    } else if a < 1e9 {
        format!("{:.1}M", d / 1e6)
    } else if a < 10e9 {
        format!("{:.1}G", d / 1e9)
    } else if a < 100e9 {
        format!("{:.1}G", d / 1e9)
    } else {
        format_g3(d)
    }
}

/// Returns the current progress-line prefix (time + memory + thread).
#[track_caller]
pub fn get_progress_prefix_str() -> String {
    let bytes = get_mem_use_bytes_l1008();
    let secs = get_elapsed_secs();
    let mut s = secs_to_hhmmss(secs);
    if bytes > 0.0 {
        s.push(' ');
        s.push_str(&format!("{:<6}", mem_bytes_to_str(bytes)));
    }
    s.push(' ');
    s
}

/// C-string view of the progress prefix.
#[track_caller]
pub fn get_progress_prefix_c_str() -> String {
    get_progress_prefix_str()
}

/// Returns elapsed wall-clock time as `HH:MM:SS`.
pub fn get_elapsed_time_str() -> String {
    secs_to_hhmmss(get_elapsed_secs())
}

/// Returns peak memory usage as a human-readable string.
pub fn get_max_ram_str() -> String {
    format!("{:>5}", mem_bytes_to_str(get_peak_mem_use_bytes()))
}

/// Enables or disables the progress prefix; returns the previous setting.
#[track_caller]
pub fn progress_prefix(on: bool) -> bool {
    let mut state = PROGRESS_STATE.lock().unwrap();
    let old_value = state.prefix_on;
    state.prefix_on = on;
    old_value
}

/// Enables or disables C++ `opt(quiet)`-style progress suppression.
#[track_caller]
pub fn set_quiet(quiet: bool) -> bool {
    QUIET_STATE.swap(quiet, std::sync::atomic::Ordering::Relaxed)
}

/// Returns true when progress output is currently suppressed.
#[track_caller]
pub fn get_quiet() -> bool {
    QUIET_STATE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Runs `f` with temporary C++ `opt(quiet)`-style progress suppression.
#[track_caller]
pub fn with_quiet<T, F: FnOnce() -> T>(quiet: bool, f: F) -> T {
    struct QuietGuard(bool);
    impl Drop for QuietGuard {
        fn drop(&mut self) {
            set_quiet(self.0);
        }
    }

    let guard = QuietGuard(set_quiet(quiet));
    let result = f();
    drop(guard);
    result
}

/// Writes `s` to the log with a progress prefix.
#[track_caller]
pub fn progress_log(s: &str) -> String {
    // Mirror C++ myutils.cpp:1504 `ProgressLog` which calls both `Log()` and
    // `Progress()`, so `-log` captures the same lines that go to stderr.
    log(s);
    let saved_prefix = {
        let mut state = PROGRESS_STATE.lock().unwrap();
        let saved_prefix = state.prefix_on;
        state.prefix_on = false;
        saved_prefix
    };
    let out = progress(s);
    let mut state = PROGRESS_STATE.lock().unwrap();
    state.prefix_on = saved_prefix;
    out
}

/// Writes `s` to the log preceded by the current progress prefix.
#[track_caller]
pub fn progress_log_prefix(s: &str) -> String {
    let out = format!("{s}\n");
    log(&out);
    progress(&out)
}

/// Prints `s` to stderr and also returns it.
#[track_caller]
pub fn pr(s: &str) -> String {
    s.to_string()
}

/// Writes `s` to caller-owned file sink, matching C++ `Pr(FILE*)` null handling.
#[track_caller]
pub fn pr_file(f: Option<&mut std::fs::File>, s: &str) {
    if let Some(f) = f {
        std::io::Write::write_all(f, s.as_bytes()).expect("Pr write failed");
    }
}

/// Writes a progress message to stderr (with the active prefix).
#[track_caller]
pub fn progress(s: &str) -> String {
    if get_quiet() {
        return String::new();
    }
    progress_unchecked(s)
}

fn progress_unchecked(s: &str) -> String {
    let mut state = PROGRESS_STATE.lock().unwrap();
    let mut out = String::new();
    for c in s.chars() {
        if state.prefix_on && state.curr_line_length == 0 {
            let prefix = get_progress_prefix_str();
            state.curr_line_length += prefix.len() as uint;
            out.push_str(&prefix);
        }
        if c == '\n' || c == '\r' {
            for _ in state.curr_line_length..state.last_line_length {
                out.push(' ');
            }
            if c == '\n' {
                state.last_line_length = 0;
            } else {
                state.last_line_length = state.curr_line_length;
            }
            state.curr_line_length = 0;
            out.push(c);
        } else {
            out.push(c);
            state.curr_line_length += 1;
        }
    }
    eprint!("{out}");
    out
}

/// Logs program name, version, command line and timestamp.
#[track_caller]
pub fn log_program_info_and_cmd_line() -> String {
    let mut out = String::new();
    out.push_str(&get_version_string());
    out.push_str(" built ");
    out.push_str("unknown unknown\n");
    out.push_str("Started ");
    out.push_str(&format!("{:?}\n", std::time::SystemTime::now()));
    out.push_str(&print_cmd_line());
    log(&out);
    out
}

/// Logs the elapsed time and peak RAM at exit.
#[track_caller]
pub fn log_elapsed_time_and_ram() -> String {
    let secs = get_elapsed_secs();
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("Finished {:?}\n", std::time::SystemTime::now()));
    out.push_str(&format!("Elapsed time {}\n", secs_to_hhmmss(secs)));
    out.push_str(&format!(
        "Max memory {}\n",
        mem_bytes_to_str(get_peak_mem_use_bytes())
    ));
    log(&out);
    out
}

/// Emits the short command progress line from C++ `main.cpp` and updates
/// translated `g_Arg1` from argv[2].
#[track_caller]
pub fn main_cpp_short_cmd_progress() -> String {
    let argv = G_ARGV.lock().unwrap().clone();
    let n = argv.len();
    assert!(n > 0);
    let mut short_cmd_line = String::new();
    if n > 1 {
        short_cmd_line.push_str(&argv[1]);
    }
    if n > 2 {
        *G_ARG1.lock().unwrap() = argv[2].clone();
        short_cmd_line.push(' ');
        short_cmd_line.push_str(&argv[2]);
    }
    if n <= 1 {
        return String::new();
    }

    let printable = short_cmd_line.get(1..).unwrap_or("");
    progress_prefix(false);
    let out = progress(&format!("[{printable}]\n"));
    progress_prefix(true);
    out
}

/// Returns the translated C++ `g_Arg1` value.
#[track_caller]
pub fn get_arg1() -> String {
    G_ARG1.lock().unwrap().clone()
}

/// Sets the translated C++ `g_Arg1` value.
#[track_caller]
pub fn set_arg1(value: &str) {
    *G_ARG1.lock().unwrap() = value.to_string();
}

/// Formats `x/y` as a percentage string.
#[track_caller]
pub fn pct_str(x: f64, y: f64) -> String {
    if y == 0.0 {
        if x == 0.0 {
            return "100%".to_string();
        }
        return "inf%".to_string();
    }
    let p = x * 100.0 / y;
    if p < 1.0 {
        let decimals = if p == 0.0 {
            0
        } else {
            let exponent = p.abs().log10().floor() as i32;
            usize::try_from((1 - exponent).max(0)).unwrap()
        };
        let mut s = format!("{p:.decimals$}");
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        format!("{s:>5}%")
    } else {
        format!("{p:5.1}%")
    }
}

/// Returns a short string describing the current progress verbosity.
#[track_caller]
pub fn get_progress_level_str() -> String {
    let state = PROGRESS_STATE.lock().unwrap();
    let level = if state.count == uint::MAX {
        if state.index == uint::MAX {
            "100%".to_string()
        } else {
            state.index.to_string()
        }
    } else {
        pct_str((state.index + 1) as f64, state.count as f64)
    };
    format!("{} {}", level, state.desc)
}

fn get_progress_time_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn progress_step_throttle(state: &mut ProgressState, is_last_step: bool) -> bool {
    if is_last_step {
        return true;
    }

    state.step_calls += 1;
    if state.counts_interval > 0 && state.step_calls % state.counts_interval != 0 {
        return false;
    }

    let now = get_progress_time_secs();
    if now == state.time_last_output_step {
        if state.counts_interval < 128 {
            state.counts_interval = (state.counts_interval * 3) / 2;
        } else {
            state.counts_interval += 64;
        }
        return false;
    }

    if state.time_last_output_step > 0 {
        let secs = now.saturating_sub(state.time_last_output_step);
        if secs > 1 {
            state.counts_interval /= (secs * 8) as uint;
        }
    }
    if state.counts_interval < 1 {
        state.counts_interval = 1;
    }
    state.time_last_output_step = now;
    true
}

/// Default progress-callback identifier (placeholder).
#[track_caller]
pub fn default_pcb() -> &'static str {
    "Processing"
}

/// Installs the progress callback.
#[track_caller]
pub fn set_pcb(pcb: fn() -> &'static str) {
    *PROGRESS_CALLBACK_STATE.lock().unwrap() = pcb;
}

/// Restores the default progress callback.
#[track_caller]
pub fn reset_pcb() {
    set_pcb(default_pcb);
}

/// Begins file-progress tracking with `msg` as the label.
#[track_caller]
pub fn progress_file_init(f: &mut std::fs::File, msg: Option<&str>) -> String {
    let file_msg = {
        let mut state = PROGRESS_STATE.lock().unwrap();
        state.prog_file_size = get_stdio_file_size64(f) as f64;
        state.prog_file_tick = 0;
        state.prog_file_msg = msg.unwrap_or("Processing").to_string();
        state.prog_file_msg.clone()
    };
    progress_step(0, 1000, &file_msg).unwrap_or_default()
}

/// Advances the file-progress counter, optionally emitting a message.
#[track_caller]
pub fn progress_file_step(f: &mut std::fs::File, msg: Option<&str>) -> Option<String> {
    let pos = get_stdio_file_pos64(f) as f64;
    let (tick, file_msg) = {
        let mut state = PROGRESS_STATE.lock().unwrap();
        if state.prog_file_size == 0.0 {
            return None;
        }
        let tick = ((pos * 998.0) / state.prog_file_size) as uint;
        if tick <= state.prog_file_tick {
            return None;
        }
        if let Some(msg) = msg {
            state.prog_file_msg = msg.to_string();
        }
        state.prog_file_tick = tick;
        (tick, state.prog_file_msg.clone())
    };
    progress_step(tick, 1000, &file_msg)
}

/// Ends file-progress tracking and emits a final message.
#[track_caller]
pub fn progress_file_done(msg: Option<&str>) -> String {
    let file_msg = {
        let mut state = PROGRESS_STATE.lock().unwrap();
        if let Some(msg) = msg {
            state.prog_file_msg = msg.to_string();
        }
        state.prog_file_msg.clone()
    };
    progress_step(999, 1000, &file_msg).unwrap_or_default()
}

/// Reports `i/n` progress; returns the formatted message.
#[track_caller]
pub fn progress_callback(i: uint, n: uint) -> String {
    if get_quiet() {
        return String::new();
    }
    let mut needs_initial_newline = false;
    {
        let mut state = PROGRESS_STATE.lock().unwrap();
        if i == 0 {
            state.index = 0;
            state.count = n;
            state.desc = (*PROGRESS_CALLBACK_STATE.lock().unwrap())().to_string();
            state.counts_interval = 1;
            state.step_calls = 0;
            state.time_last_output_step = 0;
            needs_initial_newline = state.curr_line_length > 0;
        }
        let is_last_step = i == uint::MAX || i + 1 == n;
        if !progress_step_throttle(&mut state, is_last_step) {
            return String::new();
        }
        state.index = i;
        state.count = n;
        if is_last_step {
            state.counts_interval = 1;
        }
    }
    if needs_initial_newline {
        let _ = progress("\n");
    }
    let pct = if i == uint::MAX {
        pct_str(0.0, n as f64)
    } else {
        pct_str((i + 1) as f64, n as f64)
    };
    let pcb_text = (*PROGRESS_CALLBACK_STATE.lock().unwrap())();
    let out = format!(" {} {}", pct, pcb_text);
    let _ = progress(&format!(" {pct}"));
    let _ = progress(&format!(" {pcb_text}\r"));
    if i == uint::MAX || i + 1 == n {
        eprintln!();
    }
    out
}

/// 64-bit variant of `progress_step` with custom label `msg`.
#[track_caller]
pub fn progress_step64(i64: uint64, n64: uint64, msg: &str) -> Option<String> {
    let i = if i64 == 0 {
        0
    } else if i64 + 1 == n64 {
        999
    } else {
        ((i64 as f64 * 997.0 / n64 as f64) as uint) + 1
    };
    progress_step(i, 1000, msg)
}

/// Reports `i/n` progress with label `msg`.
#[track_caller]
pub fn progress_step(i: uint, n: uint, msg: &str) -> Option<String> {
    if get_quiet() {
        return None;
    }
    progress_step_unchecked(i, n, msg)
}

/// Reports progress even when command-local quiet is enabled.
#[track_caller]
pub fn progress_step_unquiet(i: uint, n: uint, msg: &str) -> Option<String> {
    progress_step_unchecked(i, n, msg)
}

fn progress_step_unchecked(i: uint, n: uint, msg: &str) -> Option<String> {
    let (level, is_last) = {
        let mut state = PROGRESS_STATE.lock().unwrap();
        if i == 0 {
            state.desc = msg.to_string();
            state.index = 0;
            state.count = n;
            state.counts_interval = 1;
            state.step_calls = 0;
            state.time_last_output_step = 0;
        }
        assert_eq!(n, state.count);
        if i >= n && i != uint::MAX {
            if !PROGRESS_STEP_RANGE_WARNING_DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                let _ = warning(&format!("ProgressStep({i},{n})"));
            }
            return None;
        }
        let is_last = i == uint::MAX || i + 1 == n;
        if !progress_step_throttle(&mut state, is_last) {
            return None;
        }
        state.index = i;
        if i > 0 {
            state.desc = msg.to_string();
        }
        let level = if state.count == uint::MAX {
            if state.index == uint::MAX {
                "100%".to_string()
            } else {
                state.index.to_string()
            }
        } else {
            pct_str((state.index + 1) as f64, state.count as f64)
        };
        let level = format!("{} {}", level, state.desc);
        if is_last {
            state.counts_interval = 1;
        }
        (level, is_last)
    };
    let _ = progress_unchecked(&format!(" {level}\r"));
    if is_last {
        eprintln!();
        log(&format!("{} {}\n", get_progress_prefix_str(), level));
    }
    Some(level)
}

/// Returns the struct packing alignment used by the binary.
#[track_caller]
pub fn get_struct_pack() -> uint {
    1
}

/// Returns a string describing the toolchain that built this binary.
#[track_caller]
pub fn compiler_info() -> String {
    let mut s = String::new();
    let bits = if cfg!(target_pointer_width = "64") {
        64
    } else {
        32
    };
    s.push_str(&format!("{bits} bits\n"));

    #[cfg(target_env = "gnu")]
    s.push_str("__GNUC__\n");

    #[cfg(target_os = "macos")]
    s.push_str("__APPLE__\n");

    #[cfg(target_env = "msvc")]
    s.push_str("_MSC_VER\n");

    s.push_str(&format!("sizeof(int) = {}\n", std::mem::size_of::<i32>()));
    s.push_str(&format!(
        "sizeof(long) = {}\n",
        std::mem::size_of::<std::os::raw::c_long>()
    ));
    s.push_str(&format!("sizeof(float) = {}\n", std::mem::size_of::<f32>()));
    s.push_str(&format!(
        "sizeof(double) = {}\n",
        std::mem::size_of::<f64>()
    ));
    s.push_str(&format!(
        "sizeof(void *) = {}\n",
        std::mem::size_of::<*const ()>()
    ));
    s.push_str(&format!(
        "sizeof(off_t) = {}\n",
        if cfg!(target_os = "windows") { 4 } else { 8 }
    ));
    s.push_str(&format!(
        "sizeof(size_t) = {}\n",
        std::mem::size_of::<usize>()
    ));

    s.push_str(&format!("pack({})\n", get_struct_pack()));

    #[cfg(any(target_os = "linux", target_os = "android"))]
    s.push_str("_FILE_OFFSET_BITS not defined\n");
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    s.push_str("_FILE_OFFSET_BITS not defined\n");

    s
}

/// Returns true if `s` begins with `t`.
#[track_caller]
pub fn starts_with_l1955(s: &str, t: &str) -> bool {
    let s = s.as_bytes();
    let t = t.as_bytes();
    for i in 0..t.len() {
        if i >= s.len() || s[i] != t[i] {
            return false;
        }
    }
    true
}

/// Reverses `s` in place.
#[track_caller]
pub fn reverse(s: &mut String) {
    let mut t = String::new();
    for c in s.chars().rev() {
        t.push(c);
    }
    *s = t;
}

/// Variant of `starts_with_l1955`.
#[track_caller]
pub fn starts_with_l1977(s: &str, t: &str) -> bool {
    starts_with_l1955(s, t)
}

/// Variant of `starts_with_l1955`.
#[track_caller]
pub fn starts_with_l1982(s: &str, t: &str) -> bool {
    starts_with_l1955(s, t)
}

/// Returns an uppercase copy of `s` (ASCII only).
#[track_caller]
pub fn to_upper(s: &str) -> String {
    s.bytes().map(|c| c.to_ascii_uppercase() as char).collect()
}

/// Returns a lowercase copy of `s` (ASCII only).
#[track_caller]
pub fn to_lower(s: &str) -> String {
    s.bytes().map(|c| c.to_ascii_lowercase() as char).collect()
}

/// Removes ASCII whitespace from `str_` in place.
#[track_caller]
pub fn strip_white_space(str_: &mut String) {
    let bytes = str_.as_bytes();
    let mut first_non_white = uint::MAX;
    let mut last_non_white = uint::MAX;
    for (i, c) in bytes.iter().enumerate() {
        if !c.is_ascii_whitespace() {
            if first_non_white == uint::MAX {
                first_non_white = i as uint;
            }
            last_non_white = i as uint;
        }
    }
    if first_non_white == uint::MAX {
        return;
    }
    *str_ = str_[first_non_white as usize..=last_non_white as usize].to_string();
}

/// Splits `str_` on `sep` and returns the segments.
#[track_caller]
pub fn split(str_: &str, sep: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut s = String::new();
    for c in str_.chars() {
        if (sep == '\0' && c.is_whitespace()) || c == sep {
            if !s.is_empty() || sep != '\0' {
                fields.push(s);
            }
            s = String::new();
        } else {
            s.push(c);
        }
    }
    if !s.is_empty() {
        fields.push(s);
    }
    fields
}

/// Returns the MUSCLE version string (e.g. `5.3`).
#[track_caller]
pub fn get_version_string() -> String {
    let git_ver = option_env!("MUSCLE_GIT_VER").unwrap_or("-");
    let mut flags = String::new();
    if cfg!(debug_assertions) {
        flags.push('D');
    }
    if option_env!("MUSCLE_TIMING").is_some() {
        flags.push('T');
    }
    format!("muscle 5.3.{}{} [{}]", get_platform(), flags, git_ver)
}

/// Returns the formatted version banner.
#[track_caller]
pub fn print_version() -> String {
    let build_date = option_env!("MUSCLE_BUILD_DATE").unwrap_or("unknown");
    let build_time = option_env!("MUSCLE_BUILD_TIME").unwrap_or("unknown");
    format!(
        "{}\nBuilt {} {}\n",
        get_version_string(),
        build_date,
        build_time
    )
}

/// Implements the `version` subcommand.
#[track_caller]
pub fn cmd_version() -> String {
    let mut s = print_version();
    s.push('\n');
    s
}

/// Returns the multi-line program banner.
#[track_caller]
pub fn print_banner() -> String {
    let build_date = option_env!("MUSCLE_BUILD_DATE").unwrap_or("unknown");
    let build_time = option_env!("MUSCLE_BUILD_TIME").unwrap_or("unknown");
    let ram = get_phys_mem_bytes_l990();
    format!(
        "\n{}  {} RAM, {} cores\nBuilt {} {}\n(C) Copyright 2004-2021 Robert C. Edgar.\nhttps://drive5.com\n\n",
        get_version_string(),
        mem_bytes_to_str(ram),
        get_cpu_core_count(),
        build_date,
        build_time
    )
}

/// Formats the stored argv as a one-line command for logging.
#[track_caller]
pub fn print_cmd_line() -> String {
    let argv = G_ARGV.lock().unwrap();
    let mut s = String::new();
    for arg in argv.iter() {
        s.push_str(arg);
        s.push(' ');
    }
    s.push('\n');
    s
}

/// Returns the stored argv joined as a single command string.
#[track_caller]
pub fn get_cmd_line() -> String {
    let argv = G_ARGV.lock().unwrap();
    let mut s = String::new();
    for (i, arg) in argv.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(arg);
    }
    s
}

/// Returns an owned copy of `s` (string-duplicate wrapper).
#[track_caller]
pub fn mystrsave(s: &str) -> String {
    s.to_string()
}

/// Computes `x^y` for unsigned integer exponents.
#[track_caller]
pub fn myipow(x: uint, y: uint) -> uint {
    let mut result: uint = 1;
    for _ in 0..y {
        if result > uint::MAX / x {
            panic!("myipow({x}, {y}), overflow");
        }
        result *= x;
    }
    result
}

/// Computes `x^y` as a 64-bit unsigned integer.
#[track_caller]
pub fn myipow64(x: uint, y: uint) -> uint64 {
    let mut result: uint64 = 1;
    for _ in 0..y {
        if result > uint64::MAX / uint64::from(x) {
            panic!("myipow({x}, {y}), overflow");
        }
        result *= uint64::from(x);
    }
    result
}

/// Formats integer `i` into a width-`w` field for log columns.
#[track_caller]
pub fn log_int(i: uint, w: uint) -> String {
    let s = if w == uint::MAX {
        if i < 9999 {
            format!("{i}")
        } else {
            format!("{i} ({})", int_to_str(u64::from(i)))
        }
    } else if i < 9999 {
        format!("{i:>width$}", width = w as usize)
    } else {
        format!(
            "{i:>width$} ({})",
            int_to_str(u64::from(i)),
            width = w as usize
        )
    };
    log(&s);
    s
}

/// Formats unsigned `u` into a width-`w` log field with optional prefix spaces.
#[track_caller]
pub fn logu(u: uint, w: uint, prefixspaces: uint) -> String {
    let mut s = " ".repeat(prefixspaces as usize);
    if u == uint::MAX {
        s.push_str(&format!("{:>width$}", "*", width = w as usize));
    } else {
        s.push_str(&format!("{u:>width$}", width = w as usize));
    }
    log(&s);
    s
}

/// Formats float `x` into a width-`w` log field with optional prefix spaces.
#[track_caller]
pub fn logf(x: f32, w: uint, prefixspaces: uint) -> String {
    let mut s = " ".repeat(prefixspaces as usize);
    if x == f32::MAX {
        s.push_str(&format!("{:>width$}", "*", width = w as usize));
    } else {
        s.push_str(&format!("{x:>width$.2}", width = w as usize));
    }
    log(&s);
    s
}

/// Steps the simple linear-congruential PRNG and returns the next 32-bit value.
#[track_caller]
pub fn slcg_rand() -> uint32 {
    let mut state = RAND_STATE.lock().unwrap();
    state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
    state.slcg_state
}

/// Seeds the SLCG PRNG.
#[track_caller]
pub fn slcg_srand(seed: uint32) {
    let mut state = RAND_STATE.lock().unwrap();
    state.slcg_state = seed;
    for _ in 0..10 {
        state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
    }
}

/// Initialises the global PRNG state (using `-randseed` if provided).
#[track_caller]
pub fn init_rand() {
    let mut state = RAND_STATE.lock().unwrap();
    if state.init_done {
        return;
    }
    state.init_done = true;
    state.slcg_state = match *PENDING_RAND_SEED.lock().unwrap() {
        Some(seed) => {
            set_cmd_opt_used("randseed");
            seed
        }
        None => 1,
    };
    for _ in 0..10 {
        state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
    }
    for i in 0..5 {
        state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
        state.x[i] = state.slcg_state;
    }
    for _ in 0..100 {
        let sum = 2_111_111_111_u64 * u64::from(state.x[3])
            + 1492_u64 * u64::from(state.x[2])
            + 1776_u64 * u64::from(state.x[1])
            + 5115_u64 * u64::from(state.x[0])
            + u64::from(state.x[4]);
        state.x[3] = state.x[2];
        state.x[2] = state.x[1];
        state.x[1] = state.x[0];
        state.x[4] = (sum >> 32) as uint32;
        state.x[0] = sum as uint32;
    }
}

/// Advances the global PRNG state by one step.
#[track_caller]
pub fn increment_rand() {
    let mut state = RAND_STATE.lock().unwrap();
    let sum = 2_111_111_111_u64 * u64::from(state.x[3])
        + 1492_u64 * u64::from(state.x[2])
        + 1776_u64 * u64::from(state.x[1])
        + 5115_u64 * u64::from(state.x[0])
        + u64::from(state.x[4]);
    state.x[3] = state.x[2];
    state.x[2] = state.x[1];
    state.x[1] = state.x[0];
    state.x[4] = (sum >> 32) as uint32;
    state.x[0] = sum as uint32;
}

/// Returns the next 32-bit random integer from the global PRNG.
#[track_caller]
pub fn rand_int32() -> uint32 {
    let mut state = RAND_STATE.lock().unwrap();
    if !state.init_done {
        state.init_done = true;
        state.slcg_state = match *PENDING_RAND_SEED.lock().unwrap() {
            Some(seed) => {
                set_cmd_opt_used("randseed");
                seed
            }
            None => 1,
        };
        for _ in 0..10 {
            state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
        }
        for i in 0..5 {
            state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
            state.x[i] = state.slcg_state;
        }
        for _ in 0..100 {
            let sum = 2_111_111_111_u64 * u64::from(state.x[3])
                + 1492_u64 * u64::from(state.x[2])
                + 1776_u64 * u64::from(state.x[1])
                + 5115_u64 * u64::from(state.x[0])
                + u64::from(state.x[4]);
            state.x[3] = state.x[2];
            state.x[2] = state.x[1];
            state.x[1] = state.x[0];
            state.x[4] = (sum >> 32) as uint32;
            state.x[0] = sum as uint32;
        }
    }
    let sum = 2_111_111_111_u64 * u64::from(state.x[3])
        + 1492_u64 * u64::from(state.x[2])
        + 1776_u64 * u64::from(state.x[1])
        + 5115_u64 * u64::from(state.x[0])
        + u64::from(state.x[4]);
    state.x[3] = state.x[2];
    state.x[2] = state.x[1];
    state.x[1] = state.x[0];
    state.x[4] = (sum >> 32) as uint32;
    state.x[0] = sum as uint32;
    state.x[0]
}

/// Returns a 32-bit random unsigned integer.
#[track_caller]
pub fn randu32() -> uint {
    rand_int32()
}

/// Returns a 64-bit random unsigned integer.
#[track_caller]
pub fn randu64() -> uint64 {
    let lo = uint64::from(randu32());
    let hi = uint64::from(randu32());
    lo | (hi << 32)
}

/// Stores the command-line RNG seed until the C++ `InitRand()` path is reached.
#[track_caller]
pub fn set_pending_rand_seed(seed: Option<uint>) {
    *PENDING_RAND_SEED.lock().unwrap() = seed;
}

/// Resets the PRNG state to `seed`.
#[track_caller]
pub fn reset_rand(seed: uint) {
    let mut state = RAND_STATE.lock().unwrap();
    state.init_done = true;
    state.slcg_state = seed;
    for _ in 0..10 {
        state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
    }
    for i in 0..5 {
        state.slcg_state = state.slcg_state.wrapping_mul(214013).wrapping_add(2531011);
        state.x[i] = state.slcg_state;
    }
    for _ in 0..100 {
        let sum = 2_111_111_111_u64 * u64::from(state.x[3])
            + 1492_u64 * u64::from(state.x[2])
            + 1776_u64 * u64::from(state.x[1])
            + 5115_u64 * u64::from(state.x[0])
            + u64::from(state.x[4]);
        state.x[3] = state.x[2];
        state.x[2] = state.x[1];
        state.x[1] = state.x[0];
        state.x[4] = (sum >> 32) as uint32;
        state.x[0] = sum as uint32;
    }
}

/// Returns the number of CPU cores reported by the OS.
#[track_caller]
pub fn get_cpu_core_count() -> uint {
    match std::thread::available_parallelism() {
        Ok(n) => uint::try_from(n.get()).unwrap_or(uint::MAX),
        Err(_) => 1,
    }
}

/// Returns the index of the current worker thread (always 0 in this port).
#[track_caller]
pub fn get_thread_index() -> uint {
    0
}

/// Diagnoses whether the named CLI option was set but never consumed.
#[track_caller]
pub fn check_used_opt(set: bool, used: bool, name: &str) -> Option<String> {
    if set && !used {
        Some(warning(&format!("Option -{name} not used")))
    } else {
        None
    }
}

/// Returns warning strings for any CLI options that were set but never consumed.
#[track_caller]
pub fn check_used_opts(log_all: bool) -> Vec<String> {
    let _ = log_all;
    let state = CMD_OPT_STATE.lock().unwrap();
    let mut warnings = Vec::new();
    for name in FLAG_OPT_NAMES
        .iter()
        .chain(UNS_OPT_NAMES.iter())
        .chain(FLT_OPT_NAMES.iter())
        .chain(STR_OPT_NAMES.iter())
    {
        if let Some(entry) = state.get(*name) {
            if entry.value.is_some() && !entry.used {
                warnings.push(warning(&format!("Option -{name} not used")));
            }
        }
    }
    warnings
}

/// Aborts with a command-line usage error message.
#[track_caller]
pub fn cmd_line_err(msg: &str) -> ! {
    let text = cmd_line_err_text(msg);
    eprint!("{text}");
    panic!("{text}")
}

/// Formats C++ `CmdLineErr` stderr exactly.
#[track_caller]
pub fn cmd_line_err_text(msg: &str) -> String {
    format!("\n\nInvalid command line\n{msg}\n\n")
}

/// C++-literal command-line error process exit.
#[track_caller]
pub fn cmd_line_err_cpp_literal(msg: &str) -> ! {
    eprint!("{}", cmd_line_err_text(msg));
    std::process::exit(1);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CppLifecycleAction {
    Continue,
    Exit {
        status: i32,
        stdout: String,
        stderr: String,
    },
}

/// Reads whitespace-separated args from `file_name`.
#[track_caller]
pub fn get_args_from_file(file_name: &str) -> Vec<String> {
    let text = std::fs::read_to_string(file_name)
        .unwrap_or_else(|err| panic!("OpenStdioFile({file_name}): {err}"));
    let mut args = Vec::new();
    for line in text.lines() {
        let line = match line.find('#') {
            Some(n) => &line[..n],
            None => line,
        };
        let fields = split(line, '\0');
        args.extend(fields);
    }
    args
}

/// Looks up a flag option by name.
#[track_caller]
pub fn try_flag_opt(opt_name: &str) -> bool {
    if FLAG_OPT_NAMES.contains(&opt_name) {
        let mut state = CMD_OPT_STATE.lock().unwrap();
        state.insert(
            opt_name.to_string(),
            CmdOptEntry {
                value: Some(CmdOptValue::Flag(true)),
                used: false,
            },
        );
        true
    } else {
        false
    }
}

/// Looks up an unsigned-int option, parsing `value` if present.
#[track_caller]
pub fn try_uns_opt(opt_name: &str, value: &str) -> Option<uint> {
    if UNS_OPT_NAMES.contains(&opt_name) {
        let value = str_to_uint_l1278(value, false);
        let mut state = CMD_OPT_STATE.lock().unwrap();
        state.insert(
            opt_name.to_string(),
            CmdOptEntry {
                value: Some(CmdOptValue::Uns(value)),
                used: false,
            },
        );
        Some(value)
    } else {
        None
    }
}

/// Looks up a float option, parsing `value` if present.
#[track_caller]
pub fn try_float_opt(opt_name: &str, value: &str) -> Option<f64> {
    if FLT_OPT_NAMES.contains(&opt_name) {
        let value = str_to_float_l1209(value, false);
        let mut state = CMD_OPT_STATE.lock().unwrap();
        state.insert(
            opt_name.to_string(),
            CmdOptEntry {
                value: Some(CmdOptValue::Float(value)),
                used: false,
            },
        );
        Some(value)
    } else {
        None
    }
}

/// Looks up a string option.
#[track_caller]
pub fn try_str_opt(opt_name: &str, value: &str) -> Option<String> {
    if STR_OPT_NAMES.contains(&opt_name) {
        let value = mystrsave(value);
        let mut state = CMD_OPT_STATE.lock().unwrap();
        state.insert(
            opt_name.to_string(),
            CmdOptEntry {
                value: Some(CmdOptValue::Str(value.clone())),
                used: false,
            },
        );
        Some(value)
    } else {
        None
    }
}

/// Clears translated command-line option state.
#[track_caller]
pub fn reset_cmd_opt_state() {
    CMD_OPT_STATE.lock().unwrap().clear();
}

/// Marks a translated command-line option as consumed.
#[track_caller]
pub fn set_cmd_opt_used(opt_name: &str) {
    if let Some(entry) = CMD_OPT_STATE.lock().unwrap().get_mut(opt_name) {
        entry.used = true;
    }
}

/// Marks a set of translated command-line options as consumed.
#[track_caller]
pub fn set_cmd_opts_used(opt_names: &[&str]) {
    for opt_name in opt_names {
        set_cmd_opt_used(opt_name);
    }
}

/// Returns the stored translated command-line option value for tests/helpers.
#[track_caller]
pub fn get_cmd_opt_value(opt_name: &str) -> Option<String> {
    let state = CMD_OPT_STATE.lock().unwrap();
    let entry = state.get(opt_name)?;
    match entry.value.as_ref()? {
        CmdOptValue::Flag(value) => Some(value.to_string()),
        CmdOptValue::Uns(value) => Some(value.to_string()),
        CmdOptValue::Float(value) => Some(value.to_string()),
        CmdOptValue::Str(value) => Some(value.clone()),
    }
}

/// Parses the command line `args`, populating global option state.
#[track_caller]
pub fn my_cmd_line(args: &[String]) {
    reset_cmd_opt_state();
    if args.len() == 1 {
        return;
    }
    let mut expanded_argv = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "file:" && i + 1 < args.len() {
            let file_name = &args[i + 1];
            let args_from_file = get_args_from_file(file_name);
            for arg_from_file in args_from_file {
                expanded_argv.push(arg_from_file);
            }
            i += 2;
        } else {
            expanded_argv.push(arg.clone());
            i += 1;
        }
    }
    {
        let mut argv = G_ARGV.lock().unwrap();
        argv.clear();
        argv.extend(expanded_argv.iter().cloned());
    }

    let mut arg_index = 1;
    while arg_index < expanded_argv.len() {
        let arg = &expanded_argv[arg_index];
        if arg.len() > 1 && arg.as_bytes()[0] == b'-' {
            let long_name = if arg.len() > 2 && arg.as_bytes()[1] == b'-' {
                &arg[2..]
            } else {
                &arg[1..]
            };
            if long_name == "version" {
                return;
            }
            if try_flag_opt(long_name) {
                arg_index += 1;
                continue;
            }

            arg_index += 1;
            if arg_index >= expanded_argv.len() {
                cmd_line_err(&format!("Invalid option or missing value -{long_name}"));
            }
            let value = &expanded_argv[arg_index];
            if try_uns_opt(long_name, value).is_some()
                || try_float_opt(long_name, value).is_some()
                || try_str_opt(long_name, value).is_some()
            {
                arg_index += 1;
                continue;
            }
            cmd_line_err(&format!("Unknown option {long_name}"));
        } else if arg.as_bytes().first().is_some_and(|b| *b > 127) {
            cmd_line_err(&format!(
                "Invalid 8-bit byte in '{arg}' (did you paste from web page?)"
            ));
        } else {
            cmd_line_err(&format!(
                "Expected -option_name or --option_name, got '{arg}'"
            ));
        }
    }
}

/// C++-literal `MyCmdLine(argc, argv)` lifecycle without terminating tests.
#[track_caller]
pub fn my_cmd_line_cpp_literal_action(args: &[String]) -> CppLifecycleAction {
    reset_cmd_opt_state();
    if args.len() == 1 {
        return CppLifecycleAction::Exit {
            status: 0,
            stdout: usage(),
            stderr: String::new(),
        };
    }

    let mut expanded_argv = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "file:" && i + 1 < args.len() {
            let file_name = &args[i + 1];
            let args_from_file = get_args_from_file(file_name);
            for arg_from_file in args_from_file {
                expanded_argv.push(arg_from_file);
            }
            i += 2;
        } else {
            expanded_argv.push(arg.clone());
            i += 1;
        }
    }
    {
        let mut argv = G_ARGV.lock().unwrap();
        argv.clear();
        argv.extend(expanded_argv.iter().cloned());
    }

    let mut arg_index = 1;
    while arg_index < expanded_argv.len() {
        let arg = &expanded_argv[arg_index];
        if arg.len() > 1 && arg.as_bytes()[0] == b'-' {
            let long_name = if arg.len() > 2 && arg.as_bytes()[1] == b'-' {
                &arg[2..]
            } else {
                &arg[1..]
            };
            if long_name == "version" {
                return CppLifecycleAction::Exit {
                    status: 0,
                    stdout: cmd_version(),
                    stderr: String::new(),
                };
            }
            if try_flag_opt(long_name) {
                arg_index += 1;
                continue;
            }

            arg_index += 1;
            if arg_index >= expanded_argv.len() {
                return CppLifecycleAction::Exit {
                    status: 1,
                    stdout: String::new(),
                    stderr: cmd_line_err_text(&format!(
                        "Invalid option or missing value -{long_name}"
                    )),
                };
            }
            let value = &expanded_argv[arg_index];
            if try_uns_opt(long_name, value).is_some()
                || try_float_opt(long_name, value).is_some()
                || try_str_opt(long_name, value).is_some()
            {
                arg_index += 1;
                continue;
            }
            return CppLifecycleAction::Exit {
                status: 1,
                stdout: String::new(),
                stderr: cmd_line_err_text(&format!("Unknown option {long_name}")),
            };
        } else if arg.as_bytes().first().is_some_and(|b| *b > 127) {
            return CppLifecycleAction::Exit {
                status: 1,
                stdout: String::new(),
                stderr: cmd_line_err_text(&format!(
                    "Invalid 8-bit byte in '{arg}' (did you paste from web page?)"
                )),
            };
        } else {
            return CppLifecycleAction::Exit {
                status: 1,
                stdout: String::new(),
                stderr: cmd_line_err_text(&format!(
                    "Expected -option_name or --option_name, got '{arg}'"
                )),
            };
        }
    }

    if option_env!("MUSCLE_TIMING").is_some()
        && get_cmd_opt_value("threads")
            .and_then(|value| value.parse::<uint>().ok())
            .is_some_and(|threads| threads > 1)
    {
        return CppLifecycleAction::Exit {
            status: 1,
            stdout: String::new(),
            stderr: "\n---Fatal error---\n--threads > 1 && TIMING\n".to_string(),
        };
    }

    if get_cmd_opt_value("compilerinfo").as_deref() == Some("true") {
        return CppLifecycleAction::Exit {
            status: 0,
            stdout: compiler_info(),
            stderr: String::new(),
        };
    }

    CppLifecycleAction::Continue
}

/// Extracts the accession portion (before the first `|`) from a FASTA label.
#[track_caller]
pub fn get_acc_from_label(label: &str) -> String {
    let mut acc = String::new();
    for c in label.bytes() {
        if c.is_ascii_alphanumeric() || c == b'_' {
            acc.push(c as char);
        } else {
            return acc;
        }
    }
    acc
}

/// Returns the file base name (no directory or extension).
#[track_caller]
pub fn get_base_name(path_name: &str) -> String {
    let n = path_name.len();
    if n == 0 {
        return String::new();
    }
    let mut start = 0;
    for (i, c) in path_name.bytes().enumerate() {
        if i + 1 >= n {
            break;
        }
        if c == b'/' || c == b'\\' {
            start = i + 1;
        }
    }
    let mut base_name = String::new();
    for (i, c) in path_name.bytes().enumerate().skip(start) {
        if i + n == n && c == b'/' {
            break;
        }
        base_name.push(c as char);
    }
    for ext in [".afa", ".fa", ".aln", ".msa"] {
        if let Some(p) = base_name.find(ext) {
            base_name.truncate(p);
        }
    }
    base_name
}

/// Formats `seq` with `label` as a single FASTA record (variant 1).
#[track_caller]
pub fn seq_to_fasta_l2561(seq: &str, label: &str) -> String {
    seq_to_fasta_l2571(seq.as_bytes(), Some(label))
}

/// Formats `seq` with `label` as a single FASTA record (variant 2).
#[track_caller]
pub fn seq_to_fasta_l2566(seq: &str, label: &str) -> String {
    seq_to_fasta_l2571(seq.as_bytes(), Some(label))
}

/// Formats a byte sequence as a FASTA record, wrapping at the standard line width.
#[track_caller]
pub fn seq_to_fasta_l2571(seq: &[byte], label: Option<&str>) -> String {
    if seq.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    if let Some(label) = label {
        out.push('>');
        out.push_str(label);
        out.push('\n');
    }
    let rowlen = 80;
    let block_count = seq.len().div_ceil(rowlen);
    for block_index in 0..block_count {
        let from = block_index * rowlen;
        let mut to = from + rowlen;
        if to >= seq.len() {
            to = seq.len();
        }
        for &c in &seq[from..to] {
            out.push(c as char);
        }
        out.push('\n');
    }
    out
}

/// Fisher-Yates shuffle of `v` in place using the global PRNG.
#[track_caller]
pub fn shuffle(v: &mut [uint]) {
    let n = v.len();
    if n == 0 {
        return;
    }
    for i in (1..n).rev() {
        let j = (randu32() as usize) % (i + 1);
        v.swap(i, j);
    }
}

/// Ensures `dir` ends with a `/` separator (in place).
#[track_caller]
pub fn dirize(dir: &mut String) {
    if !ends_with(dir, "/") {
        dir.push('/');
    }
}
