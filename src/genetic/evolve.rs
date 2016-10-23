use super::Fitness;
use super::super::Population;
use super::super::{AstNode, Mutatable};
use super::crossover;
use super::mutate;
use rand::Rng;

pub struct Weights {
    pub reproduce: u32,
    pub mutate: u32,
    pub crossover: u32
}

/// Produce a new population of the same size based off the current one
pub fn evolve<P, F, S, R: Rng>(pop: Population<P, F>, weights: &Weights, rng: &mut R, selector: S) -> Population<P, F>
    where P: AstNode+Clone+Mutatable+Sync,
          F: Fitness+Send,
          S: for<'a> Fn(&'a Population<P, F>, &mut Rng) -> &'a P
{
    let mut ret = Population::new(pop.n(), pop.generation + 1);
    while ret.n() < pop.n() {
        pick![rng,
            weights.reproduce, {
                let winner = selector(&pop, rng);
                ret.add(winner.clone());
            },
            weights.mutate, {
                let winner = selector(&pop, rng);
                let mutation = mutate::mutate_tree(winner, rng);
                ret.add(*mutation);
            },
            weights.crossover, {
                if pop.n() < 2 { continue; }

                let one = selector(&pop, rng);
                let two = selector(&pop, rng);

                let (child1, child2) = crossover::crossover_tree(one, two, rng);

                // We insert both children, this might make the population go over size, but never
                // by more than 1.
                ret.add(*child1);
                ret.add(*child2);
            }
        ];
    }
    ret
}
