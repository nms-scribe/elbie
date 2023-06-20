use std::collections::HashMap;
use std::rc::Rc;
use std::error::Error;
use rand::Rng;
use rand::seq::SliceRandom; 
use rand::prelude::ThreadRng;
use csv::Reader;
use std::fmt::Display;
// NOTE: The term_grid that is used here is a modified variant of the original that
// 1. allows for multi-line cells
// 2. removes the ending linefeed.
// TODO: If this project ever gets published, put in pull requests for the changes to the original github project, or provide a fork.
// TODO: Better yet, I'd like to come up with one that works better than this. Something more akin to the structure of an HTML table.
use term_grid::Grid;
use term_grid::GridOptions;
use term_grid::Cell;

/*
Elbie = LB, Language Builder, and is a bunch of tools for building a constructed language.
*/

/*
FUTURE: Implementing syllable breaks, stress, etc: (relatively simple)
- replace phoneme sequences with WordElement enum, where phoneme is only one element.
- Append new items to branch choices which allow adding these indicators to the word
- Figure out how to represent these indicators in the word for writing and reading them phonetically
- spelling is a bit more difficult, but there're lots of things about spelling that are difficult to implement right now.
*/

pub const PHONEME: &'static str = "phoneme";
pub const EMPTY: &'static str = "empty";

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

  // returns a new bag containing objects in either self or other
  fn union(&self, other: &Bag<ItemType>) -> Self {
    let mut self_iter = self.0.iter();
    let mut other_iter = other.0.iter();
    let mut result: Self = Bag::new();

    // TODO: Is there some simpler way we can do this with iterators?

    // This algorithm should be more efficient because we're depending on both vectors being sorted, rather than doing the 'contains'
    // and a binary_search for every check. There could still be some improvements.

    let mut self_next = self_iter.next();
    let mut other_next = other_iter.next();

    loop {
      if let Some(self_some) = self_next {
        if let Some(other_some) = other_next {
          if self_some < other_some {
            // self is less, so add that and continue iterating self
            result.insert(self_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
            self_next = self_iter.next();
            continue;
          } else if self_some > other_some {
            // other is less, so add that and continue iterating other
            result.insert(other_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
            other_next = other_iter.next();
            continue;
          } else {
            // both are equal, so add self and continue iterating both
            result.insert(self_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
            self_next = self_iter.next();
            other_next = other_iter.next();
            continue;
          }
        } else {
          // we've eaten through all others...
          break;
        }
      } else {
        // we've eaten through all of the 
        break;
      }

    }

    // either one of the two or both are empty, so loop through each to fill the rest.
    while let Some(self_some) = self_next {
      result.insert(self_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
      self_next = self_iter.next();
    }

    while let Some(other_some) = other_next {
      result.insert(other_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
      other_next = other_iter.next();
    }

    result
  }

  // returns a new bag containing objects in self, but not in other.
  fn _difference(&self, other: &Bag<ItemType>) -> Self {
    let mut self_iter = self.0.iter();
    let mut other_iter = other.0.iter();
    let mut result: Self = Bag::new();

    // Based on union, but we're only adding in stuff that is only contained in self if it isn't in the other.

    let mut self_next = self_iter.next();
    let mut other_next = other_iter.next();

    loop {
      if let Some(self_some) = self_next {
        if let Some(other_some) = other_next {
          if self_some < other_some {
            // self is less, so add that and continue iterating self
            result.insert(self_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
            self_next = self_iter.next();
            continue;
          } else if self_some > other_some {
            // other is less, so it doesn't belong, continue iterating it...
            other_next = other_iter.next();
            continue;
          } else {
            // both are equal, so they con't belong, continue iterating both.
            self_next = self_iter.next();
            other_next = other_iter.next();
            continue;
          }
        } else {
          // we've eaten through all others...
          break;
        }
      } else {
        // we've eaten through all of the self
        break;
      }

    }

    // either one of the two or both are empty, so loop through each to fill the rest.
    while let Some(self_some) = self_next {
      result.insert(self_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
      self_next = self_iter.next();
    }

    // anything left in other iterator doesn't belong.

    result

  }

  // returns a new bag containing objects both in self and other
  fn intersection(&self, other: &Bag<ItemType>) -> Self {
    let mut self_iter = self.0.iter();
    let mut other_iter = other.0.iter();
    let mut result: Self = Bag::new();

    // Algorithm based on union, we just only put stuff in when they are equal.

    let mut self_next = self_iter.next();
    let mut other_next = other_iter.next();

    loop {
      if let Some(self_some) = self_next {
        if let Some(other_some) = other_next {
          if self_some < other_some {
            // self is less, so continue iterating self
            self_next = self_iter.next();
            continue;
          } else if self_some > other_some {
            // other is less, so continue iterating other
            other_next = other_iter.next();
            continue;
          } else {
            // both are equal, so add self and continue iterating both
            result.insert(self_some.clone()); // TODO: If I was certain that this algorithm works, then I should be able to just push
            self_next = self_iter.next();
            other_next = other_iter.next();
            continue;
          }
        } else {
          // we've eaten through all others...
          break;
        }
      } else {
        // we've eaten through all of the self
        break;
      }

    }

    // anything left in the iterators is not in the other, so they don't make it into an intersection.

    result
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

impl std::convert::From<&Word> for Cell {

  fn from(word: &Word) -> Self {
    Cell::from(format!("{}",word))
  }
}

// TODO: Is there some way I can do the environments and sets as types? Maybe phonemes, sets and environments are traits instead that you implement in structs. 
// I could use macros to make those implementations easier to code. Phonemes should really be enumerations. This would require the language to be generic
// and base itself off of phonemes. --- I think the hardest part is implementing a set that describes which phonemes can be chosen, and then to choose such a 
// type randomly?

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

#[derive(Debug)]
struct Table {
  axisses: Vec<Vec<(&'static str,&'static str)>> // first string is the caption, second string is the set name
}

impl Table {

  fn new(axisses: &[&[(&'static str,&'static str)]]) -> Self {
    let axisses = axisses.iter().map(|axis| axis.to_vec()).collect();
    Table {
      axisses
    }
  }
}

pub enum GridStyle {
  Plain, // uses spaces to delimit grids
  Pipes, // uses '|' to delimit cells, no other styling
  LaTeX, // custom for my own use
}

impl GridStyle {

  fn show_headers(&self) -> bool {
    match self {
      Self::Plain => false,
      Self::Pipes |
      Self::LaTeX => true
    }
  }

  fn ellide_empty_rows(&self) -> bool {
    match self {
      Self::Plain => true,
      Self::Pipes |
      Self::LaTeX => false
    }

  }

  fn get_filling(&self) -> term_grid::Filling {
    match self {
      GridStyle::Plain => term_grid::Filling::Spaces(1),
      GridStyle::Pipes => term_grid::Filling::Text(" │ ".to_string()),
      GridStyle::LaTeX => term_grid::Filling::Text(" & ".to_string()),
    }    
  }

// TODO: If true, then we need to add a dummy "empty" column before the table
  fn start_with_filling(&self) -> bool {
    match self {
      Self::Plain => false,
      Self::Pipes => true,
      Self::LaTeX => false
    }
  }

// TODO: If true, then we need to add a dummy "empty" column after the table
  fn end_with_filling(&self) -> bool {
    match self {
      Self::Plain => false,
      Self::Pipes => true,
      Self::LaTeX => false
    }
  }

// TODO: If set, then we need to append this to the last column
  fn end_delimiter(&self) -> Option<String> {
    match self {
      Self::Plain => None,
      Self::Pipes => None,
      Self::LaTeX => Some("\\\\".to_string()),
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

    fn _add_difference(&mut self, name: &'static str, source_a: &'static str, source_b: &'static str) -> Result<(),LanguageError> {
      
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let subset_a = self.get_set(source_a)?;
        let subset_b = self.get_set(source_b)?;
        let set = subset_a._difference(subset_b);
        self.sets.insert(name,set);
        Ok(())

      }

    }

    // will eventually be used over add_difference
    pub fn _build_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),LanguageError> {
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let mut set = self.get_set(base_set)?.clone();
        for subset in exclude_sets {
          let subset = self.get_set(subset)?;
          set = set._difference(subset);
        }
        self.sets.insert(name, set);
        Ok(())
      }
    }

    pub fn add_intersection(&mut self, name: &'static str, source_a: &'static str, source_b: &'static str) -> Result<(),LanguageError> {
      
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let subset_a = self.get_set(source_a)?;
        let subset_b = self.get_set(source_b)?;
        let set = subset_a.intersection(subset_b);
        self.sets.insert(name,set);
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

    pub fn add_union(&mut self, name: &'static str, source_a: &'static str, source_b: &'static str) -> Result<(),LanguageError> {
      
      if let Some(_) = self.sets.get(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let subset_a = self.get_set(source_a)?;
        let subset_b = self.get_set(source_b)?;
        let set = subset_a.union(subset_b);
        self.sets.insert(name,set);
        Ok(())

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

    fn print_phonemes_once(bag: &Bag<Rc<Phoneme>>, unprinted_phonemes: &mut Bag<Rc<Phoneme>>) -> String {
      let mut result = String::new();
      if !bag.is_empty() {
        let mut phonemes: Vec<Rc<Phoneme>> = bag.list();
        phonemes.sort();
        for value in phonemes {
          if !result.is_empty() {
            result.push_str(" ")
          } 
          if unprinted_phonemes.contains(&value) {
            result.push_str(&format!("{}",value));
          } else {
            result.push_str(&format!("⚠{}",value)); // FUTURE: Should I report an error?
          }
          unprinted_phonemes.remove(&value);
        };
      }
      // TODO: This is a hack, until I can figure out how to align the sub-grids in the grid thing.
      // This really only works if the user is putting out one phoneme per cell.
      if result.is_empty() {
        result.push_str("   ");
      }
      result

    }


    fn build_grid(&self, master_set: &Bag<Rc<Phoneme>>, axisses: &Vec<Vec<(&'static str,&'static str)>>, style: &GridStyle, unprinted_phonemes: &mut Bag<Rc<Phoneme>>) -> Result<(Grid,usize),LanguageError> {
      // if we have two axises, then the first is the vertical and the second is horizontal.
      // if we have one axis, then it is horizontal.
      // if there are more axes, they create "sub-tables" inside the cell.

      let mut grid = Vec::new();

      let column_count = if axisses.len() > 0 {
          let first_axis = &axisses[0];
          let second_axis = &axisses.get(1); // we might not have a second axis.
          let remaining_axisses = if axisses.len() > 2 {
            // why do they do bounds checking on this? I could possibly try to get the Skip iterator to work as well.
            axisses[2..].to_vec()
          } else {
            vec![]
          };

          let column_count = if style.start_with_filling() {
            grid.push(Cell::from(""));
            1
          } else { 
            0 
          } + if let Some(second_axis) = second_axis {
            if style.show_headers() {
              // add column headers including an extra one for the row header column
              grid.push(Cell::from(String::new()));
              for column in second_axis.iter() {
                grid.push(Cell::from(column.0.to_owned()))
              }

              second_axis.len() + 1
            } else {

              second_axis.len()
            }
          } else {
            if style.show_headers() {
              // add column headers including an extra one for the row header column
              for column in first_axis.iter() {
                grid.push(Cell::from(column.0.to_owned()))
              }
            }
            first_axis.len()

          } + if style.end_with_filling() {
            grid.push(Cell::from(""));
            1
          } else { 
            0 
          };

          if let Some(filling) = style.end_delimiter() {
            if let Some(cell) = grid.last_mut() {
              if let Some(text) = cell.contents.last_mut() {
                text.push_str(&filling)
              } else {
                cell.contents.push(filling)
              }
            } else {
              grid.push(Cell::from(filling))
            }
          }

          for row_def in first_axis.iter() {
            // get the set of phonemes in the row
            let row = self.get_set(row_def.1)?;
            // get the intersection of this and the master set.
            let row = master_set.intersection(row);

            // empty rows get ellided if the table is not fancy...
            if style.ellide_empty_rows() && row.is_empty() {
              continue;
            }

            if style.start_with_filling() {
              grid.push(Cell::from(""))
            }

            if let Some(second_axis) = second_axis {
              if style.show_headers() {
                // add a row header
                grid.push(Cell::from(row_def.0.to_owned()));
              }

              for col_def in second_axis.iter() {
                // get the set of phonemes in the column
                let column = self.get_set(col_def.1)?;
                // find the intersection of this and the row.
                let column = row.intersection(column);
                // add all phonemes in that intersection to the cell.
  
                let cell_str = if remaining_axisses.len() > 0 {
                  // build a tiny grid in the cell and print it to text.
                  let (cell_grid,columns) = self.build_grid(&column, &remaining_axisses, &GridStyle::Plain, unprinted_phonemes)?;
                  // TODO: There are still some problems, for example, /m/ isn't pulling to the right. I wonder if I have to fit into width 
                  // instead of columns? However, I can't do that until the whole thing is printed. Another alternative is to align right,
                  // but that won't work for ones that only have voiceless.
                  // TODO: I think the only real alternative is to have "sub-grids", but that makes the grid way more complicated.

                  format!("{}",cell_grid.fit_into_columns(columns))
                } else {
                  Self::print_phonemes_once(&column,unprinted_phonemes)
                };
  
                grid.push(Cell::from(cell_str))
              }
  
            } else {
              // the rows are actually columns
              let column = row;

              let cell_str = Self::print_phonemes_once(&column,unprinted_phonemes);
              grid.push(Cell::from(cell_str))
            }

            if style.end_with_filling() {
              grid.push(Cell::from(""))
            }

            if let Some(filling) = style.end_delimiter() {
              if let Some(cell) = grid.last_mut() {
                if let Some(text) = cell.contents.last_mut() {
                  text.push_str(&filling)
                } else {
                  cell.contents.push(filling)
                }
              } else {
                grid.push(Cell::from(filling))
              }
            }
  
  


          }

          column_count
        
      } else {
        let cell_str = Self::print_phonemes_once(&master_set, unprinted_phonemes);
        if style.start_with_filling() {
          grid.push(Cell::from(""))
        }

        grid.push(Cell::from(cell_str));

        if style.end_with_filling() {
          grid.push(Cell::from(""))
        }

        grid.len()
      };

      let grid = Grid::new_with_cells(GridOptions {
        direction: term_grid::Direction::LeftToRight,
        filling: style.get_filling()
      },grid);

      Ok((grid,column_count))

    }

    pub fn display_phonemes(&self, style: &GridStyle) -> Result<Vec<(String,Grid,usize)>,LanguageError> {

      let mut result = vec![];

      let mut unprinted_phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();

      for (name,set,table) in &self.tables {

        let (grid,column_count) = self.build_grid(self.get_set(set)?, &table.axisses, style, &mut unprinted_phonemes)?;

        result.push((name.to_owned().to_string(),grid,column_count));


      } 

      if !unprinted_phonemes.is_empty() {
        let (grid, column_count) = self.build_grid(&unprinted_phonemes.clone(), &vec![], &GridStyle::Plain, &mut unprinted_phonemes)?;
        result.push(("uncategorized phonemes".to_owned(),grid,column_count));
      }

      Ok(result)

    }

    pub fn display_spelling(&self, style: GridStyle) -> Result<(String,Grid,usize),LanguageError> {

      let phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();

      let mut grid = Vec::new();

      let column_count = 1 + self.orthographies.len() + if style.start_with_filling() {
        1
      } else { 
        0 
      } + if style.end_with_filling() {
        1
      } else { 
        0 
      };

      if style.show_headers() {

        if style.start_with_filling() {
          grid.push(Cell::from(""));
        }

        grid.push(Cell::from("Phoneme"));
        for orthography in self.orthographies {
          grid.push(Cell::from(orthography))
        }
  
        if style.end_with_filling() {
          grid.push(Cell::from(""));
        }

        if let Some(filling) = style.end_delimiter() {
          if let Some(cell) = grid.last_mut() {
            if let Some(text) = cell.contents.last_mut() {
              text.push_str(&filling)
            } else {
              cell.contents.push(filling)
            }
          } else {
            grid.push(Cell::from(filling))
          }
        }



      }

      
      for phoneme in phonemes.list() {
        if style.start_with_filling() {
          grid.push(Cell::from(""));
        }

        grid.push(Cell::from(format!("{}",phoneme)));
        for i in 0..ORTHOGRAPHIES {
          let mut cell = String::new();
          self.spell_phoneme(&phoneme, i, &mut cell, &mut [].iter().peekable());
          grid.push(Cell::from(cell));
        }

        if style.end_with_filling() {
          grid.push(Cell::from(""));
        }

        if let Some(filling) = style.end_delimiter() {
          if let Some(cell) = grid.last_mut() {
            if let Some(text) = cell.contents.last_mut() {
              text.push_str(&filling)
            } else {
              cell.contents.push(filling)
            }
          } else {
            grid.push(Cell::from(filling))
          }
        }



      }

      let grid = Grid::new_with_cells(GridOptions {
        direction: term_grid::Direction::LeftToRight,
        filling: style.get_filling()
      },grid);

      Ok(("Orthography".to_owned(),grid,column_count))

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
    GenerateWords(usize),
    ValidateWords(Vec<String>,ValidateOption), // words to validate, whether to trace
    ShowPhonemes(GridStyle),
    ShowSpelling(GridStyle),
    ShowUsage,
    ProcessLexicon(String,usize)
}

pub fn run_main<const ORTHOGRAPHIES: usize>(args: Vec<String>, language: Result<Language<ORTHOGRAPHIES>,LanguageError>) {
  let command = if args.len() > 1 {
    match args[1].as_str() {
        "--generate" => Command::GenerateWords(args.get(2).expect("Generate count required").parse().expect("Argument should be a number")),
        "--validate" => {
            if args.len() > 2 {
                Command::ValidateWords(args.iter().skip(2).map(|x| x.to_string()).collect(),ValidateOption::Simple)
            } else {
                panic!("No words to validate.")
            }
        },
        "--validate=explain" => {
          if args.len() > 2 {
              Command::ValidateWords(args.iter().skip(2).map(|x| x.to_string()).collect(),ValidateOption::Explain)
          } else {
              panic!("No words to validate.")
          }
        },
        "--validate=trace" => {
          if args.len() > 2 {
              Command::ValidateWords(args.iter().skip(2).map(|x| x.to_string()).collect(),ValidateOption::Trace)
          } else {
              panic!("No words to validate.")
          }
        },
        "--phonemes" => Command::ShowPhonemes(GridStyle::Pipes),
        "--phonemes=latex" => Command::ShowPhonemes(GridStyle::LaTeX),
        "--spelling" => Command::ShowSpelling(GridStyle::Pipes),
        "--spelling=latex" => Command::ShowSpelling(GridStyle::LaTeX),
        "--lexicon" => {
          if let (Some(path),Some(spelling_index)) = (args.get(2),args.get(3)) {
            Command::ProcessLexicon(path.to_owned(),spelling_index.parse().expect("orthography index must be a number"))
          } else {
            panic!("Please specify a filename and an orthography index")
          }
        },
        "--help" => Command::ShowUsage,
        _ => panic!("Unknown command {}",args[1])
    }
  } else {
    Command::GenerateWords(1)
  };
  
  match language {
      Ok(language) => {
    
        match command {
            Command::GenerateWords(count) => {
              let mut grid = Grid::new(GridOptions {
                direction: term_grid::Direction::LeftToRight,
                filling: term_grid::Filling::Spaces(1)
              });

              for _ in 0..count {
                    match language.make_word() {
                      Ok(word) => {
                        for orthography in 0..language.orthographies.len() {
                          grid.add(Cell::from(language.spell_word(&word,orthography)));
                        }
                        grid.add(Cell::from(&word));
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
                println!("{}",grid.fit_into_columns(2));
          
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
            Command::ShowPhonemes(style) => {
              match language.display_phonemes(&style) {
                Ok(grids) => {
                  for grid in grids {
                    println!("{}:",grid.0);
                    println!("{}",grid.1.fit_into_columns(grid.2));
                    println!();
                  }
                },
                Err(err) => {
                  eprintln!("!!! Couldn't display phonemes: {}",err);
                  std::process::exit(1)
                }
              }
            },
            Command::ShowSpelling(style) => {
              match language.display_spelling(style) {
                Ok(grid) => {
                  println!("{}:",grid.0);
                  println!("{}",grid.1.fit_into_columns(grid.2));
                  println!();
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
                        line.push_str(&format!("\\textsc{{{}}}: {}; ",language.orthographies[i],entry.spelling[i]));
                      }
                    }
                    line.push_str(&format!("\\ipaq{{{}}}) {}",entry.word,entry.definition));
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
              println!("usage: {} [command]",language.name);
              println!("default command: --generate 1");
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
              println!("   --spelling");
              println!("      prints out the orthographies of the language.");
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
  