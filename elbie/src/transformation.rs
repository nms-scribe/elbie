use core::fmt;
use core::iter::Peekable;
use core::fmt::Display;
use core::fmt::Formatter;
use std::rc::Rc;
use core::slice::Iter;
use crate::errors::ElbieError;
use crate::language::Language;
use crate::phoneme::Inventory;
use crate::phoneme::Phoneme;
use crate::word::Word;


pub(crate) enum TransformationTraceMessage {
  StartTransformation(Word),
  MatchedRule(&'static str,Word,Word),
  UnmatchedRule(&'static str)
}

impl Display for TransformationTraceMessage {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::StartTransformation(word) => write!(f,"Tracing Transformation: '{word}'"),
      Self::MatchedRule(name,from,to) => write!(f,"Matched '{name}': {from} 🡺 {to}"),
      Self::UnmatchedRule(name) => write!(f,"Did not match '{name}'"),
    }

  }
}


pub(crate) type TransformationTraceCallback = dyn Fn(TransformationTraceMessage);

struct WordSplice {
    start_index: usize,
    length: usize,
    replace: Vec<Rc<Phoneme>>
}

pub enum RuleStateError {
    MatchFailed,
    Elbie(ElbieError)
}

impl From<ElbieError> for RuleStateError {
    fn from(value: ElbieError) -> Self {
        Self::Elbie(value)
    }
}

pub struct RuleState<'phonemes> {
    inventory: &'phonemes Inventory,
    phonemes: Peekable<Iter<'phonemes, Rc<Phoneme>>>,
    word_index: usize,
    splices: Vec<WordSplice>
}

impl<'phonemes> RuleState<'phonemes> {

    const fn new(inventory: &'phonemes Inventory, phonemes: Peekable<Iter<'phonemes, Rc<Phoneme>>>, word_index: usize) -> Self {
        Self {
            inventory,
            phonemes,
            word_index,
            splices: Vec::new()
        }
    }
}

impl RuleState<'_> {


    /// Peek at the next phoneme in the iterator. This function does not change the position.
    pub fn peek(&mut self) -> Option<&Rc<Phoneme>> {
        self.phonemes.peek().copied()
    }

    /// Returns true if the next phoneme matches the specified name, or is in a set with that name. The position is not changed. An error will be returned if the name is neither a valid phoneme nor a valid set.
    pub fn peek_is(&mut self, name: &'static str) -> Result<bool,ElbieError> {
        if let Some(phoneme) = self.phonemes.peek().copied() {
            self.phoneme_is(phoneme, name)
        } else {
            Ok(false)
        }

    }

    /// Check if a phoneme (probably returned by `peek`) matches the specified name or is in the specified set. The position is not changed. An error will be returned if the name is neither a valid phoneme nor a valid set.
    pub fn phoneme_is(&mut self, phoneme: &Rc<Phoneme>, name: &'static str) -> Result<bool,ElbieError> {
        if self.inventory.phonemes().contains_key(name) {
            Ok(phoneme.name == name)
        } else {
            Ok(self.inventory.get_set(name)?.contains(phoneme))
        }
    }

    /// Returns true if the iterator is at the beginning of the word (word_index is 0). The position is not changed.
    #[must_use]
    pub const fn peek_initial(&self) -> bool {
        self.word_index == 0
    }

    /// If the iterator is at the beginning of the word, returns Ok, otherwise returns a MatchFailed error.
    pub const fn initial(&mut self) -> Result<(),RuleStateError> {
        if self.peek_initial() {
            Ok(())
        } else {
            Err(RuleStateError::MatchFailed)
        }
    }

    /// If the iterator is not at the beginning of the word, returns Ok, otherwise returns a MatchFailed error.
    pub const fn not_initial(&mut self) -> Result<(),RuleStateError> {
        if self.peek_initial() {
            Err(RuleStateError::MatchFailed)
        } else {
            Ok(())
        }
    }

    /// Returns true if the iterator is at the end of the word (there are no tokens when peeking). The position is not changed.
    pub fn peek_final(&mut self) -> bool {
        self.phonemes.peek().is_none()
    }

    /// If the iterator is at the end of the word, returns Ok. Otherwise, returns a MatchFailed error.
    pub fn final_(&mut self) -> Result<(),RuleStateError> {
        if self.peek_final() {
            Ok(())
        } else {
            Err(RuleStateError::MatchFailed)
        }
    }

    /// If the iterator is not at the end of the word, returns Ok. Otherwise, returns a MatchFailed error.
    pub fn not_final(&mut self) -> Result<(),RuleStateError> {
        if self.peek_final() {
            Err(RuleStateError::MatchFailed)
        } else {
            Ok(())
        }
    }


    /// Matches any phoneme. If there is a phoneme in the iterator, it shifts the position forward and returns Ok. Otherwise, returns a MatchFailed error.
    pub fn any(&mut self) -> Result<(),RuleStateError> {
        if self.phonemes.next().is_some() {
            self.word_index += 1;
            Ok(())
        } else {
            Err(RuleStateError::MatchFailed)
        }
    }

    /// Peeks at the next phoneme, useing `peek_is`. If it matches, shifts the position forward by one and returns Ok. Otherwise, returns a MatchFailed error.
    pub fn is(&mut self, name: &'static str) -> Result<(),RuleStateError> {
        if self.peek_is(name)? {
            self.any()
        } else {
            Err(RuleStateError::MatchFailed)
        }

    }

    /// Processes the current pattern in the provided closure, to allow for code reuse. The closure should immediately return any errors from pattern function calls. The closure should return `Ok(true)` if it thinks there is a successful match, or `Ok(false)` if the pattern is still considered to have failed despite successful pattern calls. Returns Ok if the sequence matched. If the sequence does not match, a MatchFailed is returned.
    pub fn seq<Sequence: Fn(&mut Self) -> Result<bool,RuleStateError>>(&mut self, sequence: Sequence) -> Result<(),RuleStateError> {
        if sequence(self)? {
            Ok(())
        } else {
            Err(RuleStateError::MatchFailed)
        }
    }

    /// Uses `is` to match the current phoneme and returns true if it matches. If the match fails, it returns false and does not move the iterator.
    pub fn opt(&mut self, name: &'static str) -> Result<bool,ElbieError> {
        match self.is(name) {
            Ok(()) => Ok(true),
            Err(RuleStateError::MatchFailed) => Ok(false),
            Err(RuleStateError::Elbie(err)) => Err(err),
        }
    }

    /// Creates a new Pattern based off of the current state, and processes it using `seq`. If the match succeeds, the state is merged back into the main Pattern and true is returned. If the match fails, the state is not merged back in, but false is returned, indicating a successful but empty match.
    pub fn opt_seq<Sequence: Fn(&mut Self) -> Result<bool,RuleStateError>>(&mut self, sequence: Sequence) -> Result<bool,ElbieError> {
        let mut inner = Self {
            inventory: self.inventory,
            phonemes: self.phonemes.clone(),
            word_index: self.word_index,
            splices: Vec::new()
        };
        match inner.seq(sequence) {
            Ok(()) => {
                self.phonemes = inner.phonemes;
                self.word_index = inner.word_index;
                self.splices.extend(inner.splices);
                Ok(true)
            },
            Err(RuleStateError::MatchFailed) => {
                Ok(false)
            },
            Err(RuleStateError::Elbie(err)) => Err(err)
        }

    }


    fn get_replacement(&self, phonemes: &[&'static str]) -> Result<Vec<Rc<Phoneme>>,ElbieError> {
        phonemes.iter().map(|phoneme| {
            self.inventory.get_phoneme(phoneme).cloned()
        }).collect()

    }

    /// Adds a new splice at the current position that replaces a length of 0 and contains the specified phonemes. Will return an error if the phonemes do not exist in the inventory.
    pub fn ins(&mut self, phonemes: &[&'static str]) -> Result<(),ElbieError> {

        let replace = self.get_replacement(phonemes)?;

        self.splices.push(WordSplice {
            start_index: self.word_index,
            length: 0,
            replace,
        });

        Ok(())
    }

    /// Calls `is`, and if there is a match adds a new splice to replace the matching phoneme with the specified phonemes. Will return an error if the phonemes do not exist in the inventory.
    pub fn repl(&mut self, name: &'static str, phonemes: &[&'static str]) -> Result<(),RuleStateError> {
        let replace = self.get_replacement(phonemes)?;
        let start_index = self.word_index;
        self.is(name)?;

        self.splices.push(WordSplice {
            start_index,
            length: 1,
            replace,
        });

        Ok(())

    }

    /// Calls `opt`, and if there is a non-empty match adds a new splice to replace the matching phoneme with the specified phonemes. On success, returns true. If the match failed, no replacement is made and returns false. Will return an error if the phonemes do not exist in the inventory.
    pub fn opt_repl(&mut self, name: &'static str, phonemes: &[&'static str]) -> Result<bool,ElbieError> {
        let replace = self.get_replacement(phonemes)?;
        let start_index = self.word_index;
        let matched = self.opt(name)?;

        if matched {
            self.splices.push(WordSplice {
                start_index,
                length: 1,
                replace,
            });
        }

        Ok(matched)

    }

    /// Calls `seq` with the specified closure, and if it matches adds a new splice which replaces the match with the specified phonemes. Returns Ok if the sequence that matched. If the sequence does not match, no replacement is done and the function returns MatchFailed.
    pub fn repl_seq<Sequence: Fn(&mut Self) -> Result<bool,RuleStateError>>(&mut self, sequence: Sequence, phonemes: &[&'static str]) -> Result<(),RuleStateError> {
        let replace = self.get_replacement(phonemes)?;
        let start_index = self.word_index;
        self.seq(sequence)?;
        self.splices.push(WordSplice {
            start_index,
            length: self.word_index - start_index,
            replace,
        });
        Ok(())
    }

    /// Calls `opt_seq` with the specified closure, and if it returns a match adds a new splice which replaces the match with the specified phonemes. If the match is empty, no splice is added and false is returned.
    pub fn opt_repl_seq<Sequence: Fn(&mut Self) -> Result<bool,RuleStateError>>(&mut self, sequence: Sequence, phonemes: &[&'static str]) -> Result<bool,ElbieError> {
        let replace = self.get_replacement(phonemes)?;
        let start_index = self.word_index;
        let matched = self.opt_seq(sequence)?;

        if matched {
            self.splices.push(WordSplice {
                start_index,
                length: self.word_index - start_index,
                replace,
            });
        }
        Ok(matched)
    }

    /// Returns a MatchFailed error. When running a choice of options, you can use `rule.opt(..)? || rule.opt(..)? || rule.fail()?` in the sequence to automatically fail. While you could also return Ok(false), that will trigger clippy's `diverging_sub_expression` lint. So this is just a convenience function to make the expression look neater.
    pub const fn fail(&self) -> Result<bool,RuleStateError> {
        Err(RuleStateError::MatchFailed)
    }


}

type Sequence = Box<dyn Fn(&mut RuleState) -> Result<bool,RuleStateError>>;

pub struct Rule {
    name: &'static str,
    sequence: Sequence

}

impl Rule {

    #[must_use]
    fn new<Sequence: Fn(&mut RuleState) -> Result<bool,RuleStateError> + 'static>(name: &'static str, sequence: Sequence) -> Self {
        Self {
            name,
            sequence: Box::new(sequence)
        }
    }

    /**
    This isn't the same algorithm as a string replace, there are some differences.

    A string replace function generally replaces one at a time, matches can not overlap. `"sss".replace("ss","test")` results in `"tests", not "testtest" or something weird, even though the last two characters also match.

    However, since the rules for language change transform based on the environment around the replacement, I can't do that here. Say we have a sound change rule `CVhC` become `CesC`, and we apply it to the word /tuhtiht/. The result should be /testest/. However, with string replace rules, one would get /testiht/. The second syllable wouldn't match because it was already part of the previous match.

    To fix this, the matches are tested starting from each phoneme, on the original word, and the replacements are spliced in after all the matches have been tested.

    One complication added to this change is a possibility of this resulting in overlapping replacements if the user defining the rule is not careful. Overlapping replacements will be reported as an error rather than try to guess what the user really meant.
    */
    fn transform(&self, transformer: &Transformation, word: Word, trace: Option<&TransformationTraceCallback>) -> Result<Word,ElbieError> {

        let mut phonemes = word.phonemes().iter();
        let mut current_index = 0;
        let mut splices = Vec::new();

        loop {
            // clone the enumerator to store it's current position, so the normal iterator isn't incremented.
            let match_phonemes = phonemes.clone().peekable();
            let mut state = RuleState::new(&transformer.inventory, match_phonemes, current_index);
            match (self.sequence)(&mut state) {
                Ok(true) => {
                    splices.extend(state.splices);
                },
                Ok(false) |
                Err(RuleStateError::MatchFailed) => (),
                Err(RuleStateError::Elbie(err)) => return Err(err)
            }

            // iterate the enumerator, if it's none we're done, otherwise we keep trying to match with the next phoneme
            // NOTE: this would return None only if the iteration we just went through was also none. Which means we do an extra
            // loop at the end. But the instructions shouldn't match anything (although there's a small possibility they do if the match
            // was optional) and everything should sort of fall through without having done anything.
            if phonemes.next().is_none() {
                break;
            }
            current_index += 1;
        }

        // sort splices first, to make finding the overlaps a little easier. (We'll need them sorted anyway.)
        splices.sort_by_key(|s| s.start_index);

        // now find all overlaps.
        for window in splices.windows(2) {
            // Using if..let to avoid having to index the element. This should always be true.
            // It doesn't like `for [prev,next] in splices.windows(2)` because it doesn't handle [_] or [_,_,..]. Here's a case where a for loop style construct which filters by match pattern as well would be useful.
            if let [prev,next,..] = window {
                // end_index is not inclusive: 5 length 1, and 6 length 1 do not intersect.
                // Also, since it's sorted, I don't have to check if the prev start is greater than next start, because it won't be.
                if (prev.start_index == next.start_index) || (prev.start_index + prev.length) > next.start_index {
                    return Err(ElbieError::TransformationCreatedOverlappingReplacements(self.name))
                }

            }

        }

        // the splices are now sorted, and unique, so I should be able to iterate through the word again and copy things in somehow...
        let mut new_phonemes = Vec::new();
        let mut old_phonemes = word.phonemes().iter().enumerate().peekable();
        let mut transformed = false;
        for next_splice in splices {
            // push through all the phonemes before the next splice and just push them through.
            while let Some((_,phoneme)) = old_phonemes.next_if(|(i,_)| i < &next_splice.start_index) {
                new_phonemes.push(phoneme.clone());
            }
            // skip the phonemes covered by the splice
            for _ in 0..next_splice.length {
                _ = old_phonemes.next();
            }
            // and insert the phonemes to be replaced
            new_phonemes.extend(next_splice.replace);
            transformed = true;
        }
        // push the remaining phonemes, after the last splice, onto the new phonemes.
        new_phonemes.extend(old_phonemes.map(|(_,p)| p.clone()));

        let transformed_word = Word::from(new_phonemes);

        if let Some(trace) = trace {
            trace(if transformed {
                TransformationTraceMessage::MatchedRule(self.name, word, transformed_word.clone())
            } else {
                TransformationTraceMessage::UnmatchedRule(self.name)
            });
        }

        Ok(transformed_word)

    }

}


pub struct Transformation {
    inventory: Inventory,
    rules: Vec<Rule>,
    dont_validate: bool,
}

impl Transformation {

    #[must_use]
    pub fn from(source: &Language) -> Self {
        let inventory = Inventory::default();
        let rules = Vec::new();
        let mut result = Self {
            inventory,
            rules,
            dont_validate: false
        };
        result.add_language(source);
        result
    }

    pub const fn set_dont_validate(&mut self, value: bool) {
        self.dont_validate = value;
    }

    #[must_use]
    pub const fn dont_validate(&self) -> bool {
        self.dont_validate
    }

    pub fn add_language(&mut self, source: &Language) {
        _ = self.add_inventory(source.name(), source.inventory());
    }

    pub fn add_inventory(&mut self, name: &'static str, inventory: &Inventory) -> Result<(),ElbieError> {
        self.inventory.extend(inventory,name)
    }

    pub fn add_rule<Sequence: Fn(&mut RuleState) -> Result<bool,RuleStateError> + 'static>(&mut self, name: &'static str, sequence: Sequence) {
        self.rules.push(Rule::new(name, sequence));
    }


    /// Applies transformation rules in order, and returns the final word if successful.
    /// The word has not been validated for any specific language, so this should still be done before reporting the result to the user.
    pub(crate) fn transform(&self, word: &Word, trace: Option<&TransformationTraceCallback>) -> Result<Word,ElbieError> {
        if let Some(trace) = trace {
            trace(TransformationTraceMessage::StartTransformation(word.clone()))
        }
        let mut transformed = word.clone();
        for rule in &self.rules {
            transformed = rule.transform(self, transformed, trace)?
        }
        Ok(transformed)

    }

}
