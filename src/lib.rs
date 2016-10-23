extern crate rand;
extern crate rustc_serialize;
extern crate rayon;
#[macro_use] extern crate downcast;
#[macro_use] extern crate log;

#[macro_use] mod pick;
#[macro_use] mod impl_astnode;

mod ast;

mod population;
pub use self::population::{Population, random_population};

pub use ast::{AstNode, Mutatable, RandNode, clone_or_replace, depth};
pub mod num;

pub mod genetic;
pub use genetic::{ScoreCard, Fitness};

pub use num::Number;
