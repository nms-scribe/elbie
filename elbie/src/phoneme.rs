use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use rand::rngs::ThreadRng;

use crate::bag::Bag;
use crate::errors::ElbieError;

pub mod ipa;

pub const PHONEME: &str = "phoneme";
pub const EMPTY: &str = "empty";


#[derive(Debug,Ord,PartialOrd,Eq,PartialEq,Hash)]
// FUTURE: When I originally wrote this, I was under the impression that Rc objects would not be equal if they aren't the same object. However, I never tested that, and a simple test shows that they can be equal and be different objects. This means that what I'm doing here is wrong, and I probably shouldn't even bother.
// Fact is, it doesn't matter too much if phonemes are equal, even across languages, because 1) as long as I'm using standard IPA then equal phonemes will probably be pronounced similarly enough to get by and 2) it makes transformations a *lot* easier if I don't have to namespace them.
// Another reason I may have originally had these objects was that I had an idea of keeping their "sets" on the phoneme itself. However, I don't need that to happen. And it seems like phonemes only make sense to have sets in the specific context of a language. Outside of that context, having a set is meaningless. (This is based on the definition of phonemes in linguistics, at least as I understand it)
// So, I feel like it would be better not to have separate Phonemes, and just use Rc<str> or Box<str> as the phoneme. The Inventory would have a Set instead of a Map. And it would simplify a bunch of the code since I could even have phonemes that didn't exist in an inventory.
pub struct Phoneme {
  pub name: &'static str
}

impl Phoneme {
  pub(crate) fn new(name: &'static str) -> Rc<Self> {
    Rc::new(Self {
      name
    })
  }

}

impl Display for Phoneme {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    write!(f,"/{}/",self.name)
  }

}


pub trait InventoryLoader {

    fn add_phoneme(&mut self, phoneme: &'static str, sets: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError>;

    fn add_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),ElbieError>;

    fn add_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError>;

    fn add_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError>;

    fn add_exclusion(&mut self, name: &'static str, set: &'static str, exclude_phoneme_strs: &[&'static str]) -> Result<(),ElbieError>;

}


#[derive(Debug)]
pub struct Inventory {
  phonemes: HashMap<&'static str,Rc<Phoneme>>,
  sets: HashMap<&'static str,Bag<Rc<Phoneme>>>, // It seems like a hashset would be better, but I can't pick randomly from it without converting to vec anyway.
}

impl Default for Inventory {
    fn default() -> Self {
        let mut sets = HashMap::new();
        _ = sets.insert(PHONEME, Bag::new());
        _ = sets.insert(EMPTY, Bag::new());
        let phonemes = HashMap::new();
        Self {
          phonemes,
          sets,
        }
    }
}

impl Inventory {

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) const fn phonemes(&self) -> &HashMap<&'static str, Rc<Phoneme>> {
        &self.phonemes
    }

    fn add_phoneme_to_set(&mut self, class: &'static str, phoneme: Rc<Phoneme>) -> Result<(),ElbieError> {
      let class = match self.sets.entry(class) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            if self.phonemes.contains_key(class) {
                return Err(ElbieError::PhonemeExistsWithSetName(class))
            }
            entry.insert(Bag::new())
        },
      };
      if !class.contains(&phoneme) {
        _ = class.insert(phoneme);
      }
      Ok(())
    }


    pub(crate) fn get_set(&self, set: &'static str) -> Result<&Bag<Rc<Phoneme>>,ElbieError> {
      match self.sets.get(set) {
        Some(set) => Ok(set),
        None => Err(ElbieError::UnknownSet(set))
      }
    }

    pub(crate) fn get_phoneme(&self, phoneme: &'static str) -> Result<&Rc<Phoneme>,ElbieError> {
      match self.phonemes.get(phoneme) {
        Some(phoneme) => Ok(phoneme),
        None => Err(ElbieError::UnknownPhoneme(phoneme))
      }
    }

    pub(crate) fn get_set_without(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>]) -> Result<Bag<Rc<Phoneme>>,ElbieError> {
      let mut set = self.get_set(set)?.clone();
      for phoneme in exclude_phonemes {
        _ = set.remove(phoneme);
      }
      Ok(set)
    }

    pub(crate) fn phoneme_is(&self, phoneme: &Rc<Phoneme>, set: &'static str) -> Result<bool,ElbieError> {
      Ok(self.get_set(set)?.contains(phoneme))
    }

    pub(crate) fn choose(&self, set: &'static str, rng: &mut ThreadRng) -> Result<Rc<Phoneme>,ElbieError> {
      match self.get_set(set)?.choose(rng) {
        Some(phoneme) => Ok(phoneme.clone()),
        None => Err(ElbieError::SetIsEmpty(set))
      }
    }

    pub(crate) fn choose_except(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>], rng: &mut ThreadRng) -> Result<Rc<Phoneme>,ElbieError> {
      match self.get_set_without(set,exclude_phonemes)?.choose(rng) {
        Some(phoneme) => Ok(phoneme.clone()),
        None => Err(ElbieError::SetIsEmptyWithFilter(set))
      }
    }

    pub(crate) fn extend(&mut self, other: &Self, containing_set: &'static str) -> Result<(),ElbieError> {
        #[expect(clippy::iter_over_hash_type,reason="Order for this doesn't matter")]
        for (name,bag) in &other.sets {
            for phoneme in bag.iter() {
                let phoneme = self.phonemes.entry(phoneme.name).or_insert_with(|| phoneme.clone()).clone();
                self.add_phoneme_to_set(name, phoneme)?;
            }
        }

        // make sure any phonemes that weren't in sets are added, and also add the phoneme to the containing set.
        #[expect(clippy::iter_over_hash_type,reason="Order for this doesn't matter")]
        for (name,phoneme) in other.phonemes() {
            let phoneme = self.phonemes.entry(name).or_insert_with(|| phoneme.clone()).clone();
            self.add_phoneme_to_set(containing_set, phoneme)?;
        }

        Ok(())
    }


}

impl InventoryLoader for Inventory {


    fn add_phoneme(&mut self, phoneme: &'static str, sets: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
      if self.phonemes.contains_key(phoneme) {
        Err(ElbieError::PhonemeAlreadyExists(phoneme))
      } else if self.sets.contains_key(phoneme) {
        Err(ElbieError::SetExistsWithPhonemeName(phoneme))
      } else {
        let phoneme = Phoneme::new(phoneme);
        _ = self.phonemes.insert(phoneme.name, phoneme.clone());
        self.add_phoneme_to_set(PHONEME,phoneme.clone())?;
        for class in sets {
          self.add_phoneme_to_set(class,phoneme.clone())?
        }
        Ok(phoneme)
      }

    }

    fn add_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),ElbieError> {
      if self.sets.contains_key(name) {
        Err(ElbieError::SetAlreadyExists(name))
      } else if self.phonemes.contains_key(name) {
        Err(ElbieError::PhonemeExistsWithSetName(name))
      } else {
        let mut set = self.get_set(base_set)?.clone();
        for subset in exclude_sets {
          let subset = self.get_set(subset)?;
          set = set.difference(subset);
        }
        _ = self.sets.insert(name, set);
        Ok(())
      }
    }

    fn add_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError> {
      if self.sets.contains_key(name) {
        Err(ElbieError::SetAlreadyExists(name))
      } else if self.phonemes.contains_key(name) {
        Err(ElbieError::PhonemeExistsWithSetName(name))
      } else {
          let mut sets = sets.iter();
          if let Some(set) = sets.next() {
              let mut set = self.get_set(set)?.clone();
              for subset in sets {
                  let subset = self.get_set(subset)?;
                  set = set.intersection(subset)
              }
              _ = self.sets.insert(name, set);
              Ok(())
          } else {
              Err(ElbieError::SetIsEmpty(name))
          }

      }

    }

    fn add_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError> {
      if self.sets.contains_key(name) {
        Err(ElbieError::SetAlreadyExists(name))
      } else if self.phonemes.contains_key(name) {
        Err(ElbieError::PhonemeExistsWithSetName(name))
      } else {
        let mut set = Bag::new();
        for subset in sets {
          let subset = self.get_set(subset)?;
          set = set.union(subset);
        }
        _ = self.sets.insert(name, set);
        Ok(())
      }

    }

    fn add_exclusion(&mut self, name: &'static str, set: &'static str, exclude_phoneme_strs: &[&'static str]) -> Result<(),ElbieError> {

      if self.sets.contains_key(name) {
        Err(ElbieError::SetAlreadyExists(name))
      } else if self.phonemes.contains_key(name) {
        Err(ElbieError::PhonemeExistsWithSetName(name))
      } else {
        let mut exclude_phonemes = vec![];
        for phoneme in exclude_phoneme_strs {
          exclude_phonemes.push(self.get_phoneme(phoneme)?);
        }
        let set = self.get_set_without(set, &exclude_phonemes)?;
        _ = self.sets.insert(name,set);
        Ok(())

      }

    }


}
