use crate::bag::Bag;
use crate::grid::Cell;
use crate::grid::ColumnHeader;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::lexicon::LexiconEntry;
use crate::phoneme::Inventory;
use crate::phoneme::InventoryLoader;
use crate::phoneme::PHONEME;
use crate::phoneme_behavior::PhonemeBehavior;
use crate::phoneme_table::Table as _;
use crate::phoneme_table::Table0D;
use crate::phoneme_table::Table0DDef;
use crate::phoneme_table::Table1D;
use crate::phoneme_table::Table2D;
use crate::phoneme_table::Table3D;
use crate::phoneme_table::Table4D;
use crate::phoneme_table::TableDef;
use core::error::Error;
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
use crate::errors::ElbieError;
use crate::phoneme_table_builder::TableEntry;
use crate::phonotactics::EnvironmentBranch;
use crate::phoneme::Phoneme;
use std::collections::HashMap;




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
pub struct Language {
  name: &'static str,
  initial_environment: &'static str,
  initial_phoneme_set: &'static str,
  inventory: Inventory,
  // These are kept separate from the phoneme structure to reduce some type dependencies.
  // For example, if this were part of the Phoneme structure, the ORTHOGRAPHIES parameter would be required on almost everything.
  // But also, keeping this separate allows me to have a separate Inventory object which is useful for phonemes out of a language context
  // (such as temporary phonemes during transformations)
  phoneme_behavior: HashMap<Rc<Phoneme>,PhonemeBehavior>,
  orthographies: Vec<&'static str>,
  environments: HashMap<&'static str,Vec<EnvironmentBranch>>,
  tables: Vec<TableEntry>
}

impl Language {

    #[must_use]
    pub fn new(name: &'static str, initial_phoneme_set: &'static str, initial_environment: &'static str, orthographies: Vec<&'static str>) -> Self {
      let inventory = Inventory::new();
      let environments = HashMap::new();
      let phoneme_behavior = HashMap::new();
      let tables = vec![];
      Self {
        name,
        initial_environment,
        initial_phoneme_set,
        inventory,
        phoneme_behavior,
        orthographies,
        environments,
        tables
      }

    }

    pub(crate) const fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub(crate) fn orthographies(&self) -> &[&'static str] {
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


    fn add_phoneme_to_inventory(&mut self, phoneme: &'static str, sets: &[&'static str], behavior: PhonemeBehavior) -> Result<Rc<Phoneme>,ElbieError> {
      if behavior.spelling_len() != self.orthographies().len() {
        return Err(ElbieError::MismatchedSpellingsForPhoneme(phoneme,self.orthographies.len(),behavior.spelling_len()))
      }

      let phoneme = self.inventory.add_phoneme(phoneme, sets)?;
      _ = self.phoneme_behavior.insert(phoneme.clone(), behavior);
      Ok(phoneme)

    }

    fn add_phoneme_with_spelling_behavior(&mut self, phoneme: &'static str, behaviors: Vec<SpellingBehavior>, classes: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
      self.add_phoneme_to_inventory(phoneme,classes,PhonemeBehavior::new(behaviors))
    }

    pub fn add_phoneme_with_spelling(&mut self, phoneme: &'static str, orthography: &[&'static str], classes: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
      let behaviors = orthography.into_iter().copied().map(SpellingBehavior::Text).collect();
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_fn(&mut self, phoneme: &'static str, callbacks: &[SpellingCallback], classes: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
      let behaviors = callbacks.into_iter().copied().map(|f| SpellingBehavior::Callback(f)).collect();
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    /// # Panics
    /// Panics if requested orthography index is out of range
    pub(crate) fn spell_phoneme(&self, phoneme: &Rc<Phoneme>, orthography: usize, result: &mut String, next: Option<&mut Peekable<Iter<Rc<Phoneme>>>>) {
      if orthography >= self.orthographies.len() {
        panic!("Language only has {} orthographies.",self.orthographies.len())
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
    #[deprecated(since="0.2.2",note="Use `<Language as InventoryLoader>::add_difference`")]
    pub fn build_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),ElbieError> {
      self.inventory.add_difference(name, base_set, exclude_sets)
    }

    #[deprecated(since="0.2.2",note="Use `<Language as InventoryLoader>::add_intersection`")]
    pub fn build_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError> {
      self.inventory.add_intersection(name, sets)
    }

    // allows building a union out of multiple sets... FUTURE: The 'add' functions will become obsolete and replace with 'build' functions.
    #[deprecated(since="0.2.2",note="Use `<Language as InventoryLoader>::add_union`")]
    pub fn build_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError> {
      self.inventory.add_union(name, sets)

    }

    pub(crate) fn get_environment(&self, environment: &'static str) -> Result<&Vec<EnvironmentBranch>,ElbieError> {
      match self.environments.get(environment) {
        Some(environment) => Ok(environment),
        None => Err(ElbieError::UnknownEnvironment(environment))
      }
    }

    pub fn add_environment(&mut self, name: &'static str, environment: &[EnvironmentBranch]) -> Result<(),ElbieError> {
      if self.environments.contains_key(name) {
        Err(ElbieError::EnvironmentAlreadyExists(name))
      } else {
        _ = self.environments.insert(name,environment.to_vec());
        Ok(())
      }

    }

    pub fn new_table(&mut self, id: &'static str, set: &'static str, caption: &'static str) -> TableBuilder {
        TableBuilder::new(self, id, caption, set)

    }


    pub(crate) fn build_word(&self, environment_name: &'static str, word: &mut Word, phoneme: &Rc<Phoneme>, rng: &mut ThreadRng) -> Result<(),ElbieError> {

        let environment = self.get_environment(environment_name)?;

        for branch in environment {
            if self.inventory.phoneme_is(phoneme, branch.set())? {
                word.push(phoneme.clone()); // have to clone because we're referencing it again later. It's an RC, so that's okay.
                match branch.choices().choose(rng) {
                    None => return Err(ElbieError::NoEnvironmentChoices(environment_name)),
                    Some(EnvironmentChoice::Done) => return Ok(()),
                    Some(EnvironmentChoice::Continuing(generate_set,continuing_environment,can_duplicate)) => {
                        let phoneme = if *can_duplicate {
                            self.inventory.choose(generate_set,rng)?
                        } else {
                            self.inventory.choose_except(generate_set,&[phoneme],rng)?
                        };
                        return self.build_word(continuing_environment, word, &phoneme, rng)
                    }
                }

            }
        }

        Err(ElbieError::IncompleteBranches(environment_name))

    }


    pub(crate) fn make_word(&self) -> Result<Word,ElbieError> {

        let mut word = Word::new(&[]);
        let mut rng = rand::rng();
        let phoneme = self.inventory.choose(self.initial_phoneme_set, &mut rng)?;
        self.build_word(self.initial_environment, &mut word, &phoneme, &mut rng)?;
        Ok(word)
    }


    pub(crate) fn read_word(&self,input: &str) -> Result<Word,ElbieError> {
        // not an efficient algorithm, but it works...
        let mut phonemes: Vec<Rc<Phoneme>> = self.inventory.phonemes().values().cloned().collect();
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
            return Err(ElbieError::UnknownPhonemeWhileReading(input.to_owned(),source.to_owned()));
        }

        Ok(Word::new(&word))
    }

    pub(crate) fn validate_word(&self, environment_name: &'static str,
                            word: &mut Enumerate<Iter<Rc<Phoneme>>>, idx: usize, phoneme: &Rc<Phoneme>,
                            level: usize, validated: &[ValidWordElement], trace: Option<&ValidationTraceCallback>) -> Result<Vec<ValidWordElement>,ElbieError> {
        let environment = self.get_environment(environment_name)?;
        let mut validated = validated.to_vec();

        let mut found_valid_path = false;
        let mut error = None;

        macro_rules! trace_error {
          ($error: expr) => {{
            if let Some(trace) = trace {
                trace(level,ValidationTraceMessage::FoundError(&$error));
            }
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
            if let Some(trace) = trace {
                trace(level,ValidationTraceMessage::FoundValid(&this_valid));
            }
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
            if self.inventory.phoneme_is(phoneme, branch.set())? {

                let next_phoneme = word.next();

                for choice in branch.choices().items() {
                    match (choice, next_phoneme) {
                        ((EnvironmentChoice::Done,_),Some((next_idx,next_phoneme))) => {
                          check_error!(ElbieError::ExpectedEndOfWord(next_idx,next_phoneme.clone(),environment_name));
                        },
                        ((EnvironmentChoice::Continuing(generate_set,_,_),_),None) => {
                          check_error!(ElbieError::ExpectedPhonemeFoundEndOfWord(idx + 1,generate_set,environment_name));
                        },
                        ((EnvironmentChoice::Continuing(generate_set,continuing_environment,can_duplicate),_),Some((next_idx,next_phoneme))) => {
                            let valid_phoneme = if *can_duplicate {
                                self.inventory.phoneme_is(next_phoneme, generate_set)?
                            } else {
                                (next_phoneme != phoneme) && self.inventory.phoneme_is(next_phoneme, generate_set)?
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
                              check_error!(ElbieError::IncorrectPhoneme(next_idx,next_phoneme.clone(),generate_set,environment_name));
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
                  check_error!(ElbieError::IncompleteBranches(environment_name));

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
              Err(trace_error!(ElbieError::NoBranchFitsPhoneme(idx,phoneme.clone(),environment_name))),
            Some(err) => Err(err)
          }
        }


    }

    pub(crate) fn check_word(&self,word: &Word, trace: Option<&ValidationTraceCallback>) -> Result<Vec<ValidWordElement>,ElbieError> {

        let mut word = word.phonemes().iter().enumerate();
        if let Some((idx,phoneme)) = word.next() {
            if self.inventory.phoneme_is(phoneme, self.initial_phoneme_set)? {
              let valid = ValidWordElement::Phoneme(idx,phoneme.clone(),self.initial_phoneme_set,self.initial_environment);
              if let Some(trace) = trace {
                  trace(0,ValidationTraceMessage::FoundValid(&valid));
              }
              self.validate_word(self.initial_environment, &mut word, idx, phoneme,1,&[valid],trace)
            } else {
              let err = ElbieError::IncorrectPhoneme(idx,phoneme.clone(),self.initial_phoneme_set,self.initial_environment);
              if let Some(trace) = trace {
                  trace(0,ValidationTraceMessage::FoundError(&err));
              }
              Err(err)
            }
        } else {
            Err(ElbieError::EmptyWord)
        }
    }



    pub(crate) fn build_phoneme_grid(&self, master_set: &Bag<Rc<Phoneme>>, table_def: &TableDef, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<Grid,ElbieError> {

        match table_def {
            TableDef::OneCell(definition) => {
                let mut table = Table0D::new(definition);

                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(ElbieError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::ListTable(definition) => {
                let mut table = Table1D::new(definition);

                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(ElbieError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::SimpleTable(definition) => {
                let mut table = Table2D::new(definition);

                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(ElbieError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::TableWithSubcolumns(definition) => {
                let mut table = Table3D::new(definition);


                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(ElbieError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
            TableDef::TableWithSubcolumnsAndSubrows(definition) => {

                let mut table = Table4D::new(definition);


                table.add_phonemes(self, master_set, unprinted_phonemes).map_err(ElbieError::InvalidAxisForPhoneme)?;

                Ok(table.build_grid())
            },
        }
    }

    pub(crate) fn build_all_phoneme_tables(&self) -> Result<Vec<(&'static str,Grid)>,ElbieError> {

      let mut result = Vec::new();

      let mut unprinted_phonemes: Bag<Rc<Phoneme>> = self.inventory.get_set(PHONEME)?.clone();

      for entry in &self.tables {

        let grid = self.build_phoneme_grid(self.inventory.get_set(entry.set())?, entry.definition(), &mut Some(&mut unprinted_phonemes))?;

        result.push((entry.id(),grid));


      }

      if !unprinted_phonemes.is_empty() {

        let grid = self.build_phoneme_grid(&unprinted_phonemes.clone(), &TableDef::OneCell(Table0DDef::new("Uncategorized Phonemes")), &mut Some(&mut unprinted_phonemes))?;
        result.push(("uncategorized",grid));

      }

      Ok(result)

    }

    pub(crate) fn build_phoneme_table(&self, table_name: &String) -> Result<Option<Grid>,ElbieError> {

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
                Ok(Some(self.build_phoneme_grid(self.inventory.get_set(entry.set())?, entry.definition(), &mut None)?))


            } else {
                Ok(None)
            }
        }

    }

    pub(crate) fn display_spelling(&self, columns: usize) -> Result<Grid,ElbieError> {

      let phonemes: Bag<Rc<Phoneme>> = self.inventory.get_set(PHONEME)?.clone();
      let phonemes = phonemes.list();

      let mut grid = Grid::new(TableClass::ElbieOrthography, format!("Spelling for {}",self.name));

      let mut header = Vec::new();
      for _ in 0..columns {
        header.push(ColumnHeader::new("Phoneme".to_owned(),1));
        for orthography in &self.orthographies {
          header.push(ColumnHeader::new(orthography.to_owned().to_owned(),1));
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
            for i in 0..self.orthographies.len() {
              let mut cell = String::new();
              self.spell_phoneme(phoneme, i, &mut cell, None);
              row.push_cell(Cell::content(cell,None));
            }

          } else {
            // add blank cells to make the table rectangular.
            row.push_cell(Cell::content(String::new(),None));
            for _ in 0..self.orthographies.len() {
                row.push_cell(Cell::content(String::new(),None));
            }
          }
        }

        grid.push_body_row(row);


      }

      Ok(grid)

    }

    pub(crate) fn load_lexicon(&self, path: &str, primary_orthography: usize) -> Result<Lexicon,Box<dyn Error>> {


      let mut reader = Reader::from_path(path)?;
      let headers = reader.headers()?;
      let word_field = headers.iter().position(|a| a.to_lowercase() == "word").ok_or_else(|| "No 'word' field found.".to_owned())?;
      let definition_field = headers.iter().position(|a| a.to_lowercase() == "definition").ok_or_else(|| "No 'definition' field found.".to_owned())?;

      let mut result = Lexicon::new(self.orthographies.clone(), primary_orthography);

      for (row,record) in reader.into_records().enumerate() {
        let record = record.map_err(|e| format!("Error reading record {row}: {e}"))?;
        let word = record.get(word_field).ok_or_else(|| format!("No word found at entry {row}"))?;
        let word = self.read_word(word).map_err(|e| format!("Error parsing word {row}: {e}"))?;
        let spelling = (0..self.orthographies.len()).map(|i| self.spell_word(&word, i)).collect();
        let entry: LexiconEntry = LexiconEntry::new(
            word,
            spelling,
            record.get(definition_field).ok_or_else(|| format!("No category found at row {row}"))?.to_owned(),
        );

        result.push(entry);

      }

      Ok(result)


    }

}


impl InventoryLoader for Language {

    fn add_phoneme(&mut self, phoneme: &'static str, sets: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
        self.add_phoneme_to_inventory(phoneme,sets,PhonemeBehavior::new((0..self.orthographies.len()).map(|_| SpellingBehavior::default()).collect()))
    }

    fn add_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(),ElbieError> {
        self.inventory.add_difference(name, base_set, exclude_sets)
    }

    fn add_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError> {
        self.inventory.add_intersection(name, sets)
    }

    fn add_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(),ElbieError> {
        self.inventory.add_union(name, sets)
    }

    fn add_exclusion(&mut self, name: &'static str, set: &'static str, exclude_phoneme_strs: &[&'static str]) -> Result<(),ElbieError> {
        self.inventory.add_exclusion(name, set, exclude_phoneme_strs)
    }


}
