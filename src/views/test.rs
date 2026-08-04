use crate::{
    define_views,
    views::{RootDataset, View, Workflow},
};

define_views! {
    TestSuite {
        types: ["TestSuite"],
        terms: ["TestSuite", "instance"],
    }

    TestInstance {
        types: ["TestInstance"],
        terms: ["TestInstance", "runsOn", "resource"],
    }

    TestService {
        types: ["TestService"],
        terms: ["TestService"],
    }

    TestDefinition {
        types: ["TestDefinition"],
        terms: ["TestDefinition", "conformsTo", "engineVersion"],
    }
}

impl<'a> RootDataset<'a> {
    /// The test suites a workflow testing crate records, via `mentions`.
    pub fn test_suites(&self) -> Vec<TestSuite<'a>> {
        self.resolve_all("mentions")
    }
}

impl<'a> TestSuite<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The workflow under test.
    pub fn main_entity(&self) -> Option<Workflow<'a>> {
        self.resolve("mainEntity")
    }

    pub fn instances(&self) -> Vec<TestInstance<'a>> {
        self.resolve_all("instance")
    }

    pub fn definition(&self) -> Option<TestDefinition<'a>> {
        self.resolve("definition")
    }
}

impl<'a> TestInstance<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The service this instance runs on, e.g. Jenkins or GitHub Actions.
    pub fn runs_on(&self) -> Option<TestService<'a>> {
        self.resolve("runsOn")
    }

    /// The service-relative path of the test job.
    pub fn resource(&self) -> Option<&'a str> {
        self.text("resource")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> TestService<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> TestDefinition<'a> {
    /// The test engine the definition is written for, e.g. `PlanemoEngine`.
    pub fn engine(&self) -> Option<&'a str> {
        self.text("conformsTo")
            .or_else(|| self.ref_ids("conformsTo").into_iter().next())
    }

    pub fn engine_version(&self) -> Option<&'a str> {
        self.text("engineVersion")
    }
}
