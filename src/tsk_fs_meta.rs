use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_dir::TskFsDir,
    bindings as tsk
};
use std::fmt::{Display, Formatter, Result as FmtReasult};
use std::convert::From;

#[derive(Debug)]
pub enum TskFsMetaFlag {
    Allocated = 1, 	    //Metadata structure is currently in an allocated state.
    Unallocated = 2, 	//Metadata structure is currently in an unallocated state.
    Used = 4, 	        //Metadata structure has been allocated at least once.
    Unused = 8, 	    //Metadata structure has never been allocated.
    Compressed = 16,    //The file contents are compressed.
    Orphan = 32, 	    //Return only metadata structures that have no file name pointing to the (inode_walk flag only)
    None
}

impl Display for TskFsMetaFlag{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtReasult {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum TskFsMetaType {
    None,
    Regular, 	        // Regular file.
    Directory, 	        // Directory file.
    NamedPipe, 	        // Named pipe (fifo)
    CharacterDevice, 	// Character device.
    BlockDevice, 	    // Block device.
    SymbolicLink, 	    // Symbolic link.
    SHAD, 	            // SOLARIS ONLY.
    UnixSocket, 	    // UNIX domain socket.
    Without, 	        // Whiteout.
    VirtualFile, 	    // "Virtual File" created by TSK for file system areas
    VirtualDirectory 	// "Virtual Directory" created by TSK to hold data like orphan files
}

impl Display for TskFsMetaType{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtReasult {
        write!(f, "{:?}", self)
    }
}

impl From<u32> for TskFsMetaType {
    fn from(type_num: u32) -> Self {
        match type_num {
            1 => TskFsMetaType::Regular,
            2 => TskFsMetaType::Directory,
            3 => TskFsMetaType::NamedPipe,
            4 => TskFsMetaType::CharacterDevice,
            5 => TskFsMetaType::BlockDevice,
            6 => TskFsMetaType::SymbolicLink,
            7 => TskFsMetaType::SHAD,
            8 => TskFsMetaType::UnixSocket,
            9 => TskFsMetaType::Without,
            10 => TskFsMetaType::VirtualFile,
            11 => TskFsMetaType::VirtualDirectory,
            _ => TskFsMetaType::None
        }
    }
}

/// Wrapper for TSK_FS_META
pub struct TskFsMeta(*const tsk::TSK_FS_META);
impl TskFsMeta {
    pub fn from_ptr(TSK_FS_META: *const tsk::TSK_FS_META) -> Result<Self, TskError> {
        if TSK_FS_META.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::tsk_fs_meta_error(
                format!("Error TSK_FS_META is null: {}", error_msg)
            ));
        }

        Ok(Self(TSK_FS_META))
    }

    /// Get the size of the file
    pub fn size(&self) -> i64 {
        unsafe { (*self.0).size }
    }

    /// Get the creation time of the file (epoch time)
    pub fn crtime(&self) -> i64 {
        unsafe { (*self.0).crtime }
    }

    /// Get the modification time of the file (epoch time)
    pub fn mtime(&self) -> i64 {
        unsafe { (*self.0).mtime }
    }

    /// Get the access time of the file (epoch time)
    pub fn atime(&self) -> i64 {
        unsafe { (*self.0).atime }
    }

    /// Get the inode
    pub fn addr(&self) -> u64 {
        unsafe { (*self.0).addr }
    }

    /// Get type
    pub fn type_(&self) -> TskFsMetaType {
        TskFsMetaType::from(unsafe { (*self.0).type_ as u32 })
    }

    /// Get flags
    pub fn flags(&self) -> Vec<TskFsMetaFlag> {
        let mut flags = vec![];
        match unsafe { (*self.0).type_ as u32 } {
            f if f & TskFsMetaFlag::Allocated as u32 > 0 => flags.push(TskFsMetaFlag::Allocated),
            f if f & TskFsMetaFlag::Unallocated as u32 > 0 => flags.push(TskFsMetaFlag::Unallocated),
            f if f & TskFsMetaFlag::Used as u32 > 0 => flags.push(TskFsMetaFlag::Used),
            f if f & TskFsMetaFlag::Unused as u32 > 0 => flags.push(TskFsMetaFlag::Unused),
            f if f & TskFsMetaFlag::Orphan as u32 > 0 => flags.push(TskFsMetaFlag::Orphan),
            _ => flags.push(TskFsMetaFlag::None)
        }
        flags
    }

    pub fn is_unallocated(&self) -> bool {
        self.flags().iter().any(|f| match f {
            TskFsMetaFlag::Unallocated => true,
            _ => false
        })
    }

}

impl std::fmt::Debug for TskFsMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsMeta")
            .field("size", &self.size())
            .field("crtime", &self.crtime())
            .field("mtime", &self.mtime())
            .field("atime", &self.atime())
            .field("addr", &self.addr())
            .field("type", &self.type_())
            .field("flags", &self.flags())
            .finish()
    }
}