//! Validates every crate in `testdata/` and prints the diagnostics.
//!
//! Run with `cargo run --example report`.
use miette::Report;
use rocrate::RoCrate;

fn main() -> std::io::Result<()> {
    for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata"))? {
        let path = entry?.path();
        let crate_: RoCrate = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        let validation = crate_.validate();

        println!("\n=== {}", path.file_name().unwrap().to_string_lossy());
        for profile in validation.profiles() {
            println!("    {}", profile.iri());
        }

        match validation.into_result() {
            Ok(warnings) => {
                println!("    ✅ conformant, {} warning(s)", warnings.len());
                for warning in warnings {
                    println!("{:?}", Report::new(warning));
                }
            }
            Err(report) => println!("{:?}", Report::new(report)),
        }
    }
    Ok(())
}
