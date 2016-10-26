/// The `mutate` trait returns a mutation version of a type that implements it.
///
/// Implementer should try to create persisted data-structures
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;
use rand;
use super::super::{AstNode, Mutatable};
use super::super::ast::{NodeInTree, find_nodes_and_parents, replace_to_root};

/// Cross two trees.
///
/// Pick two random subtrees of the same type in both trees, and return two new
/// trees with the subtrees switched.
pub fn crossover_tree<T: AstNode+Mutatable+Clone, R: rand::Rng+Sized>(ast1: &T, ast2: &T, rng: &mut R) -> (Box<T>, Box<T>) {
    let nodes1 = group_by_type(find_nodes_and_parents(ast1));
    let nodes2 = group_by_type(find_nodes_and_parents(ast2));

    // Return all types that are in both maps (there is guaranteed to be at least 1)
    let shared_node_types = nodes1.iter()
            .filter_map(|(typ, _)| nodes2.get(typ).map(|_| *typ))
            .collect::<Vec<usize>>();
    let typ = rng.choose(&shared_node_types).unwrap();

    // Swap nodes
    let nap1 = rng.choose(&nodes1.get(typ).unwrap()).unwrap();
    let nap2 = rng.choose(&nodes2.get(typ).unwrap()).unwrap();

    let child1 = replace_to_root::<T>(&nap1, nap2.node.copy());
    let child2 = replace_to_root::<T>(&nap2, nap1.node.copy());

    (child1, child2)
}

fn group_by_type(naps: Vec<Rc<NodeInTree>>) -> BTreeMap<usize, Vec<Rc<NodeInTree>>> {
    let mut ret : BTreeMap<usize, Vec<Rc<NodeInTree>>> = BTreeMap::new();
    for nap in naps {
        let values: &mut Vec<Rc<NodeInTree>> = match ret.entry(nap.node.node_type()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(vec![])
        };
        values.push(nap);
    }
    ret
}
