use crate::{
    RoCrate,
    profile::Profile,
    validate::{Level, Violation},
};

/// RO-Crate 1.1/1.2: the metadata descriptor, the root data entity and its
/// required direct properties.
pub(super) fn check(crate_: &RoCrate, out: &mut Vec<Violation>) {
    check_descriptor(crate_, out);
    check_root(crate_, out);

    for id in crate_.graph.duplicate_ids() {
        out.push(
            Violation::must("base::duplicate-id", "@id appears more than once")
                .at(id)
                .advise("entities are identified by their @id, so each one must be unique"),
        );
    }
}

fn check_descriptor(crate_: &RoCrate, out: &mut Vec<Violation>) {
    let Some(descriptor) = crate_.descriptor() else {
        out.push(
            Violation::must("base::descriptor", "the crate has no metadata descriptor").advise(
                "add an entity with @id \"ro-crate-metadata.json\", @type \"CreativeWork\" and \
                 `about` pointing at the root data entity",
            ),
        );
        return;
    };

    if !descriptor.has_type("CreativeWork") {
        out.push(
            Violation::must(
                "base::descriptor-type",
                "the metadata descriptor is not a CreativeWork",
            )
            .at(&descriptor.id),
        );
    }

    if !descriptor
        .iris("conformsTo")
        .any(|iri| matches!(Profile::from_iri(iri), Profile::RoCrate(_)))
    {
        out.push(
            Violation::must(
                "base::descriptor-conforms-to",
                "the metadata descriptor does not state the RO-Crate version it conforms to",
            )
            .at(&descriptor.id)
            .advise("add conformsTo: {\"@id\": \"https://w3id.org/ro/crate/1.1\"}"),
        );
    }

    match descriptor.iris("about").next() {
        None => out.push(
            Violation::must(
                "base::descriptor-about",
                "the metadata descriptor does not say what it is `about`",
            )
            .at(&descriptor.id),
        ),
        Some(id) if crate_.graph.get(id).is_none() => out.push(
            Violation::must(
                "base::descriptor-about",
                format!("the metadata descriptor is `about` `{id}`, which is not in the graph"),
            )
            .at(&descriptor.id),
        ),
        Some(_) => {}
    }
}

/// Direct properties of the root data entity. Only `datePublished` is a MUST;
/// the Workflow RO-Crate profile raises `license` to one.
const ROOT_PROPERTIES: [(&str, &str, Level); 4] = [
    ("datePublished", "base::root-date-published", Level::Must),
    ("name", "base::root-name", Level::Should),
    ("description", "base::root-description", Level::Should),
    ("license", "base::root-license", Level::Should),
];

fn check_root(crate_: &RoCrate, out: &mut Vec<Violation>) {
    let Some(root) = crate_.root() else {
        out.push(
            Violation::must("base::root", "the crate has no root data entity")
                .advise("the entity the metadata descriptor is `about`, conventionally @id \"./\""),
        );
        return;
    };

    if !root.has_type("Dataset") {
        out.push(
            Violation::must("base::root-type", "the root data entity is not a Dataset")
                .at(&root.id),
        );
    }

    // RO-Crate 1.1 makes this a MUST; 1.2 relaxed it to a SHOULD, so the
    // gentler reading is used for both.
    if !root.id.ends_with('/') {
        out.push(
            Violation::should(
                "base::root-id",
                "the root data entity's @id does not end with `/`",
            )
            .at(&root.id)
            .advise("use \"./\" for a crate rooted at its own directory"),
        );
    }

    for (term, rule, level) in ROOT_PROPERTIES {
        if root.get(term).is_none() {
            let message = format!("the root data entity has no `{term}`");
            out.push(match level {
                Level::Must => Violation::must(rule, message).at(&root.id),
                Level::Should => Violation::should(rule, message).at(&root.id),
            });
        }
    }

    for id in root.iris("hasPart") {
        if crate_.graph.get(id).is_none() {
            out.push(
                Violation::should(
                    "base::dangling-part",
                    format!("`hasPart` points at `{id}`, which the graph does not describe"),
                )
                .at(&root.id)
                .advise("every part of a crate should have its own entity in the graph"),
            );
        }
    }
}

/// Reports `term` as missing from `node`, for the profile rule sets.
pub(super) fn require(
    node: &crate::graph::node::GraphNode,
    term: &str,
    level: Level,
    rule: &'static str,
    out: &mut Vec<Violation>,
) {
    if node.get(term).is_some() {
        return;
    }
    let message = format!(
        "`{}` has no `{term}`",
        node.types.iter().next().unwrap_or(&node.id)
    );
    out.push(match level {
        Level::Must => Violation::must(rule, message).at(&node.id),
        Level::Should => Violation::should(rule, message).at(&node.id),
    });
}
