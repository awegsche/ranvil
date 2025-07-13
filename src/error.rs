
#[derive(Debug)]
pub enum MCLoadError {
    IoError(std::io::Error),
    PathNotFoundError,
    IncompatibleCompressionType(u8),
}

impl From<std::io::Error> for MCLoadError {
    fn from(e: std::io::Error) -> Self {
        MCLoadError::IoError(e)
    }
}

impl std::fmt::Display for MCLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCLoadError::IoError(e) => write!(f, "IO error: {}", e),
            MCLoadError::PathNotFoundError => write!(f, "Path not found"),
            MCLoadError::IncompatibleCompressionType(t) => {
                write!(f, "Incompatible compression type {}", t)
            }
        }
    }
}

impl std::error::Error for MCLoadError {}

