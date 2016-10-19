extern crate rand;
extern crate rustc_serialize;

#[macro_use]
extern crate downcast_rs;

#[macro_use]
extern crate log;

#[macro_use]
mod pick;

#[macro_use]
mod node_gen;

mod ast;

mod population;
pub use self::population::{Population, random_population};

pub use ast::{AstNode, Mutatable, RandNode, clone_or_replace};
pub mod num;

pub mod genetic;

pub use num::Number;
