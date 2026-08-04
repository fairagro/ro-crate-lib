use crate::{
    define_views,
    graph::node::GraphNode,
    views::{FormalParameter, PropertyValue, RootDataset, View, Workflow},
};

define_views! {
    /// One execution: of a workflow, of a step's tool, or of a plain
    /// application in a Process Run Crate.
    CreateAction {
        types: ["CreateAction"],
        terms: ["instrument", "result"],
    }

    /// The orchestration of a single workflow step.
    ControlAction {
        types: ["ControlAction"],
        terms: ["instrument", "object"],
    }

    /// The engine's run of the whole workflow.
    OrganizeAction {
        types: ["OrganizeAction"],
        terms: ["instrument", "object", "result"],
    }

    HowToStep {
        types: ["HowToStep"],
        terms: ["workExample"],
    }

    SoftwareApplication {
        types: ["SoftwareApplication"],
        terms: ["name"],
    }

    ContainerImage {
        types: ["ContainerImage"],
        terms: ["ContainerImage", "registry", "tag"],
    }

    ParameterConnection {
        types: ["ParameterConnection"],
        terms: ["ParameterConnection", "sourceParameter", "targetParameter"],
    }
}

impl<'a> RootDataset<'a> {
    /// The runs recorded by a Workflow Run Crate, via `mentions`.
    pub fn actions(&self) -> Vec<CreateAction<'a>> {
        self.resolve_all("mentions")
    }
}

impl<'a> Workflow<'a> {
    /// The steps, when the workflow is also a `HowTo`.
    pub fn steps(&self) -> Vec<HowToStep<'a>> {
        self.resolve_all("step")
    }

    /// Connections into the workflow's own outputs. Connections between steps
    /// hang off the steps themselves.
    pub fn connections(&self) -> Vec<ParameterConnection<'a>> {
        self.resolve_all("connection")
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

    /// The instrument as an application, in a Process Run Crate.
    pub fn application(&self) -> Option<SoftwareApplication<'a>> {
        self.resolve("instrument")
    }

    pub fn objects(&self) -> Vec<&'a GraphNode> {
        self.nodes("object")
    }

    pub fn results(&self) -> Vec<&'a GraphNode> {
        self.nodes("result")
    }

    /// The inputs that are parameter values rather than files.
    pub fn object_values(&self) -> Vec<PropertyValue<'a>> {
        self.resolve_all("object")
    }

    pub fn result_values(&self) -> Vec<PropertyValue<'a>> {
        self.resolve_all("result")
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

    pub fn container_images(&self) -> Vec<ContainerImage<'a>> {
        self.resolve_all("containerImage")
    }

    /// Environment variables the execution ran with.
    pub fn environment(&self) -> Vec<PropertyValue<'a>> {
        self.resolve_all("environment")
    }

    pub fn resource_usage(&self) -> Vec<PropertyValue<'a>> {
        self.resolve_all("resourceUsage")
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

    pub fn error(&self) -> Option<&'a str> {
        self.text("error")
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

impl<'a> HowToStep<'a> {
    /// The tool or subworkflow this step runs.
    pub fn work_example(&self) -> Option<&'a GraphNode> {
        self.node_at("workExample")
    }

    /// `position` is published as both a number and a string.
    pub fn position(&self) -> Option<&'a str> {
        self.text("position")
    }

    /// The parameter connections feeding this step.
    pub fn connections(&self) -> Vec<ParameterConnection<'a>> {
        self.resolve_all("connection")
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

impl<'a> ContainerImage<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn registry(&self) -> Option<&'a str> {
        self.text("registry")
    }

    pub fn tag(&self) -> Option<&'a str> {
        self.text("tag")
    }

    /// `DockerImage` or `SIFImage`, as the IRI the crate points at.
    pub fn kind(&self) -> Option<&'a str> {
        self.text("additionalType")
            .or_else(|| self.ref_ids("additionalType").into_iter().next())
    }

    pub fn md5(&self) -> Option<&'a str> {
        self.text("md5")
    }

    pub fn sha1(&self) -> Option<&'a str> {
        self.text("sha1")
    }

    pub fn sha256(&self) -> Option<&'a str> {
        self.text("sha256")
    }

    pub fn sha512(&self) -> Option<&'a str> {
        self.text("sha512")
    }
}

impl<'a> ParameterConnection<'a> {
    pub fn source(&self) -> Option<FormalParameter<'a>> {
        self.resolve("sourceParameter")
    }

    pub fn target(&self) -> Option<FormalParameter<'a>> {
        self.resolve("targetParameter")
    }

    /// The endpoints as they stand, for crates that connect something other
    /// than a `FormalParameter`.
    pub fn source_node(&self) -> Option<&'a GraphNode> {
        self.node_at("sourceParameter")
    }

    pub fn target_node(&self) -> Option<&'a GraphNode> {
        self.node_at("targetParameter")
    }
}
