use thiserror::Error;
use crate::phoneme_table::HeaderDef;
use crate::phoneme_table::Axis;
use crate::phoneme::Phoneme;
use crate::phoneme_table::TableOption;
use std::rc::Rc;

#[derive(Debug,Clone,Error)]
pub enum LanguageError {
    #[error("Set {0} has no phonemes.")]
    SetIsEmpty(&'static str),
    #[error("Set {0} as filtered has no phonemes.")]
    SetIsEmptyWithFilter(&'static str),
    #[error("Unknown set {0}.")]
    UnknownSet(&'static str),
    #[error("Unknown phoneme {0}.")]
    UnknownPhoneme(&'static str),
    #[error("Phoneme {0} already exists.")]
    PhonemeAlreadyExists(&'static str),
    #[error("A set already exists with the phoneme name {0}")]
    SetExistsWithPhonemeName(&'static str),
    #[error("Set {0} already exists.")]
    SetAlreadyExists(&'static str),
    #[error("A phoneme already exists with the set name {0}")]
    PhonemeExistsWithSetName(&'static str),
    #[error( "Environment {0} already exists.")]
    EnvironmentAlreadyExists(&'static str),
    #[error("Unknown environment {0}.")]
    UnknownEnvironment(&'static str),
    #[error("Environment {0} is missing some branch environment choices.")]
    NoEnvironmentChoices(&'static str),
    #[error("Environment {0} is missing some possible branches.")]
    IncompleteBranches(&'static str),

    // word validation errors //
    #[error("Word is empty")]
    EmptyWord,
    #[error("[Environment {3} at {0}]: Expected {2}, found phoneme ({1}).")]
    IncorrectPhoneme(usize, Rc<Phoneme>, &'static str, &'static str),
    #[error("[Environment {2} at {0}]: Expected end2 word, found phoneme ({1})")]
    ExpectedEndOfWord(usize, Rc<Phoneme>, &'static str),
    #[error("[Environment {2} at {0}]: Expected {2}, found end of word")]
    ExpectedPhonemeFoundEndOfWord(usize, &'static str, &'static str),
    #[error("[Environment {2} at {0}]: Phoneme ({2}) does not match any branch.")]
    NoBranchFitsPhoneme(usize, Rc<Phoneme>, &'static str),

    // word reading errors //
    #[error("In word '{0}': unknown phoneme starting at '{1}'.")]
    UnknownPhonemeWhileReading(String,String),

    // table def errors //
    #[error("Invalid option for phoneme table: '{0:?}'.")]
    InvalidOptionForTable(TableOption),
    #[error("Duplicate phoneme {0:?} definition: '{1:?}'.")]
    DuplicateTableHeaderDef(Axis,HeaderDef),
    #[error("Duplicate table definition for id '{0}'.")]
    DuplicateTableDef(String),
    #[error("Phoneme set was not added as an {0:?} to the phoneme table.")]
    InvalidAxisForPhoneme(Axis),
    #[error("Phoneme tables are limited to a maximum of four axes.")]
    TooManyAxisses
}
