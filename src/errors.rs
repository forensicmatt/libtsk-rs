#[derive(Debug)]
pub enum ErrorType {
    TskFsMeta,
    TskFsFile,
    LibTskError,
    TskFsAttr,
    TskFsName,
    TskFsDir,
    Generic
}
#[derive(Debug)]
pub struct TskError {
    pub message: String,
    pub kind: ErrorType,
}
impl TskError {
    /// Error function for TskFsDir operations
    pub fn tsk_fs_dir_error(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::TskFsDir,
        }
    }

    /// Error function for TskFsName operations
    pub fn tsk_fs_name_error(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::TskFsName,
        }
    }

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

    /// Error function for TskFsMeta operations
    pub fn tsk_fs_meta_error(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::TskFsMeta,
        }
    }

    /// Error function for TskFsFile operations
    pub fn tsk_fs_file_error(message: String) -> Self {
        Self {
            message: message,
            kind: ErrorType::TskFsFile,
        }
    }
}