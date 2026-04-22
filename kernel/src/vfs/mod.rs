//! Simple in-memory virtual file system.
//!
//! All storage uses fixed-size arrays so the kernel can operate without a heap
//! allocator.  The global [`VFS`] static is the single file-system instance.
//! Call [`init`] once at boot to set up the root directory.

use spin::Mutex;

// ── Limits ────────────────────────────────────────────────────────────────────

/// Total number of inodes (files + directories) available.
pub const MAX_INODES: usize = 64;
/// Maximum length of a single file or directory name (bytes).
pub const MAX_NAME_LEN: usize = 64;
/// Maximum number of direct children a directory can hold.
pub const MAX_CHILDREN: usize = 16;
/// Maximum number of bytes that can be stored in a single file.
pub const MAX_FILE_CONTENT: usize = 2048;
/// Maximum length of the string returned by [`Vfs::pwd`].
pub const MAX_PATH_LEN: usize = 512;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    DirectoryFull,
    FilesystemFull,
    NameTooLong,
    FileTooLarge,
}

impl VfsError {
    pub fn as_str(self) -> &'static str {
        match self {
            VfsError::NotFound => "no such file or directory",
            VfsError::NotADirectory => "not a directory",
            VfsError::NotAFile => "not a file",
            VfsError::AlreadyExists => "file already exists",
            VfsError::DirectoryFull => "directory is full",
            VfsError::FilesystemFull => "no space left on device",
            VfsError::NameTooLong => "file name too long",
            VfsError::FileTooLarge => "file too large",
        }
    }
}

// ── Inode ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InodeKind {
    Free,
    File,
    Directory,
}

#[derive(Clone, Copy)]
pub struct Inode {
    pub kind: InodeKind,
    /// Name of this entry (not the full path).
    pub name: [u8; MAX_NAME_LEN],
    pub name_len: usize,
    /// Index of the parent inode.  Root points to itself (index 0).
    pub parent: usize,
    /// Permission bits: 0o644 for files, 0o755 for directories.
    pub permissions: u16,
    // ── File-specific ─────────────────────────────────────────────────────────
    pub content: [u8; MAX_FILE_CONTENT],
    pub content_len: usize,
    // ── Directory-specific ────────────────────────────────────────────────────
    pub children: [usize; MAX_CHILDREN],
    pub child_count: usize,
    // ── Metadata ──────────────────────────────────────────────────────────────
    /// Timer ticks at creation time.
    pub created_ticks: usize,
    /// Timer ticks at last modification.
    pub modified_ticks: usize,
}

impl Inode {
    /// A zeroed-out free-slot sentinel, usable in `const` contexts.
    pub const FREE: Self = Self {
        kind: InodeKind::Free,
        name: [0; MAX_NAME_LEN],
        name_len: 0,
        parent: 0,
        permissions: 0,
        content: [0; MAX_FILE_CONTENT],
        content_len: 0,
        children: [0; MAX_CHILDREN],
        child_count: 0,
        created_ticks: 0,
        modified_ticks: 0,
    };

    /// Return the name as a `&str` slice (best-effort; falls back to `"<?>"`).
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("<?>")
    }

    /// Return the size that `ls` should display: file content length or
    /// directory child count.
    pub fn display_size(&self) -> usize {
        match self.kind {
            InodeKind::File => self.content_len,
            InodeKind::Directory => self.child_count,
            InodeKind::Free => 0,
        }
    }
}

// ── Vfs ───────────────────────────────────────────────────────────────────────

pub struct Vfs {
    pub inodes: [Inode; MAX_INODES],
    /// Inode index of the current working directory.
    cwd: usize,
}

impl Vfs {
    const fn new_const() -> Self {
        Self {
            inodes: [Inode::FREE; MAX_INODES],
            cwd: 0,
        }
    }

    /// Set up the root directory (inode 0).  Called once at boot.
    pub fn init_root(&mut self) {
        let now = crate::time::get_ticks();
        let root = &mut self.inodes[0];
        root.kind = InodeKind::Directory;
        root.name[0] = b'/';
        root.name_len = 1;
        root.parent = 0; // root is its own parent
        root.permissions = 0o755;
        root.created_ticks = now;
        root.modified_ticks = now;
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Scan for the next free inode slot (starting from index 1; 0 = root).
    fn alloc_inode(&mut self) -> Option<usize> {
        for i in 1..MAX_INODES {
            if self.inodes[i].kind == InodeKind::Free {
                return Some(i);
            }
        }
        None
    }

    /// Find the inode index of a direct child of `dir_idx` named `name`.
    fn find_child(&self, dir_idx: usize, name: &str) -> Option<usize> {
        let dir = &self.inodes[dir_idx];
        for i in 0..dir.child_count {
            let child_idx = dir.children[i];
            if self.inodes[child_idx].name_str() == name {
                return Some(child_idx);
            }
        }
        None
    }

    /// Append `child_idx` to the children list of `dir_idx`.
    fn add_child(&mut self, dir_idx: usize, child_idx: usize) -> Result<(), VfsError> {
        if self.inodes[dir_idx].child_count >= MAX_CHILDREN {
            return Err(VfsError::DirectoryFull);
        }
        let count = self.inodes[dir_idx].child_count;
        self.inodes[dir_idx].children[count] = child_idx;
        self.inodes[dir_idx].child_count += 1;
        self.inodes[dir_idx].modified_ticks = crate::time::get_ticks();
        Ok(())
    }

    // ── Path resolution ───────────────────────────────────────────────────────

    /// Resolve `path` to an inode index.
    ///
    /// Absolute paths (starting with `/`) resolve from the root.
    /// Relative paths resolve from the current working directory.
    /// Both `.` and `..` are handled.
    pub fn resolve_path(&self, path: &str) -> Result<usize, VfsError> {
        if path == "/" {
            return Ok(0);
        }

        let mut current = if path.starts_with('/') { 0 } else { self.cwd };

        for component in path.split('/') {
            match component {
                "" | "." => continue,
                ".." => {
                    // Stay at root if already there (root's parent is itself).
                    let parent = self.inodes[current].parent;
                    current = parent;
                }
                name => {
                    if self.inodes[current].kind != InodeKind::Directory {
                        return Err(VfsError::NotADirectory);
                    }
                    current = self.find_child(current, name).ok_or(VfsError::NotFound)?;
                }
            }
        }

        Ok(current)
    }

    /// Resolve the parent directory of `path` and return its inode index
    /// together with the final name component.
    fn resolve_parent<'a>(&self, path: &'a str) -> Result<(usize, &'a str), VfsError> {
        let (dir_part, name) = match path.rfind('/') {
            Some(pos) => {
                let d = &path[..pos];
                let n = &path[pos + 1..];
                (if d.is_empty() { "/" } else { d }, n)
            }
            None => (".", path),
        };

        if name.is_empty() {
            return Err(VfsError::NotFound);
        }
        if name.len() > MAX_NAME_LEN {
            return Err(VfsError::NameTooLong);
        }

        let parent_idx = self.resolve_path(dir_part)?;
        if self.inodes[parent_idx].kind != InodeKind::Directory {
            return Err(VfsError::NotADirectory);
        }

        Ok((parent_idx, name))
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Create an empty file at `path`.
    ///
    /// If the file already exists the modification timestamp is updated
    /// (POSIX `touch` semantics) and `Ok(())` is returned.
    pub fn touch(&mut self, path: &str) -> Result<(), VfsError> {
        let (parent_idx, name) = self.resolve_parent(path)?;

        // Already exists – bump mtime.
        if let Some(idx) = self.find_child(parent_idx, name) {
            self.inodes[idx].modified_ticks = crate::time::get_ticks();
            return Ok(());
        }

        let child_idx = self.alloc_inode().ok_or(VfsError::FilesystemFull)?;
        let now = crate::time::get_ticks();
        {
            let inode = &mut self.inodes[child_idx];
            inode.kind = InodeKind::File;
            inode.name_len = name.len();
            inode.name[..name.len()].copy_from_slice(name.as_bytes());
            inode.parent = parent_idx;
            inode.permissions = 0o644;
            inode.content_len = 0;
            inode.created_ticks = now;
            inode.modified_ticks = now;
        }
        self.add_child(parent_idx, child_idx)
    }

    /// Create a new directory at `path`.
    ///
    /// Returns [`VfsError::AlreadyExists`] if the path already exists.
    pub fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
        let (parent_idx, name) = self.resolve_parent(path)?;

        if self.find_child(parent_idx, name).is_some() {
            return Err(VfsError::AlreadyExists);
        }

        let child_idx = self.alloc_inode().ok_or(VfsError::FilesystemFull)?;
        let now = crate::time::get_ticks();
        {
            let inode = &mut self.inodes[child_idx];
            inode.kind = InodeKind::Directory;
            inode.name_len = name.len();
            inode.name[..name.len()].copy_from_slice(name.as_bytes());
            inode.parent = parent_idx;
            inode.permissions = 0o755;
            inode.child_count = 0;
            inode.created_ticks = now;
            inode.modified_ticks = now;
        }
        self.add_child(parent_idx, child_idx)
    }

    /// Change the current working directory to `path`.
    pub fn cd(&mut self, path: &str) -> Result<(), VfsError> {
        let idx = self.resolve_path(path)?;
        if self.inodes[idx].kind != InodeKind::Directory {
            return Err(VfsError::NotADirectory);
        }
        self.cwd = idx;
        Ok(())
    }

    /// Write `data` as the entire content of the file at `path`,
    /// creating the file if it does not yet exist.
    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        if data.len() > MAX_FILE_CONTENT {
            return Err(VfsError::FileTooLarge);
        }

        let idx = match self.resolve_path(path) {
            Ok(i) => {
                if self.inodes[i].kind != InodeKind::File {
                    return Err(VfsError::NotAFile);
                }
                i
            }
            Err(VfsError::NotFound) => {
                self.touch(path)?;
                self.resolve_path(path).map_err(|_| VfsError::NotFound)?
            }
            Err(e) => return Err(e),
        };

        let now = crate::time::get_ticks();
        let inode = &mut self.inodes[idx];
        inode.content[..data.len()].copy_from_slice(data);
        inode.content_len = data.len();
        inode.modified_ticks = now;
        Ok(())
    }

    /// Read file content at `path`, returning a slice into the inode's buffer.
    pub fn read_file(&self, path: &str) -> Result<&[u8], VfsError> {
        let idx = self.resolve_path(path)?;
        let inode = &self.inodes[idx];
        if inode.kind != InodeKind::File {
            return Err(VfsError::NotAFile);
        }
        Ok(&inode.content[..inode.content_len])
    }

    /// Return the current working directory as a path string.
    ///
    /// The path is written into `buf`; the number of bytes written is returned.
    pub fn pwd(&self) -> ([u8; MAX_PATH_LEN], usize) {
        let mut buf = [0u8; MAX_PATH_LEN];

        if self.cwd == 0 {
            buf[0] = b'/';
            return (buf, 1);
        }

        // Collect ancestors from cwd up to (but not including) root.
        let mut stack = [0usize; 32];
        let mut depth = 0usize;
        let mut idx = self.cwd;

        loop {
            if depth >= 32 {
                break;
            }
            stack[depth] = idx;
            depth += 1;
            let parent = self.inodes[idx].parent;
            if parent == idx || parent == 0 {
                // parent == 0 means the next step would be root; stop here.
                break;
            }
            idx = parent;
        }

        // Build "/comp1/comp2/..." by reversing the stack.
        let mut pos = 0usize;
        for i in (0..depth).rev() {
            let inode = &self.inodes[stack[i]];
            if pos < MAX_PATH_LEN {
                buf[pos] = b'/';
                pos += 1;
            }
            let copy_len = core::cmp::min(inode.name_len, MAX_PATH_LEN.saturating_sub(pos));
            buf[pos..pos + copy_len].copy_from_slice(&inode.name[..copy_len]);
            pos += copy_len;
        }

        (buf, pos)
    }

    /// Return the inode index of the current working directory.
    #[allow(dead_code)]
    pub fn cwd(&self) -> usize {
        self.cwd
    }
}

// ── Global instance ───────────────────────────────────────────────────────────

/// The single global VFS instance, protected by a spin-lock.
pub static VFS: Mutex<Vfs> = Mutex::new(Vfs::new_const());

/// Initialise the root directory.  Must be called once at kernel boot.
pub fn init() {
    VFS.lock().init_root();
}
