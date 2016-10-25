use super::super::num::Number;
use std::cmp::Ordering;
use std::ops::Add;

/// Trait that models fitness for an individual
///
/// Write a custom implementation that retains some additional state about the
/// evaluation if you want to.
///
/// Provides some helper classes to record scores.
pub trait Fitness: Send {
    fn score_card(&self) -> &ScoreCard;
}

/// Simple fitness result that only consists of a ScoreCard.
///
/// In case you don't need to retain any additional state, you can use this struct.
pub struct SimpleFitness {
    score_card: ScoreCard
}

impl SimpleFitness {
    pub fn new(scores: Scores) -> SimpleFitness {
        SimpleFitness { score_card: ScoreCard::new(scores) }
    }
}

impl Fitness for SimpleFitness {
    fn score_card(&self) -> &ScoreCard { &self.score_card }
}

pub type Score = (&'static str, Number);
pub type Scores = Vec<Score>;

/// Immutable tagged list of scores
///
/// Retains the total as a separate field
#[derive(Clone,RustcEncodable)]
pub struct ScoreCard(Scores, Number);

impl ScoreCard {
    pub fn new(scores: Scores) -> ScoreCard {
        let sum = scores.iter().map(|&(_, x)| x).fold(0.0, Add::add);
        ScoreCard(scores, sum)
    }

    pub fn add(self, scores: Scores) -> ScoreCard {
        let mut xs = self.0;
        xs.extend(scores);
        ScoreCard::new(xs)
    }

    pub fn scores(&self) -> &Scores {
        &self.0
    }

    pub fn total_score(&self) -> Number {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

impl PartialEq for ScoreCard {
    fn eq(&self, other: &Self) -> bool {
        return self.1.eq(&other.1);
    }
}

impl Eq for ScoreCard {
}

impl PartialOrd for ScoreCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return self.1.partial_cmp(&other.1);
    }
}

impl Ord for ScoreCard {
    fn cmp(&self, other: &Self) -> Ordering {
        if Number::is_nan(self.1) && Number::is_nan(other.1) { return Ordering::Equal; }
        if Number::is_nan(self.1) { return Ordering::Less; }
        if Number::is_nan(other.1) { return Ordering::Greater; }

        return self.1.partial_cmp(&other.1).unwrap();
    }
}

fn find_rec<'a>(scores: &'a mut Scores, name: &'static str) -> Option<&'a mut Score> {
    for x in scores.iter_mut() {
        if x.0 == name {
            return Some(x);
        }
    }
    None
}

impl Add<ScoreCard> for ScoreCard {
    type Output = ScoreCard;

    fn add(mut self, rhs: ScoreCard) -> Self::Output {
        self += &rhs;
        self
    }
}

impl <'a> ::std::ops::AddAssign<&'a ScoreCard> for ScoreCard {
    fn add_assign(&mut self, rhs: &'a ScoreCard) {
        for &(name, value) in rhs.0.iter() {
            let mut increased = false;
            match find_rec(&mut self.0, name) {
                Some(rec) => { rec.1 += value; increased = true; },
                None => { /* Moved outside match because borrow checker can't end scope early */ }
            }
            if !increased {
                self.0.push((name, value));
            }
            self.1 += value;
        }
    }
}

impl ::std::ops::Div<Number> for ScoreCard {
    type Output = ScoreCard;

    fn div(self, rhs: Number) -> Self::Output {
        ScoreCard(
            self.0.into_iter().map(|(n, v)| (n, v / rhs)).collect(),
            self.1 / rhs
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_scorecards_matching_keys() {
        let one = ScoreCard::new(vec![("a", 1.0)]);
        let two = ScoreCard::new(vec![("a", 1.0)]);

        let added = one + two;

        assert_eq!(vec![("a", 2.0)], added.0);
        assert_eq!(2.0, added.total_score());
    }

    #[test]
    fn add_scorecards_different_keys() {
        let one = ScoreCard::new(vec![("a", 1.0)]);
        let two = ScoreCard::new(vec![("b", 1.0)]);

        let added = one + two;

        assert_eq!(vec![("a", 1.0),("b", 1.0)], added.0);
        assert_eq!(2.0, added.total_score());
    }
}
