/// The `mutate` trait returns a mutation version of a type that implements it.
///
/// Implementer should try to create persisted data-structures
use rand;
use super::super::AstNode;
use super::super::ast::{replace_to_root, find_nodes_and_parents};


/// Pick a node at random to mutate
pub fn mutate_tree<T: AstNode+Clone, R: rand::Rng+Sized>(ast: &T, rng: &mut R) -> Box<T> {
    let naps = find_nodes_and_parents(ast);
    let picked = rng.choose(&naps).unwrap();
    let mutated = picked.node.mutate(rng);
    replace_to_root(&picked, mutated)
}
