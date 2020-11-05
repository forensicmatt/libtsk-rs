use crate::{
    errors::TskError,
    tsk_vs::TskVs,
    bindings as tsk
};


/// Wrapper for TSK_VS_PART_INFO 
#[derive(Debug)]
pub struct TskVsPart(tsk::TSK_VS_PART_INFO);
impl TskVsPart {
    /// Create a TSK_VS_PART_INFO wrapper given the TskImg and offset of the file system
    pub fn new(tsk_vs: &TskVs, offset: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_VS_PART_INFO sturct
        let tsk_vs_part = unsafe {*tsk::tsk_vs_part_get(
            tsk_vs.handle.as_ptr(),
            offset as _
        )};

        Ok(Self(tsk_vs_part))
    }
}
