use std::iter::Peekable;
use std::rc::Rc;
use std::slice::Iter;

use crate::errors::LanguageError;
use crate::language::Language;
use crate::phoneme::Inventory;
use crate::phoneme::Phoneme;
use crate::word::Word;



struct WordSplice {
    start_index: usize,
    length: usize,
    replace: Vec<Rc<Phoneme>>
}

#[derive(Clone)]
enum PatternEntity {
    /// Match is successful if the current phoneme has this name.
    Phoneme(&'static str),
    /// Match is successful if the current phoneme is in the specified set from the specified namespace
    Set(&'static str),
    /// Match is successful if the contents of the sequence match.
    Sequence(Vec<Pattern>)
}

impl PatternEntity {

    fn match_(&self, transformer: &Transformer, phonemes: &mut Peekable<Iter<'_, Rc<Phoneme>>>, trace: bool) -> Result<Option<usize>,LanguageError> {
        match self {
            Self::Phoneme(name) => if phonemes.next_if(|phoneme| phoneme.name == *name).is_some() {
                Ok(Some(1))
            } else {
                Ok(None)
            },
            Self::Set(name) => if let Some(phoneme) = phonemes.peek() && transformer.inventory.phoneme_is(phoneme, name)? {
                // iterate the peek. I can't use next_if for sets because phoneme_is returns a result, not a bool.
                _ = phonemes.next();
                Ok(Some(1))
            } else {
                Ok(None)
            },
            Self::Sequence(patterns) => Pattern::match_(patterns, transformer, phonemes, trace)
        }

    }

}

#[derive(Clone)]
pub struct Pattern {
    /// If true, the match can be successful even if the pattern does not match.
    optional: bool,
    /// If true, the match is repeated after the first successful attempt, and the word position will be incremented for each match, but the match will always be successful no matter how many repetitions are found.
    /// If optional is false, then at least the first one is required, the remaining are optional.
    repeatable: bool,
    /// Match is successful if the current position in the word matches this entity.
    entity: PatternEntity
}

impl Pattern {

    #[must_use]
    pub const fn phoneme(name: &'static str) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: PatternEntity::Phoneme(name)
        }
    }


    pub const fn set(entity: &'static str) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: PatternEntity::Set(entity)
        }
    }

    #[must_use]
    pub const fn sequence(patterns: Vec<Self>) -> Self {
        Self {
            optional: false,
            repeatable: false,
            entity: PatternEntity::Sequence(patterns)
        }
    }

    #[must_use]
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

    #[must_use]
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

    fn match_(patterns: &[Self], transformer: &Transformer, phonemes: &mut Peekable<Iter<'_, Rc<Phoneme>>>, trace: bool) -> Result<Option<usize>,LanguageError> {
        let mut length = 0;
        for pattern in patterns {
            length += if let Some(mut match_length) = pattern.entity.match_(transformer, phonemes, trace)? {
                if pattern.repeatable {
                    while let Some(next_length) = pattern.entity.match_(transformer, phonemes, trace)? {
                        match_length += next_length;
                    }
                }
                match_length
            } else if pattern.optional {
                0
            } else {
                return Ok(None)
            }

        }
        Ok(Some(length))

    }
}


pub enum Instruction {
    /// Attempts to match the current position of the word, incrementing through the word if successful, otherwise failing the match.
    Match(Vec<Pattern>),
    /// Attempts the match, if successful, replaces the matched content with the specified replacement and still increments through the word
    Replace(Vec<Pattern>,Vec<&'static str>)
}


pub struct Rule {
    name: &'static str,
    /// Rule only matches if the pattern starts at the beginning of the word
    initial: bool,
    /// Rule only matches if the pattern ends at the end of the word
    final_: bool,
    instructions: Vec<Instruction>
}

impl Rule {

    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            initial: false,
            final_: false,
            instructions: Vec::new()
        }
    }

    #[must_use]
    pub fn initial(self) -> Self {
        Self {
            name: self.name,
            initial: true,
            final_: self.final_,
            instructions: self.instructions,
        }
    }

    #[must_use]
    pub fn final_(self) -> Self {
        Self {
            name: self.name,
            initial: self.initial,
            final_: true,
            instructions: self.instructions,
        }
    }

    #[must_use]
    pub fn match_(self, patterns: &[Pattern]) -> Self {
        let Self {
            name,
            initial,
            final_,
            mut instructions,
        } = self;
        instructions.push(Instruction::Match(patterns.to_vec()));
        Self {
            name,
            initial,
            final_,
            instructions,
        }
    }

    #[must_use]
    pub fn replace(self, patterns: &[Pattern], phonemes: &[&'static str]) -> Self {
        let Self {
            name,
            initial,
            final_,
            mut instructions,
        } = self;
        instructions.push(Instruction::Replace(patterns.to_vec(),phonemes.to_vec()));
        Self {
            name,
            initial,
            final_,
            instructions,
        }
    }


    fn match_instructions(&self, transformer: &Transformer, mut start_index: usize, phonemes: &mut Peekable<Iter<'_, Rc<Phoneme>>>, trace: bool) -> Result<Vec<WordSplice>,LanguageError> {
        let mut splices = Vec::new();
        for instruction in &self.instructions {
            match instruction {
                Instruction::Match(pattern) => if let Some(length) = Pattern::match_(pattern,transformer,phonemes,trace)? {
                    start_index += length;
                } else {
                    // the instructions did not match, so the whole thing doesn't match. return an empty list of splices.
                    return Ok(Vec::new())
                },
                Instruction::Replace(pattern, replacement) => if let Some(length) = Pattern::match_(pattern,transformer,phonemes,trace)? {
                    start_index += length;
                    let replace = replacement.iter().map(|name| {
                        transformer.inventory.get_phoneme(name).cloned()
                    }).collect::<Result<_,_>>()?;
                    splices.push(WordSplice {
                        start_index,
                        length,
                        replace
                    });
                } else {
                    // the instructions did not match, so the whole thing doesn't match.
                    return Ok(Vec::new())
                },
            }
        }
        Ok(splices)
    }

    /**
    This isn't the same algorithm as a string replace, there are some differences.

    A string replace function generally replaces one at a time, matches can not overlap. `"sss".replace("ss","test")` results in `"tests", not "testtest" or something weird, even though the last two characters also match.

    However, since the rules for language change transform based on the environment around, I can't do that here. Say we have a sound change rule `CVhC` become `CesC`, and we apply it to the word /tuhtiht/. The result should be /testest/. However, with string replace rules, one would get /testiht/. The second syllable wouldn't match because it was already part of the previous match. This is because only part of the match is replaced.

    To fix this, the matches are tested starting from each phoneme, on the original word, and the replacements are spliced in after all the matches have been tested.

    There is a possibility of this resulting in overlapping replacements, which will be reported as an error. In theory, I could require matches to start after the replacement, but this can get complicated if there's more than one replacement section in a rule.
    */
    fn transform(&self, transformer: &Transformer, word: Word, trace: bool) -> Result<Word,LanguageError> {

        let mut phonemes = word.phonemes().iter();
        let mut current_index = 0;
        let mut splices = Vec::new();

        loop {
            // copy the enumerator to store it's current position
            let mut match_phonemes = phonemes.clone().peekable();
            let match_splices = self.match_instructions(transformer,current_index,&mut match_phonemes,trace)?;
            if self.final_ {
                // if the match enumerator has more, then it wasn't at the final, so this isn't a match if we're expecting a final.
                if match_phonemes.next().is_some() {
                    continue;
                }
            }
            splices.extend(match_splices);
            if self.initial {
                // it's only possible to match on the first one, so we can just break.
                break;
            }
            // iterate the enumerator, if it's none we're done, otherwise we keep trying to match with the next character
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
            // It doesn't like `for [prev,next] in splices.windows(2)` because it doesn't handle [_] or [_,_,..]
            if let [prev,next,..] = window {
                // end_index is not inclusive: 5 length 1, and 6 length 1 do not intersect.
                // Also, since it's sorted, I don't have to check if the prev start is greater than next start, because it won't be.
                if (prev.start_index == next.start_index) || (prev.start_index + prev.length) > next.start_index {
                    return Err(LanguageError::TransformationCreatedOverlappingReplacements(self.name))
                }

            }

        }

        // the splices are now sorted, and unique, so I should be able to iterate through the word again and copy things in somehow...
        let mut new_phonemes = Vec::new();
        let mut old_phonemes = word.into_phonemes().into_iter().enumerate().peekable();
        for next_splice in splices {
            // push through all the phonemes before the next splice and just push them through.
            while let Some((_,phoneme)) = old_phonemes.next_if(|(i,_)| i < &next_splice.start_index) {
                new_phonemes.push(phoneme);
            }
            // skip the phonemes covered by the splice
            for _ in 0..next_splice.length {
                _ = old_phonemes.next();
            }
            // and insert the phonemes to be replaced
            new_phonemes.extend(next_splice.replace);
        }
        // push the remaining phonemes, after the last splice, onto the new phonemes.
        new_phonemes.extend(old_phonemes.map(|(_,p)| p));

        Ok(Word::from(new_phonemes))

    }

}

/* TODO: Macro for building rules and sequences

rule!('^'? instruction,+ '^'?)
instruction = match ('=>' replacement)
match = (phoneme | set | sequence) ('+' | '*' | '?')?
phoneme = '/' ident '/'
set = '{' ident '}'
sequence = '(' (phoneme | set | sequence),* ')'
replacement = '/' ident '/',*


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


pub struct Transformer {
    inventory: Inventory,
    rules: Vec<Rule>
}

impl Transformer {

    #[must_use]
    pub fn from<const ORTHOGRAPHIES: usize>(source: &Language<ORTHOGRAPHIES>) -> Self {
        let inventory = Inventory::default();
        let rules = Vec::new();
        let mut result = Self {
            inventory,
            rules
        };
        result.add_language(source);
        result
    }

    pub fn add_language<const ORTHOGRAPHIES: usize>(&mut self, source: &Language<ORTHOGRAPHIES>) {
        _ = self.add_inventory(source.name(), source.inventory());
    }

    pub fn add_inventory(&mut self, name: &'static str, inventory: &Inventory) -> Result<(),LanguageError> {
        self.inventory.extend(inventory,name)
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Applies transformation rules in order, and returns the final word if successful.
    /// The word has not been validated for any specific language, so this should still be done before reporting the result to the user.
    // TODO: The 'trace' bool should indicate if messages should be reported, similar to validation tracing.
    pub fn transform(&self, word: Word, trace: bool) -> Result<Word,LanguageError> {
        let mut transformed = word;
        for rule in &self.rules {
            transformed = rule.transform(self, transformed, trace)?
        }
        Ok(transformed)


    }


}
