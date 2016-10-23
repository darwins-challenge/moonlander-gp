//! Numeric helpers

/// The general number type used by all simulations
pub type Number = f32;

/// A max() function that only requires a partial ordering.
///
/// Necessary for floats because they don't implement a total ordering, something that the regular
/// Iterator::max() function needs.
pub fn partial_max<I: Iterator>(iter: I) -> Option<I::Item>
    where I::Item : PartialOrd {
    iter.fold(None, |ret, x| {
        match ret {
            None => Some(x),
            Some(ref y) if x > *y => Some(x),
            _ => ret
        }
    })
}

pub fn partial_min<I: Iterator>(iter: I) -> Option<I::Item>
    where I::Item : PartialOrd {
    iter.fold(None, |ret, x| {
        match ret {
            None => Some(x),
            Some(ref y) if x < *y => Some(x),
            _ => ret
        }
    })
}

pub fn sum<I: Iterator<Item=Number>>(iter: I) -> Number {
    let mut total = 0.0;
    for x in iter {
        total += x;
    }
    total
}

pub fn torus(x: i32, n: i32) -> i32 {
    let k = if x < 0 { -x / n + 1 } else { 0 };
    (x + k * n) % n
}

pub fn square(x: Number) -> Number {
    x * x
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_torus() {
        assert_eq!(1, torus(33, 32));
        assert_eq!(1, torus(-31, 32));
    }
}
