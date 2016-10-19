/// The `mutate` trait returns a mutation version of a type that implements it.
///
/// Implementer should try to create persisted data-structures
use rand;
use std::rc::Rc;
use super::super::{AstNode, Mutatable};
use super::super::ast::{replace_to_root, find_nodes_and_parents};


/// Pick a node at random to mutate
pub fn mutate_tree<T: AstNode+Mutatable+Clone, R: rand::Rng+Sized>(ast: Rc<T>, rng: &mut R) -> Rc<T> {
    let naps = find_nodes_and_parents(ast.clone());
    let picked = rng.choose(&naps).unwrap();
    let mutated = picked.node.mutate(rng);
    replace_to_root::<T>(&picked, mutated.as_ref())
}
