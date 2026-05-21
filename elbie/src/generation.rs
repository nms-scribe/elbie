#![allow(dead_code)]
use crate::language::Language;
use crate::word::Word;
use rand::rngs::ThreadRng;
use rand::Rng as _;
use core::slice::Iter;
use std::rc::Rc;
use crate::phoneme::Phoneme;
use core::panic::Location;
use core::iter::Enumerate;
use crate::errors::ElbieError;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

// TODO: A rewrite of the generation/validation system. I'm not sure how to integrate, but this should look much more like the transformation thing, so much easier to do.

// TODO: Converting to the new system in a way that I don't need to mess up old programs.
// [X] Tag the git commit so we can go back to it. (v0.3.2)
// [ ] Language::new will create the pattern by declaring a simple pattern that is just an environment switch.
// [ ] Language::add_environment will add environments to the patternset that look very much like the old one, but it will keep the old stuff at first.
// [ ] duplicate word generation code in the new feature to use the patternset instead.
// [ ] convert goblin over to the new feature. It should just work if I've done it right.
// [ ] test generation and validation of goblin over and over to make sure that generated phonemes look the same in the new system.
// [ ] duplicate word validation code in the new feature to also use the patternset.
// [ ] Test that everything in goblin is working the same.
// [ ] Deprecate the old API, but do not delete. In the deprecation message, report that they can use the tag made above to get the old API, but that tag is unsupported so it's best if they convert to the new.
// [ ] Also probably deprecate the named environments in the new API. Those are only there to support the deprecated API, and I'm not sure they're that useful.
// [ ] Add in an API that lets you use the patternset system directly.
// [ ] Start converting goblin to making use of the new API, and possibly even some of it's new features.

// TODO: I think I can almost convert the old environments to the new now.
// - An Old Environment generates a phoneme, then iterates through the branches looking for one that matches the phoneme. Then it does a choice (tree) where each branch is basically another switch.
// TODO: Find a way to convert the current phonotactics system into these Patterns so I don't have to edit them on the old languages.
//       -- the phonotactics are currently supplied with 'add_environment', so I can hook into that. And add an 'add_patterns' to the Language and eventually deprecate 'add_environment'
//       -- one issue is that the input for the phonotactics are in the new function for Language. I'm not sure how to deprecate that without changing the name of the constructor.
//       -- simple answer: use it create the "default" pattern in the pattern-set.
// TODO: Then, switch to the patterns for generating words. Test with validation, then switch to validating.
// TODO: Eventually, this should go in three modules: patterns.rs, generation.rs, and validation.rs
// TODO: Also, the patterns should be on the Language.
// TODO: And switch from PatternError to ElbieError.

/* NOTE:

Probabilities in the patterns below are marked by u8 instead of f64. To check a probability, a random u8 is generated, and if the value is <= the probability, then it is true. This is slightly more efficient since I'm not bogged down by floating point precission issues, and I don't need to worry about someone adding in values higher than 1.0.

Weight in the patterns below are found in collections of choices, branches, etc. All weights in the patterns must add up to u8::MAX. When an item is chosen, a random number is generated and the items are evaluated in order, adding up their weights, until one's running weight is that value or higher.
*/

// TODO: Add these to ElbieError later.
enum PatternError {
    Elbie(ElbieError),
    PhonemeAfterTerminate,
    UnknownPattern(String)

}

trait PatternGenerate {

    fn extend_word(&self, language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError>;

}

#[derive(Clone)]
pub(crate) enum ValidationError {
    SeriesFailedToReachMinimumCount {
        source: Location<'static>,
        position: usize,
        minimum: usize,
        count: usize
    },
    NoBranchesMatched {
        source: Location<'static>,
        position: usize
    },
    InvalidPhoneme {
        source: Location<'static>,
        position: usize,
        expected: &'static str,
        found: Rc<Phoneme>
    },
    UnexpectedEnd {
        source: Location<'static>,
        position: usize,
        expected: &'static str
    },
    UnexpectedPhoneme {
        source: Location<'static>,
        position: usize,
        found: Rc<Phoneme>
    },
    UnexpectedPhonemeAfterPattern {
        position: usize,
        found: Rc<Phoneme>
    }
}

pub(crate) enum ValidationTraceStart {
    Sequence,
    Series,
    Option,
    Tree,
    // If the value is Some, then the switch called a named environment.
    Switch(Option<&'static str>),
    Environment,
    RuleReference(&'static str)
}

pub(crate) enum ValidationTraceEnd {
    Sequence,
    // number is the current count of the series.
    Series(usize),
    // bool indicates if the option has matched.
    Option(bool),
    // number indicates the current branch position
    Tree(usize),
    // If the value is Some, then the switch called a named environment.
    Switch(Option<&'static str>),
    // number indicates the current branch position, if None then this ended with the else or the initial phoneme.
    Environment(Option<usize>),
    RuleReference(&'static str),
    PhonemeSuccess(Rc<Phoneme>),
    PhonemeFail,
    Terminate,
}

pub(crate) enum ValidationTraceMessage {
    Start(Location<'static>,usize,ValidationTraceStart),
    Success(Location<'static>,usize,ValidationTraceEnd),
    Failure(ValidationTraceEnd,ValidationError)
}

pub(crate) type ValidationTraceCallback = dyn Fn(ValidationTraceMessage);

trait ValidationReport {

    fn report(&self, message: ValidationTraceMessage);

    fn start(&self, location: Location<'static>, position: usize, event: ValidationTraceStart) {
        self.report(ValidationTraceMessage::Start(location, position, event));
    }

    fn success(&self, location: Location<'static>, position: usize, event: ValidationTraceEnd) {
        self.report(ValidationTraceMessage::Success(location, position, event));
    }

    fn failure(&self, event: ValidationTraceEnd, error: ValidationError) {
        self.report(ValidationTraceMessage::Failure(event, error));
    }
}

impl ValidationReport for Option<&ValidationTraceCallback> {
    fn report(&self, message: ValidationTraceMessage) {
        if let Some(callback) = self {
            callback(message)
        }
    }
}

trait PatternValidate {

    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError>;

}

#[derive(Clone)]
// FUTURE: The Enumerate struct does have an unstable next_index function, which if I had that would mean I wouldn't need this struct. When that becomes stable I can rewrite this stuff.
struct EnumerateCount<Inner> {
    inner: Enumerate<Inner>,
    next_index: usize
}

impl<Inner: Iterator> EnumerateCount<Inner> {

    fn new(inner: Inner) -> Self {
        Self {
            inner: inner.enumerate(),
            next_index: 0
        }
    }

    const fn next_index(&self) -> usize {
        self.next_index
    }
}

impl<Iter> Iterator for EnumerateCount<Iter>
where
    Iter: Iterator,
{
    type Item = (usize, <Iter as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index,item)) = self.inner.next() {
            self.next_index = index + 1;
            Some((index,item))
        } else {
            None
        }
    }

}

#[derive(Debug)]
struct Sequence {
    patterns: Vec<Pattern>,
    defined_at: Location<'static>
}

impl PatternGenerate for Sequence {
    fn extend_word(&self,language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        for pattern in &self.patterns {
            pattern.extend_word(language, rules, rng, is_complete, result)?;
        }
        Ok(())
    }
}

impl PatternValidate for Sequence {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        trace.start(self.defined_at,word.next_index,ValidationTraceStart::Sequence);
        for pattern in &self.patterns {
            if let Err(error) = pattern.validate_word(language, rules, word, trace)? {
                trace.failure(ValidationTraceEnd::Sequence,error.clone());
                return Ok(Err(error))
            }
        }
        trace.success(self.defined_at,word.next_index,ValidationTraceEnd::Sequence);
        Ok(Ok(()))
    }
}

#[derive(Debug)]
struct Series {
    pattern: Pattern,
    probability: u8,
    minimum: usize,
    maximum: Option<usize>,
    defined_at: Location<'static>
}

impl PatternGenerate for Series {

    fn extend_word(&self,language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        for _ in 0..self.minimum {
            self.pattern.extend_word(language, rules, rng, is_complete, result)?;
        }
        let mut i = self.minimum;
        while (!*is_complete) && Pattern::is_probable(self.probability, rng) && self.maximum.is_none_or(|max| i < max) {
            self.pattern.extend_word(language, rules, rng, is_complete, result)?;
            i += 1;
        }
        Ok(())
    }

}

impl PatternValidate for Series {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        trace.start(self.defined_at,word.next_index,ValidationTraceStart::Series);
        let mut count = 0;
        // clone the word so that if we have a failure, but it's above the minimum required, we can still return the original.
        let mut working = word.clone();
        while self.pattern.validate_word(language, rules, &mut working, trace)?.is_ok() {
            count += 1;
            working = working.clone();
            if let Some(maximum) = self.maximum && count > maximum {
                // rather than returning an error, just stop validating. It's possible the remainder of the word is still valid in some other way.
                break;
            }
        };

        if count >= self.minimum {
            // reset to the last successfully matched working copy.
            *word = working;
            trace.success(self.defined_at,word.next_index,ValidationTraceEnd::Series(count));
            Ok(Ok(()))
        } else {
            let error = ValidationError::SeriesFailedToReachMinimumCount {
                source: self.defined_at,
                position: word.next_index,
                minimum: self.minimum,
                count,
            };
            trace.failure(ValidationTraceEnd::Series(count),error.clone());
            Ok(Err(error))
        }
    }
}

#[derive(Debug)]
struct Optional {
    pattern: Pattern,
    probability: u8,
    defined_at: Location<'static>
}

impl PatternGenerate for Optional {

    fn extend_word(&self,language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        if (!*is_complete) && Pattern::is_probable(self.probability, rng)  {
            self.pattern.extend_word(language, rules, rng, is_complete, result)
        } else {
            Ok(())
        }
    }

}

impl PatternValidate for Optional {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        trace.start(self.defined_at,word.next_index,ValidationTraceStart::Option);
        // clone the word so that if we have a failure, but it's above the minimum required, we can still return the original.
        let mut working = word.clone();
        if self.pattern.validate_word(language, rules, &mut working, trace)?.is_ok() {
            *word = working;
            trace.success(self.defined_at,word.next_index,ValidationTraceEnd::Option(true));
        } else {
            // failing to match an option isn't an error. So there really is no way for an option to fail.
            trace.success(self.defined_at,word.next_index,ValidationTraceEnd::Option(false));
        }

        Ok(Ok(()))
    }

}


#[derive(Debug)]
struct TreeBranch {
    head: Pattern,
    weight: u8,
    tail: Pattern,
    defined_at: Location<'static>
}


#[derive(Debug)]
struct Tree {
    branches: Vec<TreeBranch>,
    total_weight: u8,
    defined_at: Location<'static>
}

impl Tree {

    fn pick_branch<'branches>(&'branches self, rng: &mut ThreadRng) -> &'branches TreeBranch {
        let choice = rng.random_range(0..self.total_weight);
        let mut accumulated_weight = 0;
        for item in &self.branches {
            accumulated_weight += item.weight;
            if accumulated_weight >= choice {
                return item
            }
        }
        // this shouldn't happen at all unless somehow the branch weights and the total weight got messed up.
        panic!("random number generated was higher than all accumulated branch weights");
    }
}

impl PatternGenerate for Tree {

    fn extend_word(&self,language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        let branch = self.pick_branch(rng);
        branch.head.extend_word(language, rules, rng, is_complete, result)?;
        branch.tail.extend_word(language, rules, rng, is_complete, result)
    }

}

impl PatternValidate for Tree {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        trace.start(self.defined_at,word.next_index,ValidationTraceStart::Tree);
        for (branch_idx,branch) in self.branches.iter().enumerate() {
            let mut working = word.clone();
            if branch.head.validate_word(language, rules, &mut working, trace)?.is_ok() {
                *word = working;
                if let Err(error) = branch.tail.validate_word(language, rules, word, trace)? {
                    trace.failure(ValidationTraceEnd::Tree(branch_idx),error.clone());
                    return Ok(Err(error))
                }

                trace.success(self.defined_at,word.next_index,ValidationTraceEnd::Tree(branch_idx));
                return Ok(Ok(()))

            }
        }
        // none of the choices matched
        let error = ValidationError::NoBranchesMatched {
            source: self.defined_at,
            position: word.next_index,
        };
        trace.failure(ValidationTraceEnd::Tree(self.branches.len()),error.clone());
        Ok(Err(error))
    }
}


#[derive(Debug)]
struct PhonemeInSet {
    name: &'static str,
    avoid_duplicates: bool,
    defined_at: Location<'static>
}

impl PhonemeInSet {

    fn extend_with_phoneme(&self,language: &Language, rng: &mut ThreadRng, is_complete: bool, result: &mut Word) -> Result<Rc<Phoneme>,PatternError> {
        if is_complete {
            return Err(PatternError::PhonemeAfterTerminate)
        }
        let phoneme = if self.avoid_duplicates && let Some(phoneme) = result.last() {
            language.inventory().choose_except(self.name,&[phoneme],rng).map_err(PatternError::Elbie)?
        } else {
            language.inventory().choose(self.name,rng).map_err(PatternError::Elbie)?
        };

        result.push(phoneme.clone());
        Ok(phoneme)
    }

    fn validate_with_phoneme(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<Rc<Phoneme>,ValidationError>,PatternError> {
        if let Some((position,phoneme)) = word.next() {
            if language.inventory().phoneme_is(phoneme, self.name).map_err(PatternError::Elbie)? {
                trace.success(self.defined_at,position,ValidationTraceEnd::PhonemeSuccess(phoneme.clone()));
                Ok(Ok(phoneme.clone()))
            } else {
                let error = ValidationError::InvalidPhoneme {
                    source: self.defined_at,
                    position,
                    expected: self.name,
                    found: phoneme.clone(),
                };
                trace.failure(ValidationTraceEnd::PhonemeSuccess(phoneme.clone()),error.clone());
                Ok(Err(error))
            }
        } else {
            let error = ValidationError::UnexpectedEnd {
                source: self.defined_at,
                position: word.next_index,
                expected: self.name,
            };
            trace.failure(ValidationTraceEnd::PhonemeFail,error.clone());
            Ok(Err(error))
        }
    }


}

impl PatternGenerate for PhonemeInSet {
    fn extend_word(&self,language: &Language, _: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        _ = self.extend_with_phoneme(language, rng, *is_complete, result)?;
        Ok(())
    }
}

impl PatternValidate for PhonemeInSet {
    fn validate_word(&self, language: &Language, _: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        match self.validate_with_phoneme(language, word, trace)? {
            Ok(_) => Ok(Ok(())),
            Err(err) => Ok(Err(err)),
        }
    }
}



#[derive(Debug)]
struct SwitchBranch {
    condition_set: &'static str,
    body: Pattern,
    defined_at: Location<'static>
}

#[derive(Debug)]
struct Environment {
    branches: Vec<SwitchBranch>,
    else_: Pattern,
    defined_at: Location<'static>
}

impl Environment {
    fn extend_word(&self, phoneme: &Rc<Phoneme>, language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        for branch in &self.branches {
            if language.inventory().phoneme_is(&phoneme, branch.condition_set).map_err(PatternError::Elbie)? {
                return branch.body.extend_word(language, rules, rng, is_complete, result)
            }
        }
        self.else_.extend_word(language, rules, rng, is_complete, result)
    }

    fn validate_word(&self, phoneme: &Rc<Phoneme>, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        trace.start(self.defined_at,word.next_index,ValidationTraceStart::Environment);

        for (index,branch) in self.branches.iter().enumerate() {
            if language.inventory().phoneme_is(&phoneme, branch.condition_set).map_err(PatternError::Elbie)? {
                match branch.body.validate_word(language, rules, word, trace)? {
                    Ok(()) => {
                        trace.success(self.defined_at, word.next_index, ValidationTraceEnd::Environment(Some(index)));
                        return Ok(Ok(()))
                    },
                    Err(err) => {
                        trace.failure(ValidationTraceEnd::Environment(Some(index)),err.clone());
                        return Ok(Err(err))
                    },
                }
            }
        }

        match self.else_.validate_word(language, rules, word, trace)? {
            Ok(()) => {
                trace.success(self.defined_at, word.next_index, ValidationTraceEnd::Environment(None));
                Ok(Ok(()))
            },
            Err(err) => {
                trace.failure(ValidationTraceEnd::Environment(None),err.clone());
                Ok(Err(err))
            },
        }
    }
}

#[derive(Debug)]
enum SwitchEnvironment {
    Environment(Environment),
    Named(&'static str)
}

#[derive(Debug)]
struct Switch {
    initial: PhonemeInSet,
    environment: SwitchEnvironment,
    defined_at: Location<'static>
}

impl PatternGenerate for Switch {
    fn extend_word(&self,language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        let phoneme = self.initial.extend_with_phoneme(language, rng, *is_complete, result)?;
        let environment = match &self.environment {
            SwitchEnvironment::Environment(environment) => environment,
            SwitchEnvironment::Named(name) => rules.get_environment(name)?,
        };
        environment.extend_word(&phoneme, language, rules, rng, is_complete, result)
    }
}

impl PatternValidate for Switch {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        let (environment,name) = match &self.environment {
            SwitchEnvironment::Environment(environment) => (environment,None),
            SwitchEnvironment::Named(name) => (rules.get_environment(name)?,Some(*name)),
        };
        trace.start(self.defined_at,word.next_index,ValidationTraceStart::Switch(name));
        match self.initial.validate_with_phoneme(language, word, trace)? {
            Ok(phoneme) => {

                match environment.validate_word(&phoneme, language, rules, word, trace)? {
                    Ok(()) => {
                        trace.success(self.defined_at, word.next_index, ValidationTraceEnd::Switch(name));
                        Ok(Ok(()))
                    },
                    Err(err) => {
                        trace.failure(ValidationTraceEnd::Switch(name), err.clone());
                        Ok(Err(err))
                    }
                }
            },
            Err(err) => {
                trace.failure(ValidationTraceEnd::Switch(name), err.clone());
                Ok(Err(err))
            },
        }
    }
}

#[derive(Debug)]
struct TerminateWord {
    defined_at: Location<'static>
}

impl PatternGenerate for TerminateWord {
    fn extend_word(&self, _: &Language, _: &PatternSet, _: &mut ThreadRng, is_complete: &mut bool, _: &mut Word) -> Result<(),PatternError> {
        *is_complete = true;
        Ok(())
    }
}

impl PatternValidate for TerminateWord {
    fn validate_word(&self, _: &Language, _: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        if let Some((index,phoneme)) = word.next() {
            let error = ValidationError::UnexpectedPhoneme {
                source: self.defined_at,
                position: index,
                found: phoneme.clone(),
            };
            trace.failure(ValidationTraceEnd::Terminate,error.clone());
            Ok(Err(error))
        } else {
            trace.success(self.defined_at,word.next_index,ValidationTraceEnd::Terminate);
            Ok(Ok(()))
        }
    }
}

struct RuleReference {
    name: &'static str,
    defined_at: Location<'static>
}

impl PatternGenerate for RuleReference {
    fn extend_word(&self, language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        let pattern = rules.get(self.name)?;
        pattern.extend_word(language, rules, rng, is_complete, result)
    }
}

impl PatternValidate for RuleReference {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        trace.start(self.defined_at, word.next_index, ValidationTraceStart::RuleReference(self.name));
        let pattern = rules.get(self.name)?;
        match pattern.validate_word(language, rules, word, trace)? {
            Ok(()) => {
                trace.success(self.defined_at, word.next_index, ValidationTraceEnd::RuleReference(self.name));
                Ok(Ok(()))
            },
            Err(err) => {
                trace.failure(ValidationTraceEnd::RuleReference(self.name), err.clone());
                Ok(Err(err))
            }
        }

    }
}

#[derive(Debug)]
enum Pattern {
    Sequence(Sequence),
    Series(Box<Series>),
    Option(Box<Optional>),
    Tree(Tree),
    Switch(Box<Switch>),
    Set(PhonemeInSet),
    // This can be used to force completion in certain situations, such as not allowing a series to continue, or disallowing an option.
    // If used in a pattern before a non-optional pattern with phonemes, it will fail.
    Terminate(TerminateWord)
}

impl Pattern {

    fn is_probable(probability: u8, rng: &mut ThreadRng) -> bool {
        rng.random::<u8>() <= probability
    }

}

impl PatternGenerate for Pattern {

    fn extend_word(&self, language: &Language, rules: &PatternSet, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),PatternError> {
        match self {
            Self::Sequence(sequence) => sequence.extend_word(language, rules, rng, is_complete, result),
            Self::Series(series) => series.extend_word(language, rules, rng,is_complete,result),
            Self::Option(optional) => optional.extend_word(language, rules, rng, is_complete, result),
            Self::Tree(tree) => tree.extend_word(language, rules, rng, is_complete, result),
            Self::Switch(switch) => switch.extend_word(language, rules, rng, is_complete, result),
            Self::Set(set) => set.extend_word(language, rules, rng, is_complete, result),
            Self::Terminate(terminate) => terminate.extend_word(language, rules, rng, is_complete, result),
        }
    }


}

impl PatternValidate for Pattern {
    fn validate_word(&self, language: &Language, rules: &PatternSet, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
        match self {
            Self::Sequence(sequence) => sequence.validate_word(language, rules, word, trace),
            Self::Series(series) => series.validate_word(language,rules, word, trace),
            Self::Option(optional) => optional.validate_word(language,rules, word, trace),
            Self::Tree(tree) => tree.validate_word(language, rules, word, trace),
            Self::Switch(switch) => switch.validate_word(language, rules, word, trace),
            Self::Set(set) => set.validate_word(language, rules, word, trace),
            Self::Terminate(terminate) => terminate.validate_word(language, rules, word, trace),
        }
    }

}

// TODO: The rules PatternSet should be part of the Language itself, so I then shouldn't have to pass it along.
fn generate_word(language: &Language, rules: &PatternSet, pattern: &Pattern, rng: &mut ThreadRng) -> Result<Word,PatternError> {
    let mut result = Word::new(&[]);

    pattern.extend_word(language, rules, rng, &mut false, &mut result)?;
    Ok(result)

}

fn validate_word(language: &Language, rules: &PatternSet, pattern: &Pattern, word: &Word, trace: Option<&ValidationTraceCallback>) -> Result<Result<(),ValidationError>,PatternError> {
    let mut word = EnumerateCount::new(word.phonemes().iter());
    if let Err(error) = pattern.validate_word(language,rules,&mut word, trace)? {
        Ok(Err(error))
    } else if let Some((position,phoneme)) = word.next() {
        Ok(Err(ValidationError::UnexpectedPhonemeAfterPattern {
            position,
            found: phoneme.clone(),
        }))
    } else {
        Ok(Ok(()))
    }
}

pub(crate) struct PatternBuilder {
    patterns: Vec<Pattern>
}

impl PatternBuilder {

    const fn new() -> Self {
        Self {
            patterns: Vec::new()
        }
    }

    fn flatten(mut self, defined_at: Location<'static>) -> Pattern {
        let len = self.patterns.len();
        if len == 0 {
            panic!("Generation pattern is empty.")
        } else if len == 1 {
            self.patterns.remove(0)
        } else {
            Pattern::Sequence(Sequence {
                patterns: self.patterns,
                defined_at
            })
        }
    }

    pub(crate) fn seq<PatternCallback: Fn(&mut Self)>(&mut self, callback: PatternCallback) {
        let mut patterns = Self::new();
        callback(&mut patterns);
        self.patterns.append(&mut patterns.patterns);
    }

    #[track_caller]
    fn series<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize, maximum: Option<usize>) {
        let defined_at = *Location::caller();
        if maximum.is_some_and(|max| max < minimum) {
            panic!("Maximum length of series is less than minimum.")
        }
        let mut pattern = Self::new();
        callback(&mut pattern);
        let pattern = pattern.flatten(defined_at);
        self.patterns.push(Pattern::Series(Box::new(Series {
            pattern,
            probability,
            minimum,
            maximum,
            defined_at
        })));
    }

    #[track_caller]
    pub(crate) fn ser_min_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.series(probability, callback, minimum, Some(maximum));
    }

    #[track_caller]
    pub(crate) fn ser_min<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize) {
        self.series(probability, callback, minimum, None);
    }

    #[track_caller]
    pub(crate) fn ser_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, maximum: usize) {
        self.series(probability, callback, 0, Some(maximum));
    }

    #[track_caller]
    pub(crate) fn ser<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback) {
        self.series(probability, callback, 0, None);
    }

    #[track_caller]
    pub(crate) fn opt<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback) {
        let defined_at = *Location::caller();
        let mut pattern = Self::new();
        callback(&mut pattern);
        let pattern = pattern.flatten(defined_at);
        self.patterns.push(Pattern::Option(Box::new(Optional {
            pattern,
            probability,
            defined_at
        })));
    }

    #[track_caller]
    pub(crate) fn tree<BranchCallback: Fn(&mut TreeBuilder)>(&mut self, callback: BranchCallback) {
        let defined_at = *Location::caller();
        let mut builder = TreeBuilder::new();
        callback(&mut builder);
        if builder.branches.is_empty() {
            panic!("Branches are empty.")
        }
        self.patterns.push(Pattern::Tree(Tree {
            branches: builder.branches,
            total_weight: builder.total_weight,
            defined_at
        }));
    }

    fn switch_opt<BranchCallback: Fn(&mut EnvironmentBuilder)>(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, callback: BranchCallback) {
        let environment = SwitchEnvironment::Environment(EnvironmentBuilder::build(defined_at, callback));

        self.patterns.push(Pattern::Switch(Box::new(Switch {
            initial: PhonemeInSet {
                name: initial_phoneme,
                avoid_duplicates,
                defined_at
            },
            environment,
            defined_at,
        })));
    }

    #[track_caller]
    pub(crate) fn switch<BranchCallback: Fn(&mut EnvironmentBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        let defined_at = *Location::caller();
        self.switch_opt(defined_at, initial_phoneme, false, callback);
    }

    #[track_caller]
    pub(crate) fn switch_nodup<BranchCallback: Fn(&mut EnvironmentBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        let defined_at = *Location::caller();
        self.switch_opt(defined_at, initial_phoneme, true, callback);
    }

    fn switch_env_opt(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, environment: &'static str) {
        let environment = SwitchEnvironment::Named(environment);

        self.patterns.push(Pattern::Switch(Box::new(Switch {
            initial: PhonemeInSet {
                name: initial_phoneme,
                avoid_duplicates,
                defined_at
            },
            environment,
            defined_at,
        })));
    }

    #[track_caller]
    pub(crate) fn switch_env(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        let defined_at = *Location::caller();
        self.switch_env_opt(defined_at, initial_phoneme, false, environment);
    }

    #[track_caller]
    pub(crate) fn switch_env_nodup(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        let defined_at = *Location::caller();
        self.switch_env_opt(defined_at, initial_phoneme, true, environment);
    }

    #[track_caller]
    pub(crate) fn set(&mut self, name: &'static str) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::Set(PhonemeInSet {
            name,
            avoid_duplicates: false,
            defined_at,
        }));
    }

    #[track_caller]
    pub(crate) fn set_nodup(&mut self, name: &'static str) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::Set(PhonemeInSet {
            name,
            avoid_duplicates: true,
            defined_at,
        }));
    }

    #[track_caller]
    pub(crate) fn done(&mut self) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::Terminate(TerminateWord {
            defined_at
        }));
    }

}

pub(crate) struct TreeBuilder {
    total_weight: u8,
    branches: Vec<TreeBranch>
}

impl TreeBuilder {

    const fn new() -> Self {
        Self {
            total_weight: 0,
            branches: Vec::new()
        }
    }

    #[track_caller]
    fn add<HeadCallback: Fn(&mut PatternBuilder), TailCallback: Fn(&mut PatternBuilder)>(&mut self, weight: u8, head_cb: HeadCallback, tail_cb: TailCallback) {
        let defined_at = *Location::caller();
        self.total_weight = if let Some(total_weight) = self.total_weight.checked_add(weight) {
            total_weight
        } else {
            panic!("Branch choice weight overflows")
        };

        let mut head = PatternBuilder::new();
        head_cb(&mut head);
        let mut tail = PatternBuilder::new();
        tail_cb(&mut tail);
        self.branches.push(TreeBranch {
            head: head.flatten(defined_at),
            weight,
            tail: tail.flatten(defined_at),
            defined_at
        })
    }
}


pub(crate) struct EnvironmentBuilder {
    branches: Vec<SwitchBranch>,
    else_: Option<Pattern>
}

impl EnvironmentBuilder {

    fn build<BranchCallback: Fn(&mut EnvironmentBuilder)>(defined_at: Location<'static>, callback: BranchCallback) -> Environment {
        let mut builder = Self {
            branches: Vec::new(),
            else_: None
        };
        callback(&mut builder);

        Environment {
            branches: builder.branches,
            else_: builder.else_.expect("The environment at least needs an else."),
            defined_at
        }

    }

    #[track_caller]
    fn add<Callback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, body_cb: Callback) {
        let defined_at = *Location::caller();

        let mut body = PatternBuilder::new();
        body_cb(&mut body);
        self.branches.push(SwitchBranch {
            condition_set,
            body: body.flatten(defined_at),
            defined_at,
        })
    }

    #[track_caller]
    fn else_<Callback: Fn(&mut PatternBuilder)>(&mut self, body_cb: Callback) {
        let defined_at = *Location::caller();

        let mut body = PatternBuilder::new();
        body_cb(&mut body);
        self.else_ = Some(body.flatten(defined_at))
    }

}

#[derive(Debug)]
pub(crate) struct PatternSet {
    patterns: HashMap<String,Pattern>,
    environments: HashMap<String,Environment>,
    initial: Pattern
}

impl PatternSet {

    #[track_caller]
    pub(crate) fn new<Callback: Fn(&mut PatternBuilder)>(callback: Callback) -> Self {
        let mut builder = PatternBuilder::new();
        callback(&mut builder);
        let initial = builder.flatten(*Location::caller());
        Self {
            patterns: HashMap::new(),
            environments: HashMap::new(),
            initial
        }

    }

    #[track_caller]
    fn add<Callback: Fn(&mut PatternBuilder)>(&mut self, name: &'static str, callback: Callback) {
        let mut builder = PatternBuilder::new();
        callback(&mut builder);
        let pattern = builder.flatten(*Location::caller());
        match self.patterns.entry(name.to_owned()) {
            Entry::Occupied(_) => panic!("A pattern named '{name}' already exists."),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(pattern),
        }
    }

    fn get(&self, name: &str) -> Result<&Pattern,PatternError> {
        if let Some(pattern) = self.patterns.get(name) {
            Ok(pattern)
        } else {
            Err(PatternError::UnknownPattern(name.to_owned()))
        }
    }

    #[track_caller]
    fn add_environment<Callback: Fn(&mut EnvironmentBuilder)>(&mut self, name: &'static str, callback: Callback) {
        let environment = EnvironmentBuilder::build(*Location::caller(), callback);
        match self.environments.entry(name.to_owned()) {
            Entry::Occupied(_) => panic!("An environment named '{name}' already exists."),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(environment),
        }
    }

    fn get_environment(&self, name: &'static str) -> Result<&Environment,PatternError> {
        if let Some(pattern) = self.environments.get(name) {
            Ok(pattern)
        } else {
            Err(PatternError::Elbie(ElbieError::UnknownEnvironment(name)))
        }
    }
}
