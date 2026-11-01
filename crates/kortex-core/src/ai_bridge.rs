//! Proposal review never grants capabilities. Execution reuses the loaded runtime's universal dispatcher.
use crate::{GuestEngine, InstanceId, Runtime, RuntimeError, State, dispatcher::Clock};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[derive(Clone)]
struct Step {
    instance: InstanceId,
    tool: String,
    action: String,
    input: Value,
    pin: String,
    used: bool,
}
pub struct ReviewedPlan {
    steps: Vec<Step>,
    expires: u64,
}
pub fn bounded_plan(value: &Value) -> bool {
    within(value, 32768)
}
/// An approved step can carry the user's own file inline; it never comes from the cloud.
pub fn bounded_local_plan(value: &Value) -> bool {
    within(value, 2_400_000)
}
fn within(value: &Value, limit: usize) -> bool {
    let Ok(bytes) = serde_json::to_vec(value) else {
        return false;
    };
    if bytes.len() > limit {
        return false;
    }
    let Ok(schema) =
        serde_json::from_str::<Value>(include_str!("../../../shared/schemas/ai-plan.schema.json"))
    else {
        return false;
    };
    jsonschema::validator_for(&schema).is_ok_and(|v| v.is_valid(value))
}
/// Binary outputs stay out of the displayed result but must reach the next step of an approved chain.
pub fn carried(value: &Value) -> serde_json::Map<String, Value> {
    value
        .as_object()
        .map(|map| {
            map.iter()
                .filter(|(k, v)| k.ends_with("_base64") && v.is_string())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}
pub fn local_result(mut value: Value) -> Result<Value, RuntimeError> {
    fn scrub(v: &mut Value, depth: usize) {
        if depth > 16 {
            *v = Value::Null;
            return;
        }
        match v {
            Value::Object(map) => {
                for (k, v) in map {
                    if ["handle", "file_handle", "path", "file_path", "bytes"].contains(&k.as_str())
                        || k.ends_with("_base64")
                    {
                        *v = Value::String("[masqué]".into())
                    } else {
                        scrub(v, depth + 1)
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    scrub(v, depth + 1)
                }
            }
            Value::String(s)
                if s.contains(":\\") || s.starts_with("/home/") || s.starts_with("/Users/") =>
            {
                *s = "[chemin masqué]".into()
            }
            _ => {}
        }
    }
    scrub(&mut value, 0);
    if serde_json::to_vec(&value)
        .map_err(|_| RuntimeError::InvalidReply)?
        .len()
        > 32768
    {
        return Err(RuntimeError::InvalidReply);
    }
    Ok(value)
}
impl<E: GuestEngine, C: Clock> Runtime<E, C> {
    pub fn review_ai_plan(&self, value: &Value) -> Result<ReviewedPlan, RuntimeError> {
        if !bounded_local_plan(value) {
            return Err(RuntimeError::InvalidInput);
        }
        let mut steps = Vec::new();
        for item in value["steps"]
            .as_array()
            .ok_or(RuntimeError::InvalidInput)?
        {
            let tool = item["tool_id"].as_str().ok_or(RuntimeError::InvalidInput)?;
            let action = item["action_id"]
                .as_str()
                .ok_or(RuntimeError::InvalidInput)?;
            let (id, instance) = self
                .instances
                .iter()
                .find(|(_, i)| i.tool_id() == tool && i.state == State::Running)
                .ok_or(RuntimeError::UnknownInstance)?;
            let definition = instance
                .tool
                .contract
                .actions
                .iter()
                .find(|a| a.id == action)
                .ok_or(RuntimeError::InvalidInput)?;
            if !jsonschema::validator_for(&definition.input_schema)
                .map_err(|_| RuntimeError::InvalidInput)?
                .is_valid(&item["input"])
            {
                return Err(RuntimeError::InvalidInput);
            }
            let pin = format!("{:x}", Sha256::digest(instance.tool.artifact()));
            steps.push(Step {
                instance: *id,
                tool: tool.into(),
                action: action.into(),
                input: item["input"].clone(),
                pin,
                used: false,
            });
        }
        Ok(ReviewedPlan {
            steps,
            expires: self
                .clock
                .now_ms()
                .checked_add(300000)
                .ok_or(RuntimeError::Timeout)?,
        })
    }
    pub async fn execute_ai_step(
        &mut self,
        review: &mut ReviewedPlan,
        index: usize,
        approved: bool,
    ) -> Result<Value, RuntimeError> {
        local_result(self.execute_ai_step_raw(review, index, approved).await?)
    }
    /// For the host only: the unmasked output, so a chain can feed its next step.
    pub async fn execute_ai_step_raw(
        &mut self,
        review: &mut ReviewedPlan,
        index: usize,
        approved: bool,
    ) -> Result<Value, RuntimeError> {
        if self.clock.now_ms() >= review.expires {
            return Err(RuntimeError::Timeout);
        }
        let step = review
            .steps
            .get_mut(index)
            .ok_or(RuntimeError::InvalidInput)?;
        if step.used {
            return Err(RuntimeError::InvalidState);
        }
        step.used = true;
        if !approved {
            return Err(RuntimeError::InvalidState);
        }
        let instance = self
            .instances
            .get(&step.instance)
            .ok_or(RuntimeError::UnknownInstance)?;
        if instance.tool_id() != step.tool
            || format!("{:x}", Sha256::digest(instance.tool.artifact())) != step.pin
        {
            return Err(RuntimeError::IntegrityError);
        }
        let reply = self
            .invoke_action(step.instance, &step.action, step.input.clone())
            .await?;
        if !reply.ok {
            return Err(RuntimeError::GuestError(
                reply.error.ok_or(RuntimeError::InvalidReply)?,
            ));
        }
        Ok(Value::Object(reply.data.ok_or(RuntimeError::InvalidReply)?))
    }
}
