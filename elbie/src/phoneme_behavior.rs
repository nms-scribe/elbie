use crate::orthography::SpellingBehavior;

#[derive(Debug)]
pub(crate) struct PhonemeBehavior {
  spelling: Vec<SpellingBehavior>
}

impl PhonemeBehavior {

  pub(crate) const fn new(spelling: Vec<SpellingBehavior>) -> Self {
    Self {
      spelling
    }
  }

  pub(crate) fn spelling(&self) -> &[SpellingBehavior] {
      &self.spelling
  }

  pub(crate) fn spelling_len(&self) -> usize {
      self.spelling.len()
  }

}
