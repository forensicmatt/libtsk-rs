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

    /// Get an iterator based off this TskVsPart struct
    pub fn into_iter(self) -> TskVsPartIterator {
        TskVsPartIterator(self.0)
    }
}
impl std::fmt::Debug for TskVsPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TskVsPart")
         .field(&(unsafe{*self.0}))
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
