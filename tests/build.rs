use rocrate::{
    RoCrate,
    build::Entity,
    context::Context,
    graph::node::Value,
    profile::Profile,
    views::{ContainerImage, CreateAction, TestSuite},
};

fn workflow() -> Entity {
    Entity::new(
        "wf.cwl",
        &["File", "SoftwareSourceCode", "ComputationalWorkflow"],
    )
    .set("name", "Example workflow")
    .reference("programmingLanguage", "#cwl")
}

fn language() -> Entity {
    Entity::new("#cwl", "ComputerLanguage").set("name", "CWL")
}

/// A Workflow RO-Crate that breaks no rule, built from nothing.
fn workflow_crate() -> RoCrate {
    RoCrate::builder()
        .date_published("2026-01-01")
        .name("Example")
        .description("Built by the builder")
        .license("MIT")
        .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
        .main_workflow(workflow())
        .entity(language())
        .part(Entity::new("README.md", "File").set("encodingFormat", "text/markdown"))
        .build()
}

#[test]
fn a_built_crate_has_a_descriptor_and_a_root() {
    let crate_ = workflow_crate();
    let descriptor = crate_.descriptor().expect("no descriptor");
    let root = crate_.root().expect("no root");

    assert_eq!(descriptor.id, "ro-crate-metadata.json");
    assert!(descriptor.has_type("CreativeWork"));
    assert_eq!(descriptor.iris("about").next(), Some("./"));
    assert_eq!(root.id, "./");
    assert!(root.has_type("Dataset"));
    assert_eq!(root.text("name"), Some("Example"));
    assert_eq!(root.text("datePublished"), Some("2026-01-01"));
    assert!(
        root.get("about").is_none(),
        "`about` belongs to the descriptor alone"
    );
}

#[test]
fn what_the_builder_makes_passes_validation() {
    let validation = workflow_crate().validate();

    assert_eq!(
        validation.violations,
        [],
        "expected a clean crate, got {:#?}",
        validation.violations
    );
    assert!(
        RoCrate::builder()
            .date_published("2026-01-01")
            .license("MIT")
            .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
            .main_workflow(workflow())
            .entity(language())
            .part(Entity::new("README.md", "File"))
            .build_checked()
            .is_ok()
    );
}

#[test]
fn build_checked_reports_what_the_profile_wants() {
    // No main workflow, though the crate claims Workflow RO-Crate.
    let report = RoCrate::builder()
        .date_published("2026-01-01")
        .license("MIT")
        .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
        .build_checked()
        .unwrap_err();

    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.rule == "wroc::main-entity")
    );
}

#[test]
fn the_version_of_conforms_to_follows_the_context() {
    let crate_ = RoCrate::builder()
        .context(Context::ro_crate_1_1())
        .date_published("2026-01-01")
        .build();

    assert_eq!(crate_.profiles(), [Profile::RoCrate("1.1".into())]);

    let stated = RoCrate::builder()
        .context(Context::ro_crate_1_1())
        .date_published("2026-01-01")
        .conforms_to(Profile::RoCrate("1.2".into()))
        .build();

    assert_eq!(
        stated.profiles(),
        [Profile::RoCrate("1.2".into())],
        "an explicit version wins"
    );
}

#[test]
fn profiles_land_where_each_spec_looks_for_them() {
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
        .conforms_to(Profile::ProvenanceRun("0.5".into()))
        .build();

    let on_descriptor: Vec<&str> = crate_.descriptor().unwrap().iris("conformsTo").collect();
    let on_root: Vec<&str> = crate_.root().unwrap().iris("conformsTo").collect();

    assert!(on_descriptor.contains(&"https://w3id.org/workflowhub/workflow-ro-crate/1.0"));
    assert!(
        on_descriptor
            .iter()
            .any(|iri| iri.starts_with("https://w3id.org/ro/crate/"))
    );
    assert_eq!(on_root, ["https://w3id.org/ro/wfrun/provenance/0.5"]);
}

#[test]
fn workflow_run_terms_pull_in_their_context() {
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .mention(
            Entity::new("#run", "CreateAction")
                .reference("instrument", "wf.cwl")
                .reference("containerImage", "#image"),
        )
        .entity(
            Entity::new("#image", "ContainerImage")
                .set("registry", "docker.io")
                .set("tag", "v1.9"),
        )
        .part(workflow())
        .build();

    assert!(crate_.context.defines("containerImage"));
    assert!(crate_.context.defines("ContainerImage"));
    assert_eq!(crate_.views::<ContainerImage>().len(), 1);
    assert_eq!(
        crate_.views::<CreateAction>()[0].container_images()[0].registry(),
        Some("docker.io")
    );
}

#[test]
fn the_test_context_arrives_the_same_way() {
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .mention(
            Entity::new("#suite", "TestSuite")
                .set("name", "Test suite")
                .reference("instance", "#instance"),
        )
        .entity(
            Entity::new("#instance", "TestInstance")
                .set("resource", "repos/example/actions")
                .reference("runsOn", "https://w3id.org/ro/terms/test#GithubService"),
        )
        .build();

    assert!(crate_.context.defines("TestSuite"));
    assert_eq!(
        crate_.views::<TestSuite>()[0].instances()[0].resource(),
        Some("repos/example/actions")
    );
}

#[test]
fn a_context_that_already_covers_a_term_is_left_alone() {
    let mut context = Context::ro_crate_1_2();
    context.define("containerImage", "https://example.org/containerImage");

    let crate_ = RoCrate::builder()
        .context(context)
        .date_published("2026-01-01")
        .entity(Entity::new("#image", "File").set("containerImage", "docker.io/example"))
        .build();

    assert_eq!(
        crate_.context.definition("containerImage"),
        Some("https://example.org/containerImage")
    );
    assert_eq!(
        crate_.context.items().count(),
        2,
        "no extra context document was pulled in"
    );
}

#[test]
fn base_terms_never_pull_in_anything() {
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .name("Example")
        .part(workflow())
        .entity(language())
        .build();

    assert_eq!(crate_.context.items().count(), 1);
}

#[test]
fn entities_carry_lists_numbers_and_references() {
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .entity(
            Entity::new("#step", "HowToStep")
                .set("position", 1i64)
                .references("connection", ["#a", "#b"])
                .reference("workExample", "#tool"),
        )
        .build();

    let step = crate_.graph.get("#step").unwrap();
    assert_eq!(step.get("position"), Some(&Value::Number(1.into())));
    assert_eq!(step.iris("connection").collect::<Vec<_>>(), ["#a", "#b"]);
    assert_eq!(step.iris("workExample").next(), Some("#tool"));
}

#[test]
fn a_built_crate_round_trips_through_json() {
    let crate_ = workflow_crate();
    let json = serde_json::to_string_pretty(&crate_).unwrap();
    let reparsed: RoCrate = serde_json::from_str(&json).unwrap();

    assert_eq!(crate_, reparsed);
    assert!(json.contains(r#""@id": "./""#));
    assert!(json.contains(r#""@context""#));
}

#[test]
fn adding_the_same_id_twice_replaces_it() {
    let crate_ = RoCrate::builder()
        .date_published("2026-01-01")
        .part(Entity::new("wf.cwl", "File").set("name", "first"))
        .part(Entity::new("wf.cwl", "File").set("name", "second"))
        .build();

    assert_eq!(
        crate_.graph.get("wf.cwl").unwrap().text("name"),
        Some("second")
    );
    assert!(crate_.graph.duplicate_ids().is_empty());
}
