use crate::{
    define_views,
    graph::node::GraphNode,
    views::{ComputerLanguage, FormalParameter, HowToStep, Person, View},
};

define_views! {
    Workflow {
        types: ["ComputationalWorkflow"],
        terms: ["programmingLanguage", "input", "output"],
    }
}

impl<'a> Workflow<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn description(&self) -> Option<&'a str> {
        self.text("description")
    }

    pub fn version(&self) -> Option<&'a str> {
        self.text("version")
            .or_else(|| self.text("softwareVersion"))
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }

    pub fn language(&self) -> Option<ComputerLanguage<'a>> {
        self.resolve("programmingLanguage")
    }

    pub fn inputs(&self) -> Vec<FormalParameter<'a>> {
        self.resolve_all("input")
    }

    pub fn outputs(&self) -> Vec<FormalParameter<'a>> {
        self.resolve_all("output")
    }

    /// The steps, when the workflow is also a `HowTo`.
    pub fn steps(&self) -> Vec<HowToStep<'a>> {
        self.resolve_all("step")
    }

    pub fn authors(&self) -> Vec<Person<'a>> {
        self.resolve_all("author")
    }

    pub fn license(&self) -> Option<&'a str> {
        self.text("license")
            .or_else(|| self.ref_ids("license").into_iter().next())
    }

    /// Subworkflows and packed tool definitions.
    pub fn parts(&self) -> Vec<&'a GraphNode> {
        self.nodes("hasPart")
    }

    pub fn subworkflows(&self) -> Vec<Workflow<'a>> {
        self.resolve_all("hasPart")
    }

    /// The abstract CWL or diagram a Workflow RO-Crate attaches to its workflow.
    pub fn subject_of(&self) -> Vec<&'a GraphNode> {
        self.nodes("subjectOf")
    }

    pub fn is_based_on(&self) -> Vec<&'a GraphNode> {
        self.nodes("isBasedOn")
    }
}
