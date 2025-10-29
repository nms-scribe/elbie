use rand::prelude::ThreadRng;
use rand::seq::IndexedRandom as _;
use core::cmp::Ordering;

// A set that I can random access. It's more efficient than random access of a HashSet (which can't retrieve by index), and also allows retaining insert order.
// But probably could be better.
#[derive(Debug,Clone)]
pub(crate) struct Bag<ItemType>(Vec<ItemType>);

impl<ItemType: Clone + Ord> Bag<ItemType> {

  pub(crate) const fn new() -> Self {
    Self(Vec::new())
  }

  pub(crate) const fn is_empty(&self) -> bool {
    self.0.is_empty()
  }


  pub(crate) fn set_operation(&self, other: &Self, insert_if_in_self: bool, insert_if_in_other: bool, insert_if_in_both: bool) -> Self {
    let mut self_iter = self.0.iter();
    let mut other_iter = other.0.iter();
    let mut result: Self = Self::new();

    // This algorithm should be more efficient because I know both vectors are sorted, rather than doing the 'contains'
    // and a binary_search for every check. There could still be some improvements.

    let mut self_next = self_iter.next();
    let mut other_next = other_iter.next();

    // NOTE: To be very clear, this is not a iter.zip. We're only iterating items conditionally.
    loop {
      match (self_next,other_next) {
        (Some(self_some), Some(other_some)) => {
          match self_some.cmp(other_some) {
            Ordering::Less => {
              // self is less, so it is in self but not other. (if it had been in other, we would have seen it by now)
              if insert_if_in_self {
                _ = result.insert(self_some.clone());
              }
              // iterate self, but not other.
              self_next = self_iter.next();
            },
            Ordering::Greater => {
              // other is less, so it is in other, but not self (if it had been in self, we would have seen it by now)
              if insert_if_in_other {
                _ = result.insert(other_some.clone());
              }
              other_next = other_iter.next();
            },
            Ordering::Equal => {
              // both are equal, so it is in both
              if insert_if_in_both {
                _ = result.insert(self_some.clone());
              }
              self_next = self_iter.next();
              other_next = other_iter.next();
            },
          }
        },
        (Some(self_some),None) => {
            if insert_if_in_self {
              _ = result.insert(self_some.clone());
            }
            self_next = self_iter.next();
        },
        (None,Some(other_some)) => {
            if insert_if_in_other {
              _ = result.insert(other_some.clone());
            }
            other_next = other_iter.next();

        },
        (None, None) => break, // we've exhausted both
      }
    }

    result
  }

  // returns a new bag containing objects in either self or other
  pub(crate) fn union(&self, other: &Self) -> Self {
    self.set_operation(other, true, true, true)
  }

  // returns a new bag containing objects in self, but not in other.
  pub(crate) fn difference(&self, other: &Self) -> Self {
    self.set_operation(other, true, false, false)
  }

  // returns a new bag containing objects both in self and other
  pub(crate) fn intersection(&self, other: &Self) -> Self {
    self.set_operation(other, false, false, true)

  }

  // returns a new bag containing objects in self or other but not both
  pub(crate) fn _symmetric_difference(&self, other: &Self) -> Self {
    self.set_operation(other, true, true, false)
  }


  // returns true if the specified value is contained in the bag.
  pub(crate) fn contains(&self, value: &ItemType) -> bool {
    self.0.binary_search(value).is_ok()
  }

  // inserts the item if it isn't already in the bag. Returns true if it was inserted.
  pub(crate) fn insert(&mut self, value: ItemType) -> bool {
    match self.0.binary_search(&value) {
      Ok(_) => false,
      Err(pos) => {
        self.0.insert(pos, value);
        true
      }
    }

  }

  pub(crate) fn remove(&mut self, value: &ItemType) -> Option<ItemType> {
    match self.0.binary_search(value) {
      Ok(pos) => {
        Some(self.0.remove(pos))
      }
      Err(_) => None
    }
  }

  // randomly chooses an item from the bag and returns it.
  pub(crate) fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
    self.0.choose(rng)
  }

  pub(crate) fn list(&self) -> Vec<ItemType> {
    self.0.clone()
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = &ItemType> {
      self.0.iter()
  }


}
