#[derive(Debug)]
pub enum ErrorType {
    LibTskError,
    TskFsAttr,
    Generic
}
#[derive(Debug)]
pub struct TskError {
    pub message: String,
    pub kind: ErrorType,
}
impl TskError {
    /// Error function for TskFsAttr operations
    pub fn tsk_attr_error(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::TskFsAttr,
        }
    }

    /// A Generic error
    pub fn generic(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::Generic,
        }
    }

    /// Error originating form a lib tsk call
    pub fn lib_tsk_error(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::LibTskError,
        }
    }
}