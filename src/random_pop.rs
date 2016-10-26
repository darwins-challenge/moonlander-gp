use std::cmp::max;
use super::{AstNode, Population, Fitness, Mutatable, Number};
use rand::Rng;
use rustc_serialize::Encodable;

/// Implement trait to generate random subtrees of a given type.
///
/// This trait is like `rand::Rand`, but it doesn't require that the `Rng`
/// instance is `Sized`, so that we can combine it with the `Mutatable` trait.
///
/// It also takes a parameter that controls the height of trees that will be
/// randomly generated. The `NodeWeights` parameter will indicate the relative
/// weights to be used when deciding between generating internal vs. leaf nodes
/// in the tree.
pub trait RandNode: Sized {
    fn rand(weights: NodeWeights, rng: &mut Rng) -> Self;
}

impl <T: RandNode+AstNode> Mutatable for T {
    fn mutate(&self, max_height: i32, rng: &mut Rng) -> Box<AstNode> {
        Box::new(T::rand(NodeWeights::fixed(max_height), rng))
    }
}

/// Generate a random population of size N.
///
/// Trees of various depths will be generated in a uniform fashion between
/// 1 and `max_depth`.
pub fn random_population<P: RandNode+Clone+Sync, F: Fitness+Sized+Send, R: Rng>(n: usize, max_depth: usize, rng: &mut R) -> Population<P, F> {
    let mut ret = Population::new(n, 0);
    for i in 0..n {
        let height = 1 + i / (n / max_depth);
        ret.add(P::rand(NodeWeights::fixed(height as i32), rng));
    }
    ret
}

/// Take the best fraction of the population and fill back up to N with random
/// programs.
pub fn retain_best<P, F, R>(frac: Number, pop: Population<P, F>, max_depth: usize, rng: &mut R) -> Population<P, F>
    where P: RandNode+Clone+Sync+Encodable,
          F: Fitness+Sized+Send+Encodable,
          R: Rng
{
    let n = (pop.n() as Number * frac) as usize;
    let filler = pop.n() - n;
    let mut ret = Population::new(n, 0);
    ret.generation = pop.generation + 1;

    for c in pop.best_n(n) {
        ret.add(c);
    }

    for i in 0..filler {
        let height = 1 + i / (filler / max_depth);
        ret.add(P::rand(NodeWeights::fixed(height as i32), rng));
    }
    ret
}

/// Weights to use when deciding between internal and leaf nodes.
///
/// This structure is initialized with a desired target depth, and
/// every level advancing in the tree shifts the weights away from
/// internal nodes and towards leaf nodes.
#[derive(Copy,Clone)]
pub struct NodeWeights {
    current_level: i32,
    per_level: i32
}

impl NodeWeights {
    pub fn fixed(target_height: i32) -> NodeWeights {
        NodeWeights {
            current_level: 0,
            per_level: 100 / max(target_height - 1, 1)
        }
    }

    pub fn randomized(max_height: i32, rng: &mut Rng) -> NodeWeights {
        NodeWeights::fixed(1 + rng.next_u32() as i32 % (max_height - 1))
    }

    pub fn internal(&self) -> u32 {
        max(100 - self.per_level * self.current_level, MIN_WEIGHT) as u32
    }

    pub fn leaf(&self) -> u32 {
        max(self.per_level * self.current_level, MIN_WEIGHT) as u32
    }

    pub fn next_level(&self) -> NodeWeights {
        NodeWeights { current_level: self.current_level + 1, per_level: self.per_level }
    }

    pub fn gen_child<P: RandNode>(&self, rng: &mut Rng) -> Box<P> {
        Box::new(P::rand(self.next_level(), rng))
    }
}

/// Minimum weight
///
/// We need at least some weight, in case we need to make a leaf node
/// but only internal nodes are available at that point in the tree.
const MIN_WEIGHT : i32 = 1;


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{num, Number, depth};
    use super::super::genetic::mutate_tree;

    #[derive(Clone)]
    enum List {
        Cons(Box<List>),
        Nil
    }

    impl_astnode!(List, 0,
                  int Cons(next),
                  leaf Nil());

    #[test]
    fn test_node_heights_on_generation() {
        let target_height = 8;
        let n = 1000;
        let mut rng = ::rand::StdRng::new().unwrap();

        let weights = NodeWeights::fixed(target_height);
        let programs : Vec<List> = (0..n).map(|_| RandNode::rand(weights, &mut rng)).collect();
        let avg_height = num::sum(programs.iter().map(|p| depth(p) as Number)) / n as Number;

        // Since we have uniform distribution of heights, the average will be about 0.5
        // times the max height. Some fudge margin for randomness.
        assert!(avg_height <= 0.6 * target_height as Number);
    }

    #[test]
    fn test_node_heights_during_mutation() {
        // Check that during mutation, the program doesn't grow endlessly
        let n = 1000;
        let target_height = 8;
        let mut rng = ::rand::StdRng::new().unwrap();
        let weights = NodeWeights::fixed(target_height);

        let mut program : List = RandNode::rand(weights, &mut rng);
        for _ in 0..n {
            program = *mutate_tree(&program, target_height, &mut rng);
            let depth = depth(&program);
            println!("Depth: {}", depth);
            // Check that we didn't grow too large, plus a fudge factor
            assert!((depth as Number) < target_height as Number * 1.5);
        }
    }
}
