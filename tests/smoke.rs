use rocrate::{RoCrate, profile::Profile};
use rstest::rstest;

const FIXTURES: [&str; 7] = [
    "argo_workflow.json",
    "crop_modeling_workflow.json",
    "nf_core_workflow.json",
    "proc_wrroc_example.json",
    "prov_wrroc_example.json",
    "wf_wrroc_example.json",
    "wroc_example.json",
];

fn read(fixture: &str) -> String {
    std::fs::read_to_string(format!("{}/testdata/{fixture}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn load(fixture: &str) -> RoCrate {
    serde_json::from_str(&read(fixture)).unwrap()
}

#[rstest]
#[case("argo_workflow.json", "./", "workflow.yaml")]
#[case(
    "crop_modeling_workflow.json",
    "./",
    "workflows/csmWorkflow/workflow.cwl"
)]
#[case("nf_core_workflow.json", "./", "main.nf")]
#[case("prov_wrroc_example.json", "./", "packed.cwl")]
#[case("wf_wrroc_example.json", "./", "Galaxy-Workflow-Hello_World.ga")]
#[case("wroc_example.json", "./", "example_workflow.cwl")]
fn resolves_the_root_and_its_main_workflow(
    #[case] fixture: &str,
    #[case] root_id: &str,
    #[case] workflow_id: &str,
) {
    let crate_ = load(fixture);
    let root = crate_.root().expect("Root entity not found");
    assert_eq!(root.id, root_id);

    let workflow = crate_.main_entity().expect("Main workflow not found");
    assert_eq!(workflow.id, workflow_id);
    assert!(workflow.has_type("ComputationalWorkflow"));
}

#[rstest]
#[case::flat_1_1("wroc_example.json", 3)]
#[case::nested_datasets("crop_modeling_workflow.json", 19)]
fn collects_data_entities_through_nested_parts(#[case] fixture: &str, #[case] expected: usize) {
    let crate_ = load(fixture);
    assert_eq!(crate_.data_entities().len(), expected);
}

#[rstest]
fn every_entity_is_root_descriptor_data_or_contextual(#[values(0, 1, 2, 3, 4, 5, 6)] index: usize) {
    let crate_ = load(FIXTURES[index]);
    let counted = crate_.data_entities().len() + crate_.contextual_entities().len() + 2;

    assert_eq!(counted, crate_.graph.len());
}

#[rstest]
#[case("argo_workflow.json", &[Profile::RoCrate("1.1".into()), Profile::WorkflowRoCrate("1.0".into())])]
#[case("nf_core_workflow.json", &[Profile::RoCrate("1.2".into()), Profile::WorkflowRoCrate("1.0".into())])]
#[case("proc_wrroc_example.json", &[Profile::RoCrate("1.1".into()), Profile::ProcessRun("0.4".into())])]
#[case("wf_wrroc_example.json", &[
    Profile::RoCrate("1.1".into()),
    Profile::WorkflowRoCrate("1.0".into()),
    Profile::ProcessRun("0.1".into()),
    Profile::WorkflowRun("0.1".into()),
])]
#[case("prov_wrroc_example.json", &[
    Profile::RoCrate("1.1".into()),
    Profile::WorkflowRoCrate("1.0".into()),
    Profile::ProcessRun("0.1".into()),
    Profile::WorkflowRun("0.1".into()),
    Profile::ProvenanceRun("0.1".into()),
])]
fn reads_profiles_from_the_descriptor_and_the_root(
    #[case] fixture: &str,
    #[case] expected: &[Profile],
) {
    let crate_ = load(fixture);
    let profiles = crate_.profiles();

    for profile in expected {
        assert!(profiles.contains(profile), "{profile:?} missing");
        assert!(crate_.claims(profile));
    }
    assert_eq!(profiles.len(), expected.len());
}

#[rstest]
fn round_trips_through_json(#[values(0, 1, 2, 3, 4, 5, 6)] index: usize) {
    let fixture = FIXTURES[index];
    let crate_ = load(fixture);
    let reparsed: RoCrate = serde_json::from_str(&serde_json::to_string(&crate_).unwrap()).unwrap();

    assert_eq!(crate_, reparsed);
}
