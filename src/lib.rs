extern crate rand;
extern crate rustc_serialize;
extern crate rayon;
#[macro_use] extern crate downcast;
#[macro_use] extern crate log;

#[macro_use] mod pick;
#[macro_use] pub mod impl_astnode;

mod ast;
pub use ast::{AstNode, Mutatable, clone_or_replace, depth};

mod population;
pub use self::population::Population;

mod random_pop;
pub use self::random_pop::{random_population, RandNode, NodeWeights, retain_best};

pub mod num;

pub mod genetic;
pub use genetic::{ScoreCard, Fitness};

pub use num::Number;
