use std::fmt;

#[derive(Debug)]
pub enum GameError {
    Persistence(PersistenceError),
    // 必要に応じて他のエラーカテゴリを追加
}

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Decompression(std::io::Error), // zstd のエラーも含む
    InvalidFormat,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Persistence(e) => write!(f, "Persistence error: {e}"),
        }
    }
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Serialization(e) => write!(f, "Serialization error: {e}"),
            Self::Decompression(e) => write!(f, "Decompression error: {e}"),
            Self::InvalidFormat => write!(f, "Invalid save file format"),
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e)
    }
}

impl From<PersistenceError> for GameError {
    fn from(e: PersistenceError) -> Self {
        Self::Persistence(e)
    }
}
