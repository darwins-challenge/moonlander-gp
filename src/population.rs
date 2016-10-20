use super::genetic::Fitness;
use rand::Rng;
use super::{RandNode, Number};
use super::num::{sum, partial_max};

/// A population with the root of the indicated type
pub struct Population<P: Clone, F: Fitness+Sized> {
    /// Collection of algorithms
    pub population: Vec<P>,

    /// Generation index of this population
    pub generation: u32,

    /// Collection of fitness scores
    pub scores: Vec<F>
}

impl <P: Clone, F: Fitness+Sized> Population<P, F> {
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
    pub fn score<S>(&mut self, scoring_fn: S, rng: &mut Rng)
        where S: Fn(&P, &mut Rng) -> F
    {
        // FIXME: Parallelize?
        self.scores = self.population.iter().map(|p| scoring_fn(p, rng)).collect();
    }

    pub fn avg_score(&self) -> Number {
        let total_score = sum(self.scores.iter().map(|f| f.score_card().total_score()));
        total_score / (self.n() as Number)
    }

    pub fn best_score(&self) -> Number {
        partial_max(self.scores.iter().map(|f| f.score_card().total_score())).unwrap()
    }

    //// Return the best program from the population
    //pub fn champion(&self) -> CreatureScore<P> {
        //let indexes = 0..self.n();
        //let (score, winner_i) = indexes.into_iter().map(|i| (&self.scores[i], i)).max().unwrap();
        //CreatureScore::new(self.population.get(winner_i).unwrap().clone(), score.clone())
    //}

    //fn get_score(&self, i: usize) -> Number {
        //self.scores.get(i).unwrap().total_score()
    //}

    ///// Return a sorted list of all scores
    //pub fn all_scores(&self) -> Vec<CreatureScore<P>> {
        //let mut indexes = (0..self.n()).collect::<Vec<usize>>();
        //indexes.sort_by(|a, b| self.get_score(*b).partial_cmp(&self.get_score(*a)).unwrap());
        //indexes.into_iter().map(|i| CreatureScore::new(self.population.get(i).unwrap().clone(), self.scores.get(i).unwrap().clone())).collect()
    //}
}

/// Generate a random population of size n
pub fn random_population<P: RandNode+Clone, F: Fitness+Sized, R: Rng>(n: usize, rng: &mut R) -> Population<P, F> {
    let mut ret = Population::new(n, 0);
    for _ in 0..n {
        ret.add(P::rand(rng));
    }
    ret
}
