use crate::{
    RoCrate,
    constants::METADATA_FILE_NAMES,
    io::{Error, Result, parse},
};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

impl RoCrate {
    /// Reads the metadata out of a zipped crate, without unpacking it.
    ///
    /// Archives that wrap the crate in a top-level folder are read too: the
    /// shallowest metadata file wins.
    ///
    /// # Errors
    /// File does not exist
    /// or
    /// File is not readable
    pub fn from_zip(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut archive = ZipArchive::new(File::open(path)?)?;

        let Some(index) = metadata_entry(&mut archive) else {
            return Err(Error::NoMetadata {
                path: path.to_path_buf(),
            });
        };

        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        let mut source = String::new();
        entry.read_to_string(&mut source)?;

        parse(&path.join(name), source)
    }

    /// Packs the crate: the metadata file, plus every part of the crate read
    /// from `source`.
    ///
    /// # Errors
    /// Fails on a part that `source` does not hold — check first with
    /// [`RoCrate::missing_parts`] if a crate may be incomplete.
    pub fn write_zip(&self, path: impl AsRef<Path>, source: impl AsRef<Path>) -> Result<()> {
        let source = source.as_ref();
        let mut writer = ZipWriter::new(File::create(path.as_ref())?);
        let options = SimpleFileOptions::default();

        writer.start_file(METADATA_FILE_NAMES[0], options)?;
        writer.write_all(
            serde_json::to_string_pretty(self)
                .map_err(|error| Error::Filesystem(std::io::Error::other(error)))?
                .as_bytes(),
        )?;

        for part in self.local_parts() {
            let path = source.join(part);
            if !path.exists() {
                return Err(Error::MissingPart {
                    part: part.to_string(),
                    directory: source.to_path_buf(),
                });
            }

            if path.is_dir() {
                writer.add_directory(part, options)?;
                pack_directory(&mut writer, &path, part, options)?;
            } else {
                writer.start_file(part, options)?;
                writer.write_all(&std::fs::read(&path)?)?;
            }
        }

        writer.finish()?;
        Ok(())
    }
}

/// Unpacks an archive into `directory` and reads the crate it holds.
///
/// # Errors
/// File does not exist
/// or
/// File is not readable
pub fn unzip(path: impl AsRef<Path>, directory: impl AsRef<Path>) -> Result<RoCrate> {
    let directory = directory.as_ref();
    let mut archive = ZipArchive::new(File::open(path.as_ref())?)?;
    std::fs::create_dir_all(directory)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        // `enclosed_name` refuses absolute paths and `..`, so an archive cannot
        // write outside the directory it is unpacked into.
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let target = directory.join(relative);

        if entry.is_dir() {
            std::fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::io::copy(&mut entry, &mut File::create(&target)?)?;
    }

    RoCrate::from_directory(directory).or_else(|error| match nested_root(directory)? {
        Some(nested) => RoCrate::from_directory(nested),
        None => Err(error),
    })
}

/// Archives often wrap the crate in a single top-level folder.
fn nested_root(directory: &Path) -> Result<Option<PathBuf>> {
    let mut entries = std::fs::read_dir(directory)?;
    let (Some(first), None) = (entries.next(), entries.next()) else {
        return Ok(None);
    };
    let path = first?.path();
    Ok(path.is_dir().then_some(path))
}

fn pack_directory(
    writer: &mut ZipWriter<File>,
    directory: &Path,
    prefix: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let name = format!(
            "{}/{}",
            prefix.trim_end_matches('/'),
            entry.file_name().to_string_lossy()
        );

        if path.is_dir() {
            writer.add_directory(&name, options)?;
            pack_directory(writer, &path, &name, options)?;
        } else {
            writer.start_file(&name, options)?;
            writer.write_all(&std::fs::read(&path)?)?;
        }
    }
    Ok(())
}

/// The metadata file closest to the archive root.
fn metadata_entry(archive: &mut ZipArchive<File>) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for index in 0..archive.len() {
        let Ok(entry) = archive.by_index_raw(index) else {
            continue;
        };
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        let is_metadata = name
            .file_name()
            .and_then(|file| file.to_str())
            .is_some_and(|file| METADATA_FILE_NAMES.contains(&file));

        if is_metadata {
            let depth = name.components().count();
            if best.is_none_or(|(_, shallowest)| depth < shallowest) {
                best = Some((index, depth));
            }
        }
    }
    best.map(|(index, _)| index)
}
