
/*
Elbie = LB, Language Builder, and is a bunch of tools for building a constructed language.
*/


pub mod phoneme_table;
mod grid;
mod lexicon;
pub mod errors;
mod bag;
mod weighted_vec;
pub mod phoneme;
mod orthography;
mod phoneme_behavior;
pub mod word;
pub mod phonotactics;
mod validation;
pub mod phoneme_table_builder;
pub mod language;
pub mod language_cli;
#[cfg(test)] mod test;

// Old paths: remove once I'm sure I've fixed all of my languages... Or, maybe just wait until I increase the version number.
#[deprecated(since="0.2.2",note="Use `elbie::errors::LanguageError` instead.")]
pub type LanguageError = errors::LanguageError;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme::EMPTY` instead.")]
pub const EMPTY: &str = phoneme::EMPTY;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme::PHONEME` instead.")]
pub const PHONEME: &str = phoneme::PHONEME;
#[deprecated(since="0.2.2",note="Use `elbie::language::Language` instead.")]
pub type Language<const ORTHOGRAPHIES: usize> = language::Language<ORTHOGRAPHIES>;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme::Phoneme` instead.")]
pub type Phoneme = phoneme::Phoneme;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table::TableOption` instead.")]
pub type TableOption = phoneme_table::TableOption;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table::Axis` instead.")]
pub type Axis = phoneme_table::Axis;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table::HeaderDef` instead.")]
pub type HeaderDef = phoneme_table::HeaderDef;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table_builder::TableBuilder` instead.")]
pub type TableBuilder<'language,const ORTHOGRAPHIES: usize> = phoneme_table_builder::TableBuilder<'language,ORTHOGRAPHIES>;
#[deprecated(since="0.2.2",note="Use `elbie::phonotactics::EnvironmentBranch` instead.")]
pub type EnvironmentBranch = phonotactics::EnvironmentBranch;
#[deprecated(since="0.2.2",note="Use `elbie::phonotactics::EnvironmentChoice` instead.")]
pub type EnvironmentChoice = phonotactics::EnvironmentChoice;
#[deprecated(since="0.2.2",note="Use `elbie::word::Word` instead.")]
pub type Word = word::Word;



#[deprecated(since="0.2.2",note="use `elbie::language_cli::run` instead.")]
pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, language: Result<language::Language<ORTHOGRAPHIES>,errors::LanguageError>) {
    language_cli::run(args, language)
}
