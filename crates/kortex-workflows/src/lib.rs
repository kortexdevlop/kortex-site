#![allow(async_fn_in_trait)]
//! Bounded local workflow orchestration. Hosts must use the existing runtime and capability gateway.
pub mod dag;
pub mod mapper;
pub mod permissions;
pub mod runner;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
pub const MAX_BYTES: usize = 1048576;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub workflow_version: String,
    pub id: String,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub action_ref: String,
    pub input_bindings: BTreeMap<String, Binding>,
    pub requires_confirmation: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Binding {
    Literal {
        value: Value,
    },
    Source {
        from_node: String,
        source_pointer: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: String,
    pub to: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Error {
    InvalidWorkflow,
    Limit,
    Cycle,
    UnknownAction,
    MissingInput,
    TypeMismatch,
    InvalidPointer,
    AwaitingConsent,
    PermissionDenied,
    Cancelled,
    Expired,
    Changed,
    Timeout,
    ActionFailed,
    RetryForbidden,
    InvalidOutput,
    Completed,
}
#[derive(Clone)]
pub struct ActionSpec {
    pub action: kortex_shared::Action,
    pub context: kortex_capabilities::policy::Context,
    pub artifact_sha256: String,
}
impl ActionSpec {
    pub fn fingerprint(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(
                    &self.context.tool_id,
                    &self.context.version,
                    self.context.instance_id,
                    &self.artifact_sha256,
                    &self.action
                ))
                .expect("validated action")
            )
        )
    }
}
#[derive(Default, Clone)]
pub struct Registry {
    actions: BTreeMap<String, ActionSpec>,
}
impl Registry {
    pub fn register_verified(
        &mut self,
        tool: &kortex_core::VerifiedTool,
        instance_id: u64,
    ) -> Result<(), Error> {
        if self.actions.len() + tool.contract().actions.len() > 2000 {
            return Err(Error::Limit);
        }
        let prefix = format!("{}.", tool.manifest().id);
        if self.actions.keys().any(|key| key.starts_with(&prefix)) {
            return Err(Error::Changed);
        }
        for action in &tool.contract().actions {
            self.actions.insert(
                format!("{prefix}{}", action.id),
                ActionSpec {
                    action: action.clone(),
                    context: kortex_capabilities::policy::Context {
                        tool_id: tool.manifest().id.clone(),
                        version: tool.manifest().version.clone(),
                        instance_id,
                    },
                    artifact_sha256: format!("{:x}", Sha256::digest(tool.artifact())),
                },
            );
        }
        Ok(())
    }
    pub fn get(&self, reference: &str) -> Option<&ActionSpec> {
        self.actions.get(reference)
    }
}
pub struct Compiled {
    pub(crate) workflow: Workflow,
    pub(crate) order: Vec<usize>,
    pub(crate) specs: Vec<ActionSpec>,
    pub(crate) fingerprint: String,
}
impl Compiled {
    pub fn new(value: &Value, registry: &Registry) -> Result<Self, Error> {
        if serde_json::to_vec(value)
            .map_err(|_| Error::InvalidWorkflow)?
            .len()
            > MAX_BYTES
        {
            return Err(Error::Limit);
        }
        let schema: Value =
            serde_json::from_str(include_str!("../../../shared/schemas/workflow.schema.json"))
                .map_err(|_| Error::InvalidWorkflow)?;
        if !jsonschema::validator_for(&schema)
            .map_err(|_| Error::InvalidWorkflow)?
            .is_valid(value)
        {
            return Err(Error::InvalidWorkflow);
        }
        let workflow: Workflow =
            serde_json::from_value(value.clone()).map_err(|_| Error::InvalidWorkflow)?;
        let order = dag::order(&workflow)?;
        let specs = workflow
            .nodes
            .iter()
            .map(|n| {
                registry
                    .get(&n.action_ref)
                    .cloned()
                    .ok_or(Error::UnknownAction)
            })
            .collect::<Result<Vec<_>, _>>()?;
        mapper::validate(&workflow, &specs)?;
        let pins: Vec<_> = specs.iter().map(ActionSpec::fingerprint).collect();
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&workflow, pins)).map_err(|_| Error::InvalidWorkflow)?
            )
        );
        Ok(Self {
            workflow,
            order,
            specs,
            fingerprint,
        })
    }
    pub fn workflow(&self) -> &Workflow {
        &self.workflow
    }
    pub fn order(&self) -> Vec<&str> {
        self.order
            .iter()
            .map(|i| self.workflow.nodes[*i].id.as_str())
            .collect()
    }
}
