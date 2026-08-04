use rocrate::{
    RoCrate,
    context::Context,
    views::{
        ComputerLanguage, ContainerImage, ControlAction, CreateAction, FormalParameter,
        OrganizeAction, ParameterConnection, View,
    },
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

const RUN_CRATE: &str = r##"{
  "@context": [
    "https://w3id.org/ro/crate/1.1/context",
    "https://w3id.org/ro/terms/workflow-run/context"
  ],
  "@graph": [
    {"@id": "ro-crate-metadata.json", "@type": "CreativeWork", "about": {"@id": "./"},
     "conformsTo": {"@id": "https://w3id.org/ro/crate/1.1"}},
    {"@id": "./", "@type": "Dataset", "hasPart": {"@id": "wf.cwl"},
     "conformsTo": {"@id": "https://w3id.org/ro/wfrun/provenance/0.5"},
     "mainEntity": {"@id": "wf.cwl"}, "mentions": {"@id": "#run"}},
    {"@id": "wf.cwl", "@type": ["File", "SoftwareSourceCode", "ComputationalWorkflow", "HowTo"],
     "programmingLanguage": {"@id": "#cwl"}, "input": {"@id": "#in"}, "output": {"@id": "#out"},
     "step": {"@id": "#step1"}, "connection": {"@id": "#conn-out"}},
    {"@id": "#cwl", "@type": "ComputerLanguage", "name": "CWL"},
    {"@id": "#in", "@type": "FormalParameter", "additionalType": "File", "name": "in"},
    {"@id": "#out", "@type": "FormalParameter", "additionalType": "File", "name": "out"},
    {"@id": "#tool-in", "@type": "FormalParameter", "additionalType": "File", "name": "tool-in"},
    {"@id": "#tool-out", "@type": "FormalParameter", "additionalType": "File", "name": "tool-out"},
    {"@id": "#step1", "@type": "HowToStep", "position": "1", "workExample": {"@id": "#tool"},
     "connection": {"@id": "#conn-step"}},
    {"@id": "#tool", "@type": "SoftwareApplication", "name": "samtools", "version": "1.9"},
    {"@id": "#conn-step", "@type": "ParameterConnection",
     "sourceParameter": {"@id": "#in"}, "targetParameter": {"@id": "#tool-in"}},
    {"@id": "#conn-out", "@type": "ParameterConnection",
     "sourceParameter": {"@id": "#tool-out"}, "targetParameter": {"@id": "#out"}},
    {"@id": "#run", "@type": "CreateAction", "instrument": {"@id": "wf.cwl"},
     "object": {"@id": "#in-pv"}, "result": {"@id": "#out-pv"},
     "containerImage": {"@id": "#image"}, "environment": {"@id": "#env"}},
    {"@id": "#image", "@type": "ContainerImage",
     "additionalType": {"@id": "https://w3id.org/ro/terms/workflow-run#DockerImage"},
     "registry": "docker.io", "name": "biocontainers/samtools", "tag": "v1.9-4-deb_cv1",
     "sha256": "da61624fda230e94867c9429ca1112e1e77c24e500b52dfc84eaf2f5820b4a2a"},
    {"@id": "#in-pv", "@type": "PropertyValue", "name": "in", "value": "hello",
     "exampleOfWork": {"@id": "#in"}},
    {"@id": "#out-pv", "@type": "PropertyValue", "name": "out", "value": "world"},
    {"@id": "#env", "@type": "PropertyValue", "name": "HEIGHT_LIMIT", "value": "1000px"}
  ]
}"##;

fn run_crate() -> RoCrate {
    serde_json::from_str(RUN_CRATE).unwrap()
}

#[test]
fn a_run_reports_its_container_and_environment() {
    let crate_ = run_crate();
    let run = crate_.root_dataset().unwrap().actions().remove(0);

    let image = run.container_images().remove(0);
    assert_eq!(image.name(), Some("biocontainers/samtools"));
    assert_eq!(image.registry(), Some("docker.io"));
    assert_eq!(image.tag(), Some("v1.9-4-deb_cv1"));
    assert_eq!(
        image.kind(),
        Some("https://w3id.org/ro/terms/workflow-run#DockerImage")
    );
    assert!(image.sha256().is_some());
    assert_eq!(image.md5(), None);

    let environment = run.environment();
    assert_eq!(environment.len(), 1);
    assert_eq!(environment[0].name(), Some("HEIGHT_LIMIT"));
    assert_eq!(environment[0].text_value(), Some("1000px"));
}

#[test]
fn parameter_values_link_back_to_the_parameter_they_fill() {
    let crate_ = run_crate();
    let run = crate_.root_dataset().unwrap().actions().remove(0);

    let inputs = run.object_values();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].text_value(), Some("hello"));
    assert_eq!(
        inputs[0].example_of_work().map(|p| p.id.as_str()),
        Some("#in")
    );
    assert_eq!(run.result_values()[0].text_value(), Some("world"));
}

#[test]
fn connections_hang_off_the_workflow_and_its_steps() {
    let crate_ = run_crate();
    let workflow = crate_.workflow().unwrap();

    let into_output = workflow.connections();
    assert_eq!(into_output.len(), 1);
    assert_eq!(
        into_output[0].source().and_then(|p| p.name()),
        Some("tool-out")
    );
    assert_eq!(into_output[0].target().and_then(|p| p.name()), Some("out"));

    let step = workflow.steps().remove(0);
    assert_eq!(step.position(), Some("1"));
    let between_steps = step.connections();
    assert_eq!(between_steps.len(), 1);
    assert_eq!(between_steps[0].source().and_then(|p| p.name()), Some("in"));
    assert_eq!(
        between_steps[0].target_node().map(|p| p.id.as_str()),
        Some("#tool-in")
    );
}

#[test]
fn workflow_run_terms_need_the_workflow_run_context() {
    let mut crate_ = run_crate();
    assert_eq!(crate_.views::<ContainerImage>().len(), 1);
    assert_eq!(crate_.views::<ParameterConnection>().len(), 2);

    crate_.context = Context::ro_crate_1_1();

    assert!(crate_.views::<ContainerImage>().is_empty());
    assert!(crate_.views::<ParameterConnection>().is_empty());
    assert!(
        crate_.workflow().is_some(),
        "base terms still read without it"
    );
}

#[test]
fn a_testing_crate_reaches_its_suites_through_mentions() {
    let crate_ = load("nf_core_workflow.json");
    let root = crate_.root_dataset().unwrap();

    let suites = root.test_suites();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name(), Some("Test suite for nf-core/mag"));
    assert_eq!(suites[0].main_entity().map(|w| w.id()), Some("main.nf"));
    assert!(
        root.actions().is_empty(),
        "a TestSuite is not a workflow run"
    );

    let instance = suites[0].instances().remove(0);
    assert_eq!(
        instance.resource(),
        Some("repos/nf-core/mag/actions/workflows/nf-test.yml")
    );
    assert_eq!(instance.url(), Some("https://api.github.com"));

    let service = instance.runs_on().expect("no test service");
    assert_eq!(service.name(), Some("Github Actions"));
    assert_eq!(service.url(), Some("https://github.com"));
}
