use crate::{Binding, Error, Workflow};
use std::collections::{BTreeMap, BTreeSet};
pub fn order(workflow: &Workflow) -> Result<Vec<usize>, Error> {
    if workflow.nodes.is_empty() || workflow.nodes.len() > 50 || workflow.edges.len() > 100 {
        return Err(Error::Limit);
    }
    let mut ids = BTreeMap::new();
    for (i, node) in workflow.nodes.iter().enumerate() {
        if ids.insert(node.id.as_str(), i).is_some() {
            return Err(Error::InvalidWorkflow);
        }
    }
    let mut outgoing = vec![Vec::new(); workflow.nodes.len()];
    let mut incoming = vec![0; workflow.nodes.len()];
    let mut edges = BTreeSet::new();
    for edge in &workflow.edges {
        let a = *ids.get(edge.from.as_str()).ok_or(Error::InvalidWorkflow)?;
        let b = *ids.get(edge.to.as_str()).ok_or(Error::InvalidWorkflow)?;
        if a == b {
            return Err(Error::Cycle);
        }
        if !edges.insert((a, b)) {
            return Err(Error::InvalidWorkflow);
        }
        outgoing[a].push(b);
        incoming[b] += 1;
    }
    let mut result = Vec::new();
    let mut ready: BTreeSet<usize> = incoming
        .iter()
        .enumerate()
        .filter_map(|(i, n)| (*n == 0).then_some(i))
        .collect();
    let mut ancestors = vec![BTreeSet::new(); workflow.nodes.len()];
    let mut depth = vec![1; workflow.nodes.len()];
    while let Some(i) = ready.pop_first() {
        result.push(i);
        for &j in &outgoing[i] {
            let prior = ancestors[i].clone();
            ancestors[j].extend(prior);
            ancestors[j].insert(i);
            depth[j] = depth[j].max(depth[i] + 1);
            if depth[j] > 50 {
                return Err(Error::Limit);
            }
            incoming[j] -= 1;
            if incoming[j] == 0 {
                ready.insert(j);
            }
        }
    }
    if result.len() != workflow.nodes.len() {
        return Err(Error::Cycle);
    }
    for (i, node) in workflow.nodes.iter().enumerate() {
        for binding in node.input_bindings.values() {
            if let Binding::Source { from_node, .. } = binding {
                let source = *ids.get(from_node.as_str()).ok_or(Error::MissingInput)?;
                if !ancestors[i].contains(&source) {
                    return Err(Error::MissingInput);
                }
            }
        }
    }
    Ok(result)
}
