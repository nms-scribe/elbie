use std::collections::HashMap;
use std::rc::Rc;
use std::error::Error;
use rand::Rng;
use rand::seq::SliceRandom; 
use rand::prelude::ThreadRng;
use csv::Reader;
use std::fmt::Display;

mod chart;
#[cfg(test)] mod test;

use chart::Chart;
use chart::ChartStyle;

/*
Elbie = LB, Language Builder, and is a bunch of tools for building a constructed language.
*/

/*
FUTURE: Implementing syllable breaks, stress, etc, Simple Solution:
- a "word" is sequence of syllables, not phonemes. A syllable is a sequence of phonemes. I don't think we need to support onset/rhyme structure, since that could be analyzed differently.
- A syllable can also have stress, tone, etc.
- spelling callbacks are the hardest part to deal with, but I'm not sure these are great anyway. Spelling might be a type of transformation.
- Another difficulty is "converting" old words, which won't have the syllable breaks and stress indicators. The best thing I can think of is to have the validators guess when a syllable break is missing, and warn about modifiers missing without stopping the process.

There have been arguments against syllables being a real thing, but I feel like their usage in analysis is big enough that I can still use them.
https://web.archive.org/web/20150923211920/http://www.cunyphonologyforum.net/syllable.php
https://web.archive.org/web/20150918220252/http://cunyphonologyforum.wikifoundry.com/page/Paraphonological+Phenomena

FUTURE: Implement transformations:
* regular sound change for building lexicons of daughter languages
* regular sound changes for loan words from other languages (I don't expect this to be common)
* orthography -- the same pattern matching of sound change could potentially be used to create more realistic orthography
- This is mostly something very similar to regular expressions, searching for patterns in a word, possibly capturing some patterns, and replacing them with other patterns. The final test, however, would require validation to a new language, or something like that.

FUTURE: Is there some way to use types or something else to make languages easier to create? 
- One issue is the use of string constants to identify environments, sets, phonemes, etc. 
  - There is a small possibility that I could repeat the string name under two different constant names, which could cause some hard to debug issues.
  - The use of a string constant removes some useful type-checking: if I specify an environment name instead of a set name, I don't know until run-time.
  - It would be nice if I could just have "phoneme" and "phoneme_set" objects and the like that can be reference by variable, and have internal access to the language they are associated with. (For example, "fricative.intersect_with(glottal)" should work without having to retrieve things off of the language, or even without having a string name)
- Constant type parameters are now possible in rust, there might be something I could use out of that.

// FUTURE: Is there some way I can do the environments and sets as types? Maybe phonemes, sets and environments are traits instead that you implement in structs. I might be able to use generic constant parameters to help with that.
// I could use macros to make those implementations easier to code. Phonemes should really be enumerations. This would require the language to be generic
// and base itself off of phonemes. --- I think the hardest part is implementing a set that describes which phonemes can be chosen, and then to choose such a 
// type randomly?


*/

pub const PHONEME: &'static str = "phoneme";
pub const EMPTY: &'static str = "empty";

trait UsizeHelper {

  fn div_ceil(&self, rhs: Self) -> Self;
  

}

impl UsizeHelper for usize {

  fn div_ceil(&self, rhs: Self) -> Self {
    let d = self / rhs;
    let r = self % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
  }
  
}

trait VecHelper<T> {

  fn expand_to<F>(&mut self, new_len: usize, f: F)
  where
      F: FnMut() -> T;
  

}

impl<T> VecHelper<T> for Vec<T> {
  
    fn expand_to<F>(&mut self, new_len: usize, f: F)
      where
          F: FnMut() -> T {
      if self.len() < new_len {
          self.resize_with(new_len,f)
      }
    }
}


#[derive(Debug,Clone)]
pub enum LanguageError {
  SetIsEmpty(&'static str),
  SetIsEmptyWithFilter(&'static str),
  UnknownSet(&'static str),
  UnknownPhoneme(&'static str),
  PhonemeAlreadyExists(&'static str),
  SetAlreadyExists(&'static str),
  EnvironmentAlreadyExists(&'static str),
  UnknownEnvironment(&'static str),
  NoEnvironmentChoices(&'static str),
  IncompleteBranches(&'static str),
  // word validation errors
  EmptyWord,
  IncorrectPhoneme(usize, Rc<Phoneme>, &'static str, &'static str),
  ExpectedEndOfWord(usize, Rc<Phoneme>, &'static str),
  ExpectedPhonemeFoundEndOfWord(usize, &'static str, &'static str),
  NoBranchFitsPhoneme(usize, Rc<Phoneme>, &'static str),
  // word reading errors
  UnknownPhonemeWhileReading(String,String)

}

impl Display for LanguageError {
  
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
    match self {
      Self::SetIsEmpty(name) => write!(f,"Set {} has no phonemes.",name),
      Self::SetIsEmptyWithFilter(name) => write!(f,"Set {} as filtered has no phonemes.",name),
      Self::UnknownSet(name) => write!(f,"Unknown set {}.",name),
      Self::UnknownPhoneme(name) => write!(f,"Unknown phoneme {}.",name),
      Self::PhonemeAlreadyExists(name) => write!(f,"Phoneme {} already exists.",name),
      Self::SetAlreadyExists(name) => write!(f,"Set {} already exists.",name),
      Self::EnvironmentAlreadyExists(name) => write!(f, "Environment {} already exists.",name),
      Self::UnknownEnvironment(name) => write!(f,"Unknown environment {}.",name),
      Self::NoEnvironmentChoices(name) => write!(f,"Environment {} is missing some branch environment choices.",name),
      Self::IncompleteBranches(name) => write!(f,"Environment {} is missing some possible branches.",name),

      Self::EmptyWord => write!(f,"Word is empty"),
      Self::IncorrectPhoneme(index,phoneme,set,environment) => write!(f,"[Environment {} at {}]: Expected {}, found phoneme ({}).",environment,index,set,phoneme),
      Self::ExpectedEndOfWord(index,phoneme,environment) => write!(f,"[Environment {} at {}]: Expected end of word, found phoneme ({})",environment,index,phoneme),
      Self::ExpectedPhonemeFoundEndOfWord(index,set,environment) => write!(f,"[Environment {} at {}]: Expected {}, found end of word",environment,index,set),
      Self::NoBranchFitsPhoneme(index,phoneme,environment) => write!(f,"[Environment {} at {}]: Phoneme ({}) does not match any branch.",environment,index,phoneme),

      Self::UnknownPhonemeWhileReading(source,problem) => write!(f,"In word '{}': unknown phoneme starting at '{}'.",source,problem)
    }

  }
}

impl Error for LanguageError {

}

// A set that I can random access. It's more efficient than random access of a HashSet, but probably could be better.
#[derive(Debug,Clone)]
struct Bag<ItemType>(Vec<ItemType>);

impl<ItemType: Clone + Ord> Bag<ItemType> {

  fn new() -> Self {
    Bag(Vec::new())
  }

  fn is_empty(&self) -> bool {
    self.0.len() == 0
  }


  fn set_operation(&self, other: &Bag<ItemType>, insert_if_in_self: bool, insert_if_in_other: bool, insert_if_in_both: bool) -> Self {
    let mut self_iter = self.0.iter();
    let mut other_iter = other.0.iter();
    let mut result: Self = Bag::new();

    // This algorithm should be more efficient because I know both vectors are sorted, rather than doing the 'contains'
    // and a binary_search for every check. There could still be some improvements.
  
    let mut self_next = self_iter.next();
    let mut other_next = other_iter.next();

    // NOTE: To be very clear, this is not a iter.zip. We're only iterating items conditionally.
    loop {
      match (self_next,other_next) {
        (Some(self_some), Some(other_some)) => {
          match self_some.cmp(other_some) {
            std::cmp::Ordering::Less => {
              // self is less, so it is in self but not other. (if it had been in other, we would have seen it by now)
              if insert_if_in_self {
                result.insert(self_some.clone()); 
              }
              // iterate self, but not other. 
              self_next = self_iter.next();
            },
            std::cmp::Ordering::Greater => {
              // other is less, so it is in other, but not self (if it had been in self, we would have seen it by now)
              if insert_if_in_other {
                result.insert(other_some.clone()); 
              }
              other_next = other_iter.next();
            },
            std::cmp::Ordering::Equal => {
              // both are equal, so it is in both
              if insert_if_in_both {
                result.insert(self_some.clone()); 
              }
              self_next = self_iter.next();
              other_next = other_iter.next();
            },
          }
        },
        (Some(self_some),None) => {
            if insert_if_in_self {
              result.insert(self_some.clone());
            }
            self_next = self_iter.next();
        },
        (None,Some(other_some)) => {
            if insert_if_in_other {
              result.insert(other_some.clone());
            }
            other_next = other_iter.next();

        },
        (None, None) => break, // we've exhausted both
      }
    }
  
    result
  }

  // returns a new bag containing objects in either self or other
  fn union(&self, other: &Bag<ItemType>) -> Self {
    self.set_operation(other, true, true, true)
  }

  // returns a new bag containing objects in self, but not in other.
  fn difference(&self, other: &Bag<ItemType>) -> Self {
    self.set_operation(other, true, false, false)
  }

  // returns a new bag containing objects both in self and other
  fn intersection(&self, other: &Bag<ItemType>) -> Self {
    self.set_operation(other, false, false, true)

  }

  // returns a new bag containing objects in self or other but not both 
  fn _symmetric_difference(&self, other: &Bag<ItemType>) -> Self {
    self.set_operation(other, true, true, false)
  }


  // returns true if the specified value is contained in the bag.
  fn contains(&self, value: &ItemType) -> bool {
    if let Ok(_) = self.0.binary_search(value) {
      true
    } else {
      false
    }

  }

  // inserts the item if it isn't already in the bag. Returns true if it was inserted.
  fn insert(&mut self, value: ItemType) -> bool {
    match self.0.binary_search(&value) {
      Ok(_) => false,
      Err(pos) => {
        self.0.insert(pos, value);
        true
      }
    }

  }

  fn remove(&mut self, value: &ItemType) -> bool {
    match self.0.binary_search(&value) {
      Ok(pos) => {
        self.0.remove(pos);
        true
      }
      Err(_) => false
    }
  }

  // randomly chooses an item from the bag and returns it.
  fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
    self.0.choose(rng)
  }

  fn list(&self) -> Vec<ItemType> {
    self.0.clone()
  }


}

#[derive(Debug,Clone)]
struct WeightedVec<ItemType>{
  items: Vec<(ItemType,usize)>,
  total_weight: usize
}

impl<ItemType> WeightedVec<ItemType> {

  fn new() -> WeightedVec<ItemType> {
    WeightedVec {
      items: vec![],
      total_weight: 0
    }
  }

  fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
    let mut choice_weight = rng.gen_range(1..self.total_weight+1);
    for choice in &self.items {
      if choice_weight <= choice.1 {
        return Some(&choice.0)
      }
      choice_weight -= choice.1;
    }
    None
    
  }

  // NOTE: Specifying a weight of 0 is not an error, but that item will never get chosen.
  fn push(&mut self, value: ItemType, weight: usize) {
    self.items.push((value,weight));
    self.total_weight += weight;
  }

}

#[derive(Debug,Ord,PartialOrd,Eq,PartialEq,Hash)]
pub struct Phoneme {
  pub name: &'static str
}

impl Phoneme {
  fn new(name: &'static str) -> Rc<Self> {
    Rc::new(Phoneme {
      name
    })
  }

}

impl Display for Phoneme {
  
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
    write!(f,"/{}/",self.name)
  }
}


type SpellingCallback<const ORTHOGRAPHIES: usize> = fn(&Language<ORTHOGRAPHIES>, &std::rc::Rc<Phoneme>, &mut std::string::String, &mut std::iter::Peekable<std::slice::Iter<std::rc::Rc<Phoneme>>>);

#[derive(Default)]
pub enum SpellingBehavior<const ORTHOGRAPHIES: usize> {
  #[default]
  Default, // default behavior is to spell the phoneme
  Text(&'static str),
  Callback(SpellingCallback<ORTHOGRAPHIES>)
}

impl<const ORTHOGRAPHIES: usize> std::fmt::Debug for SpellingBehavior<ORTHOGRAPHIES> {

  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
    write!(f,"PhonemeBehavior::")?;
    match self {
      SpellingBehavior::Default => write!(f,"Default"),
      SpellingBehavior::Text(text) => write!(f,"Text({})",text),
      SpellingBehavior::Callback(_) => write!(f,"Callback(<...>)"),
    }

  }
}

#[derive(Debug)]
struct PhonemeBehavior<const ORTHOGRAPHIES: usize> {
  spelling: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES]
}

impl<const ORTHOGRAPHIES: usize> Default for PhonemeBehavior<ORTHOGRAPHIES> {

  fn default() -> Self {
    Self {
      spelling: std::array::from_fn(|_| SpellingBehavior::default())
    }
  }
}

impl<const ORTHOGRAPHIES: usize> PhonemeBehavior<ORTHOGRAPHIES> {

  fn new(spelling: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES]) -> Self {
    Self {
      spelling
    }
  }

}

#[derive(Debug)]
pub struct Word {
  phonemes: Vec<Rc<Phoneme>>
}

impl Word {
  fn new(phonemes: &[Rc<Phoneme>]) -> Self {
    let phonemes = phonemes.to_vec();
    Word{phonemes}
  }

  fn push(&mut self,phoneme: Rc<Phoneme>) {
    self.phonemes.push(phoneme)
  }

  fn _last(&self) -> Option<&Rc<Phoneme>> {
    self.phonemes.last()
  }

}

impl Display for Word {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
    write!(f,"/")?;
    for phoneme in &self.phonemes {
      write!(f,"{}",phoneme.name)?
    }
    write!(f,"/")?;
    Ok(())
  }

}


#[derive(Debug,Clone)]
pub enum EnvironmentChoice {
  Done,
  Continuing(&'static str,&'static str,bool),// set to generate next phoneme from, next environment to follow, whether to allow duplicate phoneme to be generated
}


#[derive(Debug,Clone)]
pub struct EnvironmentBranch(&'static str,WeightedVec<EnvironmentChoice>);

impl EnvironmentBranch {

  pub fn new(set_check: &'static str, choices: &[(EnvironmentChoice,usize)]) -> Self {
    let mut vec = WeightedVec::new();
    for choice in choices {
      vec.push(choice.0.clone(),choice.1)
    };
    EnvironmentBranch(set_check,vec)

  }
}

type TableAxis = Vec<(&'static str,&'static str)>; // a list of sets to use in the axis. The first string is the caption, the second string is the set name.

/*
NOTE: Four seems like an arbitrary limit. I used to have this all in a vector so the limit was usize. However, this is a user interface thing. The third and fourth axis basically just add more items to a cell in a table. Trying to do more than that is going to be difficult to represent in a way that a human to understand, and it makes processing the table harder to program. I believe such distinctions would not be found in most languages anyway.

The good news is that this doesn't limit the language if the user wants something really alien. They can just separate one of the lower axes into separate tables instead, and then they can still use this.
*/
#[derive(Debug)]
enum Table {
  NotATable,
  ColumnsOnly(TableAxis),
  ColumnsAndRows(TableAxis,TableAxis),
  ColumnsSubcolumnsAndRows(TableAxis,TableAxis,TableAxis),
  ColumnsSubcolumnsRowsAndSubrows(TableAxis,TableAxis,TableAxis,TableAxis)
}

impl Table {

  fn new(axisses: &[&[(&'static str,&'static str)]]) -> Self {

    macro_rules! axis {
        ($index: literal) => {
          axisses[$index].to_vec()
        };
    }

    match axisses.len() {
        0 => Self::NotATable,
        1 => Self::ColumnsOnly(axis!(0)),
        2 => Self::ColumnsAndRows(axis!(0),axis!(1)),
        3 => Self::ColumnsSubcolumnsAndRows(axis!(0),axis!(1),axis!(2)),
        4 => Self::ColumnsSubcolumnsRowsAndSubrows(axis!(0),axis!(1),axis!(2),axis!(3)),
        _ => panic!("Tables can only have from 1 to 4 axisses.")
    }
  }


  fn columns(&self) -> Option<&TableAxis> {
    match self {
      Self::NotATable => None,
      Self::ColumnsOnly(a) => Some(a),
      Self::ColumnsAndRows(a, _) => Some(a),
      Self::ColumnsSubcolumnsAndRows(a, _, _) => Some(a),
      Self::ColumnsSubcolumnsRowsAndSubrows(a, _, _, _) => Some(a),
    }
  }

  fn rows(&self) -> Option<&TableAxis> {
    match self {
      Self::NotATable => None,
      Self::ColumnsOnly(_) => None,
      Self::ColumnsAndRows(_, a) => Some(a),
      Self::ColumnsSubcolumnsAndRows(_, a, _) => Some(a),
      Self::ColumnsSubcolumnsRowsAndSubrows(_, a, _, _) => Some(a),
    }
  }

  fn subcolumns(&self) -> Option<&TableAxis> {
    match self {
      Self::NotATable => None,
      Self::ColumnsOnly(_) => None,
      Self::ColumnsAndRows(_, _) => None,
      Self::ColumnsSubcolumnsAndRows(_, _, a) => Some(a),
      Self::ColumnsSubcolumnsRowsAndSubrows(_, _, a, _) => Some(a),
    }
  }

  fn subrows(&self) -> Option<&TableAxis> {
    match self {
      Self::NotATable => None,
      Self::ColumnsOnly(_) => None,
      Self::ColumnsAndRows(_, _) => None,
      Self::ColumnsSubcolumnsAndRows(_, _, _) => None,
      Self::ColumnsSubcolumnsRowsAndSubrows(_, _, _, a) => Some(a),
    }
  }


}

#[derive(Clone)]
pub enum ValidWordElement {
  Done(usize,&'static str), // environment
  Phoneme(usize,Rc<Phoneme>,&'static str,&'static str) // found phoneme, expected set, expected environment
}

impl std::fmt::Display for ValidWordElement {

  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
    match self {
      ValidWordElement::Done(index,environment) => write!(f,"[Environment {} at {}]: end of word",environment,index),
      ValidWordElement::Phoneme(index,phoneme,set,environment) => write!(f,"[Environment {} at {}]: phoneme ({}) from {}.",environment,index,phoneme,set),
    }

  }
}


pub enum ValidationTraceMessage<'lifetime> {
  FoundValid(&'lifetime ValidWordElement),
  FoundError(&'lifetime LanguageError)
}

impl std::fmt::Display for ValidationTraceMessage<'_> {

  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
    match self {
      Self::FoundValid(valid) => write!(f,"Found valid: {}",valid),
      Self::FoundError(err) => write!(f,"!!!Found error: {}",err), 
    }

  }
}


pub type ValidationTraceCallback = dyn Fn(usize, ValidationTraceMessage);

pub struct LexiconEntry<const ORTHOGRAPHIES: usize> {
  word: Word,
  spelling: [String; ORTHOGRAPHIES],
  definition: String
}


#[derive(Debug)]
pub struct Language<const ORTHOGRAPHIES: usize> {
  name: &'static str, 
  initial_environment: &'static str,
  initial_phoneme_set: &'static str,
  phonemes: HashMap<&'static str,Rc<Phoneme>>,
  // These are kept separate from the phoneme structure to reduce some type dependencies.
  // For example, if this were part of the Phoneme structure, the ORTHOGRAPHIES parameter would be required on almost everything.
  phoneme_behavior: HashMap<Rc<Phoneme>,PhonemeBehavior<ORTHOGRAPHIES>>,
  orthographies: [&'static str; ORTHOGRAPHIES],
  environments: HashMap<&'static str,Vec<EnvironmentBranch>>,
  sets: HashMap<&'static str,Bag<Rc<Phoneme>>>, // It seems like a hashset would be better, but I can't pick randomly from it without converting to vec anyway.
  tables: Vec<(&'static str,&'static str,Table)> // (caption, set name, table axes)
}

impl<const ORTHOGRAPHIES: usize> Language<ORTHOGRAPHIES> {

    pub fn new(name: &'static str, initial_phoneme_set: &'static str, initial_environment: &'static str, orthographies: [&'static str; ORTHOGRAPHIES]) -> Self {
      let mut sets = HashMap::new();
      sets.insert(PHONEME, Bag::new());
      sets.insert(EMPTY, Bag::new());
      let phonemes = HashMap::new();
      let environments = HashMap::new();
      let phoneme_behavior = HashMap::new();
      let tables = vec![];
      Language {
        name,
        initial_environment,
        initial_phoneme_set,
        phonemes,
        phoneme_behavior,
        orthographies,
        environments,
        sets,
        tables
      }

    }

    fn add_phoneme_to_class(&mut self, class: &'static str, phoneme: Rc<Phoneme>) {
      let class = self.sets.entry(class).or_insert(Bag::new());
      if !class.contains(&phoneme) {
        class.insert(phoneme);
      }
    }

    fn add_phoneme_object(&mut self, phoneme: Rc<Phoneme>, classes: &[&'static str], behavior: PhonemeBehavior<ORTHOGRAPHIES>) -> Result<Rc<Phoneme>,LanguageError> {
      if let Some(_) = self.phonemes.get(phoneme.name) {
        Err(LanguageError::PhonemeAlreadyExists(phoneme.name))
      } else {
        self.phonemes.insert(phoneme.name, phoneme.clone());
        self.phoneme_behavior.insert(phoneme.clone(), behavior);
        self.add_phoneme_to_class(PHONEME,phoneme.clone());
        for class in classes {
          self.add_phoneme_to_class(class,phoneme.clone())
        }
        Ok(phoneme)
      }

    }

    pub fn add_phoneme(&mut self, phoneme: &'static str, classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      self.add_phoneme_object(Phoneme::new(phoneme),classes,PhonemeBehavior::default())
    }

    pub fn add_phoneme_with_spelling(&mut self, phoneme: &'static str, orthography: [&'static str; ORTHOGRAPHIES], classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      let behaviors = orthography.map(|t| SpellingBehavior::Text(t));
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_fn(&mut self, phoneme: &'static str, callbacks: [SpellingCallback<ORTHOGRAPHIES>; ORTHOGRAPHIES], classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      let behaviors = callbacks.map(|f| SpellingBehavior::Callback(f));
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_behavior(&mut self, phoneme: &'static str, behaviors: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES], classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      self.add_phoneme_object(Phoneme::new(phoneme),classes,PhonemeBehavior::new(behaviors))
    }

    pub fn spell_phoneme(&self, phoneme: &Rc<Phoneme>, orthography: usize, result: &mut String, next: &mut std::iter::Peekable<std::slice::Iter<Rc<Phoneme>>>) {
      if orthography >= ORTHOGRAPHIES {
        panic!("Language only has {} orthographies.",ORTHOGRAPHIES)
      }

      match self.phoneme_behavior.get(phoneme).map(|b| &b.spelling[orthography]) {
        None | Some(SpellingBehavior::Default) => result.push_str(phoneme.name),
        Some(SpellingBehavior::Text(text)) => result.push_str(text),
        Some(SpellingBehavior::Callback(callback)) => callback(self,phoneme,result,next)
      }

    }

    pub fn spell_word(&self, word: &Word, orthography: usize) -> String {
      let mut result = String::new();
      let mut iter = word.phonemes.iter().peekable();
      while let Some(phoneme) = iter.next() {
        self.spell_phoneme(phoneme,orthography,&mut result,&mut iter)
      }
      result
    }

    // will eventually be used over add_difference
    pub fn build_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),LanguageError> {
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let mut set = self.get_set(base_set)?.clone();
        for subset in exclude_sets {
          let subset = self.get_set(subset)?;
          set = set.difference(subset);
        }
        self.sets.insert(name, set);
        Ok(())
      }
    }

    pub fn build_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),LanguageError> {
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let set = if sets.len() > 0 {
          let mut set = self.get_set(sets[0])?.clone();
          for subset in sets.iter().skip(1) {
            let subset = self.get_set(subset)?;
            set = set.intersection(subset);
          }
          set
        } else {
          Bag::new()
        };
        if set.is_empty() {
          Err(LanguageError::SetIsEmpty(name))
        } else {
          self.sets.insert(name, set);
          Ok(())
        }
      }

    }

    // allows building a union out of multiple sets... FUTURE: The 'add' functions will become obsolete and replace with 'build' functions.
    pub fn build_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),LanguageError> {
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let mut set = Bag::new();
        for subset in sets {
          let subset = self.get_set(subset)?;
          set = set.union(subset);
        }
        self.sets.insert(name, set);
        Ok(())
      }

    }

    pub fn add_exclusion(&mut self, name: &'static str, set: &'static str, exclude_phoneme_strs: &[&'static str]) -> Result<(),LanguageError> {
      
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let mut exclude_phonemes = vec![];
        for phoneme in exclude_phoneme_strs {
          exclude_phonemes.push(self.get_phoneme(phoneme)?);
        }
        let set = self.new_set(set, &exclude_phonemes)?;
        self.sets.insert(name,set);
        Ok(())

      }

    }


    fn get_set(&self, set: &'static str) -> Result<&Bag<Rc<Phoneme>>,LanguageError> {
      match self.sets.get(set) {
        Some(set) => Ok(set),
        None => Err(LanguageError::UnknownSet(set))
      }
    }

    fn get_phoneme(&self, phoneme: &'static str) -> Result<&Rc<Phoneme>,LanguageError> {
      match self.phonemes.get(phoneme) {
        Some(phoneme) => Ok(phoneme),
        None => Err(LanguageError::UnknownPhoneme(phoneme))
      }
    }

    fn get_environment(&self, environment: &'static str) -> Result<&Vec<EnvironmentBranch>,LanguageError> {
      match self.environments.get(environment) {
        Some(environment) => Ok(environment),
        None => Err(LanguageError::UnknownEnvironment(environment))
      }
    }

    pub fn add_environment(&mut self, name: &'static str, environment: &[EnvironmentBranch]) -> Result<(),LanguageError> {
      if let Some(_) = self.environments.get(name) {
        Err(LanguageError::EnvironmentAlreadyExists(name))
      } else {
        self.environments.insert(name,environment.to_vec());
        Ok(())
      }

    }

    pub fn add_table(&mut self, caption: &'static str, set: &'static str, axisses: &[&[(&'static str,&'static str)]]) -> Result<(),LanguageError> {
      self.tables.push((caption, set, Table::new(axisses)));
      Ok(())
    }

    fn new_set(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>]) -> Result<Bag<Rc<Phoneme>>,LanguageError> { 
      let mut set = self.get_set(set)?.clone();
      for phoneme in exclude_phonemes {
        set.remove(phoneme);
      }
      Ok(set)
    }

    fn phoneme_is(&self, phoneme: &Rc<Phoneme>, set: &'static str) -> Result<bool,LanguageError> {
      Ok(self.get_set(set)?.contains(phoneme))
    }

    fn _phoneme_equals(&self, phoneme: &Rc<Phoneme>, other: &'static str) -> Result<bool,LanguageError> {
      match self.phonemes.get(other) {
        Some(other) => Ok(phoneme == other),
        None => Err(LanguageError::UnknownPhoneme(other))
      }
    }

    fn choose(&self, set: &'static str, rng: &mut ThreadRng) -> Result<Rc<Phoneme>,LanguageError> { 
      match self.get_set(set)?.choose(rng) {
        Some(phoneme) => Ok(phoneme.clone()),
        None => Err(LanguageError::SetIsEmpty(set))
      }
    }

    fn choose_except(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>], rng: &mut ThreadRng) -> Result<Rc<Phoneme>,LanguageError> { 
      match self.new_set(set,exclude_phonemes)?.choose(rng) {
        Some(phoneme) => Ok(phoneme.clone()),
        None => Err(LanguageError::SetIsEmptyWithFilter(set)) 
      }
    }

    fn build_word(&self, environment_name: &'static str, word: &mut Word, phoneme: Rc<Phoneme>, rng: &mut ThreadRng) -> Result<(),LanguageError> {

        let environment = self.get_environment(environment_name)?;

        for branch in environment {
            if self.phoneme_is(&phoneme, branch.0)? {
                word.push(phoneme.clone()); // have to clone because we're referencing it again later. It's an RC, so that's okay.
                match branch.1.choose(rng) {
                    None => return Err(LanguageError::NoEnvironmentChoices(environment_name)),
                    Some(EnvironmentChoice::Done) => return Ok(()),
                    Some(EnvironmentChoice::Continuing(generate_set,environment,can_duplicate)) => {
                        let phoneme = if *can_duplicate {
                            self.choose(generate_set,rng)?
                        } else {
                            self.choose_except(generate_set,&[&phoneme],rng)?
                        };
                        return self.build_word(environment, word, phoneme, rng)
                    }
                }

            }
        }

        Err(LanguageError::IncompleteBranches(environment_name))

    }


    pub fn make_word(&self) -> Result<Word,LanguageError> {

        let mut word = Word::new(&[]);
        let mut rng = rand::thread_rng();
        let phoneme = self.choose(self.initial_phoneme_set, &mut rng)?;
        self.build_word(self.initial_environment, &mut word, phoneme, &mut rng)?;
        Ok(word)
    }



    fn validate_word(&self, environment_name: &'static str, 
                            word: &mut std::iter::Enumerate<std::slice::Iter<Rc<Phoneme>>>, idx: usize, phoneme: &Rc<Phoneme>, 
                            level: usize, validated: &Vec<ValidWordElement>, trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,LanguageError> {
        let environment = self.get_environment(environment_name)?;
        let mut validated = validated.clone();

        let mut found_valid_path = false;
        let mut error = None;

        macro_rules! trace_error {
          ($error: expr) => {{
            trace(level,ValidationTraceMessage::FoundError(&$error));
            $error
          }};
        }

        // I want to return only the deepest error, so only set the error if one hasn't been found.
        macro_rules! check_error {
          ($error: expr) => {{
            let this_error = $error;
            if let None = error {
              error = Some(this_error.clone());
            }
            trace_error!(this_error)
          }};
        }

        macro_rules! trace_valid {
          ($valid: expr) => {{
            let this_valid = $valid;
            trace(level,ValidationTraceMessage::FoundValid(&this_valid));
            validated.push(this_valid);
          }};
        }

        macro_rules! check_valid {
          ($valid: expr) => {{
            found_valid_path = true;
            trace_valid!($valid)
          }};
        }
    
        for branch in environment {
            if self.phoneme_is(&phoneme, branch.0)? {

                let next_phoneme = word.next();

                for choice in &branch.1.items {
                    match (choice, next_phoneme) {
                        ((EnvironmentChoice::Done,_),Some((idx,next_phoneme))) => {
                          check_error!(LanguageError::ExpectedEndOfWord(idx,next_phoneme.clone(),environment_name));
                        },
                        ((EnvironmentChoice::Continuing(generate_set,_,_),_),None) => {
                          check_error!(LanguageError::ExpectedPhonemeFoundEndOfWord(idx + 1,generate_set,environment_name));
                        },
                        ((EnvironmentChoice::Continuing(generate_set,environment,can_duplicate),_),Some((idx,next_phoneme))) => {
                            let valid_phoneme = if *can_duplicate {
                                self.phoneme_is(next_phoneme, generate_set)?
                            } else {
                                (next_phoneme != phoneme) && self.phoneme_is(next_phoneme, generate_set)?
                            };
                            if !valid_phoneme {
                              check_error!(LanguageError::IncorrectPhoneme(idx,next_phoneme.clone(),generate_set,environment_name));
                            } else {
                              trace_valid!(ValidWordElement::Phoneme(idx,next_phoneme.clone(),generate_set,environment_name));
                              // NOTE: I'm cloning the iterator here so that the next branch choice looks at the same next phoneme.
                              match self.validate_word(environment, &mut word.clone(), idx, next_phoneme, level + 1, &validated, trace) {
                                Err(err) => error = Some(err),
                                Ok(sub_validated) => {
                                  validated = sub_validated;
                                  found_valid_path = true;
                                  // break out of the loop, we found a successful branch.
                                  break;
                                }
                              }
                            }
                        },
                        ((EnvironmentChoice::Done,_),None) => {
                          check_valid!(ValidWordElement::Done(idx + 1,environment_name));
                          // break out of the loop, we found a successful branch.
                          break;
                        }
                    };

                    // otherwise keep iterating branches until an Ok is found or the branches are exhausted.

                };

                if !found_valid_path && error.is_none() {

                  // no successful choices were found. Check if error was set, and if not, then we didn't find
                  // any choices at all, which is an error. 
                  check_error!(LanguageError::IncompleteBranches(environment_name));

                }

                // no further processing, if the phoneme was valid for this branch, then that's the one that would have
                // been used for generating, so there's no way any other branches should match.
                break;

            }
        }

        if !found_valid_path {
          match error {
            None => 
              // if we got here, then there were no branches that fit the current phoneme.
              Err(trace_error!(LanguageError::NoBranchFitsPhoneme(idx,phoneme.clone(),environment_name))),
            Some(err) => Err(err)
          }
        } else {
            Ok(validated)
        }

        
    }

    pub fn check_word(&self,word: &Word, trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,LanguageError> {

        let mut word = word.phonemes.iter().enumerate();
        if let Some((idx,phoneme)) = word.next() {
            if self.phoneme_is(&phoneme, self.initial_phoneme_set)? {
              let valid = ValidWordElement::Phoneme(idx,phoneme.clone(),self.initial_phoneme_set,self.initial_environment);
              trace(0,ValidationTraceMessage::FoundValid(&valid)); 
              self.validate_word(self.initial_environment, &mut word, idx, phoneme,1,&vec![valid],trace)
            } else {
              let err = LanguageError::IncorrectPhoneme(idx,phoneme.clone(),self.initial_phoneme_set,self.initial_environment);
              trace(0,ValidationTraceMessage::FoundError(&err)); 
              Err(err) 
            }
        } else {
            Err(LanguageError::EmptyWord)
        }
    }

    pub fn read_word(&self,input: &str) -> Result<Word,LanguageError> {
        // not an efficient algorithm, but it works...
        let mut phonemes: Vec<&Rc<Phoneme>> = self.phonemes.values().collect();
        phonemes.sort_by(sort_phonemes_by_length_descending);
        
        let mut word: Vec<Rc<Phoneme>> = vec![];

        let mut source = input;

        'outer: while source.len() > 0 {
            for phoneme in &phonemes {
                let name = phoneme.name;
                if let Some(after) = source.strip_prefix(name) {
                    word.push(phoneme.clone().clone()); // clone twice because apparently phoneme is a double reference
                    source = after;
                    continue 'outer;
                }
            }
            Err(LanguageError::UnknownPhonemeWhileReading(input.to_string(),source.to_string()))?
        }

        Ok(Word::new(&word))
    }

    fn print_phonemes_once(bag: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Bag<Rc<Phoneme>>, grid_style: &ChartStyle) -> String {
      let mut result = String::new();
      if !bag.is_empty() {
        let mut phonemes: Vec<Rc<Phoneme>> = bag.list();
        phonemes.sort();
        for value in phonemes {
          if !result.is_empty() {
            result.push_str(" ")
          } 
          if unprinted_phonemes.contains(&value) {
            result.push_str(&grid_style.get_phoneme_text(format!("{}",value)));
          } else {
            result.push_str(&grid_style.get_phoneme_text(format!("⚠{}",value))); // FUTURE: Should I report an error?
          }
          unprinted_phonemes.remove(&value);
        };
      }

      result

    }


    fn build_phoneme_grid(&self, master_set: &Bag<Rc<Phoneme>>, table: &Table, style: ChartStyle, unprinted_phonemes: &mut Bag<Rc<Phoneme>>) -> Result<Chart,LanguageError> {
      // If there are no columns or rows, then the phonemes are just listed horizontally.
      // If you want to do a vertical table with just one column, you need to set columns that contains only one set.

      let mut grid = Chart::new(style.clone());

      if let Some(columns) = table.columns() {

        if let Some(rows) = table.rows() {

          // we need to know about the other axises for the headers.
          let subcolumns = table.subcolumns();
          let subrows = table.subrows();
          let sub_col_count = subcolumns.map(|a| a.len()).unwrap_or_else(|| 1);
          let sub_row_count = subrows.map(|a| a.len()).unwrap_or_else(|| 1);

          // add column headers...
          for column in columns.iter() {
            grid.add_col_header_cell(column.0,sub_col_count)
          }

          // I need to place the row-headers after, because I don't know if I'm skipping rows until they're processed.
          let mut row_headers = Vec::new();
            
          for row_def in rows.iter() {


            // get the set of phonemes in the row
            let row_set = self.get_set(row_def.1)?;

            // get the intersection of this and the master set.
            let row_set = master_set.intersection(row_set);

            let mut row_header_placed = false;

            for sub_row in 0..sub_row_count {

              let sub_row_set = if let Some(subrows) = subrows {
                let sub_row_def = subrows[sub_row];
                let sub_row_set = self.get_set(sub_row_def.1)?;
                let sub_row_set = row_set.intersection(sub_row_set);
                sub_row_set
              } else {
                row_set.clone()
              };

              if sub_row_set.is_empty() {
                continue;
              }

              grid.add_row();

              if !row_header_placed {
                row_headers.push(Some(row_def.0));
                row_header_placed = true;
              } else {
                row_headers.push(None)
              }

              for col_def in columns.iter() {
                // get the set of phonemes in the column
                let col_set = self.get_set(col_def.1)?;
                // find the intersection of this and the row.
                let col_set = sub_row_set.intersection(col_set);

                for sub_col in 0..sub_col_count {

                  let sub_col_set = if let Some(subcolumns) = subcolumns {
                    let sub_col_def = subcolumns[sub_col];
                    let sub_col_set = self.get_set(sub_col_def.1)?;
                    let sub_col_set = col_set.intersection(sub_col_set);
                    sub_col_set
                  } else {
                    col_set.clone()
                  };

                  // add all phonemes in that intersection to the cell.
                  // if there are no remaining axes, this will follow the zero axis path and just print them in a row.
                  
                  let cell_str = Self::print_phonemes_once(&sub_col_set, unprinted_phonemes, &style);
    
                  grid.add_cell(&cell_str)

                }
  
  
              }
  
            }


          }

          // now, figure out row_headers. Right now, I've got a vector of "Some caption" and "None" which is supposed to match
          // up with the rows. I need to 1) figure out the correct row spans, and 2) place them on the grid.
          let mut row_header_spans = Vec::new();

          // scan the row headers
          for (i,text) in row_headers.iter().enumerate() {
            if let Some(text) = text {
              // if there is a row header,
              // - mark the index it's in, the name, and a row_span of 1.
              row_header_spans.push((i,text,1))
            } else {
              // if there isn't a header,
              // - increment the last row_span of the row header.
              if let Some(last) = row_header_spans.last_mut() {
                last.2 += 1
              }
            }
          }

          for (index,text,row_span) in row_header_spans {
            grid.add_row_header_cell_at(index, text, row_span)
          }
          
          
        } else {

          // add column headers...
          // NOTE: This is different from the two-axis branch because it doesn't add a dummy column for the row headers.
          for column in columns.iter() {
            grid.add_col_header_cell(column.0,1)
          }

          for col_def in columns.iter() {
            // get the set of phonemes in the column
            let column = self.get_set(col_def.1)?;
            // find the intersection of this and the row.
            let column = master_set.intersection(column);

            // add all phonemes in that intersection to the cell.
            let cell_str = Self::print_phonemes_once(&column, unprinted_phonemes, &style);

            grid.add_cell(&cell_str)

          }

        }


      } else {
        let cell_str = Self::print_phonemes_once(&master_set, unprinted_phonemes, &style);

        grid.add_cell(&cell_str);

      };

      Ok(grid)

    }

    pub fn display_phonemes(&self, preferred_table: &Option<String>, style: ChartStyle) -> Result<Vec<(String,Chart)>,LanguageError> {

      let preferred_table = preferred_table.as_ref().map(|a| a.to_lowercase());

      let mut result = vec![];

      let mut unprinted_phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();

      for (name,set,table) in &self.tables {

        let grid = self.build_phoneme_grid(self.get_set(set)?, &table, style.clone(), &mut unprinted_phonemes)?;

        // we have to 'continue' here, as otherwise the "uncategorized phonemes" will show all of the other phonemes.
        if let Some(preferred_table) = &preferred_table {
          if (&name.to_lowercase() != preferred_table) && (&set.to_lowercase() != preferred_table) {
            continue;
          }
        }

        result.push((name.to_owned().to_string(),grid));


      } 

      if !unprinted_phonemes.is_empty() && (if let Some(preferred_table) = &preferred_table {
          if ("uncategorized" != preferred_table) && ("uncategorized phonemes" != preferred_table) {
            true
          } else {
            false
          }
        } else {
          true
        }) {

        let grid = self.build_phoneme_grid(&unprinted_phonemes.clone(), &Table::NotATable, style, &mut unprinted_phonemes)?;
        result.push(("uncategorized phonemes".to_owned(),grid));
  

      }

      Ok(result)

    }

    pub fn display_spelling(&self, style: ChartStyle, columns: usize) -> Result<Chart,LanguageError> {

      let phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();
      let phonemes = phonemes.list();

      let mut grid = Chart::new(style.clone());

      for _ in 0..columns {
        grid.add_col_header_cell("Phoneme",1);
        for orthography in self.orthographies {
          grid.add_col_header_cell(orthography,1)
        }
      }


      // once div_ceil is stable in the library, the existence of this will cause an error.
      // But, we can get rid of our shim, then.
      #[allow(unstable_name_collisions)] let length = phonemes.len().div_ceil(columns);
      let mut chunks: Vec<std::slice::Iter<Rc<Phoneme>>> = phonemes.chunks(length).map(|a| a.iter()).collect();
  
      for _ in 0..length {
        grid.add_row();

        for chunk in &mut chunks {
          if let Some(phoneme) = chunk.next() {
            grid.add_cell(&style.get_phoneme_text(format!("{}",phoneme)));
            for i in 0..ORTHOGRAPHIES {
              let mut cell = String::new();
              self.spell_phoneme(&phoneme, i, &mut cell, &mut [].iter().peekable());
              grid.add_cell(&cell);
            }
  
          } else {
            // add blank cells to make the table rectangular.
            grid.add_cell("");
            for _ in 0..ORTHOGRAPHIES {
              grid.add_cell("");
            }
          }
        }


      }

      Ok(grid)

    }

    pub fn process_lexicon(&self, path: String) -> Result<Vec<LexiconEntry<ORTHOGRAPHIES>>,Box<dyn Error>> {


      let mut reader = Reader::from_path(path)?;
      let headers = reader.headers()?;
      let word_field = headers.iter().position(|a| a.to_lowercase() == "word").ok_or_else(|| format!("No 'word' field found."))?;
      let definition_field = headers.iter().position(|a| a.to_lowercase() == "definition").ok_or_else(|| format!("No 'definition' field found."))?;

      let mut result: Vec<LexiconEntry<ORTHOGRAPHIES>> = Vec::new();

      for (row,record) in reader.into_records().enumerate() {
        let record = record.map_err(|e| format!("Error reading record {}: {}",row,e))?;
        let word = record.get(word_field).ok_or_else(|| format!("No word found at entry {}",row))?;
        let word = self.read_word(word).map_err(|e| format!("Error parsing word {}: {}",row,e))?;
        let spelling = std::array::from_fn(|i| self.spell_word(&word, i));
        let entry: LexiconEntry<ORTHOGRAPHIES> = LexiconEntry {
          word,
          spelling,
          definition: record.get(definition_field).ok_or_else(|| format!("No category found at row {}",row))?.to_owned(),
        };

        result.push(entry);

      }

      Ok(result)


    }

}

fn sort_phonemes_by_length_descending(a: &&Rc<Phoneme>, b: &&Rc<Phoneme>)  -> std::cmp::Ordering {
    let name_a = a.name;
    let len_a = name_a.len();
    let name_b = b.name;
    let len_b = name_b.len();
    if len_a != len_b {
        len_b.partial_cmp(&len_a).expect("Can't order phoneme lengths for some reason.")
    } else {
        name_a.partial_cmp(&name_b).expect("Can't order phoneme names for some reason.")
    }
}

enum ValidateOption {
  Simple,
  Explain,
  Trace
}

enum Command {
    GenerateWords(usize), // number of words to generate
    ValidateWords(Vec<String>,ValidateOption), // words to validate, whether to trace
    ShowPhonemes(Option<String>), // specifies the table to show
    ShowSpelling(usize), // specifies number of columns
    ShowUsage,
    ProcessLexicon(String,usize)
}

fn parse_args<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args) -> (Option<ChartStyle>,Command) {
  let mut command = None;
  let mut grid_style = None;

  macro_rules! set_grid_style {
      ($style: expr) => {
        if grid_style.is_some() {
          panic!("Too many grid styles");
        } else {
          grid_style = Some($style);
        }          
      };
  }

  macro_rules! set_command {
      ($command: expr) => {
        if command.is_some() {
          panic!("Too many commands");
        } else {
          command = Some($command);
        }          
      };
  }

  while let Some(arg) = args.next() {
    match arg.as_ref() {
      "--grid=plain" => set_grid_style!(ChartStyle::Plain),
      "--grid=terminal" => set_grid_style!(ChartStyle::Terminal),
      "--grid=markdown" => set_grid_style!(ChartStyle::Markdown),
      "--grid=latex" => set_grid_style!(ChartStyle::LaTeX),
      "--generate" => set_command!(Command::GenerateWords(args.next().expect("Generate count required").as_ref().parse().expect("Argument should be a number"))),
      "--validate" => {
        let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
        words.extend(args.map(|x| x.as_ref().to_owned()));
        set_command!(Command::ValidateWords(words,ValidateOption::Simple));
      },
      "--validate=explain" => {
        let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
        words.extend(args.map(|x| x.as_ref().to_owned()));
        set_command!(Command::ValidateWords(words,ValidateOption::Explain));
      },
      "--validate=trace" => {
        let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
        words.extend(args.map(|x| x.as_ref().to_owned()));
        set_command!(Command::ValidateWords(words,ValidateOption::Trace));
      },
      "--phonemes" => set_command!(Command::ShowPhonemes(None)),
      a if a.starts_with("--phonemes=") => set_command!(Command::ShowPhonemes(Some(a.trim_start_matches("--phonemes=").to_owned()))),
      "--spelling" => set_command!(Command::ShowSpelling(1)),
      a if a.starts_with("--spelling=") => set_command!(Command::ShowSpelling(a.trim_start_matches("--spelling=").parse::<usize>().expect("Parameter should be a number").clamp(1,usize::MAX))),
      "--lexicon" => {
        let path = args.next().expect("No lexicon filename given").as_ref().to_owned();
        let spelling_index = args.next().expect("No orthography index given").as_ref().parse().expect("orthography index must be a number");
        set_command!(Command::ProcessLexicon(path,spelling_index))
      },
      "--help" => set_command!(Command::ShowUsage),
      _ => panic!("Unknown command {}",arg.as_ref())

    }
  }

  (grid_style,command.unwrap_or_else(|| Command::GenerateWords(1)))

}

pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, language: Result<Language<ORTHOGRAPHIES>,LanguageError>) {
  let (grid_style,command) = parse_args(&mut args.skip(1));
  
  match language {
      Ok(language) => {
    
        match command {
            Command::GenerateWords(count) => {
              let mut grid = Chart::new(grid_style.unwrap_or_else(|| ChartStyle::Plain));

              for _ in 0..count {
                    grid.add_row();
                    match language.make_word() {
                      Ok(word) => {
                        for orthography in 0..language.orthographies.len() {
                          grid.add_cell(&language.spell_word(&word,orthography));
                        }
                        grid.add_cell(&format!("{}",word));
                        // the following is a sanity check. It might catch some logic errors, but really it's just GIGO.
                        if let Err(err) = language.check_word(&word,&|_,_| { /* eat message, no need to report */}) {
                          println!("-- !!!! invalid word: {}",err);
                          std::process::exit(1);
                        }
                      },
                      Err(err) => {
                        eprintln!("!!! Couldn't make word: {}",err);
                        std::process::exit(1);
                      }
                    }
                }
                print!("{}",grid);
          
            },
            Command::ValidateWords(words,option) => {
              let mut invalid_found = false;
              for word in words {
                    match language.read_word(&word) {
                        Ok(word) => {
                            let trace_cb: Box<ValidationTraceCallback> = if let ValidateOption::Trace = option {
                              Box::new(|level,message| { 
                                /* eat message, no need to report */
                                println!("{}{}",str::repeat(" ",level*2),message);
                               })
                            } else {
                              Box::new(|_,_| {})
                            };
                            match language.check_word(&word,&trace_cb) {
                                Err(err) => {
                                  invalid_found = true;
                                  if let ValidateOption::Trace = option {
                                    println!("!!!! invalid word (see trace)");
                                  } else {
                                    println!("{}",err);
                                  }
                                },
                                Ok(validated) => {
                                  if let ValidateOption::Explain = option {
                                    for valid in validated {
                                      println!("{}",valid)
                                    }                                      
                                  };
                                  for orthography in 0..language.orthographies.len() {
                                    print!("{} ",language.spell_word(&word,orthography));
                                  }
                                  println!("{}",word);
                                }
                            }
                        },
                        Err(err) => {
                            eprintln!("!!!! Can't read word: {}",err);
                            std::process::exit(1);
                        }
                    }
                }
                if invalid_found {
                  std::process::exit(1);
                }
            }
            Command::ShowPhonemes(table) => {
              match language.display_phonemes(&table,grid_style.unwrap_or_else(|| ChartStyle::Terminal)) {
                Ok(grids) => {
                  if let Some(table) = &table { 
                    if grids.is_empty() {
                      println!("No phoneme table named {}. Try singular?",table);
                    }
                  }
                  for grid in grids {
                    if table.is_none() {
                      println!("{}:",grid.0);
                    }
                    println!("{}",grid.1);
                  }
                },
                Err(err) => {
                  eprintln!("!!! Couldn't display phonemes: {}",err);
                  std::process::exit(1)
                }
              }
            },
            Command::ShowSpelling(columns) => {
              match language.display_spelling(grid_style.unwrap_or_else(|| ChartStyle::Terminal),columns) {
                Ok(grid) => {
                  println!("{}",grid);
                },
                Err(err) => {
                  eprintln!("!!! Couldn't display spelling: {}",err);
                  std::process::exit(1)
                }
              }
            },
            Command::ProcessLexicon(path,ortho_index) => {

              if ortho_index >= language.orthographies.len() {
                panic!("Language only has {} orthographies.",language.orthographies.len())
              }
        
              match language.process_lexicon(path) {
                Ok(entries) => {
                  // NOTE: I'm *not* sorting the entries before grouping. The user might have some sort of custom sort in the data, however.
                  for entry in entries {
                    // NOTE: I'm not formatting because there's no easy way to format the different spellings.
                    let mut line = String::new();
                    line.push_str(&format!("\\subparagraph{{{}}} (",entry.spelling[ortho_index]));
                    for i in 0..entry.spelling.len() {
                      if i != ortho_index {
                        line.push_str(&format!("\\textit{{{}}}: {}; ",language.orthographies[i],entry.spelling[i]));
                      }
                    }
                    line.push_str(&format!("\\ipa{{{}}}) {}",entry.word,entry.definition));
                    println!("{}",line);
                  }
    
                },
                Err(err) => {
                  eprintln!("!!! Couldn't process lexicon: {}",err);
                  std::process::exit(1)
                }
              }
            },
            Command::ShowUsage => {
              println!("usage: {} [options] <command>",language.name);
              println!("default command: --generate 1");
              println!("options:");
              println!("   --grid=<plain | terminal | markdown>");
              println!("      changes the format of grid output. Default is \"plain\" for generate and \"terminal\" for phonemes and spelling.");
              println!("commands:");
              println!("   --generate <integer>");
              println!("      generates the specified number of words.");
              println!("   --validate <words>...");
              println!("      validates the specified words (verifies that it is possible to generate them).");
              println!("   --validate=trace <words>...");
              println!("      same as --validate, but traces the validation through all environment branches.");
              println!("   --validate=explain <words>...");
              println!("      same as --validate, but provides detailed explanation of valid phonemes on success.");
              println!("   --phonemes");
              println!("      prints out the phonemes of the language.");
              println!("   --phonemes=<table>");
              println!("      prints out one table of phonemes of the language.");
              println!("   --spelling");
              println!("      prints out the orthographies of the language.");
              println!("   --spelling=<2..>");
              println!("      prints spelling table in multiple columns.");
              println!("   --lexicon <path>");
              println!("      validates lexicon and outputs into a LaTeX file.");
              println!("   --help");
              println!("      display this information.");
            }
        }
    
      },
      Err(err) => eprintln!("!!! Language Incomplete: {}",err)
    }
  
  
  }
  