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
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
#[allow(deprecated)]
use crate::phonotactics::EnvironmentChoice;
use crate::phoneme_table_builder::TableBuilder;
use crate::word::Word;
use core::slice::Iter;
use core::iter::Peekable;
use crate::orthography::SpellingCallback;
use crate::orthography::SpellingBehavior;
use crate::errors::ElbieError;
use crate::phoneme_table_builder::TableEntry;
#[allow(deprecated)]
use crate::phonotactics::EnvironmentBranch;
use crate::phoneme::Phoneme;
use std::collections::HashMap;
use core::iter;
use crate::lexicon::Lexicon;
use crate::lexicon::LexiconStyle;
use crate::word_table::WordTable;
use crate::phonotactics::PatternSet;
use crate::phonotactics::PatternBuilder;
use crate::validation::ValidationTraceCallback;
use crate::validation::ValidWordElement;
use crate::phonotactics::CaseEnvironmentBuilder;




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
  inventory: Inventory,
  // These are kept separate from the phoneme structure to reduce some type dependencies.
  // For example, if this were part of the Phoneme structure, the ORTHOGRAPHIES parameter would be required on almost everything.
  // But also, keeping this separate allows me to have a separate Inventory object which is useful for phonemes out of a language context
  // (such as temporary phonemes during transformations)
  phoneme_behavior: HashMap<Rc<Phoneme>,PhonemeBehavior>,
  orthographies: Vec<&'static str>,
  #[allow(deprecated)]
  patterns: PatternSet,
  tables: Vec<TableEntry>
}

impl Language {

    #[must_use]
    #[deprecated(since = "0.4.0", note = "Use `with_pattern` instead. The phonotactics system was overhauled, and this effects constructor parameters. If you don't have time to convert yours, make sure you check out the crate from git with tag 'v0.3.2'.")]
    pub fn new(name: &'static str, initial_phoneme_set: &'static str, initial_environment: &'static str, orthographies: Vec<&'static str>) -> Self {
      let inventory = Inventory::new();
      let phoneme_behavior = HashMap::new();
      let tables = vec![];
      let patterns = PatternSet::new(|rules| {
          #[allow(deprecated)]
          rules.case_env(initial_phoneme_set, initial_environment);
      });
      Self {
        name,
        inventory,
        phoneme_behavior,
        orthographies,
        patterns,
        tables
      }

    }

    pub fn with_pattern<Pattern: Fn(&mut PatternBuilder)>(name: &'static str, orthographies: Vec<&'static str>, initial_pattern: Pattern) -> Self {
        let inventory = Inventory::new();
        let phoneme_behavior = HashMap::new();
        let tables = vec![];
        let patterns = PatternSet::new(initial_pattern);
        Self {
          name,
          inventory,
          phoneme_behavior,
          orthographies,
          patterns,
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

    pub(crate) const fn patterns(&self) -> &PatternSet {
        &self.patterns
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
      let behaviors = orthography.iter().copied().map(SpellingBehavior::Text).collect();
      self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_fn(&mut self, phoneme: &'static str, callbacks: &[SpellingCallback], classes: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
      let behaviors = callbacks.iter().copied().map(|f| SpellingBehavior::Callback(f)).collect();
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


    #[deprecated(since = "0.4.0", note = "Use the new patterns API available with `add_pattern` and `add_pattern_environment` instead. If you don't have time to convert yours, make sure you check out the crate from git with tag 'v0.3.2'.")]
    #[allow(deprecated)]
    pub fn add_environment(&mut self, name: &'static str, environment: &[EnvironmentBranch]) -> Result<(),ElbieError> {

      self.patterns.case_environment(name, |rule| {
          for branch in environment {
              let set = branch.set();
              let choices = branch.choices();
              rule.choice(set, |choice| {
                for (item,weight) in choices.items() {
                    match item {
                    EnvironmentChoice::Done => choice.done(*weight),
                    // set to generate next phoneme from, next environment to follow, whether to allow duplicate phoneme to be generated
                    EnvironmentChoice::Continuing(generate_set, next_environment, allow_duplicates) => {
                        if *allow_duplicates {
                            #[allow(deprecated)]
                            choice.case_env(*weight, generate_set, next_environment);
                        } else {
                            #[allow(deprecated)]
                            choice.case_env_nodup(*weight,generate_set, next_environment);
                        }
                    },
                    }
                }
              });
          }
      })

    }

    // track caller allows us to catch the locations of the calls, to help the user debug.
    #[track_caller]
    pub fn add_pattern<Pattern: Fn(&mut PatternBuilder)>(&mut self, name: &'static str, pattern: Pattern) -> Result<(),ElbieError> {
        self.patterns.pattern(name, pattern)
    }

    // track caller allows us to catch the locations of the calls, to help the user debug.
    #[track_caller]
    pub fn add_pattern_environment<Callback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, name: &'static str, callback: Callback) -> Result<(),ElbieError> {
        self.patterns.case_environment(name, callback)
    }

    pub fn new_table(&mut self, id: &'static str, set: &'static str, caption: &'static str) -> TableBuilder {
        TableBuilder::new(self, id, caption, set)

    }

    pub(crate) fn make_word(&self) -> Result<Word,ElbieError> {

        //let mut word = Word::new(&[]);
        // FUTURE: Should I keep an rng and re-use it, and then be able to specify a seed when generating words?
        let mut rng = rand::rng();
        self.patterns().generate(self, &mut rng)
    }


    pub(crate) fn read_word(&self,input: &str) -> Result<Word,ElbieError> {
        // not an efficient algorithm, but it works...
        let mut phonemes: Vec<Rc<Phoneme>> = self.inventory.phonemes().values().cloned().collect();
        // sort the phonemes so that longer phonemes come first. This should avoid longer graphemes from matching the shorter graphemes accidentally. For example, say there were phonemes "aw" and "a". If "aw" is sorted first for the match, then if it's found in the word it won't be mistaken for an "a" followed by a "w".
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


    pub(crate) fn check_word(&self,word: &Word, trace: Option<&ValidationTraceCallback>) -> Result<Result<Vec<ValidWordElement>,()>,ElbieError> {

        self.patterns().validate(self, word, trace)

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

    pub(crate) fn load_lexicon(&self, words: &WordTable, primary_orthography: usize, style: &LexiconStyle) -> Result<Lexicon,Box<dyn Error>> {


      let definition_field = words.find_attribute(|a| a.to_lowercase() == "definition").ok_or_else(|| "No 'definition' field found.".to_owned())?;

      let mut result = Lexicon::new(style,self.orthographies.clone(), primary_orthography);

      for (row,entry) in words.entries().enumerate() {
        let word = &entry.word();
        let word = self.read_word(word).map_err(|e| format!("Error parsing word {row}: {e}"))?;
        let spelling = (0..self.orthographies.len()).map(|i| self.spell_word(&word, i)).collect();
        let entry = LexiconEntry::new(
            word,
            spelling,
            entry.get_attribute(definition_field).ok_or_else(|| format!("No definition found at row {row}"))?.to_owned(),
        );

        result.push_entry(entry);

      }

      Ok(result)


    }

}


impl InventoryLoader for Language {

    fn add_phoneme(&mut self, phoneme: &'static str, sets: &[&'static str]) -> Result<Rc<Phoneme>,ElbieError> {
        self.add_phoneme_to_inventory(phoneme,sets,PhonemeBehavior::new(iter::repeat_with(SpellingBehavior::default).take(self.orthographies.len()).collect()))
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
