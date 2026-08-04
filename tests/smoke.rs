use rocrate::{RoCrate, graph::node::Value, profile::Profile};
use rstest::rstest;

fn read(fixture: &str) -> String {
    std::fs::read_to_string(format!("{}/testdata/{fixture}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn load(fixture: &str) -> RoCrate {
    serde_json::from_str(&read(fixture)).unwrap()
}

/// Every crate in `testdata/`, so a new fixture is covered the moment it lands.
fn fixtures() -> Vec<(String, RoCrate)> {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata");
    let mut crates: Vec<(String, RoCrate)> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let source = std::fs::read_to_string(&path).unwrap();
            let crate_ = serde_json::from_str(&source)
                .unwrap_or_else(|error| panic!("{name} does not parse: {error}"));
            (name, crate_)
        })
        .collect();
    crates.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert!(crates.len() >= 10, "fixtures went missing");
    crates
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
    let rocrate = load(fixture);
    let root = rocrate.root().expect("Root entity not found");
    assert_eq!(root.id, root_id);

    let workflow = rocrate.main_entity().expect("Main workflow not found");
    assert_eq!(workflow.id, workflow_id);
    assert!(workflow.has_type("ComputationalWorkflow"));
}

/// A crate may carry a workflow without nominating one: `mainEntity` is what
/// makes it the main workflow, and these crates leave it out.
#[rstest]
#[case::a_run_of_a_workflow_described_elsewhere("biocompute.json")]
#[case::no_workflow_at_all("minimal.json")]
#[case::a_workflow_only_listed_in_has_part("wf_bioschemas.json")]
fn a_root_without_a_main_entity_resolves_to_nothing(#[case] fixture: &str) {
    let rocrate = load(fixture);

    assert_eq!(rocrate.root().map(|root| root.id.as_str()), Some("./"));
    assert!(rocrate.main_entity().is_none());
    assert!(rocrate.workflow().is_none());
}

#[rstest]
#[case::flat_1_1("wroc_example.json", 3)]
#[case::nested_datasets("crop_modeling_workflow.json", 19)]
#[case::deeply_nested_results("biocompute.json", 32)]
#[case::one_part("wf_bioschemas.json", 1)]
#[case::no_parts("minimal.json", 0)]
fn collects_data_entities_through_nested_parts(#[case] fixture: &str, #[case] expected: usize) {
    let rocrate = load(fixture);
    assert_eq!(rocrate.data_entities().len(), expected);
}

#[test]
fn every_entity_is_root_descriptor_data_or_contextual() {
    for (name, rocrate) in fixtures() {
        let counted = rocrate.data_entities().len() + rocrate.contextual_entities().len() + 2;

        assert_eq!(counted, rocrate.graph.len(), "{name} does not add up");
    }
}

#[rstest]
#[case("argo_workflow.json", &[Profile::RoCrate("1.1".into()), Profile::WorkflowRoCrate("1.0".into())])]
#[case("nf_core_workflow.json", &[Profile::RoCrate("1.2".into()), Profile::WorkflowRoCrate("1.0".into())])]
#[case("proc_wrroc_example.json", &[Profile::RoCrate("1.1".into()), Profile::ProcessRun("0.4".into())])]
#[case::ro_crate_1_0("biocompute.json", &[Profile::RoCrate("1.0".into())])]
#[case::base_only("minimal.json", &[Profile::RoCrate("1.1".into())])]
#[case::bioschemas_without_the_profile("wf_bioschemas.json", &[Profile::RoCrate("1.1".into())])]
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
    let rocrate = load(fixture);
    let profiles = rocrate.profiles();

    for profile in expected {
        assert!(profiles.contains(profile), "{profile:?} missing");
        assert!(rocrate.claims(profile));
    }
    assert_eq!(profiles.len(), expected.len());
}

/// `@context` is a single IRI as often as it is an array.
#[rstest]
#[case::one_reference("minimal.json", 1)]
#[case::reference_and_inline_vocabulary("biocompute.json", 2)]
#[case::reference_and_inline_terms("nf_core_workflow.json", 2)]
fn contexts_come_as_one_item_or_many(#[case] fixture: &str, #[case] items: usize) {
    let rocrate = load(fixture);

    assert_eq!(rocrate.context.items().count(), items);
    assert!(
        rocrate.context.defines("hasPart"),
        "the base context is in there either way"
    );
}

/// RO-Crate 1.0, `@vocab`, and the `@reverse` links Describo writes.
#[test]
fn a_1_0_crate_keeps_its_vocabulary_and_reverse_links() {
    let rocrate = load("biocompute.json");

    assert!(rocrate.context.defines("@vocab"));
    assert_eq!(
        rocrate.context.definition("@vocab"),
        Some("https://schema.org/")
    );

    // `@reverse` is an object of terms, and survives as one.
    let results = rocrate.graph.get("results/").expect("no results dataset");
    let Some(Value::Object(reverse)) = results.get("@reverse") else {
        panic!("@reverse was dropped or flattened");
    };
    assert_eq!(
        reverse["hasPart"].refs().collect::<Vec<_>>(),
        ["./"],
        "the crate root is what has this part"
    );
}

#[test]
fn round_trips_through_json() {
    for (name, rocrate) in fixtures() {
        let reparsed: RoCrate =
            serde_json::from_str(&serde_json::to_string(&rocrate).unwrap()).unwrap();

        assert_eq!(rocrate, reparsed, "{name} changed on the way through");
    }
}
