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
    #[must_use]
    pub fn test_suites(&self) -> Vec<TestSuite<'a>> {
        self.resolve_all("mentions")
    }
}

impl<'a> TestSuite<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The workflow under test.
    #[must_use]
    pub fn main_entity(&self) -> Option<Workflow<'a>> {
        self.resolve("mainEntity")
    }

    #[must_use]
    pub fn instances(&self) -> Vec<TestInstance<'a>> {
        self.resolve_all("instance")
    }

    #[must_use]
    pub fn definition(&self) -> Option<TestDefinition<'a>> {
        self.resolve("definition")
    }
}

impl<'a> TestInstance<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// The service this instance runs on, e.g. Jenkins or GitHub Actions.
    #[must_use]
    pub fn runs_on(&self) -> Option<TestService<'a>> {
        self.resolve("runsOn")
    }

    /// The service-relative path of the test job.
    #[must_use]
    pub fn resource(&self) -> Option<&'a str> {
        self.text("resource")
    }

    #[must_use]
    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> TestService<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    #[must_use]
    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> TestDefinition<'a> {
    /// The test engine the definition is written for, e.g. `PlanemoEngine`.
    #[must_use]
    pub fn engine(&self) -> Option<&'a str> {
        self.text("conformsTo")
            .or_else(|| self.ref_ids("conformsTo").into_iter().next())
    }

    #[must_use]
    pub fn engine_version(&self) -> Option<&'a str> {
        self.text("engineVersion")
    }
}
