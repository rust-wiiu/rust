use cafe_sys::coreinit::fs::{
    self, CmdBlock, DirHandle, ErrorFlag, FileHandle, MAX_PATH, Stat, StatFlags, Status,
};

use crate::cell::UnsafeCell;
use crate::ffi::{CStr, OsString};
use crate::fs::TryLockError;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::mem::{self, MaybeUninit};
use crate::path::{Path, PathBuf};
use crate::sync::{Arc, OnceLock};
use crate::sys::fs::common::NOT_FILE_ERROR;
use crate::sys::helpers::run_path_with_cstr;
use crate::sys::os_str::Buf;
use crate::sys::time::SystemTime;
use crate::sys::{FromInner, unsupported, unsupported_err};
use crate::time::Duration;
use crate::{fmt, ptr};

#[expect(dead_code)]
#[path = "unsupported.rs"]
mod unsupported_fs;
pub use unsupported_fs::{
    Dir, ExtraHomeDirs, ExtraMediaDirs, FileTimes, canonicalize, link, readlink, set_times,
    set_times_nofollow, symlink,
};

pub use crate::sys::fs::common::{exists, remove_dir_all};

const STAT_CTIME: StatFlags = StatFlags(0x0800_0000);
const STAT_MTIME: StatFlags = StatFlags(0x0400_0000);
const MAX_MOUNTPATH_SIZE: usize = 128;

struct Client(UnsafeCell<fs::Client>);

// SAFETY: The FS library queues the commands of each client, so threads can
// share one client as long as every command uses its own command block.
unsafe impl Sync for Client {}

static CLIENT: Client = Client(UnsafeCell::new(unsafe { mem::zeroed() }));

fn client() -> io::Result<*mut fs::Client> {
    static STATUS: OnceLock<Status> = OnceLock::new();
    let status = *STATUS.get_or_init(|| unsafe {
        fs::init();
        let status = fs::add_client(CLIENT.0.get(), ErrorFlag::ALL);
        if status == Status::SUCCESS {
            mount_sd_card(CLIENT.0.get());
        }
        status
    });
    cvt(status)?;
    Ok(CLIENT.0.get())
}

/// Mounts the SD card at `/vol/external01`, where homebrew expects it. Errors
/// are ignored, since there might be no SD card.
///
/// `client` must be a registered client.
unsafe fn mount_sd_card(client: *mut fs::Client) {
    let mut block = MaybeUninit::<CmdBlock>::uninit();
    let mut source = MaybeUninit::<fs::MountSource>::uninit();
    let mut target = [0; MAX_MOUNTPATH_SIZE];
    unsafe {
        fs::init_cmd_block(block.as_mut_ptr());
        let status = fs::get_mount_source(
            client,
            block.as_mut_ptr(),
            fs::MountSourceType::SD,
            source.as_mut_ptr(),
            ErrorFlag::ALL,
        );
        if status == Status::SUCCESS {
            fs::mount(
                client,
                block.as_mut_ptr(),
                source.as_mut_ptr(),
                target.as_mut_ptr(),
                target.len() as u32,
                ErrorFlag::ALL,
            );
        }
    }
}

/// Issues one synchronous FS command and returns its raw status.
fn call(f: impl FnOnce(*mut fs::Client, *mut CmdBlock) -> Status) -> io::Result<Status> {
    let client = client()?;
    let mut block = MaybeUninit::<CmdBlock>::uninit();
    unsafe { fs::init_cmd_block(block.as_mut_ptr()) };
    Ok(f(client, block.as_mut_ptr()))
}

fn run(f: impl FnOnce(*mut fs::Client, *mut CmdBlock) -> Status) -> io::Result<u32> {
    cvt(call(f)?)
}

fn cvt(status: Status) -> io::Result<u32> {
    use io::ErrorKind::*;
    Err(match status {
        Status(n @ 0..) => return Ok(n as u32),
        Status::ALREADY_OPEN => io::const_error!(ResourceBusy, "file is already open"),
        Status::EXISTS => io::const_error!(AlreadyExists, "file exists"),
        Status::NOT_FOUND => io::const_error!(NotFound, "no such file or directory"),
        Status::NOT_FILE => io::const_error!(IsADirectory, "is a directory"),
        Status::NOT_DIR => io::const_error!(NotADirectory, "not a directory"),
        Status::PERMISSION_ERROR => io::const_error!(PermissionDenied, "permission denied"),
        Status::FILE_TOO_BIG => io::const_error!(FileTooLarge, "file too large"),
        Status::STORAGE_FULL => io::const_error!(StorageFull, "no space left on device"),
        Status::UNSUPPORTED_CMD => io::const_error!(Unsupported, "operation not supported"),
        Status::WRITE_PROTECTED => {
            io::const_error!(ReadOnlyFilesystem, "media is write protected")
        }
        Status::MAX => io::const_error!(Uncategorized, "too many open files"),
        Status::ACCESS_ERROR => io::const_error!(Uncategorized, "bad file mode"),
        _ => io::const_error!(Uncategorized, "file system error"),
    })
}

fn with_path<T>(path: &Path, f: &dyn Fn(&CStr) -> io::Result<T>) -> io::Result<T> {
    run_path_with_cstr(path, &|path| {
        if path.count_bytes() >= MAX_PATH {
            return Err(io::const_error!(io::ErrorKind::InvalidFilename, "path is too long"));
        }
        f(path)
    })
}

/// FS transfers need buffers whose address and length are multiples of 64.
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct IoBlock([u8; 64]);

/// Largest bounce buffer used for unaligned reads and writes.
const BOUNCE_MAX: usize = 128 * 1024;

fn is_io_aligned(ptr: *const u8, len: usize) -> bool {
    ptr.addr() % align_of::<IoBlock>() == 0 && len % size_of::<IoBlock>() == 0
}

fn bounce_buffer(len: usize) -> Box<[MaybeUninit<IoBlock>]> {
    Box::new_uninit_slice(len.div_ceil(size_of::<IoBlock>()))
}

/// Seconds from 1970-01-01 to 1980-01-01, the epoch of FS timestamps.
const FS_EPOCH_OFFSET: Duration = Duration::from_secs(315_532_800);

fn fs_time(valid: bool, micros: u64) -> io::Result<SystemTime> {
    if !valid {
        return unsupported();
    }
    Ok(SystemTime::from_unix(FS_EPOCH_OFFSET + Duration::from_micros(micros)))
}

pub struct File {
    handle: FileHandle,
}

#[derive(Clone)]
pub struct FileAttr(Stat);

pub struct ReadDir {
    handle: DirHandle,
    root: Arc<Path>,
}

pub struct DirEntry {
    root: Arc<Path>,
    name: OsString,
    stat: Stat,
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilePermissions {
    readonly: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileType {
    is_dir: bool,
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.0.size.into()
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions { readonly: self.0.mode.0 & 0x222 == 0 }
    }

    pub fn file_type(&self) -> FileType {
        FileType { is_dir: self.0.flags.contains(StatFlags::DIRECTORY) }
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        fs_time(self.0.flags.contains(STAT_MTIME), self.0.modified)
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        unsupported()
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        fs_time(self.0.flags.contains(STAT_CTIME), self.0.created)
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.readonly
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir
    }

    pub fn is_symlink(&self) -> bool {
        false
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ReadDir").field(&self.root).finish()
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        loop {
            let mut entry = MaybeUninit::<fs::DirectoryEntry>::uninit();
            let status = match call(|c, b| unsafe {
                fs::read_dir(c, b, self.handle, entry.as_mut_ptr(), ErrorFlag::ALL)
            }) {
                Ok(Status::END) => return None,
                Ok(status) => status,
                Err(e) => return Some(Err(e)),
            };
            if let Err(e) = cvt(status) {
                return Some(Err(e));
            }
            let entry = unsafe { entry.assume_init() };
            let name =
                CStr::from_bytes_until_nul(&entry.name).map_or(&entry.name[..], CStr::to_bytes);
            if name == b"." || name == b".." {
                continue;
            }
            return Some(Ok(DirEntry {
                root: self.root.clone(),
                name: OsString::from_inner(Buf { inner: name.to_vec() }),
                stat: entry.info,
            }));
        }
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        let _ = call(|c, b| unsafe { fs::close_dir(c, b, self.handle, ErrorFlag::ALL) });
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.root.join(&self.name)
    }

    pub fn file_name(&self) -> OsString {
        self.name.clone()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        Ok(FileAttr(self.stat))
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(FileAttr(self.stat).file_type())
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        with_path(path, &|path| File::open_cstr(path, opts))
    }

    fn open_cstr(path: &CStr, opts: &OpenOptions) -> io::Result<File> {
        let writes = opts.write || opts.append;
        if !(opts.read || writes)
            || (!writes && (opts.truncate || opts.create || opts.create_new))
            || (opts.append && opts.truncate && !opts.create_new)
        {
            return Err(io::const_error!(io::ErrorKind::InvalidInput, "invalid open options"));
        }

        let append = if opts.read { c"a+" } else { c"a" };
        let create = if opts.read { c"w+" } else { c"w" };
        let mode = if opts.create_new {
            // FIXME: not atomic, the FS has no exclusive create.
            if stat_cstr(path).is_ok() {
                return Err(io::const_error!(io::ErrorKind::AlreadyExists, "file exists"));
            }
            if opts.append { append } else { create }
        } else if opts.append {
            if !opts.create {
                stat_cstr(path)?;
            }
            append
        } else if !opts.write {
            c"r"
        } else if opts.create && opts.truncate {
            create
        } else {
            let file = match File::open_mode(path, c"r+") {
                Err(e) if opts.create && e.kind() == io::ErrorKind::NotFound => {
                    File::open_mode(path, create)?
                }
                file => file?,
            };
            if opts.truncate {
                file.truncate(0)?;
            }
            return Ok(file);
        };
        File::open_mode(path, mode)
    }

    fn open_mode(path: &CStr, mode: &CStr) -> io::Result<File> {
        let mut handle = FileHandle::INVALID;
        run(|c, b| unsafe {
            fs::open_file(c, b, path.as_ptr(), mode.as_ptr(), &mut handle, ErrorFlag::ALL)
        })?;
        Ok(File { handle })
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        let mut stat = MaybeUninit::<Stat>::uninit();
        run(|c, b| unsafe {
            fs::get_stat_file(c, b, self.handle, stat.as_mut_ptr(), ErrorFlag::ALL)
        })?;
        Ok(FileAttr(unsafe { stat.assume_init() }))
    }

    pub fn fsync(&self) -> io::Result<()> {
        run(|c, b| unsafe { fs::flush_file(c, b, self.handle, ErrorFlag::ALL) }).map(drop)
    }

    pub fn datasync(&self) -> io::Result<()> {
        self.fsync()
    }

    pub fn lock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn lock_shared(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }

    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }

    pub fn unlock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        let pos = self.tell()?;
        let len = self.file_attr()?.size();
        if size < len {
            self.set_pos(size)?;
            run(|c, b| unsafe { fs::truncate_file(c, b, self.handle, ErrorFlag::ALL) })?;
        } else if size > len {
            // FSTruncateFile cuts the file at the current position, so it cannot grow it.
            self.set_pos(len)?;
            let mut remaining = size - len;
            let chunk = usize::try_from(remaining).map_or(BOUNCE_MAX, |r| r.min(BOUNCE_MAX));
            let zeros = vec![IoBlock([0; 64]); chunk.div_ceil(size_of::<IoBlock>())];
            while remaining > 0 {
                let len = remaining.min(chunk as u64) as usize;
                match self.write_raw(zeros.as_ptr().cast(), len)? {
                    0 => {
                        return Err(io::const_error!(
                            io::ErrorKind::WriteZero,
                            "failed to extend file"
                        ));
                    }
                    n => remaining -= n as u64,
                }
            }
        }
        self.set_pos(pos.min(size))
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if is_io_aligned(buf.as_ptr(), buf.len()) {
            return self.read_raw(buf.as_mut_ptr(), buf.len());
        }
        let len = buf.len().min(BOUNCE_MAX);
        let mut bounce = bounce_buffer(len);
        let n = self.read_raw(bounce.as_mut_ptr().cast(), len)?;
        unsafe { ptr::copy_nonoverlapping(bounce.as_ptr().cast::<u8>(), buf.as_mut_ptr(), n) };
        Ok(n)
    }

    fn read_raw(&self, dest: *mut u8, len: usize) -> io::Result<usize> {
        let n = run(|c, b| unsafe {
            fs::read_file(c, b, dest.cast(), 1, len as u32, self.handle, 0, ErrorFlag::ALL)
        })?;
        Ok(n as usize)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_, u8>) -> io::Result<()> {
        crate::io::default_read_buf(|b| self.read(b), cursor)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if is_io_aligned(buf.as_ptr(), buf.len()) {
            return self.write_raw(buf.as_ptr(), buf.len());
        }
        let len = buf.len().min(BOUNCE_MAX);
        let mut bounce = bounce_buffer(len);
        unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), bounce.as_mut_ptr().cast::<u8>(), len) };
        self.write_raw(bounce.as_ptr().cast(), len)
    }

    fn write_raw(&self, source: *const u8, len: usize) -> io::Result<usize> {
        let n = run(|c, b| unsafe {
            fs::write_file(c, b, source.cast(), 1, len as u32, self.handle, 0, ErrorFlag::ALL)
        })?;
        Ok(n as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.write(b), bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let pos = match pos {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::Current(offset) => self.tell()?.checked_add_signed(offset),
            SeekFrom::End(offset) => self.file_attr()?.size().checked_add_signed(offset),
        }
        .ok_or(io::const_error!(io::ErrorKind::InvalidInput, "invalid seek position"))?;
        self.set_pos(pos)?;
        Ok(pos)
    }

    fn set_pos(&self, pos: u64) -> io::Result<()> {
        let pos = u32::try_from(pos)
            .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "position exceeds 4 GiB"))?;
        run(|c, b| unsafe { fs::set_pos_file(c, b, self.handle, pos, ErrorFlag::ALL) }).map(drop)
    }

    pub fn size(&self) -> Option<io::Result<u64>> {
        Some(self.file_attr().map(|attr| attr.size()))
    }

    pub fn tell(&self) -> io::Result<u64> {
        let mut pos = 0;
        run(|c, b| unsafe { fs::get_pos_file(c, b, self.handle, &mut pos, ErrorFlag::ALL) })?;
        Ok(pos.into())
    }

    pub fn duplicate(&self) -> io::Result<File> {
        unsupported()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported()
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        unsupported()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = call(|c, b| unsafe { fs::close_file(c, b, self.handle, ErrorFlag::ALL) });
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("handle", &self.handle).finish()
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, path: &Path) -> io::Result<()> {
        with_path(path, &|path| {
            run(|c, b| unsafe { fs::make_dir(c, b, path.as_ptr(), ErrorFlag::ALL) }).map(drop)
        })
    }
}

fn stat_cstr(path: &CStr) -> io::Result<FileAttr> {
    let mut stat = MaybeUninit::<Stat>::uninit();
    run(|c, b| unsafe {
        fs::get_stat(c, b, path.as_ptr(), stat.as_mut_ptr(), ErrorFlag::ALL)
    })?;
    Ok(FileAttr(unsafe { stat.assume_init() }))
}

fn remove_cstr(path: &CStr) -> io::Result<()> {
    run(|c, b| unsafe { fs::remove(c, b, path.as_ptr(), ErrorFlag::ALL) }).map(drop)
}

pub fn readdir(path: &Path) -> io::Result<ReadDir> {
    let handle = with_path(path, &|path| {
        let mut handle = DirHandle::INVALID;
        run(|c, b| unsafe { fs::open_dir(c, b, path.as_ptr(), &mut handle, ErrorFlag::ALL) })?;
        Ok(handle)
    })?;
    Ok(ReadDir { handle, root: Arc::from(path) })
}

pub fn unlink(path: &Path) -> io::Result<()> {
    with_path(path, &|path| {
        // FSRemove also removes empty directories.
        if stat_cstr(path)?.file_type().is_dir() {
            return Err(io::const_error!(io::ErrorKind::IsADirectory, "is a directory"));
        }
        remove_cstr(path)
    })
}

pub fn rmdir(path: &Path) -> io::Result<()> {
    with_path(path, &|path| {
        if !stat_cstr(path)?.file_type().is_dir() {
            return Err(io::const_error!(io::ErrorKind::NotADirectory, "not a directory"));
        }
        match remove_cstr(path) {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                Err(io::const_error!(io::ErrorKind::DirectoryNotEmpty, "directory not empty"))
            }
            result => result,
        }
    })
}

pub fn rename(old: &Path, new: &Path) -> io::Result<()> {
    with_path(old, &|old| {
        with_path(new, &|new| {
            run(|c, b| unsafe { fs::rename(c, b, old.as_ptr(), new.as_ptr(), ErrorFlag::ALL) })
                .map(drop)
        })
    })
}

pub fn set_perm(_path: &Path, _perm: FilePermissions) -> io::Result<()> {
    unsupported()
}

pub fn set_perm_nofollow(_path: &Path, _perm: FilePermissions) -> io::Result<()> {
    unsupported()
}

pub fn stat(path: &Path) -> io::Result<FileAttr> {
    with_path(path, &stat_cstr)
}

pub fn lstat(path: &Path) -> io::Result<FileAttr> {
    stat(path)
}

pub fn getcwd() -> io::Result<PathBuf> {
    // One spare byte, so the path is always nul-terminated.
    let mut buf = [0u8; MAX_PATH + 1];
    run(|c, b| unsafe {
        fs::get_cwd(c, b, buf.as_mut_ptr().cast(), MAX_PATH as u32, ErrorFlag::ALL)
    })?;
    let mut len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    if len > 1 && buf[len - 1] == b'/' {
        len -= 1;
    }
    Ok(PathBuf::from(OsString::from_inner(Buf { inner: buf[..len].to_vec() })))
}

pub fn chdir(path: &Path) -> io::Result<()> {
    with_path(path, &|path| {
        // FSChangeDir accepts any path, even missing ones and files.
        if !stat_cstr(path)?.file_type().is_dir() {
            return Err(io::const_error!(io::ErrorKind::NotADirectory, "not a directory"));
        }
        run(|c, b| unsafe { fs::change_dir(c, b, path.as_ptr(), ErrorFlag::ALL) }).map(drop)
    })
}

pub fn copy(from: &Path, to: &Path) -> io::Result<u64> {
    let mut reader = crate::fs::File::open(from)?;
    if !reader.metadata()?.is_file() {
        return Err(NOT_FILE_ERROR);
    }
    let mut writer = crate::fs::File::create(to)?;
    io::copy(&mut reader, &mut writer)
}
