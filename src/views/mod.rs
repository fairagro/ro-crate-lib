use crate::{
    RoCrate,
    graph::node::{GraphNode, Value},
};

mod base;
mod test;
mod workflow;
pub use base::{
    ComputerLanguage, ControlAction, CreateAction, FormalParameter, HowToStep, Organization,
    OrganizeAction, Person, RootDataset, SoftwareApplication,
};
pub use test::{TestInstance, TestService, TestSuite};
pub use workflow::Workflow;

pub trait View<'a>: Sized {
    const TYPES: &'static [&'static str];
    const REQUIRED: &'static [&'static str];

    fn try_new(crate_: &'a RoCrate, entity: &'a GraphNode) -> Option<Self>;
    fn rocrate(&self) -> &'a RoCrate;
    fn node(&self) -> &'a GraphNode;

    fn id(&self) -> &'a str {
        &self.node().id
    }

    fn get(&self, term: &str) -> Option<&'a Value> {
        self.rocrate()
            .context
            .defines(term)
            .then(|| self.node().get(term))
            .flatten()
    }

    fn text(&self, term: &str) -> Option<&'a str> {
        self.get(term)?.as_str()
    }

    fn flag(&self, term: &str) -> Option<bool> {
        self.get(term)?.as_bool()
    }

    fn texts(&self, term: &str) -> Vec<&'a str> {
        self.get(term)
            .into_iter()
            .flat_map(Value::strings)
            .collect()
    }

    fn ref_ids(&self, term: &str) -> Vec<&'a str> {
        self.get(term).into_iter().flat_map(Value::refs).collect()
    }

    fn nodes(&self, term: &str) -> Vec<&'a GraphNode> {
        self.ref_ids(term)
            .into_iter()
            .filter_map(|i| self.rocrate().graph.get(i))
            .collect()
    }

    fn node_at(&self, term: &str) -> Option<&'a GraphNode> {
        self.nodes(term).into_iter().next()
    }

    fn resolve<V: View<'a>>(&self, term: &str) -> Option<V> {
        self.resolve_all(term).into_iter().next()
    }

    fn resolve_all<V: View<'a>>(&self, term: &str) -> Vec<V> {
        self.nodes(term)
            .into_iter()
            .filter_map(|entity| V::try_new(self.rocrate(), entity))
            .collect()
    }
}

impl RoCrate {
    /// A typed view of `id`, when that entity carries the view's types and the
    /// context defines the terms the view reads.
    pub fn view<'a, V: View<'a>>(&'a self, id: &str) -> Option<V> {
        V::try_new(self, self.graph.get(id)?)
    }

    /// Every entity the view applies to, in document order.
    pub fn views<'a, V: View<'a>>(&'a self) -> Vec<V> {
        self.graph
            .iter()
            .filter_map(|node| V::try_new(self, node))
            .collect()
    }

    pub fn root_dataset(&self) -> Option<RootDataset<'_>> {
        RootDataset::try_new(self, self.root()?)
    }

    /// The main workflow of a Workflow RO-Crate.
    pub fn workflow(&self) -> Option<Workflow<'_>> {
        Workflow::try_new(self, self.main_entity()?)
    }
}

/// Defines a view struct plus its [`View`] impl.
#[macro_export]
macro_rules! define_views {
    ($(
        $(#[$meta:meta])*
        $name:ident {
            types: [$($type_name:literal),* $(,)?],
            terms: [$($term:literal),* $(,)?] $(,)?
        }
    )*) => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name<'a> {
            rocrate: &'a $crate::RoCrate,
            node: &'a $crate::graph::node::GraphNode,
        }

        impl<'a> $crate::views::View<'a> for $name<'a> {
            const TYPES: &'static [&'static str] = &[$($type_name),*];
            const REQUIRED: &'static [&'static str] = &[$($term),*];

            fn try_new(
                rocrate: &'a $crate::RoCrate,
                node: &'a $crate::graph::node::GraphNode,
            ) -> Option<Self> {
                let typed = node.has_types(Self::TYPES);
                let defined = Self::REQUIRED
                    .iter()
                    .all(|term| rocrate.context.defines(term));
                (typed && defined).then_some(Self { rocrate, node})
            }

            fn rocrate(&self) -> &'a $crate::RoCrate {
                self.rocrate
            }

            fn node(&self) -> &'a $crate::graph::node::GraphNode {
                self.node
            }
        }
    )*};
}
