use crate::{
    RoCrate,
    graph::node::GraphNode,
    profile::Profile,
    validate::{Level, Violation, base::require},
};

const MAIN_WORKFLOW_TYPES: [&str; 3] = ["File", "SoftwareSourceCode", "ComputationalWorkflow"];

/// Workflow RO-Crate 1.0.
pub(super) fn check(crate_: &RoCrate, out: &mut Vec<Violation>) {
    if let Some(descriptor) = crate_.descriptor()
        && !descriptor
            .iris("conformsTo")
            .any(|iri| matches!(Profile::from_iri(iri), Profile::WorkflowRoCrate(_)))
    {
        out.push(
            Violation::should(
                "wroc::descriptor-conforms-to",
                "the metadata descriptor does not list the Workflow RO-Crate profile",
            )
            .at(&descriptor.id)
            .advise("conformsTo should hold both the RO-Crate version and https://w3id.org/workflowhub/workflow-ro-crate/1.0"),
        );
    }

    let Some(root) = crate_.root() else {
        return;
    };

    // The profile is stricter than base RO-Crate here.
    require(root, "license", Level::Must, "wroc::root-license", out);

    let Some(main_id) = root.iris("mainEntity").next() else {
        out.push(
            Violation::must("wroc::main-entity", "the crate has no main workflow")
                .at(&root.id)
                .advise("the root data entity must point at the workflow with `mainEntity`"),
        );
        return;
    };

    let Some(workflow) = crate_.graph.get(main_id) else {
        out.push(
            Violation::must(
                "wroc::main-entity",
                format!("`mainEntity` points at `{main_id}`, which is not in the graph"),
            )
            .at(&root.id),
        );
        return;
    };

    check_workflow(crate_, workflow, out);

    if !root.iris("hasPart").any(|id| id == main_id) {
        out.push(
            Violation::should(
                "wroc::main-entity-part",
                "the main workflow is not listed in the root's `hasPart`",
            )
            .at(&workflow.id),
        );
    }

    if !root
        .iris("hasPart")
        .any(|id| id.rsplit('/').next().unwrap_or(id) == "README.md")
    {
        out.push(
            Violation::should("wroc::readme", "the crate has no README.md")
                .at(&root.id)
                .advise("Workflow RO-Crate expects a text/markdown README.md at the crate root"),
        );
    }
}

fn check_workflow(crate_: &RoCrate, workflow: &GraphNode, out: &mut Vec<Violation>) {
    let missing: Vec<&str> = MAIN_WORKFLOW_TYPES
        .iter()
        .copied()
        .filter(|type_name| !workflow.has_type(type_name))
        .collect();
    if !missing.is_empty() {
        out.push(
            Violation::must(
                "wroc::main-entity-type",
                format!("the main workflow is missing @type {}", missing.join(", ")),
            )
            .at(&workflow.id)
            .advise("the main workflow must be [\"File\", \"SoftwareSourceCode\", \"ComputationalWorkflow\"]"),
        );
    }

    // Bioschemas ComputationalWorkflow, which the profile says workflows should
    // comply with.
    require(workflow, "name", Level::Should, "wroc::workflow-name", out);

    match workflow.iris("programmingLanguage").next() {
        None => out.push(
            Violation::must(
                "wroc::programming-language",
                "the main workflow has no `programmingLanguage`",
            )
            .at(&workflow.id),
        ),
        Some(id) => match crate_.graph.get(id) {
            Some(language) if language.has_type("ComputerLanguage") => {}
            Some(language) => out.push(
                Violation::must(
                    "wroc::programming-language",
                    "`programmingLanguage` does not point at a ComputerLanguage",
                )
                .at(&language.id),
            ),
            None => out.push(
                Violation::must(
                    "wroc::programming-language",
                    format!("`programmingLanguage` points at `{id}`, which is not in the graph"),
                )
                .at(&workflow.id),
            ),
        },
    }

    check_attachment(
        crate_,
        workflow,
        "image",
        &["File", "ImageObject"],
        "wroc::image-type",
        out,
    );
    check_attachment(
        crate_,
        workflow,
        "subjectOf",
        &["File", "SoftwareSourceCode", "HowTo"],
        "wroc::subject-of-type",
        out,
    );
}

/// The diagram and the abstract CWL are optional, but typed exactly when present.
fn check_attachment(
    crate_: &RoCrate,
    workflow: &GraphNode,
    term: &str,
    types: &[&str],
    rule: &'static str,
    out: &mut Vec<Violation>,
) {
    for id in workflow.iris(term) {
        let Some(node) = crate_.graph.get(id) else {
            continue;
        };
        if !node.has_types(types) {
            out.push(
                Violation::must(
                    rule,
                    format!("`{term}` should be @type {:?}, not {:?}", types, node.types),
                )
                .at(&node.id),
            );
        }
    }
}
