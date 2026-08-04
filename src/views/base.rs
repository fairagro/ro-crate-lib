use crate::{define_views, graph::node::GraphNode, views::View, views::Workflow};

define_views! {
    RootDataset {
        types: ["Dataset"],
        terms: ["hasPart", "mainEntity"],
    }

    CreateAction {
        types: ["CreateAction"],
        terms: ["instrument", "result"],
    }

    ControlAction {
        types: ["ControlAction"],
        terms: ["instrument", "object"],
    }

    OrganizeAction {
        types: ["OrganizeAction"],
        terms: ["instrument", "object", "result"],
    }

    Person {
        types: ["Person"],
        terms: ["name"],
    }

    Organization {
        types: ["Organization"],
        terms: ["name"],
    }

    ComputerLanguage {
        types: ["ComputerLanguage"],
        terms: ["name"],
    }

    FormalParameter {
        types: ["FormalParameter"],
        terms: ["additionalType"],
    }
    SoftwareApplication {
        types: ["SoftwareApplication"],
        terms: ["name"],
    }

    HowToStep {
        types: ["HowToStep"],
        terms: ["workExample"],
    }
}

impl<'a> RootDataset<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn description(&self) -> Option<&'a str> {
        self.text("description")
    }

    pub fn date_published(&self) -> Option<&'a str> {
        self.text("datePublished")
    }

    /// `license` is a text IRI as often as it is a reference.
    pub fn license(&self) -> Option<&'a str> {
        self.text("license")
            .or_else(|| self.ref_ids("license").into_iter().next())
    }

    pub fn parts(&self) -> Vec<&'a GraphNode> {
        self.nodes("hasPart")
    }

    /// The main workflow, when the crate has one that reads as a workflow.
    pub fn main_entity(&self) -> Option<Workflow<'a>> {
        self.resolve("mainEntity")
    }

    pub fn authors(&self) -> Vec<Person<'a>> {
        self.resolve_all("author")
    }

    pub fn publisher(&self) -> Option<Organization<'a>> {
        self.resolve("publisher")
    }

    pub fn mentions(&self) -> Vec<&'a GraphNode> {
        self.nodes("mentions")
    }

    /// The runs recorded by a Workflow Run Crate, via `mentions`.
    pub fn actions(&self) -> Vec<CreateAction<'a>> {
        self.resolve_all("mentions")
    }
}

impl<'a> CreateAction<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn description(&self) -> Option<&'a str> {
        self.text("description")
    }

    /// What was run: a workflow, a step's tool, or a plain application.
    pub fn instrument(&self) -> Option<&'a GraphNode> {
        self.node_at("instrument")
    }

    /// The instrument as a workflow, when this action ran one.
    pub fn workflow(&self) -> Option<Workflow<'a>> {
        self.resolve("instrument")
    }

    pub fn objects(&self) -> Vec<&'a GraphNode> {
        self.nodes("object")
    }

    pub fn results(&self) -> Vec<&'a GraphNode> {
        self.nodes("result")
    }

    pub fn agent(&self) -> Option<&'a GraphNode> {
        self.node_at("agent")
    }

    pub fn start_time(&self) -> Option<&'a str> {
        self.text("startTime")
    }

    pub fn end_time(&self) -> Option<&'a str> {
        self.text("endTime")
    }

    pub fn error(&self) -> Option<&'a str> {
        self.text("error")
    }

    /// `containerImage`, from the Workflow Run terms.
    pub fn container_images(&self) -> Vec<&'a GraphNode> {
        self.nodes("containerImage")
    }

    pub fn resource_usage(&self) -> Vec<&'a GraphNode> {
        self.nodes("resourceUsage")
    }
}

impl<'a> ControlAction<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The step this action orchestrates.
    pub fn instrument(&self) -> Option<HowToStep<'a>> {
        self.resolve("instrument")
    }

    /// The step's runs.
    pub fn objects(&self) -> Vec<CreateAction<'a>> {
        self.resolve_all("object")
    }
}

impl<'a> OrganizeAction<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The engine that ran the workflow.
    pub fn instrument(&self) -> Option<SoftwareApplication<'a>> {
        self.resolve("instrument")
    }

    /// The step orchestrations this run performed.
    pub fn objects(&self) -> Vec<ControlAction<'a>> {
        self.resolve_all("object")
    }

    /// The workflow run this orchestration produced.
    pub fn result(&self) -> Option<CreateAction<'a>> {
        self.resolve("result")
    }

    pub fn agent(&self) -> Option<&'a GraphNode> {
        self.node_at("agent")
    }

    pub fn start_time(&self) -> Option<&'a str> {
        self.text("startTime")
    }

    pub fn end_time(&self) -> Option<&'a str> {
        self.text("endTime")
    }
}

impl<'a> Person<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn identifier(&self) -> Option<&'a str> {
        self.text("identifier")
    }

    pub fn affiliations(&self) -> Vec<Organization<'a>> {
        self.resolve_all("affiliation")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
    }
}

impl<'a> Organization<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
    }
}

impl<'a> ComputerLanguage<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn alternate_name(&self) -> Option<&'a str> {
        self.text("alternateName")
    }

    pub fn version(&self) -> Option<&'a str> {
        self.text("version")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }

    pub fn identifier(&self) -> Option<&'a str> {
        self.text("identifier")
            .or_else(|| self.ref_ids("identifier").into_iter().next())
    }
}

impl<'a> FormalParameter<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The parameter's data type, e.g. `File` or `Integer`.
    pub fn additional_type(&self) -> Option<&'a str> {
        self.text("additionalType")
    }

    pub fn default_value(&self) -> Option<&'a str> {
        self.text("defaultValue")
    }

    pub fn encoding_formats(&self) -> Vec<&'a str> {
        let mut formats = self.texts("encodingFormat");
        formats.extend(self.ref_ids("encodingFormat"));
        formats
    }

    pub fn value_required(&self) -> Option<bool> {
        self.flag("valueRequired")
    }
}

impl<'a> SoftwareApplication<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn version(&self) -> Option<&'a str> {
        self.text("version")
            .or_else(|| self.text("softwareVersion"))
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> HowToStep<'a> {
    /// The tool or subworkflow this step runs.
    pub fn work_example(&self) -> Option<&'a GraphNode> {
        self.node_at("workExample")
    }

    /// `position` is published as both a number and a string.
    pub fn position(&self) -> Option<&'a str> {
        self.text("position")
    }
}
