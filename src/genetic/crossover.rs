/// The `mutate` trait returns a mutation version of a type that implements it.
///
/// Implementer should try to create persisted data-structures
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;
use rand;
use super::super::{AstNode, Mutatable};
use super::super::ast::{NodeAndParent, find_nodes_and_parents, replace_to_root};

/// Pick two random nodes and cross them over
pub fn crossover_tree<T: AstNode+Mutatable+Clone, R: rand::Rng+Sized>(ast1: Rc<T>, ast2: Rc<T>, rng: &mut R) -> (Rc<T>, Rc<T>) {
    let nodes1 = group_by_type(find_nodes_and_parents(ast1.clone()));
    let nodes2 = group_by_type(find_nodes_and_parents(ast2.clone()));

    // Return all types that are in both maps (there is guaranteed to be at least 1)
    let shared_node_types = nodes1.iter()
            .filter_map(|(typ, _)| nodes2.get(typ).map(|_| *typ))
            .collect::<Vec<usize>>();
    let typ = rng.choose(&shared_node_types).unwrap();

    // Swap nodes
    let nap1 = rng.choose(&nodes1.get(typ).unwrap()).unwrap();
    let nap2 = rng.choose(&nodes2.get(typ).unwrap()).unwrap();

    let child1 = replace_to_root::<T>(&nap1, nap2.node.as_ref());
    let child2 = replace_to_root::<T>(&nap2, nap1.node.as_ref());

    (child1, child2)
}

fn group_by_type(naps: Vec<Rc<NodeAndParent>>) -> BTreeMap<usize, Vec<Rc<NodeAndParent>>> {
    let mut ret : BTreeMap<usize, Vec<Rc<NodeAndParent>>> = BTreeMap::new();
    for nap in naps {
        let values: &mut Vec<Rc<NodeAndParent>> = match ret.entry(nap.node.node_type()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(vec![])
        };
        values.push(nap);
    }
    ret
}
