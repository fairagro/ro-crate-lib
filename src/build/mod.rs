//! Building crates term by term, with the context kept in step.
//!
//! Every entity that goes in is scanned for terms the context does not cover
//! yet, and the context document that defines them is referenced. Adding a
//! `ContainerImage` therefore pulls in the Workflow Run terms on its own.
//!
//! ```
//! use rocrate::{RoCrate, build::Entity, profile::Profile, views::View};
//!
//! let crate_ = RoCrate::builder()
//!     .date_published("2026-01-01")
//!     .name("Example")
//!     .license("MIT")
//!     .conforms_to(Profile::WorkflowRoCrate("1.0".into()))
//!     .main_workflow(
//!         Entity::new("wf.cwl", &["File", "SoftwareSourceCode", "ComputationalWorkflow"])
//!             .set("name", "Example workflow")
//!             .reference("programmingLanguage", "#cwl"),
//!     )
//!     .entity(Entity::new("#cwl", "ComputerLanguage").set("name", "CWL"))
//!     .build();
//!
//! assert_eq!(crate_.workflow().unwrap().id(), "wf.cwl");
//! ```

use crate::{
    RoCrate,
    constants::{METADATA_FILE_NAMES, ROOT_ID},
    context::{Context, ContextItem},
    graph::{
        Graph,
        node::{GraphNode, Reference, Value},
    },
    profile::Profile,
    terms,
    validate::InvalidCrate,
};
use oneormany::OneOrMany;

/// One entity, built up before it goes into a crate.
///
/// Entities carry arbitrary terms, so this stays a plain map rather than a
/// generated builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    node: GraphNode,
}

impl Entity {
    pub fn new(id: impl Into<String>, types: impl Into<OneOrMany<String>>) -> Self {
        Entity {
            node: GraphNode::new(id, types),
        }
    }

    pub fn set(mut self, term: impl Into<String>, value: impl Into<Value>) -> Self {
        self.node.set(term, value);
        self
    }

    /// Points `term` at another entity's `@id`.
    pub fn reference(mut self, term: impl Into<String>, id: impl Into<String>) -> Self {
        self.node.set(term, Reference::new(id));
        self
    }

    pub fn references<I>(mut self, term: impl Into<String>, ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let references: Vec<Reference> = ids.into_iter().map(Reference::new).collect();
        self.node.set(term, references);
        self
    }
}

impl From<Entity> for GraphNode {
    fn from(entity: Entity) -> Self {
        entity.node
    }
}

impl From<GraphNode> for Entity {
    fn from(node: GraphNode) -> Self {
        Entity { node }
    }
}

/// How an entity relates to the root data entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Described by the crate, but not part of it.
    Contextual,
    /// Listed in the root's `hasPart`.
    Part,
    /// A part the root also points at with `mainEntity`.
    MainWorkflow,
    /// Listed in the root's `mentions`, as the run profiles record executions.
    Mentioned,
}

/// An entity waiting to be placed in the graph.
#[derive(Debug, Clone)]
pub struct Addition {
    node: GraphNode,
    role: Role,
}

#[bon::bon]
impl RoCrate {
    /// Builds a crate around its root data entity.
    ///
    /// `date_published` is the one direct property RO-Crate requires of a root,
    /// so it is required here too — the crate does not compile without it.
    #[builder(
        builder_type = RoCrateBuilder,
        state_mod = ro_crate_builder,
        finish_fn = build
    )]
    pub fn builder(
        #[builder(field)] additions: Vec<Addition>,
        #[builder(field)] profiles: Vec<Profile>,
        #[builder(default = Context::default())] context: Context,
        #[builder(into)] date_published: Value,
        #[builder(into)] name: Option<Value>,
        #[builder(into)] description: Option<Value>,
        #[builder(into)] license: Option<Value>,
    ) -> RoCrate {
        let mut context = context;
        let descriptor = GraphNode::new(METADATA_FILE_NAMES[0], "CreativeWork");
        let mut graph = Graph::from(vec![descriptor, GraphNode::new(ROOT_ID, "Dataset")]);

        for (term, value) in [
            ("datePublished", Some(date_published)),
            ("name", name),
            ("description", description),
            ("license", license),
        ] {
            if let Some(value) = value {
                root_of(&mut graph).set(term, value);
            }
        }

        for Addition { node, role } in additions {
            ensure_terms(&mut context, &node);
            let id = node.id.clone();
            graph.insert(node);

            let root = root_of(&mut graph);
            match role {
                Role::Contextual => {}
                Role::Part => {
                    root.push_ref("hasPart", Reference::new(id));
                }
                Role::MainWorkflow => {
                    root.push_ref("hasPart", Reference::new(&id));
                    root.set("mainEntity", Reference::new(id));
                }
                Role::Mentioned => {
                    root.push_ref("mentions", Reference::new(id));
                }
            }
        }

        let (on_descriptor, on_root) = sort_profiles(&profiles, &context);
        if !on_descriptor.is_empty() {
            graph
                .get_mut(METADATA_FILE_NAMES[0])
                .expect("the descriptor was just inserted")
                .set("conformsTo", on_descriptor);
        }
        if !on_root.is_empty() {
            root_of(&mut graph).set("conformsTo", on_root);
        }
        graph
            .get_mut(METADATA_FILE_NAMES[0])
            .expect("the descriptor was just inserted")
            .set("about", Reference::new(ROOT_ID));

        ensure_all_terms(&mut context, &graph);
        RoCrate { context, graph }
    }
}

impl<S: ro_crate_builder::State> RoCrateBuilder<S> {
    /// Claims a profile. Base and Workflow RO-Crate profiles land on the
    /// metadata descriptor, the run profiles on the root data entity, which is
    /// where each spec looks for them.
    pub fn conforms_to(mut self, profile: Profile) -> Self {
        if !self.profiles.contains(&profile) {
            self.profiles.push(profile);
        }
        self
    }

    /// Adds a contextual entity: something the crate describes but does not
    /// contain.
    pub fn entity(self, entity: impl Into<Entity>) -> Self {
        self.add(entity, Role::Contextual)
    }

    /// Adds a data entity and lists it in the root's `hasPart`.
    pub fn part(self, entity: impl Into<Entity>) -> Self {
        self.add(entity, Role::Part)
    }

    /// Adds the main workflow: a part the root also points at with
    /// `mainEntity`.
    pub fn main_workflow(self, entity: impl Into<Entity>) -> Self {
        self.add(entity, Role::MainWorkflow)
    }

    /// Adds an entity the root `mentions`, which is how the run profiles hang
    /// executions off a crate.
    pub fn mention(self, entity: impl Into<Entity>) -> Self {
        self.add(entity, Role::Mentioned)
    }

    fn add(mut self, entity: impl Into<Entity>, role: Role) -> Self {
        self.additions.push(Addition {
            node: entity.into().into(),
            role,
        });
        self
    }
}

impl<S: ro_crate_builder::IsComplete> RoCrateBuilder<S> {
    /// Builds, then holds the crate to the profiles it claims.
    pub fn build_checked(self) -> Result<RoCrate, InvalidCrate> {
        let crate_ = self.build();
        crate_.validate().into_result()?;
        Ok(crate_)
    }
}

fn root_of(graph: &mut Graph) -> &mut GraphNode {
    graph
        .get_mut(ROOT_ID)
        .expect("the builder always keeps a root data entity")
}

/// The descriptor states the RO-Crate version and the Workflow RO-Crate
/// profile; the root states the run profiles.
fn sort_profiles(profiles: &[Profile], context: &Context) -> (Vec<Reference>, Vec<Reference>) {
    let mut on_descriptor: Vec<Reference> = Vec::new();
    let mut on_root: Vec<Reference> = Vec::new();

    for profile in profiles {
        let reference = Reference::new(profile.iri());
        match profile {
            Profile::RoCrate(_) | Profile::WorkflowRoCrate(_) | Profile::Other(_) => {
                on_descriptor.push(reference)
            }
            Profile::ProcessRun(_) | Profile::WorkflowRun(_) | Profile::ProvenanceRun(_) => {
                on_root.push(reference)
            }
        }
    }

    // The version the crate conforms to is the version of the context it uses,
    // so the two cannot drift apart.
    if !profiles
        .iter()
        .any(|profile| matches!(profile, Profile::RoCrate(_)))
        && let Some(version) = context_version(context)
    {
        on_descriptor.insert(0, Reference::new(Profile::RoCrate(version).iri()));
    }
    (on_descriptor, on_root)
}

fn context_version(context: &Context) -> Option<String> {
    context.items().find_map(|item| match item {
        ContextItem::Reference(iri) => terms::base_context_version(iri).map(str::to_string),
        ContextItem::Definitions(_) => None,
    })
}

fn ensure_all_terms(context: &mut Context, graph: &Graph) {
    for node in graph.iter() {
        ensure_terms(context, node);
    }
}

/// Pulls in the context documents an entity's types and terms need.
fn ensure_terms(context: &mut Context, node: &GraphNode) {
    for type_name in node.types.iter() {
        context.ensure(type_name);
    }
    for (term, value) in &node.properties {
        context.ensure(term);
        ensure_nested(context, value);
    }
}

/// Terms hidden inside nested objects need their context too.
fn ensure_nested(context: &mut Context, value: &Value) {
    match value {
        Value::Object(fields) => {
            for (term, nested) in fields {
                context.ensure(term);
                ensure_nested(context, nested);
            }
        }
        Value::List(items) => items.iter().for_each(|item| ensure_nested(context, item)),
        _ => {}
    }
}
