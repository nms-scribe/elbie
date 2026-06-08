use rand::Rng as _;
use rand::prelude::ThreadRng;

#[derive(Debug, Clone)]
pub(crate) struct WeightedVec<ItemType> {
    items: Vec<(ItemType, usize)>,
    total_weight: usize
}

impl<ItemType> WeightedVec<ItemType> {
    pub(crate) const fn new() -> Self {
        Self { items: vec![],
               total_weight: 0 }
    }

    pub(crate) const fn items(&self) -> &Vec<(ItemType, usize)> {
        &self.items
    }

    pub(crate) fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
        // the range starting at 1 ensures that if the first items hav a weight of 0, they will not get chosen.
        // In every other case, if an item has a weight of 0, the item before it would get chosen before it gets chosen.
        let mut choice_weight = rng.random_range(1..=self.total_weight);
        for choice in &self.items {
            if choice_weight <= choice.1 {
                return Some(&choice.0);
            }
            choice_weight -= choice.1;
        }
        None
    }

    // NOTE: Specifying a weight of 0 is not an error, but that item will never get chosen.
    // It will still validate as an option, however. This is useful for certain cases where
    // a phonemic pattern is only found in specific fixed-vocabulary words, like prepositions
    // or pronouns.
    pub(crate) fn push(&mut self, value: ItemType, weight: usize) {
        self.items.push((value, weight));
        self.total_weight += weight;
    }
}
