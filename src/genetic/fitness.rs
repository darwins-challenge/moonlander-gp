use super::super::num::Number;
use std::cmp::Ordering;
use std::ops::Add;

/// Trait that models fitness for an individual
///
/// Write a custom implementation that retains some additional state about the
/// evaluation if you want to.
///
/// Provides some helper classes to record scores.
pub trait Fitness {
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

pub type Scores = Vec<(&'static str, Number)>;

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
