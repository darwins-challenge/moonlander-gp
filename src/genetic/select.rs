use rand::Rng;
use super::fitness::Fitness;
use super::super::AstNode;
use super::super::Population;

pub fn tournament_selection<'a, P, F>(tournament_size: usize, pop: &'a Population<P, F>, rng: &mut Rng) -> &'a P
    where P: AstNode+Clone,
          F: Fitness
{
    // Generate N random indexes. Slightly faster than rand::sample(), don't care about
    // the inaccuracy introduced by sampling with replacement.
    let count = pop.n();
    let candidate_indexes = (0..tournament_size).map(|_| rng.next_u64() as usize % count);

    let (_, winner_i) = candidate_indexes.map(|i| (&pop.scores[i], i)).max_by_key(|f| f.0.score_card()).unwrap();
    &pop.population[winner_i]
}
