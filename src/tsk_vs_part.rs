use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_vs::TskVs,
    tsk_vs_part_handle::TskVsPartHandle,
    bindings as tsk
};


/// Wrapper for TSK_VS_PART_INFO.
/// The TskVs reference must live for the lifetime of
/// *const tsk::TSK_VS_PART_INFO.
pub struct TskVsPart<'vs> {
    tsk_vs: &'vs TskVs,
    tsk_part_info: *const tsk::TSK_VS_PART_INFO
}
impl<'vs> TskVsPart<'vs> {
    /// Create a TSK_VS_PART_INFO wrapper given the TskImg and offset of the file system
    pub fn new(tsk_vs: &'vs TskVs, offset: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_VS_PART_INFO sturct
        // TODO: HANDLE NULL!
        let tsk_vs_part = unsafe {tsk::tsk_vs_part_get(
            tsk_vs.handle.as_ptr(),
            offset as _
        )};

        Ok( Self{
            tsk_vs: tsk_vs,
            tsk_part_info: tsk_vs_part
        })
    }

    /// Get the len in blocks of this partition
    pub fn len(&self) -> u64 {
        unsafe {*self.tsk_part_info}.len
    }

    /// Get the byte size of the volume
    pub fn size(&self) -> u64 {
        unsafe {*self.tsk_part_info}.len *
        unsafe {(*(*self.tsk_part_info).vs).block_size} as u64
    }

    /// Get a IO handle to the partition
    pub fn get_handle<'p>(&'p self) -> TskVsPartHandle<'vs, 'p> {
        TskVsPartHandle::new(&self)
    }

    /// Get the description string
    pub fn desc(&self) -> String {
        let desc = unsafe { CStr::from_ptr((*self.tsk_part_info).desc) }.to_string_lossy();
        desc.to_string().clone()
    }

    /// Get an iterator based off this TskVsPart struct
    pub fn into_iter<'p>(&'vs mut self) -> TskVsPartIterator<'p, 'vs> {
        TskVsPartIterator(self)
    }
}
impl<'vs> std::fmt::Debug for TskVsPart<'vs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskVsPart")
         .field("addr", &(unsafe{*self.tsk_part_info}.addr))
         .field("desc", &self.desc())
         .field("flags", &(unsafe{*self.tsk_part_info}.flags))
         .field("len", &(unsafe{*self.tsk_part_info}.len))
         .field("slot_num", &(unsafe{*self.tsk_part_info}.slot_num))
         .field("start", &(unsafe{*self.tsk_part_info}.	start))
         .field("table_num", &(unsafe{*self.tsk_part_info}.	table_num))
         .field("tag", &(unsafe{*self.tsk_part_info}.tag))
         .finish()
    }
}


/// An iterator over a TSK_VS_PART_INFO pointer which uses the
/// structs next attribute to iterate.
pub struct TskVsPartIterator<'p, 'vs>(&'p mut TskVsPart<'vs>);
impl<'p, 'vs> Iterator for TskVsPartIterator<'p, 'vs> {
    type Item = &'p mut TskVsPart<'vs>;
    
    fn next(&mut self) -> Option<&'p mut TskVsPart<'vs>> {
        if self.0.tsk_part_info.is_null() {
            return None;
        }

        let next = unsafe {
            *self.0.tsk_part_info
        }.next as *const tsk::TSK_VS_PART_INFO;

        self.0.tsk_part_info = next;
        
        Some(self.0)
    }
}
