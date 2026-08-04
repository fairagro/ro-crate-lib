pub const WORKFLOW_RUN_CONTEXT: &str = "https://w3id.org/ro/terms/workflow-run/context";
pub const TEST_CONTEXT: &str = "https://w3id.org/ro/terms/test/context";

pub const RO_CRATE_1_1_CONTEXT: &str = "https://w3id.org/ro/crate/1.1/context";
pub const RO_CRATE_1_2_CONTEXT: &str = "https://w3id.org/ro/crate/1.2/context";
pub const RO_CRATE_1_3_CONTEXT: &str = "https://w3id.org/ro/crate/1.3/context";

/// Profile IRIs, without their trailing version segment.
pub const RO_CRATE_PROFILE: &str = "https://w3id.org/ro/crate/";
pub const WORKFLOW_RO_CRATE_PROFILE: &str = "https://w3id.org/workflowhub/workflow-ro-crate/";
pub const PROCESS_RUN_PROFILE: &str = "https://w3id.org/ro/wfrun/process/";
pub const WORKFLOW_RUN_PROFILE: &str = "https://w3id.org/ro/wfrun/workflow/";
pub const PROVENANCE_RUN_PROFILE: &str = "https://w3id.org/ro/wfrun/provenance/";

/// The metadata descriptor's file name. `.jsonld` is the RO-Crate 1.0 spelling.
pub const METADATA_FILE_NAMES: &[&str] = &["ro-crate-metadata.json", "ro-crate-metadata.jsonld"];

pub const ROOT_ID: &str = "./";

/// Terms defined by `workflow-run/context.json`.
pub const WORKFLOW_RUN_TERMS: &[(&str, &str)] = &[
    (
        "ParameterConnection",
        "https://w3id.org/ro/terms/workflow-run#ParameterConnection",
    ),
    (
        "ContainerImage",
        "https://w3id.org/ro/terms/workflow-run#ContainerImage",
    ),
    (
        "DockerImage",
        "https://w3id.org/ro/terms/workflow-run#DockerImage",
    ),
    (
        "SIFImage",
        "https://w3id.org/ro/terms/workflow-run#SIFImage",
    ),
    (
        "connection",
        "https://w3id.org/ro/terms/workflow-run#connection",
    ),
    (
        "sourceParameter",
        "https://w3id.org/ro/terms/workflow-run#sourceParameter",
    ),
    (
        "targetParameter",
        "https://w3id.org/ro/terms/workflow-run#targetParameter",
    ),
    ("md5", "https://w3id.org/ro/terms/workflow-run#md5"),
    ("sha1", "https://w3id.org/ro/terms/workflow-run#sha1"),
    ("sha256", "https://w3id.org/ro/terms/workflow-run#sha256"),
    ("sha512", "https://w3id.org/ro/terms/workflow-run#sha512"),
    (
        "environment",
        "https://w3id.org/ro/terms/workflow-run#environment",
    ),
    (
        "registry",
        "https://w3id.org/ro/terms/workflow-run#registry",
    ),
    ("tag", "https://w3id.org/ro/terms/workflow-run#tag"),
    (
        "containerImage",
        "https://w3id.org/ro/terms/workflow-run#containerImage",
    ),
    (
        "resourceUsage",
        "https://w3id.org/ro/terms/workflow-run#resourceUsage",
    ),
];

/// Terms defined by `test/context.json`.
pub const TEST_TERMS: &[(&str, &str)] = &[
    ("TestSuite", "https://w3id.org/ro/terms/test#TestSuite"),
    (
        "TestInstance",
        "https://w3id.org/ro/terms/test#TestInstance",
    ),
    ("TestService", "https://w3id.org/ro/terms/test#TestService"),
    (
        "TestDefinition",
        "https://w3id.org/ro/terms/test#TestDefinition",
    ),
    (
        "PlanemoEngine",
        "https://w3id.org/ro/terms/test#PlanemoEngine",
    ),
    (
        "JenkinsService",
        "https://w3id.org/ro/terms/test#JenkinsService",
    ),
    (
        "TravisService",
        "https://w3id.org/ro/terms/test#TravisService",
    ),
    (
        "GithubService",
        "https://w3id.org/ro/terms/test#GithubService",
    ),
    ("instance", "https://w3id.org/ro/terms/test#instance"),
    ("runsOn", "https://w3id.org/ro/terms/test#runsOn"),
    ("resource", "https://w3id.org/ro/terms/test#resource"),
    ("definition", "https://w3id.org/ro/terms/test#definition"),
    (
        "engineVersion",
        "https://w3id.org/ro/terms/test#engineVersion",
    ),
];

/// Base RO-Crate (schema.org / Bioschemas) terms the typed views read.
pub const BASE_TERMS: &[&str] = &[
    // classes
    "Dataset",
    "File",
    "ComputationalWorkflow",
    "SoftwareSourceCode",
    "SoftwareApplication",
    "ComputerLanguage",
    "CreativeWork",
    "FormalParameter",
    "CreateAction",
    "ControlAction",
    "OrganizeAction",
    "HowTo",
    "HowToStep",
    "Person",
    "Organization",
    "PropertyValue",
    // properties
    "about",
    "additionalType",
    "affiliation",
    "agent",
    "alternateName",
    "author",
    "conformsTo",
    "contributor",
    "dateCreated",
    "dateModified",
    "datePublished",
    "defaultValue",
    "description",
    "encodingFormat",
    "endTime",
    "error",
    "exampleOfWork",
    "hasPart",
    "identifier",
    "input",
    "instrument",
    "isBasedOn",
    "keywords",
    "license",
    "mainEntity",
    "mentions",
    "name",
    "object",
    "output",
    "position",
    "programmingLanguage",
    "result",
    "sdPublisher",
    "softwareVersion",
    "startTime",
    "step",
    "subjectOf",
    "url",
    "value",
    "valueRequired",
    "version",
    "workExample",
];
