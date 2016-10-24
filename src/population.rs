use super::genetic::Fitness;
use rand::Rng;
use super::Number;
use super::num::{sum, partial_max};
use rayon::prelude::*;

/// A population with the root of the indicated type
pub struct Population<P: Clone+Sync, F: Fitness+Sized+Send> {
    /// Collection of algorithms
    pub population: Vec<P>,

    /// Generation index of this population
    pub generation: u32,

    /// Collection of fitness scores
    pub scores: Vec<F>
}

impl <P: Clone+Sync, F: Fitness+Sized+Send> Population<P, F> {
    /// Create a new population with an estimated size
    ///
    /// This does not create programs yet but simply allocates memory.
    pub fn new(n: usize, generation: u32) -> Population<P, F> {
        Population {
            population: Vec::with_capacity(n),
            scores: Vec::with_capacity(n),
            generation: generation
        }
    }

    /// Add a single program to the population
    pub fn add(&mut self, program: P) {
        self.population.push(program);
    }

    pub fn n(&self) -> usize {
        self.population.len()
    }

    /// Apply a scoring function to the entire population
    pub fn score<S>(&mut self, scoring_fn: S, _: &mut Rng)
        where S: Fn(&P, &mut Rng) -> F + Sync
    {
        self.population.par_iter().weight_max().map(|p| scoring_fn(p, &mut ::rand::thread_rng())).collect_into(&mut self.scores);
    }

    pub fn avg_score(&self) -> Number {
        let total_score = sum(self.scores.iter().map(|f| f.score_card().total_score()));
        total_score / (self.n() as Number)
    }

    pub fn best_score(&self) -> Number {
        partial_max(self.scores.iter().map(|f| f.score_card().total_score())).unwrap()
    }
}

