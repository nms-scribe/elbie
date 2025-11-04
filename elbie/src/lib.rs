
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
pub mod validation;
pub mod phoneme_table_builder;
pub mod language;
pub mod transformation;
pub mod language_cli;
pub mod transformation_cli;
#[cfg(test)] mod test;

// Old paths: remove once I'm sure I've fixed all of my languages... Or, maybe just wait until I increase the version number.
#[deprecated(since="0.2.2",note="Use `elbie::errors::LanguageError` instead.")]
pub type LanguageError = errors::ElbieError;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme::EMPTY` instead.")]
pub const EMPTY: &str = phoneme::EMPTY;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme::PHONEME` instead.")]
pub const PHONEME: &str = phoneme::PHONEME;
#[deprecated(since="0.2.2",note="Use `elbie::language::Language` instead.")]
pub type Language = language::Language;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme::Phoneme` instead.")]
pub type Phoneme = phoneme::Phoneme;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table::TableOption` instead.")]
pub type TableOption = phoneme_table::TableOption;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table::Axis` instead.")]
pub type Axis = phoneme_table::Axis;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table::HeaderDef` instead.")]
pub type HeaderDef = phoneme_table::HeaderDef;
#[deprecated(since="0.2.2",note="Use `elbie::phoneme_table_builder::TableBuilder` instead.")]
pub type TableBuilder<'language,const ORTHOGRAPHIES: usize> = phoneme_table_builder::TableBuilder<'language>;
#[deprecated(since="0.2.2",note="Use `elbie::phonotactics::EnvironmentBranch` instead.")]
pub type EnvironmentBranch = phonotactics::EnvironmentBranch;
#[deprecated(since="0.2.2",note="Use `elbie::phonotactics::EnvironmentChoice` instead.")]
pub type EnvironmentChoice = phonotactics::EnvironmentChoice;
#[deprecated(since="0.2.2",note="Use `elbie::word::Word` instead.")]
pub type Word = word::Word;



#[deprecated(since="0.2.2",note="use `elbie::language_cli::run` instead.")]
pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args, language: Result<language::Language,errors::ElbieError>) {
    language_cli::run(args, language)
}
