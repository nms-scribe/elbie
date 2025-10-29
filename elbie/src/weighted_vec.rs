use rand::prelude::ThreadRng;
use rand::Rng as _;

#[derive(Debug,Clone)]
pub(crate) struct WeightedVec<ItemType>{
  items: Vec<(ItemType,usize)>,
  total_weight: usize
}

impl<ItemType> WeightedVec<ItemType> {

  pub(crate) const fn new() -> Self {
    Self {
      items: vec![],
      total_weight: 0
    }
  }

  pub(crate) const fn items(&self) -> &Vec<(ItemType, usize)> {
      &self.items
  }

  pub(crate) fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
    let mut choice_weight = rng.random_range(1..self.total_weight+1);
    for choice in &self.items {
      if choice_weight <= choice.1 {
        return Some(&choice.0)
      }
      choice_weight -= choice.1;
    }
    None

  }

  // NOTE: Specifying a weight of 0 is not an error, but that item will never get chosen.
  pub(crate) fn push(&mut self, value: ItemType, weight: usize) {
    self.items.push((value,weight));
    self.total_weight += weight;
  }

}
