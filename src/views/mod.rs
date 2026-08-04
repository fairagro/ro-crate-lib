use crate::{
    RoCrate,
    graph::node::{GraphNode, Value},
};

mod base;
mod workflow;
pub use base::{
    ComputerLanguage, ControlAction, CreateAction, FormalParameter, HowToStep, Organization,
    OrganizeAction, Person, RootDataset, SoftwareApplication,
};
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
