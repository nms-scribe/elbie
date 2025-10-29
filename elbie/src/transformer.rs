use std::collections::HashMap;
use std::rc::Rc;

use crate::errors::LanguageError;
use crate::language::Language;
use crate::phoneme::Inventory;
use crate::phoneme::Phoneme;
use crate::word::Word;

/*
/*
pattern: Vec<Pattern>

enum Pattern:
  Match(Match),
  Replace(Match,Replacement)

TODO: Possible values for Match:
-- matches a single phoneme in a set, or identified by phoneme name
-- matches a sequence of phonemes, identified by sequence name, a sequence is a separate entity containing a Vec<Match> itself.
-- matches an optional phoneme identified by set or name
-- matches an optional sequence, identified by sequence name
-- matches a specific number of matches
   -- note: a range of matches could be a sequence of Matches to represent minimum, and optionals to represent maximum.
-- choices: these can be handled by using sets, or adding new rules to handle the alternatives (as long as the rules are defined to be mutually exclusive) If it becomes a real problem, I may add choices/branches as an entity.



*/
*/

struct Inventoried {
    namespace: &'static str,
    entity: &'static str
}

impl From<&(&'static str, &'static str)> for Inventoried {
    fn from(value: &(&'static str, &'static str)) -> Self {
        Self {
            namespace: value.0,
            entity: value.1
        }
    }
}

enum Entity {
    /// Match is successful if the current phoneme has this name.
    Phoneme(&'static str),
    /// Match is successful if the current phoneme has this name, and a phoneme with that name also found in the specified namespace.
    InventoriedPhoneme(Inventoried),
    /// Match is successful if the current phoneme is in the specified set from the specified namespace
    Set(Inventoried),
    Sequence(&'static str)
}

pub struct Match {
    /// If true, the match can be successful even if the pattern does not match.
    optional: bool,
    /// If true, the match is repeated after the first successful attempt, and the word position will be incremented for each match, but the match will always be successful no matter how many repetitions are found.
    /// If optional is false, then at least the first one is required, the remaining are optional.
    repeatable: bool,
    /// Match is successful if the current position in the word matches this entity.
    entity: Entity
}

impl Match {

    pub fn phoneme(name: &'static str) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: Entity::Phoneme(name)
        }
    }

    pub fn inventoried_phoneme(namespace: &'static str, entity: &'static str) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: Entity::InventoriedPhoneme(Inventoried { namespace, entity })
        }
    }

    pub fn set(namespace: &'static str, entity: &'static str) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: Entity::Set(Inventoried { namespace, entity })
        }
    }

    pub fn sequence(name: &'static str) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: Entity::Sequence(name)
        }
    }

    pub fn optional(self) -> Self {
        let Self {
            optional: _,
            repeatable,
            entity,
        } = self;
        Self {
            optional: true,
            repeatable,
            entity
        }
    }

    pub fn repeatable(self) -> Self {
        let Self {
            optional,
            repeatable: _,
            entity,
        } = self;
        Self {
            optional,
            repeatable: true,
            entity
        }
    }
}

pub enum Replacement {
    /// Replaces the matched pattern with the series of phonemes in a namespace
    Phonemes(Vec<Inventoried>),
    /// For **all** matching phonemes, attempts to find an equivalent phoneme in a collection that is an intersection of the second list and original phonemes sets without the first list
    /// In easier terms: It finds all of the sets that the phoneme is in, then removes from those sets the first list, and adds the sets in the second. It then looks for an intersection
    /// between all of these sets, and tries to find a single phoneme.
    /// If less or more than one phoneme is found, an error will occur.
    /// No error is returned if the original phoneme is not contained in the first list of sets.
    /// This might be used, for example, to switch phonemes from voiced to unvoiced.
    ConvertSet(Vec<Inventoried>,Vec<Inventoried>),
    /// For **all** matching phonemes, looks for a phoneme with the same name in the specified namespace, and replaces them with that.
    /// If no phoneme is found, an error will occur.
    ConvertNamespace(&'static str)

}

impl Replacement {

    pub fn phonemes(phonemes: &[(&'static str,&'static str)]) -> Self {
        let phonemes = phonemes.iter().map(Inventoried::from).collect();
        Self::Phonemes(phonemes)
    }

    pub fn convert_set(from: &[(&'static str,&'static str)], to: &[(&'static str,&'static str)]) -> Self {
        let from = from.iter().map(Inventoried::from).collect();
        let to = to.into_iter().map(Inventoried::from).collect();
        Self::ConvertSet(from,to)
    }

    pub fn convert_namespace(to: &'static str) -> Self {
        Self::ConvertNamespace(to)
    }

}

pub enum Instruction {
    /// Attempts to match the current position of the word, incrementing through the word if successful, otherwise failing the match.
    Match(Match),
    /// Attempts the match, if successful, replaces the matched content with the specified replacement and still increments through the word
    Replace(Match,Replacement)
}

pub struct Rule {
    /// Rule only matches if the pattern starts at the beginning of the word
    initial: bool,
    /// Rule only matches if the pattern ends at the end of the word
    final_: bool,
    instructions: Vec<Instruction>
}

impl Rule {

    pub fn new(initial: bool, final_: bool, instructions: Vec<Instruction>) -> Self {
        Self {
            initial,
            final_,
            instructions
        }

    }

}

/* TODO: Macro for building rules and sequences

rule!(^...) -- specifies initial is true
rule!(...^) -- specifies final is true
rule!(^...^) -- initial and final are true
rule!(match,*) -- list of matches
rule!(match => replacement,*) -- list of replacements
match!(/ident/) -- phoneme match
match!(/ident:ident/) -- inventoried phoneme
match!({ident:ident}) -- set
match!(ident) -- sequence
match!(...+) -- optional is false, repeatable is true
match!(...*) -- optional is true, repeatable is true
match!(...?) -- optional is true, repeatable is false
replacement!(/ident:ident/,*) -- replace with specified phonemes
replacement!({ident:ident,*} -> {ident:ident,*}) -- replace sets
replacement!(ident) -- convert_namespace

*/


pub struct Transformer<'inventories> {
    inventories: HashMap<&'static str,&'inventories Inventory>,
    sequences: HashMap<&'static str,Vec<Match>>,
    rules: Vec<Rule>
}

impl<'inventories> Transformer<'inventories> {

    pub fn from<const ORTHOGRAPHIES: usize>(source: &'inventories Language<ORTHOGRAPHIES>) -> Self {
        let inventories = HashMap::new();
        let sequences = HashMap::new();
        let rules = Vec::new();
        let mut result = Self {
            inventories,
            sequences,
            rules
        };
        result.add_language(source);
        result
    }

    pub fn add_language<const ORTHOGRAPHIES: usize>(&mut self, source: &'inventories Language<ORTHOGRAPHIES>) {
        _ = self.add_inventory(source.name(), source.inventory());
    }

    pub fn add_inventory(&mut self, name: &'static str, inventory: &'inventories Inventory) {
        _ = self.inventories.insert(name, inventory)
    }

    pub fn add_sequence(&mut self, name: &'static str, sequence: Vec<Match>) -> Result<(),LanguageError> {
        if self.sequences.insert(name, sequence).is_some() {
            Err(LanguageError::TransformationSequenceAlreadyExists(name))
        } else {
            Ok(())
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Applies transformation rules in order, and returns the final word if successful.
    /// The word has not been validated for any specific language, so this should still be done before reporting the result to the user.
    // TODO: The 'trace' bool should indicate if messages should be reported, similar to validation tracing.
    pub fn transform(&self, word: Word, trace: bool) -> Result<Word,LanguageError> {
        // TODO: for each rule:
        // - create a new word
        // - attempt to match the patterns with the old word at the first character
        //   - if pattern is "initial" and pattern does not match, then the match fails.
        //   - otherwise, if pattern does not match, start again at the first character and match.
        //   - if a piece of the rule simply matches, the matching phonemes should be returned
        //   - if a piece of the rule replaces, the replacement should be returned
        //   - if the match is successful, but it is final and we are not at the end of the word, then the match fails.
        //   - the successful match will indicate: where the match starts, what is replaced in the match, and where the match ends.
        // - if the match is successful, copy the word up until the match starts, then insert the replacement, and then copy the remainder of the word, that is the new word for the next rule.
        // - if the match fails, and there are also no errors, then just use the existing word as the word for the next rule.
        todo!()
    }


}
/* TODO:

Transformations, possibly:

A LanguageTransformer basically looks like this:


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
