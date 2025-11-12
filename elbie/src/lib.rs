
/*
Elbie = LB, Language Builder, and is a bunch of tools for building a constructed language.
*/

// TODO: I could allow translations in IPA ASCII formats with this crate: https://github.com/tirimid/ipa-translate

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

/* FUTURE: Another attempt at making a DSL for this.

elbie-file = language+
-- note that you can define multiple languages in a single file. If you are generating words, you will have to specify the language to generate from.

language = 'language' identifier ((phonology table*) | (table+ phonology? table*)) phonotactics orthography? lexicon?

phonology = 'phonology' phonology-declaration*

phonology-declaration = phoneme-category-declaration | alias-declaration

phoneme-category-declaration = phoneme-declaration | category-declaration

phoneme-declaration = phoneme (identifier? string? (('{' identifier* '}') | ';'))?
-- the first identifier is an alias for the phoneme to make it easier to read in the definitions
-- the string can be used in orthography, it's a shortcut for specifying a string to spell a phoneme as.
-- identifiers in brackets are categories the phoneme is supposed to be in
-- note that the semi-colon is only required if the categories clause is not included, and even then only if an alias or a spelling is given.
-- there should be some way to check that the phoneme is mutually-exclusive with other phonemes, so that when a word is read in, it won't be confused
with others.

category-declaration = 'add' identifier '{' phoneme-category-declaration+ '}'
-- another way of defining categories and their contents. the declarations contain phonemes that are defined as part of the category
-- phonemes can have their own further categories defined as a regular phoneme definition
-- contained categories mean that all categories up to the root are applied to contained phonemes -- they do not mean the subcategories are subsets of the above.
-- this form can be repeated for the same category, in which case it adds further phonemes and subcategories to the category.
-- the 'add' part of the statement is used to make it clear that this is adding to the categories according to the current state. If the categories
   in the expression are modified later, the result category in this statement will not change.

alias = identifier '=' set-expression ';'
-- an alias defines a 'set' which is based on combinations of other sets and phonemes, or rarely an alias defines an identifier that represents a phoneme (the preferred
way of doing that is using the mechanism in phoneme-declaration).

set-expression = term | (set-expression ('+' | '&' | '-' ) term)
-- '+' is union, '&' is intersection, '-' is set difference
-- early versions of this might not allow mixed operators (except a '-' at the end, which might only allow a phoneme-list) for efficiency, requiring all sets to be named.

term = group-expression | phoneme-list | identifier
-- early versions of this might not allow groups.

group-expression = '(' set-expression ')'

phoneme-list = phoneme ( ',' phoneme)*
-- if the phoneme-list only has one phoneme, and it is the main expression, the alias is a phoneme alias.

table = 'table' identifier sub-cell-def table-def 'end'
-- the identifier specifies the master category in which the phonemes in the table will be found.

sub-cell-def = '(' table-identifier-term* ')'
-- if multiple phonemes are found in a cell, this defines which categories get assigned to each, in order. Multi-line isn't possible.

table-def = table-header-def ';' (table-row-def ';'+)+ table-row-def? 'end'

table-header-def = '|' (table-identifier-term '|')+
-- the phonemes in the matching columns below are assigned to the category in the column header
-- column categories can be repeated

table-row-def = table-identifier-term '|' ((phoneme)* '|')+
-- the first identifier specifies that phonemes in this row are assigned to that category, as well as their column categories, and the sub-cell categories.
-- row categories can be repeated.

table-identifier-term = identifier | '(' identifier* ')'
-- basically, you can assign the phonemes to multiple categories at once in the column by grouping them in parantheses.

phonotactics = 'phonotactics' phonotactics-environment* initial-phonotactics phonotactics-environment*
-- in the future I may come up with another mechanism for defining this, that looks more like Linguistic academic standards. This is why 'environment' must be
specified at each line, so I can change to 'rule' or something like that.

initial-phonotactics = 'initial' 'phonemes' identifier '>' identifier
-- first identifier is the set of phonemes to generate the first phoneme from
-- second identifier is the name of an environment to follow the branches of after generating the phoneme.

phonotactics-environment = 'environment' identifier ':' phonotactics-branch (';' phonotactics-branch) '.'
-- the identifier is the name of the environment being defined

phonotactics-branch = 'on' identifier 'choose' phonotactics-choice (',' phonotactics-choice)
-- identifier is a set of phonemes, if the last generated phoneme is in this set, this branch will be followed.

phonotactics-choice = 'done' | phonotactics-continuing-choice ('(' integer ')')?
-- done means the word can end here. (I need a way to check for infinite recursion)
-- integer provides a weight to the choice

phontactics-continuing-choice = 'phonemes' identifier 'nocopy'? '>' identifier
-- first identifier is the name of a a set of phonemes to generate the next phoneme from.
-- second identifier is the name of an environment to enter after generating that phoneme.
-- nocopy means that the previous phoneme can not be duplicated in this choice, even if it's a member of the set.

orthography = 'orthography' orthography-definition*

orthography-definition = phoneme ':' string ';'
-- at some point I might allow some sort of scripting language, or at least just a few rules. For now this is a useless definition since we can already specify this in the phoneme definition.

lexicon = 'lexicon' lexicon-entry
-- **** if this section is included, all words in the lexicon will be validated against the current language at load time, and an invalid word will cause a syntax error.

lexicon-entry = spelled-word phonetic-word

spelled-word = .... this is like an identifier except that it's pretty much everything allowed except spaces, and I might even allow that with escapes.

phonetic-word = '/'...'/' basically defines a word made of a series of phonemes.

-----

FUTURE: Elbie Annotated Text Mode:

It should be possible to specify a regular text file with the Elbie code embedded in it, allowing you to mix documentation and the Elbie code. For this mode, you specify
'delimiters' for the elbie code. For example, in Markdown you might specify '<!--' and '-->', or more likely you'll specify a special indicator as well so you can still use regular
comments: '<!--%%' and '%%-->'.

Although, since some of the stuff in here needs to be included in the documentation, maybe something else should be used instead, like a backtick to separate code, so that the
Elbie stuff still shows up in the final document.

When parsing a document in this mode, elbie will ignore all content that is not within a matching pair of delimiters, and parse the remaining content as Elbie code.

Alternatively, I could have a mode which takes the comments and extracts them into an output document, along with the output of macros that reference the Elbie code,
such as phoneme tables, lexicon words, etc.

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
mod cli_functions;
#[deprecated(since="0.2.2",note="Use `cli::run_language` instead.")]
pub mod language_cli;
pub mod family;
pub mod cli;
#[cfg(test)] mod test;

pub use constcat;

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


#[deprecated(since="0.2.2",note="use `elbie::cli::run_language` instead (includes changes to arguments).")]
pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args, language: Result<language::Language,errors::ElbieError>) {
    #[expect(deprecated)]
    language_cli::run(args, language);
}
