use miette::{Diagnostic, Severity};
use rocrate::{RoCrate, profile::Profile, validate::Level};
use rstest::rstest;

/// A crate that breaks no rule of any profile it claims.
const VALID: &str = r##"{
  "@context": [
    "https://w3id.org/ro/crate/1.1/context",
    "https://w3id.org/ro/terms/workflow-run/context"
  ],
  "@graph": [
    {"@id": "ro-crate-metadata.json", "@type": "CreativeWork", "about": {"@id": "./"},
     "conformsTo": [{"@id": "https://w3id.org/ro/crate/1.1"},
                    {"@id": "https://w3id.org/workflowhub/workflow-ro-crate/1.0"}]},
    {"@id": "./", "@type": "Dataset",
     "conformsTo": [{"@id": "https://w3id.org/ro/wfrun/process/0.5"},
                    {"@id": "https://w3id.org/ro/wfrun/workflow/0.5"},
                    {"@id": "https://w3id.org/ro/wfrun/provenance/0.5"}],
     "name": "Example run", "description": "A crate that validates cleanly",
     "datePublished": "2026-01-01", "license": "MIT",
     "hasPart": [{"@id": "wf.cwl"}, {"@id": "README.md"}],
     "mainEntity": {"@id": "wf.cwl"},
     "mentions": [{"@id": "#run"}, {"@id": "#step-run"}]},
    {"@id": "README.md", "@type": "File", "encodingFormat": "text/markdown"},
    {"@id": "wf.cwl", "@type": ["File", "SoftwareSourceCode", "ComputationalWorkflow", "HowTo"],
     "name": "Example workflow", "programmingLanguage": {"@id": "#cwl"},
     "input": {"@id": "#in"}, "output": {"@id": "#out"}, "step": {"@id": "#step"}},
    {"@id": "#cwl", "@type": "ComputerLanguage", "name": "CWL"},
    {"@id": "#in", "@type": "FormalParameter", "additionalType": "File", "name": "in"},
    {"@id": "#out", "@type": "FormalParameter", "additionalType": "File", "name": "out"},
    {"@id": "#step", "@type": "HowToStep", "position": "1", "workExample": {"@id": "#tool"}},
    {"@id": "#tool", "@type": "SoftwareApplication", "name": "samtools"},
    {"@id": "#engine", "@type": "SoftwareApplication", "name": "cwltool"},
    {"@id": "#organize", "@type": "OrganizeAction", "instrument": {"@id": "#engine"},
     "object": {"@id": "#control"}, "result": {"@id": "#run"},
     "startTime": "2026-01-01T10:00:00Z"},
    {"@id": "#control", "@type": "ControlAction", "instrument": {"@id": "#step"},
     "object": {"@id": "#step-run"}},
    {"@id": "#run", "@type": "CreateAction", "instrument": {"@id": "wf.cwl"},
     "name": "Run of the example workflow", "description": "cwltool wf.cwl job.json",
     "endTime": "2026-01-01T10:05:00Z", "agent": {"@id": "#person"},
     "object": {"@id": "#in-pv"}, "result": {"@id": "#out-pv"}},
    {"@id": "#step-run", "@type": "CreateAction", "instrument": {"@id": "#tool"},
     "name": "Run of samtools", "description": "samtools sort", "agent": {"@id": "#person"},
     "endTime": "2026-01-01T10:03:00Z", "result": {"@id": "#out-pv"}},
    {"@id": "#person", "@type": "Person", "name": "Alice"},
    {"@id": "#in-pv", "@type": "PropertyValue", "name": "in", "value": "hello",
     "exampleOfWork": {"@id": "#in"}},
    {"@id": "#out-pv", "@type": "PropertyValue", "name": "out", "value": "world",
     "exampleOfWork": {"@id": "#out"}}
  ]
}"##;

fn valid() -> RoCrate {
    serde_json::from_str(VALID).unwrap()
}

/// `VALID` with one piece of JSON swapped out.
fn broken(from: &str, to: &str) -> RoCrate {
    let source = VALID.replace(from, to);
    assert_ne!(source, VALID, "`{from}` is not in the fixture");
    serde_json::from_str(&source).unwrap()
}

fn load(fixture: &str) -> RoCrate {
    let path = format!("{}/testdata/{fixture}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn a_conformant_crate_reports_nothing_at_all() {
    let validation = valid().validate();

    assert_eq!(
        validation.violations,
        [],
        "expected a clean crate, got {:#?}",
        validation.violations
    );
    assert!(validation.is_conformant());
    assert_eq!(validation.profiles().len(), 5);
    assert!(validation.into_result().is_ok());
}

#[test]
fn warnings_do_not_cost_conformance() {
    let validation = broken(r##", {"@id": "README.md"}"##, "").validate();

    assert!(validation.is_conformant());
    assert!(validation.broke("wroc::readme"));
    assert_eq!(validation.errors().count(), 0);
    assert_eq!(validation.into_result().unwrap().len(), 1);
}

/// Published crates are otherwise in good shape, so anything else here is a
/// rule that fires when it should not.
#[rstest]
fn the_published_crates_only_miss_dates_and_licenses(
    #[values(
        "argo_workflow.json",
        "crop_modeling_workflow.json",
        "nf_core_workflow.json",
        "proc_wrroc_example.json",
        "prov_wrroc_example.json",
        "wf_wrroc_example.json",
        "wroc_example.json"
    )]
    fixture: &str,
) {
    let validation = load(fixture).validate();
    let unexpected: Vec<&str> = validation
        .errors()
        .map(|violation| violation.rule)
        .filter(|rule| !["base::root-date-published", "wroc::root-license"].contains(rule))
        .collect();

    assert_eq!(unexpected, [] as [&str; 0]);
}

#[rstest]
#[case::descriptor(
    r##""@id": "ro-crate-metadata.json""##,
    r##""@id": "metadata.txt""##,
    "base::descriptor"
)]
#[case::descriptor_type(
    r##""ro-crate-metadata.json", "@type": "CreativeWork""##,
    r##""ro-crate-metadata.json", "@type": "File""##,
    "base::descriptor-type"
)]
#[case::descriptor_about(
    r##""about": {"@id": "./"}"##,
    r##""about": {"@id": "#nowhere"}"##,
    "base::descriptor-about"
)]
#[case::root_type(
    r##""./", "@type": "Dataset""##,
    r##""./", "@type": "Collection""##,
    "base::root-type"
)]
#[case::date_published(
    r##""datePublished": "2026-01-01", "##,
    "",
    "base::root-date-published"
)]
#[case::dangling_part(
    r##"{"@id": "README.md"}"##,
    r##"{"@id": "MISSING.md"}"##,
    "base::dangling-part"
)]
fn base_rules_catch_a_broken_root(
    #[case] from: &str,
    #[case] to: &str,
    #[case] rule: &'static str,
) {
    let validation = broken(from, to).validate();

    assert!(
        validation.broke(rule),
        "expected {rule}, got {:#?}",
        validation.violations
    );
}

#[rstest]
#[case::no_main_entity(r##""mainEntity": {"@id": "wf.cwl"},"##, "", "wroc::main-entity")]
#[case::unresolved_main_entity(
    r##""mainEntity": {"@id": "wf.cwl"}"##,
    r##""mainEntity": {"@id": "#gone"}"##,
    "wroc::main-entity"
)]
#[case::wrong_types(
    r##""@type": ["File", "SoftwareSourceCode", "ComputationalWorkflow", "HowTo"]"##,
    r##""@type": ["File", "ComputationalWorkflow"]"##,
    "wroc::main-entity-type"
)]
#[case::language_is_not_a_language(
    r##""programmingLanguage": {"@id": "#cwl"}"##,
    r##""programmingLanguage": {"@id": "#tool"}"##,
    "wroc::programming-language"
)]
#[case::no_language(
    r##""programmingLanguage": {"@id": "#cwl"},"##,
    "",
    "wroc::programming-language"
)]
#[case::no_license(r##""license": "MIT","##, "", "wroc::root-license")]
fn workflow_ro_crate_rules_catch_a_broken_workflow(
    #[case] from: &str,
    #[case] to: &str,
    #[case] rule: &'static str,
) {
    let validation = broken(from, to).validate();

    assert!(
        validation.errors().any(|violation| violation.rule == rule),
        "expected {rule} as an error, got {:#?}",
        validation.violations
    );
}

#[rstest]
#[case::no_instrument(r##""instrument": {"@id": "wf.cwl"},"##, "", "process::instrument")]
#[case::unresolved_instrument(
    r##""instrument": {"@id": "wf.cwl"}"##,
    r##""instrument": {"@id": "#gone"}"##,
    "process::instrument"
)]
#[case::nothing_ran_the_workflow(
    r##""instrument": {"@id": "wf.cwl"}"##,
    r##""instrument": {"@id": "#tool"}"##,
    "wfrun::workflow-run"
)]
#[case::untyped_parameter(
    r##""additionalType": "File", "name": "in""##,
    r##""name": "in""##,
    "wfrun::parameter-type"
)]
#[case::no_engine(
    r##""@id": "#organize", "@type": "OrganizeAction", "instrument": {"@id": "#engine"},"##,
    r##""@id": "#organize", "@type": "OrganizeAction", "instrument": {"@id": "#step"},"##,
    "prov::engine"
)]
#[case::control_without_a_step(
    r##""@id": "#control", "@type": "ControlAction", "instrument": {"@id": "#step"}"##,
    r##""@id": "#control", "@type": "ControlAction", "instrument": {"@id": "#tool"}"##,
    "prov::control-instrument"
)]
#[case::step_without_a_tool(
    r##""position": "1", "workExample": {"@id": "#tool"}"##,
    r##""position": "1""##,
    "prov::step-work-example"
)]
fn run_crate_rules_catch_a_broken_run(
    #[case] from: &str,
    #[case] to: &str,
    #[case] rule: &'static str,
) {
    let validation = broken(from, to).validate();

    assert!(
        validation.errors().any(|violation| violation.rule == rule),
        "expected {rule} as an error, got {:#?}",
        validation.violations
    );
}

#[test]
fn a_run_that_is_not_mentioned_is_only_a_warning() {
    let validation = broken(r##", {"@id": "#step-run"}]"##, "]").validate();

    assert!(validation.is_conformant());
    assert!(validation.broke("process::mentions"));
}

#[test]
fn a_crate_can_be_checked_against_a_profile_it_does_not_claim() {
    let crate_ = load("wroc_example.json");
    assert!(!crate_.claims(&Profile::ProcessRun("0.5".into())));

    let validation = crate_.validate_as(&Profile::ProcessRun("0.5".into()));

    assert!(validation.broke("process::action"));
    assert_eq!(validation.profiles().len(), 1);
    assert!(
        !validation.broke("wroc::readme"),
        "only the named profile is checked"
    );
}

#[test]
fn duplicate_ids_are_reported_once_per_id() {
    let doubled = VALID.replace(
        r##"{"@id": "#person", "@type": "Person", "name": "Alice"},"##,
        r##"{"@id": "#person", "@type": "Person", "name": "Alice"},
           {"@id": "#person", "@type": "Person", "name": "Bob"},"##,
    );
    let crate_: RoCrate = serde_json::from_str(&doubled).unwrap();

    let validation = crate_.validate();
    let duplicates: Vec<_> = validation
        .violations
        .iter()
        .filter(|violation| violation.rule == "base::duplicate-id")
        .collect();

    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].entity.as_deref(), Some("#person"));
}

#[test]
fn violations_render_as_miette_diagnostics() {
    let source = VALID
        .replace(r##""datePublished": "2026-01-01", "##, "")
        .replace(r##", {"@id": "README.md"}"##, "");
    let crate_: RoCrate = serde_json::from_str(&source).unwrap();
    let validation = crate_.validate();
    let error = validation.errors().next().unwrap();

    assert_eq!(error.severity(), Some(Severity::Error));
    assert_eq!(
        error.code().unwrap().to_string(),
        "rocrate::base::root-date-published"
    );
    assert_eq!(
        error.to_string(),
        "the root data entity has no `datePublished` (`./`)"
    );

    let warning = validation.warnings().next().unwrap();
    assert_eq!(warning.severity(), Some(Severity::Warning));
    assert_eq!(warning.level, Level::Should);
}

#[test]
fn a_failed_validation_becomes_one_report_with_related_violations() {
    let validation = broken(r##""datePublished": "2026-01-01", "##, "").validate();
    let expected = validation.violations.len();

    let report = validation.into_result().unwrap_err();

    assert!(
        report
            .profiles
            .contains("https://w3id.org/ro/wfrun/provenance/0.5")
    );
    assert_eq!(report.related().unwrap().count(), expected);
    assert_eq!(report.violations.len(), expected);
}
