use crate::{
    RoCrate,
    graph::node::GraphNode,
    validate::{Level, Violation, base::require},
};

/// The types Process Run Crate accepts for an execution.
const ACTION_TYPES: [&str; 3] = ["CreateAction", "ActivateAction", "UpdateAction"];

fn of_type<'a>(crate_: &'a RoCrate, type_name: &'a str) -> impl Iterator<Item = &'a GraphNode> {
    crate_.graph.iter().filter(move |n| n.has_type(type_name))
}

fn actions(crate_: &RoCrate) -> Vec<&GraphNode> {
    crate_
        .graph
        .iter()
        .filter(|node| ACTION_TYPES.iter().any(|t| node.has_type(t)))
        .collect()
}

/// Process Run Crate.
pub(super) fn check_process(crate_: &RoCrate, out: &mut Vec<Violation>) {
    let actions = actions(crate_);
    if actions.is_empty() {
        out.push(
            Violation::must("process::action", "the crate records no execution")
                .advise("a Process Run Crate describes at least one CreateAction"),
        );
        return;
    }

    let mentioned: Vec<&str> = crate_
        .root()
        .map(|root| root.iris("mentions").collect())
        .unwrap_or_default();

    for action in actions {
        match action.iris("instrument").next() {
            None => out.push(
                Violation::must("process::instrument", "the execution has no `instrument`")
                    .at(&action.id)
                    .advise("`instrument` identifies the tool or workflow that was run"),
            ),
            Some(id) if crate_.graph.get(id).is_none() => out.push(
                Violation::must(
                    "process::instrument",
                    format!("`instrument` points at `{id}`, which is not in the graph"),
                )
                .at(&action.id),
            ),
            Some(_) => {}
        }

        require(action, "name", Level::Should, "process::name", out);
        require(
            action,
            "description",
            Level::Should,
            "process::description",
            out,
        );
        require(action, "endTime", Level::Should, "process::end-time", out);
        require(action, "agent", Level::Should, "process::agent", out);
        require(action, "result", Level::Should, "process::result", out);

        if !mentioned.contains(&action.id.as_str()) {
            out.push(
                Violation::should(
                    "process::mentions",
                    "the root data entity does not `mention` this execution",
                )
                .at(&action.id),
            );
        }
    }
}

/// Workflow Run Crate: one of the executions must be the main workflow's.
pub(super) fn check_workflow_run(crate_: &RoCrate, out: &mut Vec<Violation>) {
    if let Some(main_id) = crate_
        .root()
        .and_then(|root| root.iris("mainEntity").next())
    {
        let ran_the_workflow = actions(crate_)
            .iter()
            .any(|action| action.iris("instrument").any(|id| id == main_id));

        if !ran_the_workflow {
            out.push(
                Violation::must(
                    "wfrun::workflow-run",
                    "no execution has the main workflow as its `instrument`",
                )
                .at(main_id)
                .advise("a Workflow Run Crate records the run of the workflow itself, not only of its steps"),
            );
        }
    }

    for parameter in of_type(crate_, "FormalParameter") {
        require(
            parameter,
            "additionalType",
            Level::Must,
            "wfrun::parameter-type",
            out,
        );
        require(
            parameter,
            "name",
            Level::Should,
            "wfrun::parameter-name",
            out,
        );
    }

    // Values that stand in for a parameter should say which one.
    let in_actions: Vec<&str> = actions(crate_)
        .iter()
        .flat_map(|action| action.iris("object").chain(action.iris("result")))
        .collect();

    for value in of_type(crate_, "PropertyValue") {
        if in_actions.contains(&value.id.as_str()) {
            require(
                value,
                "exampleOfWork",
                Level::Should,
                "wfrun::example-of-work",
                out,
            );
        }
    }
}

/// Provenance Run Crate: the engine, the steps and their executions.
pub(super) fn check_provenance(crate_: &RoCrate, out: &mut Vec<Violation>) {
    let organize: Vec<&GraphNode> = of_type(crate_, "OrganizeAction").collect();
    if organize.is_empty() {
        out.push(
            Violation::must(
                "prov::organize-action",
                "the crate records no OrganizeAction",
            )
            .advise("a Provenance Run Crate describes the engine's orchestration of the run"),
        );
    }

    for action in &organize {
        match action
            .iris("instrument")
            .next()
            .map(|id| crate_.graph.get(id))
        {
            Some(Some(engine)) if engine.has_type("SoftwareApplication") => {}
            _ => out.push(
                Violation::must(
                    "prov::engine",
                    "the OrganizeAction's `instrument` is not a SoftwareApplication in the graph",
                )
                .at(&action.id)
                .advise("`instrument` identifies the workflow engine that ran the workflow"),
            ),
        }
        require(
            action,
            "result",
            Level::Should,
            "prov::organize-result",
            out,
        );
    }

    let controls: Vec<&GraphNode> = of_type(crate_, "ControlAction").collect();
    for control in &controls {
        check_reference(
            crate_,
            control,
            "instrument",
            "HowToStep",
            "prov::control-instrument",
            out,
        );
        check_reference(
            crate_,
            control,
            "object",
            "CreateAction",
            "prov::control-object",
            out,
        );
    }

    for step in of_type(crate_, "HowToStep") {
        require(
            step,
            "workExample",
            Level::Must,
            "prov::step-work-example",
            out,
        );
    }

    if !controls.is_empty()
        && let Some(workflow) = crate_.main_entity()
        && workflow.get("step").is_none()
    {
        out.push(
            Violation::should(
                "prov::workflow-steps",
                "the main workflow has no `step`, but the crate orchestrates steps",
            )
            .at(&workflow.id)
            .advise("a workflow with recorded step executions should also be a HowTo with steps"),
        );
    }
}

/// Every value of `term` must resolve to an entity of `expected` type.
fn check_reference(
    crate_: &RoCrate,
    node: &GraphNode,
    term: &str,
    expected: &str,
    rule: &'static str,
    out: &mut Vec<Violation>,
) {
    let mut targets = node.iris(term).peekable();
    if targets.peek().is_none() {
        out.push(Violation::must(rule, format!("`{term}` is missing")).at(&node.id));
        return;
    }

    for id in targets {
        match crate_.graph.get(id) {
            Some(target) if target.has_type(expected) => {}
            Some(target) => out.push(
                Violation::must(
                    rule,
                    format!("`{term}` points at `{id}`, which is not a {expected}"),
                )
                .at(&target.id),
            ),
            None => out.push(
                Violation::must(
                    rule,
                    format!("`{term}` points at `{id}`, which is not in the graph"),
                )
                .at(&node.id),
            ),
        }
    }
}
