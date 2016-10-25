/// The `mutate` trait returns a mutation version of a type that implements it.
///
/// Implementer should try to create persisted data-structures
use rand;
use super::super::AstNode;
use super::super::ast::{replace_to_root, find_nodes_and_parents, NodeAndParent};


/// Pick a node at random to mutate
pub fn mutate_tree<T: AstNode+Clone, R: rand::Rng+Sized>(ast: &T, target_height: i32, rng: &mut R) -> Box<T> {
    let naps = find_nodes_and_parents(ast);
    let picked = rng.choose(&naps).unwrap();
    let height_diff = target_height - depth(picked);
    let mutated = picked.node.mutate(height_diff, rng);
    replace_to_root(&picked, mutated)
}

fn depth<'a>(nap: &NodeAndParent<'a>) -> i32 {
    1 + match nap.root_path {
        None => 0,
        Some(ref parent) => depth(parent.as_ref())
    }
}
