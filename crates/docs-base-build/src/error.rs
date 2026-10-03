use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    Invalid(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Document {
        path: PathBuf,
        source: Box<Error>,
    },
    Highlight(syntect::Error),
    Format(fmt::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Json { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Document { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Highlight(source) => write!(formatter, "syntax highlighting failed: {source}"),
            Self::Format(source) => write!(formatter, "formatting documentation failed: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid(_) => None,
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            Self::Document { source, .. } => Some(source),
            Self::Highlight(source) => Some(source),
            Self::Format(source) => Some(source),
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Invalid(message.to_owned())
    }
}

impl From<syntect::Error> for Error {
    fn from(source: syntect::Error) -> Self {
        Self::Highlight(source)
    }
}

impl From<fmt::Error> for Error {
    fn from(source: fmt::Error) -> Self {
        Self::Format(source)
    }
}

pub(crate) fn read(path: &Path) -> crate::Result<String> {
    std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_owned(),
        source,
    })
}

pub(crate) fn write(path: &Path, contents: &str) -> crate::Result<()> {
    match std::fs::read_to_string(path) {
        Ok(previous) if previous == contents => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(Error::Io {
                path: path.to_owned(),
                source,
            });
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    std::fs::write(path, contents).map_err(|source| Error::Io {
        path: path.to_owned(),
        source,
    })
}
