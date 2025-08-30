use core::array;
use core::cmp::Ordering;
use std::collections::HashMap;
use core::fmt;
use core::fmt::Formatter;
use core::iter::Enumerate;
use core::iter::Peekable;
use std::process;
use std::rc::Rc;
use core::error::Error;
use core::slice::Iter;
use rand::Rng as _;
use rand::seq::IndexedRandom as _;
use rand::prelude::ThreadRng;
use csv::Reader;
use core::fmt::Display;

mod chart;
mod phoneme_table;
pub mod grid;
pub mod lexicon;
#[cfg(test)] mod test;

pub use chart::Chart;
pub use chart::ChartStyle;

use crate::grid::Cell;
use crate::grid::ColumnHeader;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::lexicon::Lexicon;
use crate::lexicon::LexiconEntry;
use crate::phoneme_table::Table;
use crate::phoneme_table::Table0D;
pub use crate::phoneme_table::Table0DDef;
use crate::phoneme_table::Table1D;
pub use crate::phoneme_table::Table1DDef;
use crate::phoneme_table::Table2D;
pub use crate::phoneme_table::Table2DDef;
use crate::phoneme_table::Table3D;
pub use crate::phoneme_table::Table3DDef;
use crate::phoneme_table::Table4D;
pub use crate::phoneme_table::Table4DDef;
pub use crate::grid::TableStyle;

/*
Elbie = LB, Language Builder, and is a bunch of tools for building a constructed language.
*/

/*
FUTURE: Implementing syllable breaks, stress, etc, Simple Solution:
- a "word" is sequence of syllables, not phonemes. A syllable is a sequence of phonemes. I don't think we need to support onset/rhyme structure, since that could be analyzed differently. In fact, some languages might not be able to analyze syllables, in which case each word would have to be one big syllable.
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


pub const PHONEME: &str = "phoneme";
pub const EMPTY: &str = "empty";

/* NMS: Seems to be implemented in core now
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
*/

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
  UnknownPhonemeWhileReading(String,String),
  // table def errors
  InvalidOptionForTable(TableOption)
}

impl Display for LanguageError {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::SetIsEmpty(name) => write!(f,"Set {name} has no phonemes."),
      Self::SetIsEmptyWithFilter(name) => write!(f,"Set {name} as filtered has no phonemes."),
      Self::UnknownSet(name) => write!(f,"Unknown set {name}."),
      Self::UnknownPhoneme(name) => write!(f,"Unknown phoneme {name}."),
      Self::PhonemeAlreadyExists(name) => write!(f,"Phoneme {name} already exists."),
      Self::SetAlreadyExists(name) => write!(f,"Set {name} already exists."),
      Self::EnvironmentAlreadyExists(name) => write!(f, "Environment {name} already exists."),
      Self::UnknownEnvironment(name) => write!(f,"Unknown environment {name}."),
      Self::NoEnvironmentChoices(name) => write!(f,"Environment {name} is missing some branch environment choices."),
      Self::IncompleteBranches(name) => write!(f,"Environment {name} is missing some possible branches."),

      Self::EmptyWord => write!(f,"Word is empty"),
      Self::IncorrectPhoneme(index,phoneme,set,environment) => write!(f,"[Environment {environment} at {index}]: Expected {set}, found phoneme ({phoneme})."),
      Self::ExpectedEndOfWord(index,phoneme,environment) => write!(f,"[Environment {environment} at {index}]: Expected end of word, found phoneme ({phoneme})"),
      Self::ExpectedPhonemeFoundEndOfWord(index,set,environment) => write!(f,"[Environment {environment} at {index}]: Expected {set}, found end of word"),
      Self::NoBranchFitsPhoneme(index,phoneme,environment) => write!(f,"[Environment {environment} at {index}]: Phoneme ({phoneme}) does not match any branch."),

      Self::UnknownPhonemeWhileReading(source,problem) => write!(f,"In word '{source}': unknown phoneme starting at '{problem}'."),

      Self::InvalidOptionForTable(option) => write!(f,"Invalid option for phoneme table: '{option:?}'.")
    }

  }
}

impl Error for LanguageError {

}

// A set that I can random access. It's more efficient than random access of a HashSet, but probably could be better.
#[derive(Debug,Clone)]
struct Bag<ItemType>(Vec<ItemType>);

impl<ItemType: Clone + Ord> Bag<ItemType> {

  const fn new() -> Self {
    Self(Vec::new())
  }

  const fn is_empty(&self) -> bool {
    self.0.is_empty()
  }


  fn set_operation(&self, other: &Self, insert_if_in_self: bool, insert_if_in_other: bool, insert_if_in_both: bool) -> Self {
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
  fn union(&self, other: &Self) -> Self {
    self.set_operation(other, true, true, true)
  }

  // returns a new bag containing objects in self, but not in other.
  fn difference(&self, other: &Self) -> Self {
    self.set_operation(other, true, false, false)
  }

  // returns a new bag containing objects both in self and other
  fn intersection(&self, other: &Self) -> Self {
    self.set_operation(other, false, false, true)

  }

  // returns a new bag containing objects in self or other but not both
  fn _symmetric_difference(&self, other: &Self) -> Self {
    self.set_operation(other, true, true, false)
  }


  // returns true if the specified value is contained in the bag.
  fn contains(&self, value: &ItemType) -> bool {
    self.0.binary_search(value).is_ok()
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

  fn remove(&mut self, value: &ItemType) -> Option<ItemType> {
    match self.0.binary_search(value) {
      Ok(pos) => {
        Some(self.0.remove(pos))
      }
      Err(_) => None
    }
  }

  // randomly chooses an item from the bag and returns it.
  fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
    self.0.choose(rng)
  }

  fn list(&self) -> Vec<ItemType> {
    self.0.clone()
  }

  fn iter(&self) -> impl Iterator<Item = &ItemType> {
      self.0.iter()
  }


}

#[derive(Debug,Clone)]
struct WeightedVec<ItemType>{
  items: Vec<(ItemType,usize)>,
  total_weight: usize
}

impl<ItemType> WeightedVec<ItemType> {

  const fn new() -> Self {
    Self {
      items: vec![],
      total_weight: 0
    }
  }

  fn choose(&self, rng: &mut ThreadRng) -> Option<&ItemType> {
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


type SpellingCallback<const ORTHOGRAPHIES: usize> = fn(&Language<ORTHOGRAPHIES>, &Rc<Phoneme>, &mut String, Option<&mut Peekable<Iter<Rc<Phoneme>>>>);

#[derive(Default)]
pub enum SpellingBehavior<const ORTHOGRAPHIES: usize> {
  #[default]
  Default, // default behavior is to spell the phoneme
  Text(&'static str),
  Callback(SpellingCallback<ORTHOGRAPHIES>)
}

impl<const ORTHOGRAPHIES: usize> fmt::Debug for SpellingBehavior<ORTHOGRAPHIES> {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    write!(f,"PhonemeBehavior::")?;
    match self {
      Self::Default => write!(f,"Default"),
      Self::Text(text) => write!(f,"Text({text})"),
      Self::Callback(_) => write!(f,"Callback(<...>)"),
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
      spelling: array::from_fn(|_| SpellingBehavior::default())
    }
  }
}

impl<const ORTHOGRAPHIES: usize> PhonemeBehavior<ORTHOGRAPHIES> {

  const fn new(spelling: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES]) -> Self {
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
    Self{phonemes}
  }

  fn push(&mut self,phoneme: Rc<Phoneme>) {
    self.phonemes.push(phoneme)
  }

  fn _last(&self) -> Option<&Rc<Phoneme>> {
    self.phonemes.last()
  }

}

impl Display for Word {
  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
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

  #[must_use]
  pub fn new(set_check: &'static str, choices: &[(EnvironmentChoice,usize)]) -> Self {
    let mut vec = WeightedVec::new();
    for choice in choices {
      vec.push(choice.0.clone(),choice.1)
    };
    Self(set_check,vec)

  }
}

#[derive(Debug)]
pub struct ColumnDef {
    caption: &'static str,
    set: &'static str
}

impl From<&(&'static str, &'static str)> for ColumnDef {
    // The first string is the caption, the second string is the set name.
    fn from(value: &(&'static str, &'static str)) -> Self {
        Self {
            caption: value.0,
            set: value.1
        }
    }
}

impl From<(&'static str, &'static str)> for ColumnDef {
    // The first string is the caption, the second string is the set name.
    fn from(value: (&'static str, &'static str)) -> Self {
        (&value).into()
    }
}

/// These are options you can add to some `TableDef`
#[derive(Debug,Clone)]
pub enum TableOption {
    /// For 3D and 4D phoneme tables, this will hide the captions for the subcolumns, which can compress the "appearance" of the table.
    HideSubcolumnCaptions,
    /// For 3D and 4D phoneme tables, this will hide the captions for the subcolumns, and blend the contents of the cells into one, while retaining order of phonemes and alignment of the subcolumns.
    ///
    /// This is probably most commonly used for the Voiced/Unvoiced dimension on consonants.
    BlendSubcolumns,
    /// For 4D phoneme tables, this will hide the captions for subrows, which can compress the "appearance" of the table.
    HideSubrowCaptions,
}

/*
NOTE: Four seems like an arbitrary limit. I used to have this all in a vector so the limit was usize. However, this is a user interface thing. The third and fourth axis basically just add more items to a cell in a table. Trying to do more than that is going to be difficult to represent in a way that a human to understand, and it makes processing the table harder to program. I believe such distinctions would not be found in most languages anyway.

The good news is that this doesn't limit the language if the user wants something really alien. They can just separate one of the lower axes into separate tables instead, and then they can still use this.
*/
#[derive(Debug)]
pub enum TableDef {
  OneCell(Table0DDef),
  ListTable(Table1DDef),
  SimpleTable(Table2DDef),
  TableWithSubcolumns(Table3DDef),
  TableWithSubcolumnsAndSubrows(Table4DDef)
}


impl TableDef {

  pub fn new_with_subcolumns_and_subrows(axis_1: &[(&'static str,&'static str)], axis_2: &[(&'static str,&'static str)], axis_3: &[(&'static str,&'static str)], axis_4: &[(&'static str,&'static str)]) -> Self {
      let columns: Vec<_> = axis_1.iter().map(Into::into).collect();
      let rows: Vec<_> = axis_2.iter().map(Into::into).collect();
      let subcolumns: Vec<_> = axis_3.iter().map(Into::into).collect();
      let subrows: Vec<_> = axis_4.iter().map(Into::into).collect();

      let mut definition = Table4DDef::default();

      // fill rows
      // TODO: These should be language errors
      definition.add_columns(&columns).expect("Duplicate column in table definition");
      definition.add_subcolumns(&subcolumns).expect("Duplicate subcolumn in table definition");
      definition.add_rows(&rows).expect("Duplicate row in table definition");
      definition.add_subrows(&subrows).expect("Duplicate subrow in table definition");

      Self::TableWithSubcolumnsAndSubrows(definition)
  }

  pub fn new_with_subcolumns(axis_1: &[(&'static str,&'static str)], axis_2: &[(&'static str,&'static str)], axis_3: &[(&'static str,&'static str)]) -> Self {
      let columns: Vec<_> = axis_1.iter().map(Into::into).collect();
      let rows: Vec<_> = axis_2.iter().map(Into::into).collect();
      let subcolumns: Vec<_> = axis_3.iter().map(Into::into).collect();

      let mut definition = Table3DDef::default();

      // fill rows
      // TODO: These should be language errors
      definition.add_columns(&columns).expect("Duplicate column in table definition");
      definition.add_subcolumns(&subcolumns).expect("Duplicate subcolumn in table definition");
      definition.add_rows(&rows).expect("Duplicate row in table definition");

      Self::TableWithSubcolumns(definition)
  }

  pub fn new_simple_table(axis_1: &[(&'static str,&'static str)], axis_2: &[(&'static str,&'static str)]) -> Self {
      let columns: Vec<_> = axis_1.iter().map(Into::into).collect();
      let rows: Vec<_> = axis_2.iter().map(Into::into).collect();

      let mut definition = Table2DDef::default();

      // fill rows
      // TODO: These should be language errors
      definition.add_columns(&columns).expect("Duplicate column in table definition");
      definition.add_rows(&rows).expect("Duplicate row in table definition");

      Self::SimpleTable(definition)
  }

  pub fn new_list_table(caption: &'static str, axis_1: &[(&'static str,&'static str)]) -> Self {
      let rows: Vec<_> = axis_1.iter().map(Into::into).collect();

      let mut definition = Table1DDef::new(caption);

      // fill rows
      // TODO: These should be language errors
      definition.add_rows(&rows).expect("Duplicate column in table definition");

      Self::ListTable(definition)
  }

  /// The table is just a column header and a single cell containing all phonemes
  pub fn new_single_cell(caption: &'static str) -> Self {
      let definition = Table0DDef::new(caption);

      Self::OneCell(definition)
  }

  pub fn with(self, option: TableOption) -> Result<Self,LanguageError> {
      match (self,&option) {
        (Self::TableWithSubcolumnsAndSubrows(mut definition), TableOption::HideSubcolumnCaptions) => {
            definition.hide_subcolumn_captions();
            Ok(Self::TableWithSubcolumnsAndSubrows(definition))
        },
        (Self::TableWithSubcolumnsAndSubrows(mut definition), TableOption::BlendSubcolumns) => {
            definition.hide_subcolumn_captions();
            definition.blend_subcolumns();
            Ok(Self::TableWithSubcolumnsAndSubrows(definition))
        },
        (Self::TableWithSubcolumnsAndSubrows(mut definition), TableOption::HideSubrowCaptions) => {
            definition.hide_subrow_captions();
            Ok(Self::TableWithSubcolumnsAndSubrows(definition))
        },
        (Self::TableWithSubcolumns(mut definition), TableOption::HideSubcolumnCaptions) => {
            definition.hide_subcolumn_captions();
            Ok(Self::TableWithSubcolumns(definition))
        },
        (Self::TableWithSubcolumns(mut definition), TableOption::BlendSubcolumns) => {
            definition.hide_subcolumn_captions();
            definition.blend_subcolumns();
            Ok(Self::TableWithSubcolumns(definition))
        },
        (Self::TableWithSubcolumns(_), TableOption::HideSubrowCaptions) |
        (Self::OneCell(_), TableOption::HideSubcolumnCaptions) |
        (Self::OneCell(_), TableOption::BlendSubcolumns) |
        (Self::OneCell(_), TableOption::HideSubrowCaptions) |
        (Self::ListTable(_), TableOption::HideSubcolumnCaptions) |
        (Self::ListTable(_), TableOption::BlendSubcolumns) |
        (Self::ListTable(_), TableOption::HideSubrowCaptions) |
        (Self::SimpleTable(_), TableOption::HideSubcolumnCaptions) |
        (Self::SimpleTable(_), TableOption::BlendSubcolumns) |
        (Self::SimpleTable(_), TableOption::HideSubrowCaptions) => Err(LanguageError::InvalidOptionForTable(option.clone()))
    }

  }

}

#[derive(Clone)]
pub enum ValidWordElement {
  Done(usize,&'static str), // environment
  Phoneme(usize,Rc<Phoneme>,&'static str,&'static str) // found phoneme, expected set, expected environment
}

impl Display for ValidWordElement {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::Done(index,environment) => write!(f,"[Environment {environment} at {index}]: end of word"),
      Self::Phoneme(index,phoneme,set,environment) => write!(f,"[Environment {environment} at {index}]: phoneme ({phoneme}) from {set}."),
    }

  }
}


pub enum ValidationTraceMessage<'lifetime> {
  FoundValid(&'lifetime ValidWordElement),
  FoundError(&'lifetime LanguageError)
}

impl Display for ValidationTraceMessage<'_> {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::FoundValid(valid) => write!(f,"Found valid: {valid}"),
      Self::FoundError(err) => write!(f,"!!!Found error: {err}"),
    }

  }
}


pub type ValidationTraceCallback = dyn Fn(usize, ValidationTraceMessage);

#[derive(Debug)]
struct TableEntry {
    id: &'static str,
    caption: &'static str,
    set: &'static str,
    definition: TableDef
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
  tables: Vec<TableEntry>
}

impl<const ORTHOGRAPHIES: usize> Language<ORTHOGRAPHIES> {

    #[must_use]
    pub fn new(name: &'static str, initial_phoneme_set: &'static str, initial_environment: &'static str, orthographies: [&'static str; ORTHOGRAPHIES]) -> Self {
      let mut sets = HashMap::new();
      _ = sets.insert(PHONEME, Bag::new());
      _ = sets.insert(EMPTY, Bag::new());
      let phonemes = HashMap::new();
      let environments = HashMap::new();
      let phoneme_behavior = HashMap::new();
      let tables = vec![];
      Self {
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
      let class = self.sets.entry(class).or_insert_with(Bag::new);
      if !class.contains(&phoneme) {
        _ = class.insert(phoneme);
      }
    }

    fn add_phoneme_object(&mut self, phoneme: Rc<Phoneme>, classes: &[&'static str], behavior: PhonemeBehavior<ORTHOGRAPHIES>) -> Result<Rc<Phoneme>,LanguageError> {
      if self.phonemes.contains_key(phoneme.name) {
        Err(LanguageError::PhonemeAlreadyExists(phoneme.name))
      } else {
        _ = self.phonemes.insert(phoneme.name, phoneme.clone());
        _ = self.phoneme_behavior.insert(phoneme.clone(), behavior);
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
      let behaviors = orthography.map(SpellingBehavior::Text);
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_fn(&mut self, phoneme: &'static str, callbacks: [SpellingCallback<ORTHOGRAPHIES>; ORTHOGRAPHIES], classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      let behaviors = callbacks.map(|f| SpellingBehavior::Callback(f));
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_behavior(&mut self, phoneme: &'static str, behaviors: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES], classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      self.add_phoneme_object(Phoneme::new(phoneme),classes,PhonemeBehavior::new(behaviors))
    }

    /// # Panics
    /// Panics if requested orthography index is out of range
    pub fn spell_phoneme(&self, phoneme: &Rc<Phoneme>, orthography: usize, result: &mut String, next: Option<&mut Peekable<Iter<Rc<Phoneme>>>>) {
      if orthography >= ORTHOGRAPHIES {
        panic!("Language only has {ORTHOGRAPHIES} orthographies.")
      }

      match self.phoneme_behavior.get(phoneme).and_then(|b| b.spelling.get(orthography)) {
        None | Some(SpellingBehavior::Default) => result.push_str(phoneme.name),
        Some(SpellingBehavior::Text(text)) => result.push_str(text),
        Some(SpellingBehavior::Callback(callback)) => callback(self,phoneme,result,next)
      }

    }

    #[must_use]
    pub fn spell_word(&self, word: &Word, orthography: usize) -> String {
      let mut result = String::new();
      let mut iter = word.phonemes.iter().peekable();
      while let Some(phoneme) = iter.next() {
        self.spell_phoneme(phoneme,orthography,&mut result,Some(&mut iter))
      }
      result
    }

    // will eventually be used over add_difference
    pub fn build_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),LanguageError> {
      if self.sets.contains_key(name) {
        Err(LanguageError::SetAlreadyExists(name))
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

    pub fn build_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),LanguageError> {
      if self.sets.contains_key(name) {
        Err(LanguageError::SetAlreadyExists(name))
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
              Err(LanguageError::SetIsEmpty(name))
          }

      }

    }

    // allows building a union out of multiple sets... FUTURE: The 'add' functions will become obsolete and replace with 'build' functions.
    pub fn build_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),LanguageError> {
      if self.sets.contains_key(name) {
        Err(LanguageError::SetAlreadyExists(name))
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

    pub fn add_exclusion(&mut self, name: &'static str, set: &'static str, exclude_phoneme_strs: &[&'static str]) -> Result<(),LanguageError> {

      if self.sets.contains_key(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else {
        let mut exclude_phonemes = vec![];
        for phoneme in exclude_phoneme_strs {
          exclude_phonemes.push(self.get_phoneme(phoneme)?);
        }
        let set = self.new_set(set, &exclude_phonemes)?;
        _ = self.sets.insert(name,set);
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
      if self.environments.contains_key(name) {
        Err(LanguageError::EnvironmentAlreadyExists(name))
      } else {
        _ = self.environments.insert(name,environment.to_vec());
        Ok(())
      }

    }

    pub fn add_table(&mut self, id: &'static str, caption: &'static str, set: &'static str, def: TableDef) -> Result<(),LanguageError> {
      self.tables.push(TableEntry {
        id,
        caption,
        set,
        definition: def
      });
      Ok(())
    }

    fn new_set(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>]) -> Result<Bag<Rc<Phoneme>>,LanguageError> {
      let mut set = self.get_set(set)?.clone();
      for phoneme in exclude_phonemes {
        _ = set.remove(phoneme);
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

    fn build_word(&self, environment_name: &'static str, word: &mut Word, phoneme: &Rc<Phoneme>, rng: &mut ThreadRng) -> Result<(),LanguageError> {

        let environment = self.get_environment(environment_name)?;

        for branch in environment {
            if self.phoneme_is(phoneme, branch.0)? {
                word.push(phoneme.clone()); // have to clone because we're referencing it again later. It's an RC, so that's okay.
                match branch.1.choose(rng) {
                    None => return Err(LanguageError::NoEnvironmentChoices(environment_name)),
                    Some(EnvironmentChoice::Done) => return Ok(()),
                    Some(EnvironmentChoice::Continuing(generate_set,continuing_environment,can_duplicate)) => {
                        let phoneme = if *can_duplicate {
                            self.choose(generate_set,rng)?
                        } else {
                            self.choose_except(generate_set,&[phoneme],rng)?
                        };
                        return self.build_word(continuing_environment, word, &phoneme, rng)
                    }
                }

            }
        }

        Err(LanguageError::IncompleteBranches(environment_name))

    }


    pub fn make_word(&self) -> Result<Word,LanguageError> {

        let mut word = Word::new(&[]);
        let mut rng = rand::rng();
        let phoneme = self.choose(self.initial_phoneme_set, &mut rng)?;
        self.build_word(self.initial_environment, &mut word, &phoneme, &mut rng)?;
        Ok(word)
    }



    fn validate_word(&self, environment_name: &'static str,
                            word: &mut Enumerate<Iter<Rc<Phoneme>>>, idx: usize, phoneme: &Rc<Phoneme>,
                            level: usize, validated: &[ValidWordElement], trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,LanguageError> {
        let environment = self.get_environment(environment_name)?;
        let mut validated = validated.to_vec();

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
          ($error: expr) => {_ = {
            let this_error = $error;
            if error.is_none() {
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
            if self.phoneme_is(phoneme, branch.0)? {

                let next_phoneme = word.next();

                for choice in &branch.1.items {
                    match (choice, next_phoneme) {
                        ((EnvironmentChoice::Done,_),Some((next_idx,next_phoneme))) => {
                          check_error!(LanguageError::ExpectedEndOfWord(next_idx,next_phoneme.clone(),environment_name));
                        },
                        ((EnvironmentChoice::Continuing(generate_set,_,_),_),None) => {
                          check_error!(LanguageError::ExpectedPhonemeFoundEndOfWord(idx + 1,generate_set,environment_name));
                        },
                        ((EnvironmentChoice::Continuing(generate_set,continuing_environment,can_duplicate),_),Some((next_idx,next_phoneme))) => {
                            let valid_phoneme = if *can_duplicate {
                                self.phoneme_is(next_phoneme, generate_set)?
                            } else {
                                (next_phoneme != phoneme) && self.phoneme_is(next_phoneme, generate_set)?
                            };

                            if valid_phoneme {
                              trace_valid!(ValidWordElement::Phoneme(next_idx,next_phoneme.clone(),generate_set,environment_name));
                              // NOTE: I'm cloning the iterator here so that the next branch choice looks at the same next phoneme.
                              match self.validate_word(continuing_environment, &mut word.clone(), next_idx, next_phoneme, level + 1, &validated, trace) {
                                Err(err) => error = Some(err),
                                Ok(sub_validated) => {
                                  validated = sub_validated;
                                  found_valid_path = true;
                                  // break out of the loop, we found a successful branch.
                                  break;
                                }
                              }
                            } else {
                              check_error!(LanguageError::IncorrectPhoneme(next_idx,next_phoneme.clone(),generate_set,environment_name));
                            }
                        },
                        ((EnvironmentChoice::Done,_),None) => {
                          check_valid!(ValidWordElement::Done(idx + 1,environment_name));
                          // break out of the loop, we found a successful branch.
                          break;
                        }
                    }

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

        if found_valid_path {
            Ok(validated)
        } else {
          match error {
            None =>
              // if we got here, then there were no branches that fit the current phoneme.
              Err(trace_error!(LanguageError::NoBranchFitsPhoneme(idx,phoneme.clone(),environment_name))),
            Some(err) => Err(err)
          }
        }


    }

    pub fn check_word(&self,word: &Word, trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,LanguageError> {

        let mut word = word.phonemes.iter().enumerate();
        if let Some((idx,phoneme)) = word.next() {
            if self.phoneme_is(phoneme, self.initial_phoneme_set)? {
              let valid = ValidWordElement::Phoneme(idx,phoneme.clone(),self.initial_phoneme_set,self.initial_environment);
              trace(0,ValidationTraceMessage::FoundValid(&valid));
              self.validate_word(self.initial_environment, &mut word, idx, phoneme,1,&[valid],trace)
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
        let mut phonemes: Vec<Rc<Phoneme>> = self.phonemes.values().cloned().collect();
        phonemes.sort_by(sort_phonemes_by_length_descending);

        let mut word: Vec<Rc<Phoneme>> = vec![];

        let mut source = input;

        'outer: while !source.is_empty() {
            for phoneme in &phonemes {
                let name = phoneme.name;
                if let Some(after) = source.strip_prefix(name) {
                    word.push((*phoneme).clone()); // clone twice because apparently phoneme is a double reference
                    source = after;
                    continue 'outer;
                }
            }
            return Err(LanguageError::UnknownPhonemeWhileReading(input.to_owned(),source.to_owned()));
        }

        Ok(Word::new(&word))
    }

    fn build_phoneme_grid(&self, master_set: &Bag<Rc<Phoneme>>, table: &TableDef, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<Grid,LanguageError> {

        match table {
            TableDef::OneCell(definition) => {
                let mut table = Table0D::new(definition);

                // TODO: Should be language error
                table.add_phonemes(self, master_set, unprinted_phonemes).expect("A phoneme was added with an non-existing axis");

                Ok(table.build_grid())
            },
            TableDef::ListTable(definition) => {
                let mut table = Table1D::new(definition);

                // TODO: Should be language error
                table.add_phonemes(self, master_set, unprinted_phonemes).expect("A phoneme was added with an non-existing axis");

                Ok(table.build_grid())
            },
            TableDef::SimpleTable(definition) => {
                let mut table = Table2D::new(definition);

                // TODO: Should be language error
                table.add_phonemes(self, master_set, unprinted_phonemes).expect("A phoneme was added with an non-existing axis");

                Ok(table.build_grid())
            },
            TableDef::TableWithSubcolumns(definition) => {
                let mut table = Table3D::new(definition);

                // TODO: Should be language error
                table.add_phonemes(self, master_set, unprinted_phonemes).expect("A phoneme was added with an non-existing axis");

                Ok(table.build_grid())
            },
            TableDef::TableWithSubcolumnsAndSubrows(definition) => {
                let mut table = Table4D::new(definition);

                // TODO: Should be language error
                table.add_phonemes(self, master_set, unprinted_phonemes).expect("A phoneme was added with an non-existing axis");

                Ok(table.build_grid())
            },
        }
    }

    pub fn build_all_phoneme_tables(&self) -> Result<Vec<(&'static str,&'static str,Grid)>,LanguageError> {

      let mut result = Vec::new();

      let mut unprinted_phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();

      for entry in &self.tables {

        let grid = self.build_phoneme_grid(self.get_set(entry.set)?, &entry.definition, &mut Some(&mut unprinted_phonemes))?;

        result.push((entry.id,entry.caption,grid));


      }

      if !unprinted_phonemes.is_empty() {

        let grid = self.build_phoneme_grid(&unprinted_phonemes.clone(), &TableDef::OneCell(Table0DDef::new("Uncategorized Phonemes")), &mut Some(&mut unprinted_phonemes))?;
        result.push(("uncategorized","Uncategorized",grid));

      }

      Ok(result)

    }

    pub fn build_phoneme_table(&self, table_name: &String) -> Result<Option<(&'static str,Grid)>,LanguageError> {

        if table_name == "uncategorized" { // FUTURE: Make that a constant
            // we need to build all of the tables to find the uncategorized phonemes
            let all_tables = self.build_all_phoneme_tables()?;
            Ok(all_tables.into_iter().find_map(|(id,caption,grid)| {
                if id == "uncategorized" {
                    Some((caption,grid))
                } else {
                    None
                }
            }))

        } else {
            let table = self.tables.iter().find(|entry| {
               entry.id == table_name || entry.caption == table_name
            });

            if let Some(entry) = table {
                Ok(Some((entry.caption,self.build_phoneme_grid(self.get_set(entry.set)?, &entry.definition, &mut None)?)))


            } else {
                Ok(None)
            }
        }

    }

    pub fn display_spelling(&self, columns: usize) -> Result<Grid,LanguageError> {

      let phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();
      let phonemes = phonemes.list();

      let mut grid = Grid::new();

      let mut header = Vec::new();
      for _ in 0..columns {
        header.push(ColumnHeader::new("Phoneme".to_owned(),1));
        for orthography in self.orthographies {
          header.push(ColumnHeader::new(orthography.to_owned(),1));
        }
      }
      grid.push_header_row(header);


      // once div_ceil is stable in the library, the existence of this will cause an error.
      // But, we can get rid of our shim, then.
      #[allow(unstable_name_collisions)] let length = phonemes.len().div_ceil(columns);
      let mut chunks: Vec<Iter<Rc<Phoneme>>> = phonemes.chunks(length).map(|a| a.iter()).collect();

      for _ in 0..length {
        let mut row = GridRow::new();

        for chunk in &mut chunks {
          if let Some(phoneme) = chunk.next() {
            row.push_cell(Cell::content(phoneme.to_string()));
            for i in 0..ORTHOGRAPHIES {
              let mut cell = String::new();
              self.spell_phoneme(phoneme, i, &mut cell, None);
              row.push_cell(Cell::content(cell));
            }

          } else {
            // add blank cells to make the table rectangular.
            row.push_cell(Cell::content(String::new()));
            for _ in 0..ORTHOGRAPHIES {
                row.push_cell(Cell::content(String::new()));
            }
          }
        }

        grid.push_body_row(row);


      }

      Ok(grid)

    }

    pub fn load_lexicon(&self, path: String, primary_orthography: usize) -> Result<Lexicon<ORTHOGRAPHIES>,Box<dyn Error>> {


      let mut reader = Reader::from_path(path)?;
      let headers = reader.headers()?;
      let word_field = headers.iter().position(|a| a.to_lowercase() == "word").ok_or_else(|| "No 'word' field found.".to_owned())?;
      let definition_field = headers.iter().position(|a| a.to_lowercase() == "definition").ok_or_else(|| "No 'definition' field found.".to_owned())?;

      let mut result = Lexicon::new(self.orthographies, primary_orthography);

      for (row,record) in reader.into_records().enumerate() {
        let record = record.map_err(|e| format!("Error reading record {row}: {e}"))?;
        let word = record.get(word_field).ok_or_else(|| format!("No word found at entry {row}"))?;
        let word = self.read_word(word).map_err(|e| format!("Error parsing word {row}: {e}"))?;
        let spelling = array::from_fn(|i| self.spell_word(&word, i));
        let entry: LexiconEntry<ORTHOGRAPHIES> = LexiconEntry::new(
            word,
            spelling,
            record.get(definition_field).ok_or_else(|| format!("No category found at row {row}"))?.to_owned(),
        );

        result.push(entry);

      }

      Ok(result)


    }

}

fn sort_phonemes_by_length_descending(a: &Rc<Phoneme>, b: &Rc<Phoneme>)  -> Ordering {
    let name_a = a.name;
    let len_a = name_a.len();
    let name_b = b.name;
    let len_b = name_b.len();
    if len_a == len_b {
        name_a.partial_cmp(name_b).expect("Can't order phoneme names for some reason.")
    } else {
        len_b.partial_cmp(&len_a).expect("Can't order phoneme lengths for some reason.")
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

fn parse_args<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args) -> (Option<TableStyle>,Command) {
  let mut command = None;
  let mut grid_style = None;
  let mut spanning = true;

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
      "--format=plain" => set_grid_style!(TableStyle::Plain),
      "--format=terminal" => set_grid_style!(TableStyle::Terminal{ spans: spanning }),
      "--format=markdown" => set_grid_style!(TableStyle::Markdown{ spans: spanning }),
      "--format=latex" => todo!(), //set_grid_style!(TableStyle::LaTeX),
      "--no-spans" => if let Some(style) = &mut grid_style {
          match style {
            TableStyle::Plain => (),
            TableStyle::Terminal { spans } => *spans = false,
            TableStyle::Markdown { spans } => *spans = false,
        }
      } else {
          spanning = false
      },
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

  (grid_style,command.unwrap_or(Command::GenerateWords(1)))

}


fn show_usage<const ORTHOGRAPHIES: usize>(language: &Language<ORTHOGRAPHIES>) {
    println!("usage: {} [options] <command>",language.name);
    println!("default command: --generate 1");
    println!("options:");
    println!("   --format=<plain | terminal | markdown | latex>");
    println!("      changes the format of grid output. Default is \"plain\" for generate and lexicon, and \"terminal\" for phonemes and spelling.");
    println!("commands:");
    println!("   --no-spans");
    println!("      turns off column and row spanning in headers of grid output.");
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

fn format_lexicon<const ORTHOGRAPHIES: usize>(grid_style: Option<TableStyle>, language: &Language<ORTHOGRAPHIES>, path: String, ortho_index: usize) {
  if ortho_index >= language.orthographies.len() {
        panic!("Language only has {} orthographies.",language.orthographies.len())
  }

  let grid_style = grid_style.unwrap_or(TableStyle::Plain);

  match language.load_lexicon(path,ortho_index) {
    Ok(lexicon) => {
        let result = lexicon.into_string(&grid_style);
        println!("{result}")

    },
    Err(err) => {
      eprintln!("!!! Couldn't process lexicon: {err}");
      process::exit(1)
    }
  }
}

fn show_spelling<const ORTHOGRAPHIES: usize>(grid_style: Option<TableStyle>, language: &Language<ORTHOGRAPHIES>, columns: usize) {
    match language.display_spelling(columns) {
        Ok(grid) => {
            println!("{}",grid.into_string(&grid_style.unwrap_or(TableStyle::Terminal { spans: false })));
        },
        Err(err) => {
            eprintln!("!!! Couldn't display spelling: {err}");
            process::exit(1)
        }
    }
}

fn show_phonemes<const ORTHOGRAPHIES: usize>(grid_style: Option<TableStyle>, language: &Language<ORTHOGRAPHIES>, table: Option<&String>) {
    let style = grid_style.as_ref().unwrap_or(&TableStyle::Terminal{ spans: true });
    let result = match table {
        Some(table) => match language.build_phoneme_table(table) {
            Ok(Some(grid)) => {
                println!("{}:",grid.0);
                println!("{}",grid.1.into_string(&style));
                Ok(())
            },
            Ok(None) => {
                println!("No phoneme table named {table}. Try singular?");
                Ok(())
            }
            Err(err) => Err(err),
        },
        None => match language.build_all_phoneme_tables() {
            Ok(grids) => {
                for grid in grids {
                    println!("{}:",grid.1);
                    println!("{}",grid.2.into_string(&style));
                    println!();

                }

                Ok(())
            },
            Err(err) => Err(err),
        },
    };

    if let Err(err) = result {
        eprintln!("!!! Couldn't display phonemes: {err}");
        process::exit(1)

    }
}

fn validate_words<const ORTHOGRAPHIES: usize>(language: &Language<ORTHOGRAPHIES>, words: Vec<String>, option: &ValidateOption) {
    let mut invalid_found = false;
    for word in words {
        match language.read_word(&word) {
            Ok(word) => {
                let trace_cb: Box<ValidationTraceCallback> = if matches!(option,ValidateOption::Trace) {
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
                      if matches!(option,ValidateOption::Trace) {
                        println!("!!!! invalid word (see trace)");
                      } else {
                        println!("{err}");
                      }
                    },
                    Ok(validated) => {
                      if matches!(option,ValidateOption::Explain) {
                        for valid in validated {
                          println!("{valid}")
                        }
                      }

                      for orthography in 0..language.orthographies.len() {
                        print!("{} ",language.spell_word(&word,orthography));
                      }
                      println!("{word}");
                    }
                }
            },
            Err(err) => {
                eprintln!("!!!! Can't read word: {err}");
                process::exit(1);
            }
        }
    }
    if invalid_found {
      process::exit(1);
    }
}

fn generate_words<const ORTHOGRAPHIES: usize>(grid_style: Option<TableStyle>, language: &Language<ORTHOGRAPHIES>, count: usize) {
    let mut grid = Grid::new();

    // FUTURE: Should I have a header?

    for _ in 0..count {
        let mut row = GridRow::new();

        match language.make_word() {
            Ok(word) => {
                for orthography in 0..language.orthographies.len() {
                    row.push_cell(Cell::content(language.spell_word(&word,orthography)));
                }
                row.push_cell(Cell::content(format!("{word}")));

                // the following is a sanity check. It might catch some logic errors, but really it's just GIGO.
                if let Err(err) = language.check_word(&word,&|_,_| { /* eat message, no need to report */}) {
                    println!("-- !!!! invalid word: {err}");
                    process::exit(1);
                }
            },
            Err(err) => {
                eprintln!("!!! Couldn't make word: {err}");
                process::exit(1);
            }
        }

        grid.push_body_row(row);
    }
    println!("{}",grid.into_string(grid_style.as_ref().unwrap_or(&TableStyle::Plain)));
}


pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, language: Result<Language<ORTHOGRAPHIES>,LanguageError>) {
  let (grid_style,command) = parse_args(&mut args.skip(1));

  // TODO: For blending the columns, I can just specify a column number and width and combine regular cells, then I don't need the MultiCell.
  // TODO: something like that also makes it easy to blend rows as well.
  // TODO: I need two more styles: HTML and JSON (which formats everything into a form of json for use in other tools)
  // TODO: Make sure the library exposes a way to get the phoneme table as a 'Grid' so that they can do whatever they want to with it in rust.


  match language {
      Ok(language) => {

        match command {
            Command::GenerateWords(count) => generate_words(grid_style, &language, count),
            Command::ValidateWords(words,option) => validate_words(&language, words, &option),
            Command::ShowPhonemes(table) => show_phonemes(grid_style, &language, table.as_ref()),
            Command::ShowSpelling(columns) => show_spelling(grid_style, &language, columns),
            Command::ProcessLexicon(path,ortho_index) => format_lexicon(grid_style, &language, path, ortho_index),
            Command::ShowUsage => show_usage(&language),
        }

      },
      Err(err) => eprintln!("!!! Language Incomplete: {err}")
    }


}
