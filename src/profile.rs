use crate::constants::{
    PROCESS_RUN_PROFILE, PROVENANCE_RUN_PROFILE, RO_CRATE_PROFILE, WORKFLOW_RO_CRATE_PROFILE,
    WORKFLOW_RUN_PROFILE,
};
use crate::terms::normalize;

/// A profile a crate claims to conform to, with the version it claims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Profile {
    RoCrate(String),
    WorkflowRoCrate(String),
    ProcessRun(String),
    WorkflowRun(String),
    ProvenanceRun(String),
    Other(String),
}

/// A profile's base IRI paired with the variant its versions build.
type KnownProfile = (&'static str, fn(String) -> Profile);

const KNOWN: &[KnownProfile] = &[
    (RO_CRATE_PROFILE, Profile::RoCrate),
    (WORKFLOW_RO_CRATE_PROFILE, Profile::WorkflowRoCrate),
    (PROCESS_RUN_PROFILE, Profile::ProcessRun),
    (WORKFLOW_RUN_PROFILE, Profile::WorkflowRun),
    (PROVENANCE_RUN_PROFILE, Profile::ProvenanceRun),
];

impl Profile {
    pub fn from_iri(iri: &str) -> Self {
        let candidate = normalize(iri);
        for (base, build) in KNOWN {
            let Some(version) = candidate
                .strip_prefix(normalize(base))
                .map(|rest| rest.trim_start_matches('/'))
            else {
                continue;
            };
            if !version.is_empty() && !version.contains('/') {
                return build(version.to_string());
            }
        }
        Profile::Other(iri.to_string())
    }

    /// The version segment, for the profiles that carry one.
    pub fn version(&self) -> Option<&str> {
        match self {
            Profile::Other(_) => None,
            Profile::RoCrate(v)
            | Profile::WorkflowRoCrate(v)
            | Profile::ProcessRun(v)
            | Profile::WorkflowRun(v)
            | Profile::ProvenanceRun(v) => Some(v),
        }
    }

    /// Whether this is the same profile as `other`, ignoring versions.
    pub fn is_same_profile(&self, other: &Profile) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub fn iri(&self) -> String {
        let base = match self {
            Profile::RoCrate(_) => RO_CRATE_PROFILE,
            Profile::WorkflowRoCrate(_) => WORKFLOW_RO_CRATE_PROFILE,
            Profile::ProcessRun(_) => PROCESS_RUN_PROFILE,
            Profile::WorkflowRun(_) => WORKFLOW_RUN_PROFILE,
            Profile::ProvenanceRun(_) => PROVENANCE_RUN_PROFILE,
            Profile::Other(iri) => return iri.clone(),
        };
        format!("{base}{}", self.version().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_profiles_keep_their_version() {
        assert_eq!(
            Profile::from_iri("https://w3id.org/ro/wfrun/provenance/0.5"),
            Profile::ProvenanceRun("0.5".into())
        );
        assert_eq!(
            Profile::from_iri("http://w3id.org/workflowhub/workflow-ro-crate/1.0/"),
            Profile::WorkflowRoCrate("1.0".into())
        );
    }

    #[test]
    fn context_iris_are_not_profiles() {
        let context = "https://w3id.org/ro/crate/1.1/context";
        assert_eq!(Profile::from_iri(context), Profile::Other(context.into()));
    }

    #[test]
    fn iri_round_trips() {
        for iri in [
            "https://w3id.org/ro/crate/1.1",
            "https://w3id.org/ro/wfrun/process/0.4",
            "https://example.org/some/profile",
        ] {
            assert_eq!(Profile::from_iri(iri).iri(), iri);
        }
    }
}
