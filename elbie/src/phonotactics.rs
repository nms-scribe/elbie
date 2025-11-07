use crate::weighted_vec::WeightedVec;

/*
FUTURE: This is relatively easy, but it's also hard to wrap ones head around how this works. One option to fix this is to do the same thing here than I'm doing in transformations, turn it into functions.

My idea would be to return functions that take an implentation of a trait, say: trait Phonotactics. The trait would have methods for matching phonemes in sets, similar to the way transformations work. But it would also have more stuff, like repeats which couldn't be done with loops, and weighting of choices.

The trait would be implemented by two objects: A Generator and a Validator. That way the user still only has to write one function.

The Generator uses the commands (and their weightings) to "choose" the phonemes to output.

The Validator ignores the weightings, but uses the commands to match a word and make sure it is valid. It might make use of a rule-name argument to help with tracing.

*/

#[derive(Debug,Clone)]
pub enum EnvironmentChoice {
  Done,
  Continuing(&'static str,&'static str,bool),// set to generate next phoneme from, next environment to follow, whether to allow duplicate phoneme to be generated
}

#[derive(Debug,Clone)]
pub struct EnvironmentBranch(&'static str, WeightedVec<EnvironmentChoice>);

impl EnvironmentBranch {

  #[must_use]
  pub fn new(set_check: &'static str, choices: &[(EnvironmentChoice,usize)]) -> Self {
    let mut vec = WeightedVec::new();
    for choice in choices {
      vec.push(choice.0.clone(),choice.1)
    };
    Self(set_check,vec)

  }

  pub(crate) const fn set(&self) -> &'static str {
      self.0
  }

  pub(crate) const fn choices(&self) -> &WeightedVec<EnvironmentChoice> {
      &self.1
  }

}
