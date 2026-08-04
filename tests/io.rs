use miette::Diagnostic;
use rocrate::{RoCrate, build::Entity, io, profile::Profile};
use std::path::Path;
use tempfile::TempDir;

/// A crate whose parts exist on disk.
fn crate_on_disk() -> (TempDir, RoCrate) {
    let directory = TempDir::new().unwrap();
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .name("Packed example")
        .license("MIT")
        .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
        .main_workflow(
            Entity::new(
                "wf.cwl",
                &["File", "SoftwareSourceCode", "ComputationalWorkflow"],
            )
            .set("name", "Example workflow")
            .reference("programmingLanguage", "#cwl"),
        )
        .entity(Entity::new("#cwl", "ComputerLanguage").set("name", "CWL"))
        .part(Entity::new("README.md", "File"))
        .part(Entity::new("data/", "Dataset"))
        .part(Entity::new("https://example.org/remote.txt", "File"))
        .build();

    std::fs::write(directory.path().join("wf.cwl"), "cwlVersion: v1.2\n").unwrap();
    std::fs::write(directory.path().join("README.md"), "# Example\n").unwrap();
    std::fs::create_dir(directory.path().join("data")).unwrap();
    std::fs::write(directory.path().join("data/input.txt"), "hello\n").unwrap();
    crate_.write_directory(directory.path()).unwrap();

    (directory, crate_)
}

#[test]
fn a_crate_survives_a_trip_through_a_directory() {
    let (directory, crate_) = crate_on_disk();

    assert!(directory.path().join("ro-crate-metadata.json").is_file());
    assert_eq!(RoCrate::from_directory(directory.path()).unwrap(), crate_);
}

#[test]
fn every_published_fixture_reads_from_a_directory() {
    for fixture in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata")).unwrap() {
        let source = std::fs::read_to_string(fixture.unwrap().path()).unwrap();
        let crate_: RoCrate = serde_json::from_str(&source).unwrap();

        let directory = TempDir::new().unwrap();
        crate_.write_directory(directory.path()).unwrap();

        assert_eq!(RoCrate::from_directory(directory.path()).unwrap(), crate_);
    }
}

#[test]
fn the_1_0_metadata_file_name_is_read_too() {
    let directory = TempDir::new().unwrap();
    let (_source, crate_) = crate_on_disk();
    let json = serde_json::to_string(&crate_).unwrap();
    std::fs::write(directory.path().join("ro-crate-metadata.jsonld"), json).unwrap();

    assert_eq!(RoCrate::from_directory(directory.path()).unwrap(), crate_);
}

#[test]
fn a_directory_without_metadata_says_so() {
    let directory = TempDir::new().unwrap();
    let error = RoCrate::from_directory(directory.path()).unwrap_err();

    assert!(matches!(error, io::Error::NoMetadata { .. }));
    assert_eq!(
        error.code().unwrap().to_string(),
        "rocrate::io::no_metadata"
    );
}

#[test]
fn broken_metadata_points_at_the_place_it_broke() {
    let directory = TempDir::new().unwrap();
    std::fs::write(
        directory.path().join("ro-crate-metadata.json"),
        "{\n  \"@context\": \"https://w3id.org/ro/crate/1.1/context\",\n  \"@graph\": [oops]\n}\n",
    )
    .unwrap();

    let error = RoCrate::from_directory(directory.path()).unwrap_err();
    let io::Error::InvalidMetadata(details) = &error else {
        panic!("expected invalid metadata, got {error:?}");
    };

    assert_eq!(
        details.path,
        directory.path().join("ro-crate-metadata.json")
    );
    assert!(details.message.contains("line 3"));
    assert!(error.source_code().is_some(), "the document is attached");
    assert_eq!(
        error.labels().unwrap().count(),
        1,
        "the offending place is labelled"
    );
}

#[test]
fn parts_are_told_apart_from_identifiers_and_urls() {
    let (directory, crate_) = crate_on_disk();

    let mut parts = crate_.local_parts();
    parts.sort_unstable();
    assert_eq!(parts, ["README.md", "data/", "wf.cwl"]);
    assert!(crate_.missing_parts(directory.path()).is_empty());

    std::fs::remove_file(directory.path().join("README.md")).unwrap();
    assert_eq!(crate_.missing_parts(directory.path()), ["README.md"]);
}

#[cfg(feature = "zip")]
mod archive {
    use super::*;

    #[test]
    fn a_crate_survives_a_trip_through_a_zip() {
        let (directory, crate_) = crate_on_disk();
        let zip = directory.path().join("crate.zip");
        crate_.write_zip(&zip, directory.path()).unwrap();

        assert_eq!(RoCrate::from_zip(&zip).unwrap(), crate_);

        let unpacked = TempDir::new().unwrap();
        let read_back = io::unzip(&zip, unpacked.path()).unwrap();

        assert_eq!(read_back, crate_);
        assert_eq!(
            std::fs::read_to_string(unpacked.path().join("data/input.txt")).unwrap(),
            "hello\n"
        );
        assert!(unpacked.path().join("README.md").is_file());
        assert!(read_back.missing_parts(unpacked.path()).is_empty());
    }

    #[test]
    fn packing_stops_at_a_part_that_is_not_there() {
        let (directory, crate_) = crate_on_disk();
        std::fs::remove_file(directory.path().join("wf.cwl")).unwrap();

        let error = crate_
            .write_zip(directory.path().join("crate.zip"), directory.path())
            .unwrap_err();

        assert!(matches!(
            error,
            io::Error::MissingPart { ref part, .. } if part == "wf.cwl"
        ));
    }

    #[test]
    fn a_crate_wrapped_in_a_folder_is_still_found() {
        let (source, crate_) = crate_on_disk();
        let zip = wrap_in_folder(source.path(), "my-crate");

        assert_eq!(RoCrate::from_zip(&zip).unwrap(), crate_);

        let unpacked = TempDir::new().unwrap();
        assert_eq!(io::unzip(&zip, unpacked.path()).unwrap(), crate_);
    }

    #[test]
    fn an_archive_cannot_write_outside_the_directory_it_unpacks_into() {
        let directory = TempDir::new().unwrap();
        let zip = directory.path().join("evil.zip");
        let (source, crate_) = crate_on_disk();
        crate_.write_zip(&zip, source.path()).unwrap();
        append_entry(&zip, "../escaped.txt", b"nope");

        let unpacked = TempDir::new().unwrap();
        io::unzip(&zip, unpacked.path()).unwrap();

        assert!(
            !unpacked
                .path()
                .parent()
                .unwrap()
                .join("escaped.txt")
                .exists()
        );
        assert!(!unpacked.path().join("escaped.txt").exists());
    }

    #[test]
    fn a_zip_without_metadata_says_so() {
        let directory = TempDir::new().unwrap();
        let zip = directory.path().join("empty.zip");
        append_entry(&zip, "notes.txt", b"nothing to see");

        let error = RoCrate::from_zip(&zip).unwrap_err();
        assert!(matches!(error, io::Error::NoMetadata { .. }));
    }

    /// Repacks a crate directory under `folder/`, as published archives do.
    fn wrap_in_folder(directory: &Path, folder: &str) -> std::path::PathBuf {
        let zip = directory.join("wrapped.zip");
        let file = std::fs::File::create(&zip).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() || path.extension().is_some_and(|e| e == "zip") {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            writer
                .start_file(format!("{folder}/{name}"), options)
                .unwrap();
            std::io::Write::write_all(&mut writer, &std::fs::read(&path).unwrap()).unwrap();
        }
        writer.finish().unwrap();
        zip
    }

    fn append_entry(zip: &Path, name: &str, contents: &[u8]) {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(zip)
            .unwrap();

        let mut writer = match zip::ZipWriter::new_append(file) {
            Ok(writer) => writer,
            Err(_) => zip::ZipWriter::new(std::fs::File::create(zip).unwrap()),
        };
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut writer, contents).unwrap();
        writer.finish().unwrap();
    }
}
