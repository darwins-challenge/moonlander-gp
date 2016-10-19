mod mutate;
pub use self::mutate::mutate_tree;

mod crossover;
pub use self::crossover::crossover_tree;

pub mod fitness;
pub use self::fitness::{Fitness, SimpleFitness, ScoreCard, Scores};

mod select;
pub use self::select::{tournament_selection};

mod evolve;
pub use self::evolve::{evolve, Weights};
