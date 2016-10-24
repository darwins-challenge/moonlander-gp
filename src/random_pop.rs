use std::cmp::max;
use super::{AstNode, Population, Fitness, Mutatable};
use rand::Rng;

const RANDOMPOP_MAX_HEIGHT : usize = 6;

/// A trait like rand::Rand, but one that doesn't require that the Rng instance is Sized,
/// so that we can combine it with the Mutatable trait.
///
/// We also will encode relative weights for tree depth into here. If the node has a choice
/// between generating an internal node or a leaf node, it should use these weights.
pub trait RandNode: Sized {
    fn rand(weights: TargetHeight, rng: &mut Rng) -> Self;
}

impl <T: RandNode+AstNode> Mutatable for T {
    fn mutate(&self, rng: &mut Rng) -> Box<AstNode> {
        Box::new(T::rand(TargetHeight::randomized(4, rng), rng))
    }
}


/// Generate a random population of size n
pub fn random_population<P: RandNode+Clone+Sync, F: Fitness+Sized+Send, R: Rng>(n: usize, rng: &mut R) -> Population<P, F> {
    let mut ret = Population::new(n, 0);
    for i in 0..n {
        let height = 1 + i / (n / RANDOMPOP_MAX_HEIGHT);
        ret.add(P::rand(TargetHeight::fixed(height as u32), rng));
    }
    ret
}

#[derive(Copy,Clone)]
pub struct TargetHeight {
    current_level: u32,
    per_level: u32
}

impl TargetHeight {
    pub fn fixed(target_height: u32) -> TargetHeight {
        TargetHeight {
            current_level: 0,
            per_level: 100 / max(target_height, 1)
        }
    }

    pub fn randomized(max_height: u32, rng: &mut Rng) -> TargetHeight {
        TargetHeight::fixed(1 + rng.next_u32() % (max_height - 1))
    }

    pub fn internal(&self) -> u32 {
        max(100 - self.per_level * self.current_level, MIN_WEIGHT)
    }

    pub fn leaf(&self) -> u32 {
        max(self.per_level * self.current_level, MIN_WEIGHT)
    }

    pub fn next_level(&self) -> TargetHeight {
        TargetHeight { current_level: self.current_level + 1, per_level: self.per_level }
    }

    pub fn gen_child<P: RandNode>(&self, rng: &mut Rng) -> Box<P> {
        Box::new(P::rand(self.next_level(), rng))
    }
}

/// Minimum weight
///
/// We need at least some weight, in case we need to make a leaf node
/// but only internal nodes are available at that point in the tree.
const MIN_WEIGHT : u32 = 1;
