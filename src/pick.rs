//! pick an expression according to weights

extern crate rand;

/// The `pick` macro allows to randomly select one expression from a sequence of
/// expressions according to associated weights.
#[macro_export]
macro_rules! pick {
    ($rng: expr, $( $weight: expr, $expression: expr),+) => {{
        let total = 0 $(+ $weight)+;
        let mut bound = 0;

        let random_number = $rng.next_u32() % total;
        let result = $( if bound <= random_number && random_number < { bound += $weight; bound } {
            $expression
        } else )+ {
            panic!();
        };
        result
    }}
}
