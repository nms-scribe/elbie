
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
pub mod cli;
#[cfg(test)] mod test;

// Old paths: remove once I'm sure I've fixed all of my languages...
#[deprecated="Use `elbie::errors::LanguageError` instead."]
pub type LanguageError = errors::LanguageError;
#[deprecated="Use `elbie::phoneme::EMPTY` instead."]
pub const EMPTY: &str = phoneme::EMPTY;
#[deprecated="Use `elbie::phoneme::PHONEME` instead."]
pub const PHONEME: &str = phoneme::PHONEME;
#[deprecated="Use `elbie::language::Language` instead."]
pub type Language<const ORTHOGRAPHIES: usize> = language::Language<ORTHOGRAPHIES>;
#[deprecated="Use `elbie::phoneme::Phoneme` instead."]
pub type Phoneme = phoneme::Phoneme;
#[deprecated="Use `elbie::phoneme_table::TableOption` instead."]
pub type TableOption = phoneme_table::TableOption;
#[deprecated="Use `elbie::phoneme_table::Axis` instead."]
pub type Axis = phoneme_table::Axis;
#[deprecated="Use `elbie::phoneme_table::HeaderDef` instead."]
pub type HeaderDef = phoneme_table::HeaderDef;
#[deprecated="Use `elbie::phoneme_table_builder::TableBuilder` instead."]
pub type TableBuilder<'language,const ORTHOGRAPHIES: usize> = phoneme_table_builder::TableBuilder<'language,ORTHOGRAPHIES>;
#[deprecated="Use `elbie::phonotactics::EnvironmentBranch` instead."]
pub type EnvironmentBranch = phonotactics::EnvironmentBranch;
#[deprecated="Use `elbie::phonotactics::EnvironmentChoice` instead."]
pub type EnvironmentChoice = phonotactics::EnvironmentChoice;
#[deprecated="Use `elbie::word::Word` instead."]
pub type Word = word::Word;



#[deprecated="use `elbie::cli::run_main` instead."]
pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, language: Result<language::Language<ORTHOGRAPHIES>,errors::LanguageError>) {
    cli::run_main(args, language)
}


/* TODO:

TODO: First, clean up cargo clippy, then publish.

TODO: This will be easier if I can separate the phoneme/set stuff from the language, as I will need a similar non-language structure to do that.

Transformations, possibly:

A LanguageTransformer basically looks like this:

struct Transformer
  phonemes: HashMap<str,Rc<Phoneme>>
  sets: HashMap<str,Bag<Rc<Phoneme>>>
  rules: Vec<(Vec<Match>,Vec<Replace>)>

Transformer::from_language(Language) -> Self
-- copies sets and phonemes from the specified language and inserts them into Namespaced entities. (str,str) where the first is the language and the second is the value.

Transformer::add_target_language(&mut self, Language)
-- copies sets and phonemes from the target language, into namespaces
-- Note that this is optional, as Transformers could be used to implement orthography instead
-- more than one language can also be used, which allows the user to transform *through* another language.

Transformer::add_phoneme(&mut self, Rc<Phoneme>, namespace: str, sets: [(str)])
-- adds a specified phoneme to a temporary namespace with the specified sets (the namespace can not be the same as one of the languages)

Transformer::build_*
-- similar to all of the build_* sets in the languages.

Transformer::add_rule(Match)
--- see below for Matches and Replaces
--- adds a "transformation" rule.

Transformer::transform(Word,trace) -> Word
-- applies each rule, one at a time and in order, to the word, transforming it as it goes, and returns the specified word
-- if the pattern matches, any Captures in the rule are replaced with the Replace expression. There must be one Replace expression for each Capture, all phonemes in the replace will be inserted where the capture was found.

struct Match
  initial: indicates that the match must occur at the beginning of the word. The matcher will not continue on to see other phonems in the word if the first phoneme doesn't match the first pattern
  final: indicates that the match must occur at the end of the word. The matcher will not match if the end of the word is not reached when the pattern is matched.
  patterns: Vec<Pattern>

enum Pattern
  Match(name,Usize,NonZeroUsize) -- matches from x to y phonemes of the specified set or phoneme name if the set doesn't exist, a match can match 0 phonemes.
  Replace(name,Usize,NonZeroUsize,Replace) -- "captures" from x to y phonemes of the specified set or phoneme name, if a capture has a min of 0, then it will always capture there, and any replace expression is inserted in it's place.
  -- repeats are built into the enums, so I don't need anything special
  -- sequences are part of the pattern
  -- choices can be handled two ways:
     -- a choice of a single phoneme can be a set
     -- a choice of two different sequences can be a new rule.

enum Replace
  Sequence(Vec<Rc<Phoneme>>) -- replaces with specified phoneme
  ConvertSet(Vec<str>,Vec<str>) -- Extracts the sets from the phoneme, removes the specified sets, and then adds the new sets. It then looks for a single phoneme in the new set and replaces the phoneme with that one. Error if not exactly 1 phoneme is returned, or if the original phoneme doesn't have the source sets. Note that sets can be empty in source or target.
  SwitchSet(str,str) -- switches a "set" from one to the other
  SetLanguage(str) -- switches the phoneme from it's source to the same phoneme in the target language, error if it doesn't exist in the new language


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
