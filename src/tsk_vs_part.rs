use std::ffi::{CStr};
use crate::{
    errors::TskError,
    tsk_vs::TskVs,
    bindings as tsk
};


/// Wrapper for TSK_VS_PART_INFO 
pub struct TskVsPart(*const tsk::TSK_VS_PART_INFO);
impl TskVsPart {
    /// Create a TSK_VS_PART_INFO wrapper given the TskImg and offset of the file system
    pub fn new(tsk_vs: &TskVs, offset: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_VS_PART_INFO sturct
        let tsk_vs_part = unsafe {tsk::tsk_vs_part_get(
            tsk_vs.handle.as_ptr(),
            offset as _
        )};

        Ok(Self(tsk_vs_part))
    }

    /// Get the description string
    pub fn desc(&self) -> String {
        let desc = unsafe { CStr::from_ptr((*self.0).desc) }.to_string_lossy();
        desc.to_string().clone()
    }

    /// Get an iterator based off this TskVsPart struct
    pub fn into_iter(self) -> TskVsPartIterator {
        TskVsPartIterator(self.0)
    }
}
impl std::fmt::Debug for TskVsPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskVsPart")
         .field("addr", &(unsafe{*self.0}.addr))
         .field("desc", &self.desc())
         .field("flags", &(unsafe{*self.0}.flags))
         .field("len", &(unsafe{*self.0}.len))
         .field("slot_num", &(unsafe{*self.0}.slot_num))
         .field("start", &(unsafe{*self.0}.	start))
         .field("table_num", &(unsafe{*self.0}.	table_num))
         .field("tag", &(unsafe{*self.0}.tag))
         .finish()
    }
}


/// An iterator over a TSK_VS_PART_INFO pointer which uses the
/// structs next attribute to iterate.
pub struct TskVsPartIterator(*const tsk::TSK_VS_PART_INFO);
impl Iterator for TskVsPartIterator {
    type Item = TskVsPart;
    
    fn next(&mut self) -> Option<TskVsPart> {
        if self.0.is_null() {
            return None;
        }

        let next = unsafe { *self.0 }.next as *const tsk::TSK_VS_PART_INFO;
        let current = TskVsPart(self.0);
        self.0 = next;
        
        Some(current)
    }
}
