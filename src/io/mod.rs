//! Reading and writing crates as directories and as zip archives.

use crate::{RoCrate, constants::METADATA_FILE_NAMES};
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[cfg(feature = "zip")]
mod archive;
#[cfg(feature = "zip")]
pub use archive::unzip;

pub type Result<T> = std::result::Result<T, Error>;

/// Metadata that is not JSON, or not a crate. Boxed inside [`Error`], since it
/// carries the whole document to point into it.
#[derive(Debug, Error, Diagnostic)]
#[error("`{}` is not valid RO-Crate metadata: {message}", path.display())]
#[diagnostic(code(rocrate::io::invalid_metadata))]
pub struct InvalidMetadata {
    pub path: PathBuf,
    pub message: String,
    #[source_code]
    document: NamedSource<String>,
    #[label("here")]
    span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("no RO-Crate metadata in `{}`", path.display())]
    #[diagnostic(
        code(rocrate::io::no_metadata),
        help("an RO-Crate keeps a `ro-crate-metadata.json` beside its payload")
    )]
    NoMetadata { path: PathBuf },

    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidMetadata(Box<InvalidMetadata>),

    #[error("`{part}` is part of the crate but missing from `{}`", directory.display())]
    #[diagnostic(
        code(rocrate::io::missing_part),
        help("check the crate's `hasPart` against what is on disk with `missing_parts`")
    )]
    MissingPart { part: String, directory: PathBuf },

    #[error(transparent)]
    #[diagnostic(code(rocrate::io::filesystem))]
    Filesystem(#[from] std::io::Error),

    #[cfg(feature = "zip")]
    #[error(transparent)]
    #[diagnostic(code(rocrate::io::archive))]
    Archive(#[from] ::zip::result::ZipError),
}

impl RoCrate {
    /// Reads the metadata file of a crate directory.
    ///
    /// # Errors
    /// Path does not exists or file is not readable
    pub fn from_directory(directory: impl AsRef<Path>) -> Result<Self> {
        let directory = directory.as_ref();
        let path = metadata_path(directory).ok_or_else(|| Error::NoMetadata {
            path: directory.to_path_buf(),
        })?;
        parse(&path, std::fs::read_to_string(&path)?)
    }

    /// Writes the metadata file into `directory`, creating it if needed, and
    /// returns the path written. Payload files are left alone.
    /// 
    /// # Errors
    /// Path is not writable
    pub fn write_directory(&self, directory: impl AsRef<Path>) -> Result<PathBuf> {
        let directory = directory.as_ref();
        std::fs::create_dir_all(directory)?;

        let path = directory.join(METADATA_FILE_NAMES[0]);
        std::fs::write(&path, serde_json::to_string_pretty(self).map_err(io_error)?)?;
        Ok(path)
    }

    /// The crate's own parts that are not in `directory`.
    pub fn missing_parts(&self, directory: impl AsRef<Path>) -> Vec<&str> {
        let directory = directory.as_ref();
        self.local_parts()
            .into_iter()
            .filter(|part| !directory.join(part).exists())
            .collect()
    }

    /// Data entities that live in the crate directory, rather than out on the
    /// web or inside the metadata itself.
    #[must_use]
    pub fn local_parts(&self) -> Vec<&str> {
        self.data_entities()
            .into_iter()
            .map(|entity| entity.id.as_str())
            .filter(|id| is_local(id))
            .collect()
    }
}

/// A part that a crate directory can hold: a relative path, not an IRI, not an
/// identifier internal to the metadata, and not an escape from the directory.
fn is_local(id: &str) -> bool {
    !id.starts_with('#')
        && !id.starts_with('/')
        && !id.contains("://")
        && !id.split(['/', '\\']).any(|segment| segment == "..")
        && !id.is_empty()
}

fn metadata_path(directory: &Path) -> Option<PathBuf> {
    METADATA_FILE_NAMES
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.is_file())
}

pub(crate) fn parse(path: &Path, source: String) -> Result<RoCrate> {
    match serde_json::from_str(&source) {
        Ok(crate_) => Ok(crate_),
        Err(error) => {
            let span = span_of(&source, &error);
            Err(Error::InvalidMetadata(Box::new(InvalidMetadata {
                path: path.to_path_buf(),
                message: error.to_string(),
                document: NamedSource::new(path.display().to_string(), source)
                    .with_language("json"),
                span,
            })))
        }
    }
}

/// Turns `serde_json`'s line and column into an offset miette can point at.
fn span_of(source: &str, error: &serde_json::Error) -> SourceSpan {
    let mut offset = 0;
    for (number, line) in source.lines().enumerate() {
        if number + 1 == error.line() {
            offset += error.column().saturating_sub(1);
            break;
        }
        offset += line.len() + 1;
    }
    SourceSpan::from(offset.min(source.len().saturating_sub(1))..offset.min(source.len()))
}

fn io_error(error: serde_json::Error) -> Error {
    Error::Filesystem(std::io::Error::other(error))
}
