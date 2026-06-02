use std::fmt;

#[derive(Debug)]
pub enum SaveEditorError {
    Io(String),
    Xml(String),
    NotFound(String),
    InvalidStructure(String),
    BackupFailed(String),
    WriteFailed(String),
}

impl fmt::Display for SaveEditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(m) => write!(f, "IO error: {}", m),
            Self::Xml(m) => write!(f, "XML parse error: {}", m),
            Self::NotFound(m) => write!(f, "Not found: {}", m),
            Self::InvalidStructure(m) => write!(f, "Invalid save structure: {}", m),
            Self::BackupFailed(m) => write!(f, "Backup failed: {}", m),
            Self::WriteFailed(m) => write!(f, "Write failed: {}", m),
        }
    }
}

impl std::error::Error for SaveEditorError {}

impl From<std::io::Error> for SaveEditorError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<quick_xml::Error> for SaveEditorError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<SaveEditorError> for String {
    fn from(e: SaveEditorError) -> Self {
        e.to_string()
    }
}

pub type Result<T> = std::result::Result<T, SaveEditorError>;
