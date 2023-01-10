use std::ffi::CStr;
use std::ptr::NonNull;
use crate::{
    errors::TskError,
    tsk_img::TskImg,
    tsk_fs_file::TskFsFile,
    tsk_fs_name::TskFsName,
    tsk_fs_dir::{TskFsDir, IntoDirNameIter},
    bindings as tsk
};


/// Wrapper for TSK_FS_INFO
pub struct TskFs {
    /// The ptr to the TSK_FS_INFO struct
    tsk_fs_ptr: *mut tsk::TSK_FS_INFO,
    /// We dont always want to free a file pointer
    _release: bool,
}
impl TskFs {
    /// Create a TSK_FS_INFO wrapper given the TskImg and offset of the file system
    pub fn from_fs_offset(tsk_img: &TskImg, offset: u64) -> Result<TskFs, TskError> {
        // Get a pointer to the TSK_FS_INFO sturct
        let tsk_fs_ptr = unsafe {tsk::tsk_fs_open_img(
            tsk_img.handle.as_ptr(),
            offset as i64 as _,
            0
        )};

        if tsk_fs_ptr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                .ok_or(
                    TskError::lib_tsk_error(
                        format!("There was an error opening the fs handle at offset {}. (no context)", offset)
                    )
                )?;
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::lib_tsk_error(
                format!("There was an error opening the fs handle at offset {}: {}", offset, error_msg)
            ));
        }

        Ok( Self { tsk_fs_ptr, _release: true } )
    }

    /// Open a file by a given path. (use '/' as separators)
    pub fn file_open(&self, path: &str) -> Result<TskFsFile, TskError> {
        TskFsFile::from_path(&self, path)
    }

    /// Open a file by a given inode.
    pub fn file_open_meta(&self, inode: u64) -> Result<TskFsFile, TskError> {
        TskFsFile::from_meta(&self, inode)
    }

    /// Open a directory by a given path
    pub fn dir_open(&self, path: &str) -> Result<TskFsDir, TskError> {
        TskFsDir::from_path(self, path)
    }

    /// Open a file name iterator based on root inode
    pub fn iter_file_names<'fs>(&'fs self) -> Result<FsNameIter<'fs>, TskError> {
        let root_inode = unsafe {(*self.tsk_fs_ptr).root_inum};
        let root_dir = TskFsDir::from_meta(self, root_inode)?;

        Ok( FsNameIter {
            tsk_fs: self,
            dir_iter_stack: vec![root_dir.into_name_iter()],
            path_stack: Vec::new()
        } )
    }

    /// Open a file name iterator based on path
    pub fn iter_file_names_from_inode<'fs>(&'fs self, inode: u64) -> Result<FsNameIter<'fs>, TskError> {
        FsNameIter::from_inode(self, inode)
    }

    /// Open a file name iterator based on path
    pub fn iter_file_names_from_path<'fs>(&'fs self, path: &str) -> Result<FsNameIter<'fs>, TskError> {
        FsNameIter::from_path(self, path)
    }

    /// Number of blocks in fs.
    pub fn block_count(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).block_count }
    }

    /// Number of bytes that precede each block (currently only used for RAW CDs) 
    pub fn block_pre_size(&self) -> u32 {
        unsafe { (*self.tsk_fs_ptr).block_pre_size }
    }

    /// Number of bytes that follow each block (currently only used for RAW CDs) 
    pub fn block_post_size(&self) -> u32 {
        unsafe { (*self.tsk_fs_ptr).block_post_size }
    }

    /// Size of each block (in bytes) 
    pub fn block_size(&self) -> u32 {
        unsafe { (*self.tsk_fs_ptr).block_size }
    }

    /// Size of device block (typically always 512) 
    pub fn dev_bsize(&self) -> u32 {
        unsafe { (*self.tsk_fs_ptr).dev_bsize }
    }

    /// Address of first block. 
    pub fn first_block(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).first_block }
    }

    /// First valid metadata address.  
    pub fn first_inum(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).first_inum }
    }

    /// Number of metadata addresses. 
    pub fn inum_count(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).inum_count }
    }

    /// Address of journal inode. 
    pub fn journ_inum(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).journ_inum }
    }

    /// Address of last block as reported by file system (could be larger than 
    /// last_block in image if end of image does not exist) 
    pub fn last_block(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).last_block }
    }

    /// Address of last block – adjusted so that it is equal to the last block 
    /// in the image or volume (if image is not complete) 
    pub fn last_block_act(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).last_block_act }
    }

    /// Last valid metadata address. 
    pub fn last_inum(&self) -> u64 {
        unsafe { (*self.tsk_fs_ptr).last_inum }
    }
}
impl Into<*mut tsk::TSK_FS_INFO> for &TskFs {
    fn into(self) -> *mut tsk::TSK_FS_INFO {
        self.tsk_fs_ptr
    }
}
impl Into<*mut tsk::TSK_FS_INFO> for TskFs {
    fn into(self) -> *mut tsk::TSK_FS_INFO {
        self.tsk_fs_ptr
    }
}
impl<'fs> Into<&'fs *mut tsk::TSK_FS_INFO> for &'fs TskFs {
    fn into(self) -> &'fs *mut tsk::TSK_FS_INFO {
        &self.tsk_fs_ptr
    }
}
impl Drop for TskFs {
    fn drop(&mut self) {
        if self._release {
            unsafe { tsk::tsk_fs_close(self.tsk_fs_ptr) };
        }
    }
}
impl std::fmt::Debug for TskFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFs")
         .field("block_count", &self.block_count())
         .field("block_pre_size", &self.block_pre_size())
         .field("block_post_size", &self.block_post_size())
         .field("dev_bsize", &self.dev_bsize())
         .field("first_block", &self.first_block())
         .field("first_inum", &self.first_inum())
         .field("inum_count", &self.inum_count())
         .field("journ_inum", &self.journ_inum())
         .field("last_block", &self.last_block())
         .field("last_block_act", &self.last_block_act())
         .field("last_inum", &self.last_inum())
         .finish()
    }
}


#[derive(Debug)]
pub struct FsNameIter<'fs> {
    tsk_fs: &'fs TskFs,
    dir_iter_stack: Vec<IntoDirNameIter<'fs>>,
    path_stack: Vec<String>
}
impl<'fs> FsNameIter<'fs> {
    pub fn from_path(
        tsk_fs: &'fs TskFs,
        path: &str
    ) -> Result<FsNameIter<'fs>, TskError> {
        let dir = TskFsDir::from_path(tsk_fs, path)?;

        Ok( FsNameIter {
            tsk_fs,
            dir_iter_stack: vec![dir.into_name_iter()],
            path_stack: Vec::new()
        } )
    }

    pub fn from_inode(
        tsk_fs: &'fs TskFs,
        inode: u64
    ) -> Result<FsNameIter<'fs>, TskError> {
        let dir = TskFsDir::from_meta(tsk_fs, inode)?;

        Ok( FsNameIter {
            tsk_fs,
            dir_iter_stack: vec![dir.into_name_iter()],
            path_stack: Vec::new()
        } )
    }
}
impl<'fs> Iterator for FsNameIter<'fs> {
    type Item = (String, TskFsName);

    fn next(&mut self) -> Option<(String, TskFsName)> {
        let fs = self.tsk_fs;

        loop {
            if let Some(current_dir_itr) = self.dir_iter_stack.last_mut() {
                if let Some(tsk_fn) = current_dir_itr.next() {
                    if tsk_fn.is_dir() {
                        let file_name = match tsk_fn.name() {
                            Some(n) => n,
                            None => break
                        };

                        if &file_name == "." || &file_name == ".." {
                            continue;
                        }

                        let tsk_fs_dir = match TskFsDir::from_meta(fs, tsk_fn.get_inode()) {
                            Ok(d) => d,
                            Err(_e) => continue
                        };

                        let new_dir_iter = tsk_fs_dir.into_name_iter();
                        self.dir_iter_stack.push(new_dir_iter);

                        let path = self.path_stack.join("/");
                        self.path_stack.push(file_name);
                        return Some((path, tsk_fn))
                    } else {
                        let path = self.path_stack.join("/");
                        return Some((path, tsk_fn))
                    }
                } else {
                    self.dir_iter_stack.pop();
                    self.path_stack.pop();
                }
            } else {
                self.dir_iter_stack.pop();

                // No more dir iterators in stack
                if self.dir_iter_stack.is_empty(){
                    break;
                }
            }
        }

        None
    }
}