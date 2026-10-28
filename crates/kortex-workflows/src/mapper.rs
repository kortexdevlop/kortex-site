use crate::{ActionSpec, Binding, Error, MAX_BYTES, Node, Workflow};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn tokens(pointer: &str) -> Result<Vec<String>, Error> {
    if pointer.len() > 1024 || !pointer.is_empty() && !pointer.starts_with('/') {
        return Err(Error::InvalidPointer);
    }
    if pointer.is_empty() {
        return Ok(vec![]);
    }
    pointer[1..]
        .split('/')
        .map(|s| {
            let mut out = String::new();
            let mut chars = s.chars();
            while let Some(c) = chars.next() {
                if c == '~' {
                    out.push(match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => return Err(Error::InvalidPointer),
                    })
                } else {
                    out.push(c)
                }
            }
            Ok(out)
        })
        .collect()
}
fn index(s: &str) -> Result<usize, Error> {
    if s.is_empty()
        || s.len() > 4
        || s.len() > 1 && s.starts_with('0')
        || !s.bytes().all(|c| c.is_ascii_digit())
    {
        return Err(Error::InvalidPointer);
    }
    s.parse::<usize>()
        .ok()
        .filter(|i| *i < 1000)
        .ok_or(Error::InvalidPointer)
}
pub fn schema_at<'a>(schema: &'a Value, path: &[String]) -> Result<&'a Value, Error> {
    let mut s = schema;
    for key in path {
        s = match s["type"].as_str() {
            Some("object") => s["properties"].get(key).ok_or(Error::InvalidPointer)?,
            Some("array") => {
                index(key)?;
                s.get("items").ok_or(Error::InvalidPointer)?
            }
            _ => return Err(Error::InvalidPointer),
        }
    }
    Ok(s)
}
pub fn compatible(source: &Value, target: &Value) -> bool {
    if let Some(values) = source["enum"].as_array() {
        return values.iter().all(|v| valid(target, v));
    }
    if source["type"] != target["type"]
        && !(source["type"] == "integer" && target["type"] == "number")
    {
        return false;
    }
    if target.get("enum").is_some() {
        return false;
    }
    for (min, max) in [
        ("minimum", "maximum"),
        ("minLength", "maxLength"),
        ("minItems", "maxItems"),
    ] {
        if target[min]
            .as_f64()
            .is_some_and(|v| source[min].as_f64().is_none_or(|s| s < v))
        {
            return false;
        }
        if target[max]
            .as_f64()
            .is_some_and(|v| source[max].as_f64().is_none_or(|s| s > v))
        {
            return false;
        }
    }
    if source["type"] == "object" {
        let empty = serde_json::Map::new();
        let source_props = source["properties"].as_object().unwrap_or(&empty);
        let target_props = target["properties"].as_object().unwrap_or(&empty);
        if let Some(required) = target["required"].as_array() {
            for key in required {
                let Some(key) = key.as_str() else {
                    return false;
                };
                if !source["required"]
                    .as_array()
                    .is_some_and(|r| r.contains(&json!(key)))
                    || !source_props
                        .get(key)
                        .zip(target_props.get(key))
                        .is_some_and(|(s, t)| compatible(s, t))
                {
                    return false;
                }
            }
        }
        for (key, s) in source_props {
            if let Some(t) = target_props.get(key) {
                if !compatible(s, t) {
                    return false;
                }
            } else if target["additionalProperties"] == false {
                return false;
            }
        }
        if source["additionalProperties"] != false
            && (target["additionalProperties"] == false
                || target_props.keys().any(|k| !source_props.contains_key(k)))
        {
            return false;
        }
    }
    if source["type"] == "array" {
        return compatible(&source["items"], &target["items"]);
    }
    true
}
pub fn valid(schema: &Value, value: &Value) -> bool {
    jsonschema::validator_for(schema).is_ok_and(|v| v.is_valid(value))
}
fn sample(s: &Value, depth: usize) -> Result<Value, Error> {
    if depth > 16 {
        return Err(Error::Limit);
    }
    if let Some(value) = s["enum"].as_array().and_then(|e| e.first()) {
        return Ok(value.clone());
    }
    Ok(match s["type"].as_str() {
        Some("object") => {
            let mut out = serde_json::Map::new();
            for key in s["required"].as_array().into_iter().flatten() {
                let key = key.as_str().ok_or(Error::TypeMismatch)?;
                out.insert(key.into(), sample(&s["properties"][key], depth + 1)?);
            }
            Value::Object(out)
        }
        Some("array") => {
            let n = s["minItems"].as_u64().unwrap_or(0);
            if n > 1000 {
                return Err(Error::Limit);
            }
            Value::Array(
                (0..n)
                    .map(|_| sample(&s["items"], depth + 1))
                    .collect::<Result<_, _>>()?,
            )
        }
        Some("string") => {
            let n = s["minLength"].as_u64().unwrap_or(0);
            if n > MAX_BYTES as u64 {
                return Err(Error::Limit);
            }
            json!("x".repeat(n as usize))
        }
        Some("number") | Some("integer") => {
            let lo = s["minimum"].as_f64().unwrap_or(-9007199254740991.0);
            let hi = s["maximum"].as_f64().unwrap_or(9007199254740991.0);
            {
                let n = 0f64.max(lo).min(hi);
                if s["type"] == "integer" {
                    let n = n.ceil();
                    if n > hi.floor() {
                        return Err(Error::TypeMismatch);
                    }
                    json!(n as i64)
                } else {
                    json!(n)
                }
            }
        }
        Some("boolean") => json!(false),
        Some("null") => Value::Null,
        _ => return Err(Error::TypeMismatch),
    })
}
fn put(value: &mut Value, schema: &Value, path: &[String], new: Value) -> Result<(), Error> {
    if path.is_empty() {
        *value = new;
        return Ok(());
    }
    let key = &path[0];
    match schema["type"].as_str() {
        Some("object") => {
            if !value.is_object() {
                *value = json!({})
            }
            let target = schema["properties"].get(key).ok_or(Error::InvalidPointer)?;
            let child = value
                .as_object_mut()
                .unwrap()
                .entry(key.clone())
                .or_insert(Value::Null);
            put(child, target, &path[1..], new)
        }
        Some("array") => {
            if !value.is_array() {
                *value = json!([])
            }
            let index = index(key)?;
            let values = value.as_array_mut().unwrap();
            if values.len() <= index {
                values.resize(index + 1, Value::Null)
            }
            put(&mut values[index], &schema["items"], &path[1..], new)
        }
        _ => Err(Error::InvalidPointer),
    }
}
fn overlap(node: &Node) -> Result<(), Error> {
    let paths = node
        .input_bindings
        .keys()
        .map(|p| tokens(p))
        .collect::<Result<Vec<_>, _>>()?;
    for (i, a) in paths.iter().enumerate() {
        for b in paths.iter().skip(i + 1) {
            if a.starts_with(b) || b.starts_with(a) {
                return Err(Error::InvalidPointer);
            }
        }
    }
    Ok(())
}
pub fn validate(workflow: &Workflow, specs: &[ActionSpec]) -> Result<(), Error> {
    for (i, node) in workflow.nodes.iter().enumerate() {
        overlap(node)?;
        let schema = &specs[i].action.input_schema;
        let mut input = if schema["type"] == "object" {
            json!({})
        } else {
            Value::Null
        };
        for (pointer, binding) in &node.input_bindings {
            let path = tokens(pointer)?;
            let target = schema_at(schema, &path)?;
            let value = match binding {
                Binding::Literal { value } => {
                    if !valid(target, value) {
                        return Err(Error::TypeMismatch);
                    }
                    value.clone()
                }
                Binding::Source {
                    from_node,
                    source_pointer,
                } => {
                    let j = workflow
                        .nodes
                        .iter()
                        .position(|n| &n.id == from_node)
                        .ok_or(Error::MissingInput)?;
                    let source =
                        schema_at(&specs[j].action.output_schema, &tokens(source_pointer)?)?;
                    if !compatible(source, target) {
                        return Err(Error::TypeMismatch);
                    }
                    sample(source, 0)?
                }
            };
            put(&mut input, schema, &path, value)?
        }
        if !valid(schema, &input) {
            return Err(Error::MissingInput);
        }
    }
    Ok(())
}
pub fn input(
    node: &Node,
    spec: &ActionSpec,
    outputs: &BTreeMap<String, Value>,
) -> Result<Value, Error> {
    let schema = &spec.action.input_schema;
    let mut result = if schema["type"] == "object" {
        json!({})
    } else {
        Value::Null
    };
    for (pointer, binding) in &node.input_bindings {
        let value = match binding {
            Binding::Literal { value } => value.clone(),
            Binding::Source {
                from_node,
                source_pointer,
            } => outputs
                .get(from_node)
                .and_then(|v| v.pointer(source_pointer))
                .cloned()
                .ok_or(Error::MissingInput)?,
        };
        put(&mut result, schema, &tokens(pointer)?, value)?
    }
    if serde_json::to_vec(&result).map_err(|_| Error::Limit)?.len() > MAX_BYTES {
        return Err(Error::Limit);
    }
    if !valid(schema, &result) {
        return Err(Error::TypeMismatch);
    }
    Ok(result)
}
