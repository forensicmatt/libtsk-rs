use std::ptr::NonNull;
use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_img::TskImg,
    tsk_vs_part::{TskVsPart, TskVsPartIterator},
    bindings as tsk
};


/// Wrapper for TSK_VS_INFO 
#[derive(Debug)]
pub struct TskVs {
    /// The ptr to the TSK_VS_INFO struct
    pub handle: NonNull<tsk::TSK_VS_INFO>
}
impl TskVs {
    /// Create a TSK_VS_INFO wrapper given the TskImg and offset of the file system
    pub fn new(tsk_img: &TskImg, offset: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_VS_INFO sturct
        let tsk_vs = unsafe {tsk::tsk_vs_open(
            tsk_img.handle.as_ptr(),
            offset as _,
            0
        )};

        // Ensure that the ptr is not null
        let handle = match NonNull::new(tsk_vs) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                    .ok_or(
                        TskError::lib_tsk_error(
                            format!(
                                "There was an error opening the TSK_VS_INFO handle at offset {}. (no context)",
                                offset
                            )
                        )
                    )?;

                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!(
                        "There was an error opening the TSK_VS_INFO handle at offset {}: {}",
                        offset,
                        error_msg
                    )
                ));
            },
            Some(h) => h
        };

        Ok( Self { handle } )
    }

    /// Get a specific TskVsPart at the given index
    pub fn get_partition_at_index(&self, index: u64) -> Result<TskVsPart, TskError> {
        TskVsPart::new(self, index)
    }

    /// Get a partition iterator that yields TskVsPart structs
    pub fn get_partition_iter(&self) -> Result<TskVsPartIterator, TskError> {
        Ok(TskVsPart::new(self, 0)?.into_iter())
    }
}
impl Drop for TskVs {
    fn drop(&mut self) {
        unsafe { tsk::tsk_vs_close(self.handle.as_ptr()) };
    }
}