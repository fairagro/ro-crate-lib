use crate::{
    define_views,
    graph::node::{GraphNode, Value},
    views::View,
};

define_views! {
    RootDataset {
        types: ["Dataset"],
        terms: ["hasPart", "mainEntity"],
    }

    Person {
        types: ["Person"],
        terms: ["name"],
    }

    Organization {
        types: ["Organization"],
        terms: ["name"],
    }

    PropertyValue {
        types: ["PropertyValue"],
        terms: ["value"],
    }
}

impl<'a> RootDataset<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    #[must_use]
    pub fn description(&self) -> Option<&'a str> {
        self.text("description")
    }

    #[must_use]
    pub fn date_published(&self) -> Option<&'a str> {
        self.text("datePublished")
    }

    /// `license` is a text IRI as often as it is a reference.
    #[must_use]
    pub fn license(&self) -> Option<&'a str> {
        self.text("license")
            .or_else(|| self.ref_ids("license").into_iter().next())
    }

    #[must_use]
    pub fn parts(&self) -> Vec<&'a GraphNode> {
        self.nodes("hasPart")
    }

    #[must_use]
    pub fn authors(&self) -> Vec<Person<'a>> {
        self.resolve_all("author")
    }

    #[must_use]
    pub fn publisher(&self) -> Option<Organization<'a>> {
        self.resolve("publisher")
    }

    #[must_use]
    pub fn mentions(&self) -> Vec<&'a GraphNode> {
        self.nodes("mentions")
    }
}

impl<'a> Person<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    #[must_use]
    pub fn identifier(&self) -> Option<&'a str> {
        self.text("identifier")
            .or_else(|| self.ref_ids("identifier").into_iter().next())
    }

    #[must_use]
    pub fn affiliations(&self) -> Vec<Organization<'a>> {
        self.resolve_all("affiliation")
    }

    #[must_use]
    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> Organization<'a> {
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

impl<'a> PropertyValue<'a> {
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// Values are published as text, numbers and booleans alike.
    #[must_use]
    pub fn value(&self) -> Option<&'a Value> {
        self.get("value")
    }

    #[must_use]
    pub fn text_value(&self) -> Option<&'a str> {
        self.text("value")
    }

    /// What this value is an instance of — a workflow parameter, in a
    /// Workflow Run Crate.
    #[must_use]
    pub fn example_of_work(&self) -> Option<&'a GraphNode> {
        self.node_at("exampleOfWork")
    }
}
