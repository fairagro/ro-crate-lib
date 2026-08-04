use crate::{
    define_views,
    graph::node::GraphNode,
    views::{Person, RootDataset, View},
};

define_views! {
    Workflow {
        types: ["ComputationalWorkflow"],
        terms: ["programmingLanguage", "input", "output"],
    }

    ComputerLanguage {
        types: ["ComputerLanguage"],
        terms: ["name"],
    }

    FormalParameter {
        types: ["FormalParameter"],
        terms: ["additionalType"],
    }
}

impl<'a> RootDataset<'a> {
    /// The main workflow of a Workflow RO-Crate.
    #[must_use]
    pub fn main_entity(&self) -> Option<Workflow<'a>> {
        self.resolve("mainEntity")
    }
}

impl<'a> Workflow<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    #[must_use]
    pub fn description(&self) -> Option<&'a str> {
        self.text("description")
    }

    #[must_use]
    pub fn version(&self) -> Option<&'a str> {
        self.text("version")
            .or_else(|| self.text("softwareVersion"))
    }

    #[must_use]
    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }

    #[must_use]
    pub fn language(&self) -> Option<ComputerLanguage<'a>> {
        self.resolve("programmingLanguage")
    }

    #[must_use]
    pub fn inputs(&self) -> Vec<FormalParameter<'a>> {
        self.resolve_all("input")
    }

    #[must_use]
    pub fn outputs(&self) -> Vec<FormalParameter<'a>> {
        self.resolve_all("output")
    }

    #[must_use]
    pub fn authors(&self) -> Vec<Person<'a>> {
        self.resolve_all("author")
    }

    #[must_use]
    pub fn license(&self) -> Option<&'a str> {
        self.text("license")
            .or_else(|| self.ref_ids("license").into_iter().next())
    }

    /// Subworkflows and packed tool definitions.
    #[must_use]
    pub fn parts(&self) -> Vec<&'a GraphNode> {
        self.nodes("hasPart")
    }

    #[must_use]
    pub fn subworkflows(&self) -> Vec<Workflow<'a>> {
        self.resolve_all("hasPart")
    }

    /// The abstract CWL or diagram a Workflow RO-Crate attaches to its workflow.
    #[must_use]
    pub fn subject_of(&self) -> Vec<&'a GraphNode> {
        self.nodes("subjectOf")
    }

    #[must_use]
    pub fn is_based_on(&self) -> Vec<&'a GraphNode> {
        self.nodes("isBasedOn")
    }
}

impl<'a> ComputerLanguage<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    #[must_use]
    pub fn alternate_name(&self) -> Option<&'a str> {
        self.text("alternateName")
    }

    #[must_use]
    pub fn version(&self) -> Option<&'a str> {
        self.text("version")
    }

    #[must_use]
    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }

    #[must_use]
    pub fn identifier(&self) -> Option<&'a str> {
        self.text("identifier")
            .or_else(|| self.ref_ids("identifier").into_iter().next())
    }
}

impl<'a> FormalParameter<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The parameter's data type: a schema.org name like `File`, or a
    /// reference to an ontology term, as Bioschemas crates write it.
    #[must_use]
    pub fn additional_type(&self) -> Option<&'a str> {
        self.text("additionalType")
            .or_else(|| self.ref_ids("additionalType").into_iter().next())
    }

    #[must_use]
    pub fn default_value(&self) -> Option<&'a str> {
        self.text("defaultValue")
    }

    #[must_use]
    pub fn encoding_formats(&self) -> Vec<&'a str> {
        let mut formats = self.texts("encodingFormat");
        formats.extend(self.ref_ids("encodingFormat"));
        formats
    }

    #[must_use]
    pub fn value_required(&self) -> Option<bool> {
        self.flag("valueRequired")
    }
}
