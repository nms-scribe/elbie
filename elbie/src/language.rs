use crate::bag::Bag;
use crate::grid::Cell;
use crate::grid::ColumnHeader;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::lexicon::LexiconEntry;
use crate::phoneme_behavior::PhonemeBehavior;
use crate::phoneme_table::Table as _;
use crate::Table0D;
use crate::Table0DDef;
use crate::Table1D;
use crate::Table2D;
use crate::Table3D;
use crate::Table4D;
use crate::TableDef;
use core::error::Error;
use core::array;
use core::cmp::Ordering;
use std::rc::Rc;
use crate::lexicon::Lexicon;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use crate::validation::ValidationTraceMessage;
use crate::validation::ValidationTraceCallback;
use crate::validation::ValidWordElement;
use core::iter::Enumerate;
use crate::phonotactics::EnvironmentChoice;
use csv::Reader;
use rand::prelude::ThreadRng;
use crate::phoneme_table_builder::TableBuilder;
use crate::word::Word;
use core::slice::Iter;
use core::iter::Peekable;
use crate::orthography::SpellingCallback;
use crate::orthography::SpellingBehavior;
use std::collections::hash_map::Entry;
use crate::errors::LanguageError;
use crate::phoneme_table_builder::TableEntry;
use crate::phonotactics::EnvironmentBranch;
use crate::phoneme::Phoneme;
use std::collections::HashMap;

pub const PHONEME: &str = "phoneme";
pub const EMPTY: &str = "empty";




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

    pub(crate) const fn orthographies(&self) -> &[&'static str; ORTHOGRAPHIES] {
        &self.orthographies
    }

    pub(crate) const fn tables(&self) -> &Vec<TableEntry> {
        &self.tables
    }

    pub(crate) const fn tables_mut(&mut self) -> &mut Vec<TableEntry> {
        &mut self.tables
    }

    pub(crate) const fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn add_phoneme_to_set(&mut self, class: &'static str, phoneme: Rc<Phoneme>) -> Result<(),LanguageError> {
      let class = match self.sets.entry(class) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            if self.phonemes.contains_key(class) {
                return Err(LanguageError::PhonemeExistsWithSetName(class))
            }
            entry.insert(Bag::new())
        },
      };
      if !class.contains(&phoneme) {
        _ = class.insert(phoneme);
      }
      Ok(())
    }

    pub(crate) fn add_phoneme_object(&mut self, phoneme: Rc<Phoneme>, sets: &[&'static str], behavior: PhonemeBehavior<ORTHOGRAPHIES>) -> Result<Rc<Phoneme>,LanguageError> {
      if self.phonemes.contains_key(phoneme.name) {
        Err(LanguageError::PhonemeAlreadyExists(phoneme.name))
      } else if self.sets.contains_key(phoneme.name) {
        Err(LanguageError::SetExistsWithPhonemeName(phoneme.name))
      } else {
        _ = self.phonemes.insert(phoneme.name, phoneme.clone());
        _ = self.phoneme_behavior.insert(phoneme.clone(), behavior);
        self.add_phoneme_to_set(PHONEME,phoneme.clone())?;
        for class in sets {
          self.add_phoneme_to_set(class,phoneme.clone())?
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

    pub(crate) fn add_phoneme_with_spelling_behavior(&mut self, phoneme: &'static str, behaviors: [SpellingBehavior<ORTHOGRAPHIES>; ORTHOGRAPHIES], classes: &[&'static str]) -> Result<Rc<Phoneme>,LanguageError> {
      self.add_phoneme_object(Phoneme::new(phoneme),classes,PhonemeBehavior::new(behaviors))
    }

    /// # Panics
    /// Panics if requested orthography index is out of range
    pub(crate) fn spell_phoneme(&self, phoneme: &Rc<Phoneme>, orthography: usize, result: &mut String, next: Option<&mut Peekable<Iter<Rc<Phoneme>>>>) {
      if orthography >= ORTHOGRAPHIES {
        panic!("Language only has {ORTHOGRAPHIES} orthographies.")
      }

      match self.phoneme_behavior.get(phoneme).and_then(|b| b.spelling().get(orthography)) {
        None | Some(SpellingBehavior::Default) => result.push_str(phoneme.name),
        Some(SpellingBehavior::Text(text)) => result.push_str(text),
        Some(SpellingBehavior::Callback(callback)) => callback(self,phoneme,result,next)
      }

    }

    #[must_use]
    pub(crate) fn spell_word(&self, word: &Word, orthography: usize) -> String {
      let mut result = String::new();
      let mut iter = word.phonemes().iter().peekable();
      while let Some(phoneme) = iter.next() {
        self.spell_phoneme(phoneme,orthography,&mut result,Some(&mut iter))
      }
      result
    }

    // will eventually be used over add_difference
    pub fn build_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),LanguageError> {
      if self.sets.contains_key(name) {
        Err(LanguageError::SetAlreadyExists(name))
      } else if self.phonemes.contains_key(name) {
        Err(LanguageError::PhonemeExistsWithSetName(name))
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
      } else if self.phonemes.contains_key(name) {
        Err(LanguageError::PhonemeExistsWithSetName(name))
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
      } else if self.phonemes.contains_key(name) {
        Err(LanguageError::PhonemeExistsWithSetName(name))
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
      } else if self.phonemes.contains_key(name) {
        Err(LanguageError::PhonemeExistsWithSetName(name))
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


    pub(crate) fn get_set(&self, set: &'static str) -> Result<&Bag<Rc<Phoneme>>,LanguageError> {
      match self.sets.get(set) {
        Some(set) => Ok(set),
        None => Err(LanguageError::UnknownSet(set))
      }
    }

    pub(crate) fn get_phoneme(&self, phoneme: &'static str) -> Result<&Rc<Phoneme>,LanguageError> {
      match self.phonemes.get(phoneme) {
        Some(phoneme) => Ok(phoneme),
        None => Err(LanguageError::UnknownPhoneme(phoneme))
      }
    }

    pub(crate) fn get_environment(&self, environment: &'static str) -> Result<&Vec<EnvironmentBranch>,LanguageError> {
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

    pub fn new_table(&mut self, id: &'static str, set: &'static str, caption: &'static str) -> TableBuilder<ORTHOGRAPHIES> {
        TableBuilder::new(self, id, caption, set)

    }

    pub(crate) fn new_set(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>]) -> Result<Bag<Rc<Phoneme>>,LanguageError> {
      let mut set = self.get_set(set)?.clone();
      for phoneme in exclude_phonemes {
        _ = set.remove(phoneme);
      }
      Ok(set)
    }

    pub(crate) fn phoneme_is(&self, phoneme: &Rc<Phoneme>, set: &'static str) -> Result<bool,LanguageError> {
      Ok(self.get_set(set)?.contains(phoneme))
    }

    pub(crate) fn _phoneme_equals(&self, phoneme: &Rc<Phoneme>, other: &'static str) -> Result<bool,LanguageError> {
      match self.phonemes.get(other) {
        Some(other) => Ok(phoneme == other),
        None => Err(LanguageError::UnknownPhoneme(other))
      }
    }

    pub(crate) fn choose(&self, set: &'static str, rng: &mut ThreadRng) -> Result<Rc<Phoneme>,LanguageError> {
      match self.get_set(set)?.choose(rng) {
        Some(phoneme) => Ok(phoneme.clone()),
        None => Err(LanguageError::SetIsEmpty(set))
      }
    }

    pub(crate) fn choose_except(&self, set: &'static str, exclude_phonemes: &[&Rc<Phoneme>], rng: &mut ThreadRng) -> Result<Rc<Phoneme>,LanguageError> {
      match self.new_set(set,exclude_phonemes)?.choose(rng) {
        Some(phoneme) => Ok(phoneme.clone()),
        None => Err(LanguageError::SetIsEmptyWithFilter(set))
      }
    }

    pub(crate) fn build_word(&self, environment_name: &'static str, word: &mut Word, phoneme: &Rc<Phoneme>, rng: &mut ThreadRng) -> Result<(),LanguageError> {

        let environment = self.get_environment(environment_name)?;

        for branch in environment {
            if self.phoneme_is(phoneme, branch.set())? {
                word.push(phoneme.clone()); // have to clone because we're referencing it again later. It's an RC, so that's okay.
                match branch.choices().choose(rng) {
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


    pub(crate) fn make_word(&self) -> Result<Word,LanguageError> {

        let mut word = Word::new(&[]);
        let mut rng = rand::rng();
        let phoneme = self.choose(self.initial_phoneme_set, &mut rng)?;
        self.build_word(self.initial_environment, &mut word, &phoneme, &mut rng)?;
        Ok(word)
    }



    pub(crate) fn validate_word(&self, environment_name: &'static str,
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
            if self.phoneme_is(phoneme, branch.set())? {

                let next_phoneme = word.next();

                for choice in branch.choices().items() {
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

    pub(crate) fn check_word(&self,word: &Word, trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,LanguageError> {

        let mut word = word.phonemes().iter().enumerate();
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

    pub(crate) fn read_word(&self,input: &str) -> Result<Word,LanguageError> {
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

    pub(crate) fn build_phoneme_grid(&self, master_set: &Bag<Rc<Phoneme>>, table_def: &TableDef, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<Grid,LanguageError> {

        match table_def {
            TableDef::OneCell(definition) => {
                let mut table = Table0D::new(definition);

                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(LanguageError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::ListTable(definition) => {
                let mut table = Table1D::new(definition);

                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(LanguageError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::SimpleTable(definition) => {
                let mut table = Table2D::new(definition);

                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(LanguageError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::TableWithSubcolumns(definition) => {
                let mut table = Table3D::new(definition);


                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(LanguageError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::TableWithSubcolumnsAndSubrows(definition) => {

                let mut table = Table4D::new(definition);


                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(LanguageError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
        }
    }

    pub(crate) fn build_all_phoneme_tables(&self) -> Result<Vec<(&'static str,Grid)>,LanguageError> {

      let mut result = Vec::new();

      let mut unprinted_phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();

      for entry in &self.tables {

        let grid = self.build_phoneme_grid(self.get_set(entry.set())?, entry.definition(), &mut Some(&mut unprinted_phonemes))?;

        result.push((entry.id(),grid));


      }

      if !unprinted_phonemes.is_empty() {

        let grid = self.build_phoneme_grid(&unprinted_phonemes.clone(), &TableDef::OneCell(Table0DDef::new("Uncategorized Phonemes")), &mut Some(&mut unprinted_phonemes))?;
        result.push(("uncategorized",grid));

      }

      Ok(result)

    }

    pub(crate) fn build_phoneme_table(&self, table_name: &String) -> Result<Option<Grid>,LanguageError> {

        if table_name == "uncategorized" { // FUTURE: Make that a constant
            // we need to build all of the tables to find the uncategorized phonemes
            let all_tables = self.build_all_phoneme_tables()?;
            Ok(all_tables.into_iter().find_map(|(id,grid)| {
                if id == "uncategorized" {
                    Some(grid)
                } else {
                    None
                }
            }))

        } else {
            let table = self.tables.iter().find(|entry| {
               entry.id() == table_name
            });

            if let Some(entry) = table {
                Ok(Some(self.build_phoneme_grid(self.get_set(entry.set())?, entry.definition(), &mut None)?))


            } else {
                Ok(None)
            }
        }

    }

    pub(crate) fn display_spelling(&self, columns: usize) -> Result<Grid,LanguageError> {

      let phonemes: Bag<Rc<Phoneme>> = self.get_set(PHONEME)?.clone();
      let phonemes = phonemes.list();

      let mut grid = Grid::new(TableClass::ElbieOrthography, format!("Spelling for {}",self.name));

      let mut header = Vec::new();
      for _ in 0..columns {
        header.push(ColumnHeader::new("Phoneme".to_owned(),1));
        for orthography in self.orthographies {
          header.push(ColumnHeader::new(orthography.to_owned(),1));
        }
      }
      grid.set_headers(header);


      // once div_ceil is stable in the library, the existence of this will cause an error.
      // But, we can get rid of our shim, then.
      #[allow(unstable_name_collisions)] let length = phonemes.len().div_ceil(columns);
      let mut chunks: Vec<Iter<Rc<Phoneme>>> = phonemes.chunks(length).map(|a| a.iter()).collect();

      for _ in 0..length {
        let mut row = GridRow::new(TRBodyClass::BodyRow);

        for chunk in &mut chunks {
          if let Some(phoneme) = chunk.next() {
            row.push_cell(Cell::content(phoneme.to_string(),None));
            for i in 0..ORTHOGRAPHIES {
              let mut cell = String::new();
              self.spell_phoneme(phoneme, i, &mut cell, None);
              row.push_cell(Cell::content(cell,None));
            }

          } else {
            // add blank cells to make the table rectangular.
            row.push_cell(Cell::content(String::new(),None));
            for _ in 0..ORTHOGRAPHIES {
                row.push_cell(Cell::content(String::new(),None));
            }
          }
        }

        grid.push_body_row(row);


      }

      Ok(grid)

    }

    pub(crate) fn load_lexicon(&self, path: String, primary_orthography: usize) -> Result<Lexicon<ORTHOGRAPHIES>,Box<dyn Error>> {


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
