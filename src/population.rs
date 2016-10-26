use super::genetic::Fitness;
use rand::Rng;
use super::Number;
use super::num::{sum, partial_max};
use rayon::prelude::*;
use rustc_serialize::Encodable;


/// Collection of programs
///
/// The root of each program is of type `P`, and fitness structures will be
/// represented by type `F`.
pub struct Population<P: Clone+Sync, F: Fitness+Sized+Send> {
    /// Collection of algorithms
    pub population: Vec<P>,

    /// Generation index of this population
    pub generation: u32,

    /// Collection of fitness scores
    pub scores: Vec<F>
}

impl <P: Clone+Sync, F: Fitness+Sized+Send> Population<P, F> {
    /// Create a new population with an estimated size.
    ///
    /// This does not create programs yet but simply allocates memory.
    pub fn new(n: usize, generation: u32) -> Population<P, F> {
        Population {
            population: Vec::with_capacity(n),
            scores: Vec::with_capacity(n),
            generation: generation
        }
    }

    /// Add a single program to the population.
    pub fn add(&mut self, program: P) {
        self.population.push(program);
    }

    pub fn n(&self) -> usize {
        self.population.len()
    }

    /// Apply a scoring function to the entire population.
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

    /// Return the best program from the population.
    pub fn champion<'a>(&'a self) -> CreatureScore<'a, P, F>
        where P: Encodable, F: Encodable
    {
        let indexes = 0..self.n();
        let winner_i = indexes.into_iter().max_by_key(|i| self.scores[*i].score_card()).unwrap();
        CreatureScore {
            generation: self.generation,
            program: &self.population[winner_i],
            fitness: &self.scores[winner_i]
        }
    }

    /// Return the best N programs from the population.
    pub fn best_n<'a>(&self, n: usize) -> Vec<P>
    {
        let mut indexes : Vec<usize> = (0..self.n()).collect();
        indexes.sort_by_key(|i| self.scores[*i].score_card());
        indexes[indexes.len() - n..].into_iter().map(|i| self.population[*i].clone()).collect()
    }
}

#[derive(RustcEncodable)]
pub struct CreatureScore<'a, P: 'a, F: 'a>
    where P: Encodable, F: Encodable
{
    pub generation: u32,
    pub program: &'a P,
    pub fitness: &'a F
}
