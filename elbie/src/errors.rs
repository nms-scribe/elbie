use thiserror::Error;
use crate::phoneme_table::HeaderDef;
use crate::phoneme_table::Axis;
use crate::phoneme_table::TableOption;

#[derive(Debug,Clone,Error)]
pub enum ElbieError {
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
    #[error("Phoneme '{0}' was added with {2} spellings, but {1} were expected.")]
    MismatchedSpellingsForPhoneme(&'static str, usize,usize),

    // word validation errors //
    #[error("Word is empty")]
    EmptyWord,

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
    TooManyAxisses,

    // transformation errors
    #[error("Transformation rule '{0}' created an overlapping splice.")]
    TransformationCreatedOverlappingReplacements(&'static str),

    // family errors
    #[error("Language '{0}' was already added.")]
    LanguageAlreadyAdded(String),
    #[error("Transformation for '{0}' to '{1}' was already added.")]
    TransformationAlreadyAdded(String,String),
    #[error("Unknown language '{0}'")]
    UnknownLanguage(String),
    #[error("Language '{0}' needs to be loaded before it can be accessed.")]
    LanguageNotLoaded(String),
    #[error("No transformation available for '{0}' => '{1}'")]
    UnknownTransformation(String,String),
    #[error("Transformation for '{0}' => '{1}' needs to be loaded before it can be accessed.")]
    TransformationNotLoaded(String,String),
    #[error("No default language available.")]
    NoDefaultLanguage,
}

#[deprecated(since="0.2.2",note="Use `ElbieError` instead.")]
pub type LanguageError = ElbieError;
