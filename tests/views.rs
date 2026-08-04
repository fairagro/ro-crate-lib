use rocrate::{
    RoCrate,
    context::Context,
    views::{ComputerLanguage, ControlAction, CreateAction, FormalParameter, OrganizeAction, View},
};
use rstest::rstest;

fn load(fixture: &str) -> RoCrate {
    let path = format!("{}/testdata/{fixture}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[rstest]
#[case("prov_wrroc_example.json", "packed.cwl", 2, 1, 2)]
#[case("wf_wrroc_example.json", "Hello World (Galaxy Workflow)", 2, 2, 0)]
#[case("wroc_example.json", "Example Workflow", 0, 0, 0)]
fn the_main_workflow_exposes_its_signature(
    #[case] fixture: &str,
    #[case] name: &str,
    #[case] inputs: usize,
    #[case] outputs: usize,
    #[case] steps: usize,
) {
    let crate_ = load(fixture);
    let workflow = crate_.workflow().expect("main entity is not a workflow");

    assert_eq!(workflow.name(), Some(name));
    assert_eq!(workflow.inputs().len(), inputs);
    assert_eq!(workflow.outputs().len(), outputs);
    assert_eq!(workflow.steps().len(), steps);
    assert!(workflow.language().is_some());
}

#[rstest]
#[case::references(
    "prov_wrroc_example.json",
    "https://www.commonwl.org/",
    "https://w3id.org/cwl/v1.0/"
)]
#[case::plain_text(
    "wroc_example.json",
    "https://www.commonwl.org/",
    "https://w3id.org/cwl/v1.2/"
)]
#[case::plain_text(
    "wf_wrroc_example.json",
    "https://galaxyproject.org/",
    "https://galaxyproject.org/"
)]
fn language_iris_read_the_same_as_references_or_text(
    #[case] fixture: &str,
    #[case] url: &str,
    #[case] identifier: &str,
) {
    let crate_ = load(fixture);
    let language = crate_.workflow().unwrap().language().unwrap();

    assert_eq!(language.url(), Some(url));
    assert_eq!(language.identifier(), Some(identifier));
}

#[test]
fn formal_parameters_carry_their_type_and_default() {
    let crate_ = load("prov_wrroc_example.json");
    let workflow = crate_.workflow().unwrap();
    let input = workflow
        .inputs()
        .into_iter()
        .find(|p| p.id() == "packed.cwl#main/input")
        .expect("input parameter missing");

    assert_eq!(input.name(), Some("main/input"));
    assert_eq!(input.additional_type(), Some("File"));
    assert_eq!(
        input.default_value(),
        Some("file:///home/stain/src/cwltool/tests/wf/hello.txt")
    );
    assert_eq!(
        input.encoding_formats(),
        ["https://www.iana.org/assignments/media-types/text/plain"]
    );
}

#[test]
fn steps_resolve_to_the_tools_they_run() {
    let crate_ = load("prov_wrroc_example.json");
    let steps = crate_.workflow().unwrap().steps();

    assert_eq!(steps[0].position(), Some("0"));
    assert_eq!(
        steps[0].work_example().map(|tool| tool.id.as_str()),
        Some("packed.cwl#revtool.cwl")
    );
}

#[test]
fn a_provenance_run_walks_engine_to_step_to_run() {
    let crate_ = load("prov_wrroc_example.json");
    let organize = crate_.views::<OrganizeAction>();
    let organize = organize.first().expect("no OrganizeAction");

    let engine = organize.instrument().expect("no engine");
    assert_eq!(engine.name(), Some("cwltool 1.0.20181012180214"));
    assert_eq!(organize.start_time(), Some("2018-10-25T15:46:35.210973"));

    let workflow_run = organize.result().expect("no workflow run");
    assert_eq!(
        workflow_run.workflow().map(|w| w.id()),
        Some("packed.cwl"),
        "the run's instrument is the main workflow"
    );
    assert_eq!(workflow_run.end_time(), Some("2018-10-25T15:46:43.020168"));

    let orchestrations = organize.objects();
    assert_eq!(orchestrations.len(), 2);
    let step_runs: Vec<CreateAction> = orchestrations
        .iter()
        .flat_map(ControlAction::objects)
        .collect();
    assert_eq!(step_runs.len(), 2);
    assert!(
        orchestrations
            .iter()
            .all(|action| action.instrument().is_some()),
        "each ControlAction orchestrates a HowToStep"
    );
}

#[test]
fn the_root_reaches_its_runs_through_mentions() {
    let crate_ = load("wf_wrroc_example.json");
    let root = crate_.root_dataset().expect("root is not a Dataset");

    assert_eq!(root.license(), Some("http://spdx.org/licenses/CC0-1.0"));
    assert_eq!(root.main_entity().map(|w| w.id()), Some(root_workflow_id()));

    let actions = root.actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].workflow().map(|w| w.id()),
        Some(root_workflow_id())
    );
}

fn root_workflow_id() -> &'static str {
    "Galaxy-Workflow-Hello_World.ga"
}

#[test]
fn a_process_run_has_an_application_not_a_workflow() {
    let crate_ = load("proc_wrroc_example.json");
    let actions = crate_.views::<CreateAction>();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name(), Some("Convert dog image to sepia"));
    assert!(actions[0].instrument().is_some());
    assert!(
        actions[0].workflow().is_none(),
        "a Process Run Crate runs an application"
    );
    assert!(crate_.workflow().is_none());
}

#[test]
fn licenses_read_as_text_too() {
    let crate_ = load("wroc_example.json");

    assert_eq!(crate_.root_dataset().unwrap().license(), Some("Apache-2.0"));
}

#[test]
fn views_need_the_terms_they_read_to_be_in_the_context() {
    let mut crate_ = load("prov_wrroc_example.json");
    assert!(crate_.workflow().is_some());

    crate_.context = Context::new_from_iri("https://example.org/unknown/context");

    assert!(crate_.workflow().is_none());
    assert!(crate_.root_dataset().is_none());
    assert!(crate_.views::<FormalParameter>().is_empty());
    assert!(crate_.views::<ComputerLanguage>().is_empty());
}

#[test]
fn a_view_only_applies_to_entities_of_its_type() {
    let crate_ = load("prov_wrroc_example.json");

    assert!(crate_.view::<CreateAction>("packed.cwl").is_none());
    assert!(crate_.view::<CreateAction>("does-not-exist").is_none());
    assert_eq!(crate_.views::<CreateAction>().len(), 3);
}
