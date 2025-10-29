use crate::weighted_vec::WeightedVec;

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
