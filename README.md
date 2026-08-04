# rocrate
[![🦆 Continuous Integration](https://github.com/fairagro/ro-crate-lib/actions/workflows/ci.yaml/badge.svg)](https://github.com/fairagro/ro-crate-lib/actions/workflows/ci.yaml) ![Crates.io License](https://img.shields.io/crates/l/rocrate) 
![Crates.io Version](https://img.shields.io/crates/v/rocrate) ![Crates.io Total Downloads](https://img.shields.io/crates/d/rocrate)

A Rust library for reading, writing, building, validating, and navigating RO-Crates.

`rocrate` provides a strongly typed API over the RO-Crate JSON-LD model while preserving lossless serialization. It supports the base RO-Crate specification as well as Workflow RO-Crates and Workflow Run Crates.

## Features
- Read RO-Crates from JSON, directories, or ZIP archives
- Build RO-Crates using a fluent builder
- Validate crates against their declared profiles
- Validate against arbitrary profiles
- Automatic context management
- Typed access to workflows, workflow runs, datasets, software, parameters, actions, and more
- Lossless JSON round-tripping
- Read and write complete RO-Crate directories
- Optional ZIP archive support

## Supported specifications
- RO-Crate 1.0
- RO-Crate 1.1
- RO-Crate 1.2
- Workflow RO-Crate 1.0
- Workflow Run Crates
- Process Run
- Workflow Run
- Provenance Run
- Workflow Testing RO-Crate terms

## Installation
```toml
rocrate = "0.1"
```

ZIP support is enabled, by default. To disable the zip feature 
```toml
rocrate = { version = "0.1", default-features = false }
```
or to be explicit about it
```toml
rocrate = { version = "0.1",  features = ["zip"] }
```

## Reading a RO-Crate
```rust
use rocrate::RoCrate;

let crate_: RoCrate =
    serde_json::from_str(&std::fs::read_to_string("ro-crate-metadata.json")?)?;
```
or from a directory
```rust
let crate_ = RoCrate::from_directory("my-crate")?;
```
or a ZIP archive
```rust
let crate_ = RoCrate::from_zip("crate.zip")?;
```

## Building a crate
```rust
use rocrate::{
    RoCrate,
    build::Entity,
    profile::Profile,
};

let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .name("Example")
        .description("Built by the builder")
        .license("MIT")
        .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
        .main_workflow(workflow())
        .entity(language())
        .part(Entity::new("README.md", "File").set("encodingFormat", "text/markdown"))
        .build();
```

## Validation
Validate against the profiles claimed by the crate.
```rust
let validation = crate_.validate();

if validation.is_conformant() {
    println!("crate is conformant");
}
```

## Typed Views
Rather than manually navigating JSON-LD, the library exposes strongly typed views.
```rust
let workflow = crate_.workflow().unwrap();

println!("{}", workflow.name().unwrap());

for input in workflow.inputs() {
    println!("{}", input.name().unwrap());
}

for step in workflow.steps() {
    println!("{:?}", step.position());
}
```

## AI Disclosure
Claude Code (Opus 5 Model) was used to build the initial scaffold/prototype. Code was reviewed and further developed by a human being.

## License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT).