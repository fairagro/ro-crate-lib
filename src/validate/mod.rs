use crate::{RoCrate, profile::Profile};
use miette::{Diagnostic, Severity};
use std::fmt::{self, Display};
use thiserror::Error;

mod base;
mod run;
mod workflow;

/// How strongly a profile states a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Must,
    Should,
}

/// A single rule a crate breaks. `Must` renders as an error, `Should` as a
/// warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub level: Level,
    /// Stable identifier, e.g. `wroc::main-entity-type`.
    pub rule: &'static str,
    /// The `@id` the rule is about, when it is about one entity.
    pub entity: Option<String>,
    pub message: String,
    pub advice: Option<String>,
}

impl Violation {
    pub fn must(rule: &'static str, message: impl Into<String>) -> Self {
        Violation {
            level: Level::Must,
            rule,
            entity: None,
            message: message.into(),
            advice: None,
        }
    }

    pub fn should(rule: &'static str, message: impl Into<String>) -> Self {
        Violation {
            level: Level::Should,
            ..Violation::must(rule, message)
        }
    }

    pub fn at(mut self, entity: impl Into<String>) -> Self {
        self.entity = Some(entity.into());
        self
    }

    pub fn advise(mut self, advice: impl Into<String>) -> Self {
        self.advice = Some(advice.into());
        self
    }
}

impl Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.entity {
            Some(id) => write!(f, "{} (`{id}`)", self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for Violation {}

impl Diagnostic for Violation {
    fn code(&self) -> Option<Box<dyn Display + '_>> {
        Some(Box::new(format!("rocrate::{}", self.rule)))
    }

    fn severity(&self) -> Option<Severity> {
        Some(match self.level {
            Level::Must => Severity::Error,
            Level::Should => Severity::Warning,
        })
    }

    fn help(&self) -> Option<Box<dyn Display + '_>> {
        self.advice
            .as_ref()
            .map(|advice| Box::new(advice) as Box<dyn Display>)
    }
}

/// The error a crate with `Must` violations turns into.
#[derive(Debug, Error, Diagnostic)]
#[error("crate does not conform to {profiles}")]
#[diagnostic(code(rocrate::invalid))]
pub struct InvalidCrate {
    pub profiles: String,
    #[related]
    pub violations: Vec<Violation>,
}

/// Everything the checked profiles have to say about a crate.
#[derive(Debug, Clone, Default)]
pub struct Validation {
    pub violations: Vec<Violation>,
    profiles: Vec<Profile>,
}

impl Validation {
    /// The profiles that were checked.
    #[must_use]
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn errors(&self) -> impl Iterator<Item = &Violation> {
        self.of_level(Level::Must)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Violation> {
        self.of_level(Level::Should)
    }

    fn of_level(&self, level: Level) -> impl Iterator<Item = &Violation> {
        self.violations.iter().filter(move |v| v.level == level)
    }

    /// Whether the crate breaks no `Must` rule. `Should` violations still leave
    /// a crate conformant.
    #[must_use]
    pub fn is_conformant(&self) -> bool {
        self.errors().next().is_none()
    }

    #[must_use]
    pub fn broke(&self, rule: &str) -> bool {
        self.violations.iter().any(|v| v.rule == rule)
    }

    /// The warnings on success, a [`miette`] report on failure.
    pub fn into_result(self) -> Result<Vec<Violation>, InvalidCrate> {
        if self.is_conformant() {
            return Ok(self.violations);
        }

        let profiles = if self.profiles.is_empty() {
            "RO-Crate".to_string()
        } else {
            self.profiles
                .iter()
                .map(Profile::iri)
                .collect::<Vec<_>>()
                .join(", ")
        };
        Err(InvalidCrate {
            profiles,
            violations: self.violations,
        })
    }
}

impl RoCrate {
    /// Check the crate against base RO-Crate plus every profile it claims.
    #[must_use]
    pub fn validate(&self) -> Validation {
        self.validate_profiles(self.profiles())
    }

    /// Check the crate against one profile, claimed or not.
    #[must_use]
    pub fn validate_as(&self, profile: &Profile) -> Validation {
        self.validate_profiles(vec![profile.clone()])
    }

    fn validate_profiles(&self, profiles: Vec<Profile>) -> Validation {
        let mut violations = Vec::new();
        base::check(self, &mut violations);

        // The run profiles build on each other: Provenance extends Workflow
        // Run, which extends both Process Run and Workflow RO-Crate.
        let (mut wroc, mut process, mut workflow_run, mut provenance) =
            (false, false, false, false);
        for profile in &profiles {
            match profile {
                Profile::WorkflowRoCrate(_) => wroc = true,
                Profile::ProcessRun(_) => process = true,
                Profile::WorkflowRun(_) => (wroc, process, workflow_run) = (true, true, true),
                Profile::ProvenanceRun(_) => {
                    (wroc, process, workflow_run, provenance) = (true, true, true, true);
                }
                Profile::RoCrate(_) | Profile::Other(_) => {}
            }
        }

        if wroc {
            workflow::check(self, &mut violations);
        }
        if process {
            run::check_process(self, &mut violations);
        }
        if workflow_run {
            run::check_workflow_run(self, &mut violations);
        }
        if provenance {
            run::check_provenance(self, &mut violations);
        }

        Validation {
            violations,
            profiles,
        }
    }
}
