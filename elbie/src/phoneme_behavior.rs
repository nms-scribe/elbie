use core::array;
use crate::orthography::SpellingBehavior;

#[derive(Debug)]
pub(crate) struct PhonemeBehavior<const ORTHOGRAPHIES: usize> {
  spelling: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES]
}

impl<const ORTHOGRAPHIES: usize> Default for PhonemeBehavior<ORTHOGRAPHIES> {

  fn default() -> Self {
    Self {
      spelling: array::from_fn(|_| SpellingBehavior::default())
    }
  }
}

impl<const ORTHOGRAPHIES: usize> PhonemeBehavior<ORTHOGRAPHIES> {

  pub(crate) const fn new(spelling: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES]) -> Self {
    Self {
      spelling
    }
  }

  pub(crate) const fn spelling(&self) -> &[SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES] {
      &self.spelling
  }

}
