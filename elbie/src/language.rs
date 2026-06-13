use crate::bag::Bag;
use crate::errors::ElbieError;
use crate::grid::Cell;
use crate::grid::ColumnHeader;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use crate::lexicon::Lexicon;
use crate::lexicon::LexiconEntry;
use crate::lexicon::LexiconStyle;
use crate::orthography::SpellingBehavior;
use crate::orthography::SpellingCallback;
use crate::phoneme::Inventory;
use crate::phoneme::InventoryLoader;
use crate::phoneme::PHONEME;
use crate::phoneme::Phoneme;
use crate::phoneme_behavior::PhonemeBehavior;
use crate::phoneme_table::Table as _;
use crate::phoneme_table::Table0D;
use crate::phoneme_table::Table0DDef;
use crate::phoneme_table::Table1D;
use crate::phoneme_table::Table2D;
use crate::phoneme_table::Table3D;
use crate::phoneme_table::Table4D;
use crate::phoneme_table::TableDef;
use crate::phoneme_table_builder::TableBuilder;
use crate::phoneme_table_builder::TableEntry;
#[allow(deprecated)]
use crate::phonotactics::EnvironmentBranch;
#[allow(deprecated)]
use crate::phonotactics::EnvironmentChoice;
use crate::phonotactics::PatternBuilder;
use crate::phonotactics::PatternSet;
use crate::phonotactics::TreeBranchesBuilder;
use crate::validation::ValidWordElement;
use crate::validation::ValidationTraceCallback;
use crate::word::Word;
use crate::word_table::WordTable;
use core::cmp::Ordering;
use core::error::Error;
use core::iter;
use core::iter::Peekable;
use core::slice::Iter;
use std::collections::HashMap;
use std::rc::Rc;
use unicode_normalization::UnicodeNormalization as _;

fn sort_phonemes_by_length_descending(a: &Rc<Phoneme>, b: &Rc<Phoneme>) -> Ordering {
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
    phoneme_behavior: HashMap<Rc<Phoneme>, PhonemeBehavior>,
    orthographies: Vec<&'static str>,
    #[allow(deprecated)]
    patterns: PatternSet,
    tables: Vec<TableEntry>,
    analysis_cluster_sets: Option<Vec<&'static str>>,
    analysis_structure_sets: Option<Vec<&'static str>>

}

impl Language {
    #[must_use]
    #[deprecated(since = "0.4.0",
                 note = "Use `with_pattern` instead. The phonotactics system was overhauled, and this effects constructor parameters. If you don't have time to convert yours, make sure you check out the crate from git with tag 'v0.3.2'.")]
    pub fn new(name: &'static str, initial_phoneme_set: &'static str, initial_environment: &'static str, orthographies: Vec<&'static str>) -> Self {
        let inventory = Inventory::new();
        let phoneme_behavior = HashMap::new();
        let tables = vec![];
        let patterns = PatternSet::new(|rules| {
            #[allow(deprecated)]
            rules.tree_named(initial_phoneme_set, initial_environment);
        });
        let analysis_cluster_sets = None;
        let analysis_structural_sets = None;
        Self { name,
               inventory,
               phoneme_behavior,
               orthographies,
               patterns,
               tables,
               analysis_cluster_sets,
               analysis_structure_sets: analysis_structural_sets }
    }

    pub fn with_pattern<Pattern: Fn(&mut PatternBuilder)>(name: &'static str, orthographies: Vec<&'static str>, initial_pattern: Pattern) -> Self {
        let inventory = Inventory::new();
        let phoneme_behavior = HashMap::new();
        let tables = vec![];
        let patterns = PatternSet::new(initial_pattern);
        let analysis_cluster_sets = None;
        let analysis_structural_sets = None;
        Self { name,
               inventory,
               phoneme_behavior,
               orthographies,
               patterns,
               tables,
               analysis_cluster_sets,
               analysis_structure_sets: analysis_structural_sets }
    }

    #[must_use]
    pub fn with_rule(name: &'static str, orthographies: Vec<&'static str>, initial_rule: &'static str) -> Self {
        Self::with_pattern(name, orthographies, |pattern| {
            pattern.rule(initial_rule);
        })
    }

    /// If this is true, phonemes added to the language must be unicode [normalized](https://www.unicode.org/reports/tr15/) (canonical decomposition), and words will be normalized before they are read. This simplifies certain processes when reading in phonemes from external sources.
    pub const fn set_normalize_phonemes(&mut self, value: bool) {
        self.inventory.set_normalize_phonemes(value);
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

    fn add_phoneme_to_inventory(&mut self, phoneme: &'static str, sets: &[&'static str], behavior: PhonemeBehavior) -> Result<Rc<Phoneme>, ElbieError> {
        if behavior.spelling_len() != self.orthographies().len() {
            return Err(ElbieError::MismatchedSpellingsForPhoneme(phoneme, self.orthographies.len(), behavior.spelling_len()));
        }

        let phoneme = self.inventory.add_phoneme(phoneme, sets)?;
        _ = self.phoneme_behavior.insert(phoneme.clone(), behavior);
        Ok(phoneme)
    }

    fn add_phoneme_with_spelling_behavior(&mut self, phoneme: &'static str, behaviors: Vec<SpellingBehavior>, classes: &[&'static str]) -> Result<Rc<Phoneme>, ElbieError> {
        self.add_phoneme_to_inventory(phoneme, classes, PhonemeBehavior::new(behaviors))
    }

    pub fn add_phoneme_with_spelling(&mut self, phoneme: &'static str, orthography: &[&'static str], classes: &[&'static str]) -> Result<Rc<Phoneme>, ElbieError> {
        let behaviors = orthography.iter().copied().map(SpellingBehavior::Text).collect();
        self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    pub fn add_phoneme_with_spelling_fn(&mut self, phoneme: &'static str, callbacks: &[SpellingCallback], classes: &[&'static str]) -> Result<Rc<Phoneme>, ElbieError> {
        let behaviors = callbacks.iter().copied().map(|f| SpellingBehavior::Callback(f)).collect();
        self.add_phoneme_with_spelling_behavior(phoneme, behaviors, classes)
    }

    /// # Panics
    /// Panics if requested orthography index is out of range
    pub(crate) fn spell_phoneme(&self, phoneme: &Rc<Phoneme>, orthography: usize, result: &mut String, next: Option<&mut Peekable<Iter<Rc<Phoneme>>>>) {
        if orthography >= self.orthographies.len() {
            panic!("Language only has {} orthographies.", self.orthographies.len())
        }

        match self.phoneme_behavior.get(phoneme).and_then(|b| b.spelling().get(orthography)) {
            None | Some(SpellingBehavior::Default) => result.push_str(phoneme.name),
            Some(SpellingBehavior::Text(text)) => result.push_str(text),
            Some(SpellingBehavior::Callback(callback)) => callback(self, phoneme, result, next)
        }
    }

    #[must_use]
    pub(crate) fn spell_word(&self, word: &Word, orthography: usize) -> String {
        let mut result = String::new();
        let mut iter = word.phonemes().iter().peekable();
        while let Some(phoneme) = iter.next() {
            self.spell_phoneme(phoneme, orthography, &mut result, Some(&mut iter))
        }
        result
    }

    // will eventually be used over add_difference
    #[deprecated(since = "0.2.2", note = "Use `<Language as InventoryLoader>::add_difference`")]
    pub fn build_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_difference(name, base_set, exclude_sets)
    }

    #[deprecated(since = "0.2.2", note = "Use `<Language as InventoryLoader>::add_intersection`")]
    pub fn build_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_intersection(name, sets)
    }

    // allows building a union out of multiple sets... FUTURE: The 'add' functions will become obsolete and replace with 'build' functions.
    #[deprecated(since = "0.2.2", note = "Use `<Language as InventoryLoader>::add_union`")]
    pub fn build_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_union(name, sets)
    }

    #[deprecated(since = "0.4.0",
                 note = "Use the new patterns API available with `add_pattern` and `add_pattern_environment` instead. If you don't have time to convert yours, make sure you check out the crate from git with tag 'v0.3.2'.")]
    #[allow(deprecated)]
    pub fn add_environment(&mut self, name: &'static str, environment: &[EnvironmentBranch]) -> Result<(), ElbieError> {
        self.patterns.named_branches(name, |rule| {
                         for branch in environment {
                             let set = branch.set();
                             let choices = branch.choices();
                             rule.choice(set, |choice| {
                                     for (item, weight) in choices.items() {
                                         match item {
                                             EnvironmentChoice::Done => choice.done(*weight),
                                             // set to generate next phoneme from, next environment to follow, whether to allow duplicate phoneme to be generated
                                             EnvironmentChoice::Continuing(generate_set, next_environment, allow_duplicates) => {
                                                 if *allow_duplicates {
                                                     #[allow(deprecated)]
                                                     choice.tree_named(*weight, generate_set, next_environment);
                                                 } else {
                                                     #[allow(deprecated)]
                                                     choice.tree_named_nodup(*weight, generate_set, next_environment);
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
    pub fn add_pattern<Pattern: Fn(&mut PatternBuilder)>(&mut self, name: &'static str, pattern: Pattern) -> Result<(), ElbieError> {
        self.patterns.pattern(name, pattern)
    }

    pub fn format_pattern_for_debug(&mut self, name: &'static str) -> Result<String, ElbieError> {
        self.patterns.get(name).map(|p| format!("{p}"))
    }

    // track caller allows us to catch the locations of the calls, to help the user debug.
    #[track_caller]
    pub fn add_pattern_branches<Callback: Fn(&mut TreeBranchesBuilder)>(&mut self, name: &'static str, callback: Callback) -> Result<(), ElbieError> {
        self.patterns.named_branches(name, callback)
    }

    pub fn format_branches_for_debug(&mut self, name: &'static str) -> Result<String, ElbieError> {
        self.patterns.get_named_branches(name).map(|p| format!("{p}"))
    }

    pub fn new_table(&mut self, id: &'static str, set: &'static str, caption: &'static str) -> TableBuilder<'_> {
        TableBuilder::new(self, id, caption, set)
    }

    pub(crate) fn make_word(&self) -> Result<Word, ElbieError> {
        //let mut word = Word::new(&[]);
        // FUTURE: Should I keep an rng and re-use it, and then be able to specify a seed when generating words?
        let mut rng = rand::rng();
        self.patterns().generate(self, &mut rng)
    }

    pub fn read_word(&self, input: &str) -> Result<Word, ElbieError> {
        // not an efficient algorithm, but it works...
        // FUTURE: This should be "cached" somehow to speed up the process. Perhaps by using a BTreeMap instead of a HashMap, and forcing insertion in order when adding phonemes to it.
        let mut phonemes: Vec<Rc<Phoneme>> = self.inventory.phonemes().values().cloned().collect();
        // sort the phonemes so that longer phonemes come first. This should avoid longer graphemes from matching the shorter graphemes accidentally. For example, say there were phonemes "aw" and "a". If "aw" is sorted first for the match, then if it's found in the word it won't be mistaken for an "a" followed by a "w".
        phonemes.sort_by(sort_phonemes_by_length_descending);

        let mut word: Vec<Rc<Phoneme>> = vec![];

        let mut source = if self.inventory.normalize_phonemes() {
            &input.nfd().collect::<String>()
        } else {
            input
        };

        'outer: while !source.is_empty() {
            for phoneme in &phonemes {
                let name = phoneme.name;
                if let Some(after) = source.strip_prefix(name) {
                    word.push((*phoneme).clone()); // clone twice because apparently phoneme is a double reference
                    source = after;
                    continue 'outer;
                }
            }
            return Err(ElbieError::UnknownPhonemeWhileReading(input.to_owned(), source.to_owned()));
        }

        Ok(Word::new(&word))
    }

    pub(crate) fn check_word(&self, word: &Word, trace: Option<&ValidationTraceCallback>) -> Result<Result<Vec<ValidWordElement>, ()>, ElbieError> {
        self.patterns().validate(self, word, trace)
    }

    pub(crate) fn build_phoneme_grid(&self, master_set: &Bag<Rc<Phoneme>>, table_def: &TableDef, unprinted_phonemes: &mut Option<&mut Bag<Rc<Phoneme>>>) -> Result<Grid, ElbieError> {
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
            }
        }
    }

    pub(crate) fn build_all_phoneme_tables(&self) -> Result<Vec<(&'static str, Grid)>, ElbieError> {
        let mut result = Vec::new();

        let mut unprinted_phonemes: Bag<Rc<Phoneme>> = self.inventory.get_set(PHONEME)?.clone();

        for entry in &self.tables {
            let grid = self.build_phoneme_grid(self.inventory.get_set(entry.set())?, entry.definition(), &mut Some(&mut unprinted_phonemes))?;

            result.push((entry.id(), grid));
        }

        if !unprinted_phonemes.is_empty() {
            let grid = self.build_phoneme_grid(&unprinted_phonemes.clone(), &TableDef::OneCell(Table0DDef::new("Uncategorized Phonemes")), &mut Some(&mut unprinted_phonemes))?;
            result.push(("uncategorized", grid));
        }

        Ok(result)
    }

    pub(crate) fn build_phoneme_table(&self, table_name: &String) -> Result<Option<Grid>, ElbieError> {
        if table_name == "uncategorized" {
            // FUTURE: Make that a constant
            // we need to build all of the tables to find the uncategorized phonemes
            let all_tables = self.build_all_phoneme_tables()?;
            Ok(all_tables.into_iter().find_map(|(id, grid)| {
                                         if id == "uncategorized" {
                                             Some(grid)
                                         } else {
                                             None
                                         }
                                     }))
        } else {
            let table = self.tables.iter().find(|entry| entry.id() == table_name);

            if let Some(entry) = table {
                Ok(Some(self.build_phoneme_grid(self.inventory.get_set(entry.set())?, entry.definition(), &mut None)?))
            } else {
                Ok(None)
            }
        }
    }

    pub(crate) fn display_spelling(&self, columns: usize) -> Result<Grid, ElbieError> {
        let phonemes: Bag<Rc<Phoneme>> = self.inventory.get_set(PHONEME)?.clone();
        let phonemes = phonemes.list();

        let mut grid = Grid::new(TableClass::ElbieOrthography, format!("Spelling for {}", self.name));

        let mut header = Vec::new();
        for _ in 0..columns {
            header.push(ColumnHeader::new("Phoneme".to_owned(), 1));
            for orthography in &self.orthographies {
                header.push(ColumnHeader::new(orthography.to_owned().to_owned(), 1));
            }
        }
        grid.set_headers(header);

        // once div_ceil is stable in the library, the existence of this will cause an error.
        // But, we can get rid of our shim, then.
        #[allow(unstable_name_collisions)]
        let length = phonemes.len().div_ceil(columns);
        let mut chunks: Vec<Iter<Rc<Phoneme>>> = phonemes.chunks(length).map(|a| a.iter()).collect();

        for _ in 0..length {
            let mut row = GridRow::new(TRBodyClass::BodyRow);

            for chunk in &mut chunks {
                if let Some(phoneme) = chunk.next() {
                    row.push_cell(Cell::content(phoneme.to_string(), None));
                    for i in 0..self.orthographies.len() {
                        let mut cell = String::new();
                        self.spell_phoneme(phoneme, i, &mut cell, None);
                        row.push_cell(Cell::content(cell, None));
                    }
                } else {
                    // add blank cells to make the table rectangular.
                    row.push_cell(Cell::content(String::new(), None));
                    for _ in 0..self.orthographies.len() {
                        row.push_cell(Cell::content(String::new(), None));
                    }
                }
            }

            grid.push_body_row(row);
        }

        Ok(grid)
    }

    pub(crate) fn load_lexicon(&self, words: &WordTable, primary_orthography: usize, style: &LexiconStyle) -> Result<Lexicon, Box<dyn Error>> {
        let definition_field = words.find_attribute(|a| a.to_lowercase() == "definition").ok_or_else(|| "No 'definition' field found.".to_owned())?;

        let mut result = Lexicon::new(style, self.orthographies.clone(), primary_orthography);

        for (row, entry) in words.entries().enumerate() {
            let word = &entry.word();
            let word = self.read_word(word).map_err(|e| format!("Error parsing word {row}: {e}"))?;
            let spelling = (0..self.orthographies.len()).map(|i| self.spell_word(&word, i)).collect();
            let entry = LexiconEntry::new(word, spelling, entry.get_attribute(definition_field).ok_or_else(|| format!("No definition found at row {row}"))?.to_owned());

            result.push_entry(entry);
        }

        Ok(result)
    }

    /**
    The analyze command lets you review phonotactic information for existing words, helping you build patterns and rules out of words from an external source. Part of the analysis breaks the words into types of clusters, for example clusters of vowels or clusters of consonants. By default, these clusters are taken from the main sets for each table added to the language. If you wish to use different sets (for example, you have two different types of vowels in separate tables), you can specify them with this command.

    Sets must be exclusive (each phoneme is only in one of them) and have full coverage (no phoneme exists that isn't in at least one of them).

    Also see `set_analysis_structure_sets`
    */
    pub fn set_analysis_cluster_sets(&mut self, sets: &[&'static str]) {
        self.analysis_cluster_sets = Some(sets.to_vec())
    }

    pub(crate) const fn analysis_cluster_sets(&self) -> Option<&Vec<&'static str>> {
        self.analysis_cluster_sets.as_ref()
    }

    /**
    See `set_analysis_cluster_sets`. The analysis tool described there also builds trees of phoneme patterns based on their structural set. So, for example, you will be able to see how many fricatives follow plosives in a consonant cluster. The structure sets determine how to classify phonemes, so you don't just have a complex tree of individual phonemes. By default the structure sets are taken from the sets for the rows of every table (or the main set for a 0-dimensional table). If you want to override them (for example, to split or join some of the rows, or to take a different window into the phonemes), you can specify them with this command.

    Sets must be exclusive (each phoneme is only in one of them) and have full coverage (no phoneme exists that isn't in at least one of them).

    */
    pub fn set_analysis_structure_sets(&mut self, sets: &[&'static str]) {
        self.analysis_structure_sets = Some(sets.to_vec())
    }

    pub(crate) const fn analysis_structure_sets(&self) -> Option<&Vec<&'static str>> {
        self.analysis_structure_sets.as_ref()
    }

}

impl InventoryLoader for Language {
    fn add_phoneme(&mut self, phoneme: &'static str, sets: &[&'static str]) -> Result<Rc<Phoneme>, ElbieError> {
        self.add_phoneme_to_inventory(phoneme, sets, PhonemeBehavior::new(iter::repeat_with(SpellingBehavior::default).take(self.orthographies.len()).collect()))
    }

    fn add_difference(&mut self, name: &'static str, base_set: &'static str, exclude_sets: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_difference(name, base_set, exclude_sets)
    }

    fn add_intersection(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_intersection(name, sets)
    }

    fn add_union(&mut self, name: &'static str, sets: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_union(name, sets)
    }

    fn add_exclusion(&mut self, name: &'static str, set: &'static str, exclude_phoneme_strs: &[&'static str]) -> Result<(), ElbieError> {
        self.inventory.add_exclusion(name, set, exclude_phoneme_strs)
    }
}
