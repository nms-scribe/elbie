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
use core::fmt::Display;
use std::fmt;
use crate::weighted_vec::WeightedVec;

// TODO: A rewrite of the generation/validation system. I'm not sure how to integrate, but this should look much more like the transformation thing, so much easier to do.

// TODO: I think I do need a "choice" instead of the tree, at least for the old environment conversion.

// TODO: Converting to the new system in a way that I don't need to mess up old programs.
// [X] Tag the git commit so we can go back to it. (v0.3.2)
// [X] Language::new will create the pattern by declaring a simple pattern that is just an environment switch.
// [X] Language::add_environment will add environments to the patternset that look very much like the old one, but it will keep the old stuff at first.
// [X] change generation code to use new patterns instead.
// [X] test generation and validation of goblin over and over to make sure that generated phonemes look the same in the new system.
// [X] change validation code to work with the new patterns. -- This is more difficult since I've change the explain and validation API to fit the new structure.
// [X] Test that everything in goblin is working the same.
// [X] Deprecate the old API, but do not delete. In the deprecation message, report that they can use the tag made above to get the old API, but that tag is unsupported so it's best if they convert to the new.
// [X] Also probably deprecate the named environments in the new API. Those are only there to support the deprecated API, and I'm not sure they're that useful.
// [X] Add in an API that lets you use the patternset system directly.
// [ ] Start converting goblin to making use of the new API, and possibly even some of it's new features.
// [ ] Separate this into three modules: patterns.rs, generation.rs, and validation.rs
// [ ] Time to set up rustfmt rules so that I can get this thing easier to contribute to.

// TODO: Get rid of panics and replace with ElbieErrors.

/* NOTE:

Probabilities in the patterns below are marked by u8 instead of f64. To check a probability, a random u8 is generated, and if the value is <= the probability, then it is true. This is slightly more efficient since I'm not bogged down by floating point precission issues, and I don't need to worry about someone adding in values higher than 1.0.

Weight in the patterns below are found in collections of choices, branches, etc. All weights in the patterns must add up to u8::MAX. When an item is chosen, a random number is generated and the items are evaluated in order, adding up their weights, until one's running weight is that value or higher.
*/


trait GenerateWord {

    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError>;

}

#[derive(Clone)]
pub(crate) enum ValidationFailure {
    InnerSequencePatternFailed {
        index: usize
    },
    SeriesFailedToReachMinimumCount {
        minimum: usize,
        count: usize
    },
    NoChoiceBranchesMatched,
    BranchTailPatternFailed {
        index: usize
    },
    NoTreeBranchesMatched,
    CaseBranchBodyFailed {
        index: usize
    },
    NoCaseBranchesMatched,
    CaseEnvironmentFailed,
    CaseConditionFailed,
    ReferencedRuleFailed {
        name: &'static str
    },
    InvalidPhoneme {
        expected: &'static str,
        found: Rc<Phoneme>
    },
    UnexpectedEnd {
        expected: &'static str
    },
    UnexpectedPhoneme {
        found: Rc<Phoneme>
    },
    UnexpectedPhonemeAfterPattern {
        found: Rc<Phoneme>
    },
    InitialPatternFailed
}

impl Display for ValidationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InnerSequencePatternFailed { index } => write!(f,"inner pattern {index} failed"),
            Self::SeriesFailedToReachMinimumCount { minimum, count } => write!(f,"Series incomplete, only found {count} of {minimum}."),
            Self::InvalidPhoneme { expected, found } => write!(f,"Expected phoneme in set '{expected}', found {found}."),
            Self::UnexpectedEnd { expected } => write!(f,"Expected phoneme in set '{expected}', found end of word."),
            Self::UnexpectedPhoneme { found } => write!(f,"Expected end of word, found {found}."),
            Self::UnexpectedPhonemeAfterPattern { found } => write!(f,"Found phoneme {found} after pattern was complete."),
            Self::NoChoiceBranchesMatched => write!(f,"No branches in choice matched."),
            Self::BranchTailPatternFailed { index } => write!(f,"Tail pattern in branch {index} failed."),
            Self::NoTreeBranchesMatched => write!(f,"No branches in tree matched."),
            Self::CaseBranchBodyFailed { index } => write!(f,"Case body pattern {index} failed."),
            Self::NoCaseBranchesMatched => write!(f,"No branches in case matched."),
            Self::CaseEnvironmentFailed => write!(f,"Case environment failed."),
            Self::CaseConditionFailed => write!(f,"Phoneme did not match initial set for case."),
            Self::ReferencedRuleFailed { name } => write!(f,"Rule '{name}' failed."),
            Self::InitialPatternFailed => write!(f,"Initial pattern failed."),
        }
    }
}

#[derive(Clone)]
pub(crate) enum ValidationTraceStart {
    Sequence,
    Series,
    Option,
    Choice,
    Tree,
    // If the value is Some, then the switch called a named environment.
    Case(Option<&'static str>),
    CaseEnvironment,
    RuleReference(&'static str)
}

impl Display for ValidationTraceStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sequence => write!(f,"start sequence"),
            Self::Series => write!(f,"start series"),
            Self::Option => write!(f,"start option"),
            Self::Choice => write!(f,"start choice"),
            Self::Tree => write!(f,"start tree"),
            Self::Case(Some(case)) => write!(f,"start case for environment '{case}'"),
            Self::Case(None) => write!(f,"start case"),
            Self::CaseEnvironment => write!(f,"start case environment"),
            Self::RuleReference(name) => write!(f,"start rule reference '{name}'"),
        }
    }
}

#[derive(Clone)]
pub(crate) enum ValidationTraceEnd {
    Sequence,
    // number is the current count of the series.
    Series(usize),
    // bool indicates if the option has matched.
    Option(bool),
    // number indicates the current branch position
    Choice(usize),
    // number indicates the current branch position
    Tree(usize),
    // If the value is Some, then the switch called a named environment.
    Case(Option<&'static str>),
    // number indicates the current branch position, if None then this ended with the else or the initial phoneme.
    CaseEnvironment(Option<usize>),
    RuleReference(&'static str),
    PhonemeFound(Rc<Phoneme>),
    PhonemeNotFound,
    Terminate,
    Word
}

impl Display for ValidationTraceEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sequence => write!(f,"end sequence"),
            Self::Series(count) => write!(f,"end series with count {count}"),
            Self::Option(matched) => write!(f,"end option {} matched", if *matched { "" } else { "not" }),
            Self::Choice(branch) => write!(f,"end choice at branch {branch}"),
            Self::Tree(branch) => write!(f,"end tree at branch {branch}"),
            Self::Case(Some(environment)) => write!(f,"end case for environment '{environment}'"),
            Self::Case(None) => write!(f,"end case"),
            Self::CaseEnvironment(Some(branch)) => write!(f,"end case environment at branch {branch}"),
            Self::CaseEnvironment(None) => write!(f,"end case environment without a branch"),
            Self::RuleReference(name) => write!(f,"end rule_reference '{name}'"),
            Self::PhonemeFound(phoneme) => write!(f,"phoneme found {phoneme}"),
            Self::PhonemeNotFound => write!(f,"phoneme not found"),
            Self::Terminate => write!(f,"terminate"),
            Self::Word => write!(f,"word")
        }
    }
}

pub(crate) enum ValidationTraceMessage {
    Start(Location<'static>,usize,ValidationTraceStart),
    Success(Location<'static>,usize,ValidationTraceEnd),
    Failure(Location<'static>,usize,ValidationTraceEnd,ValidationFailure)
}

impl Display for ValidationTraceMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start(source, position, event) => write!(f,"[{source}; phoneme {position}]: {event}"),
            Self::Success(source, position, event) => write!(f,"[{source}; phoneme {position}]: {event}"),
            Self::Failure(source, position, event, error) => write!(f,"[{source}; phoneme {position}]: {event} failed -- {error}"),
        }
    }
}

pub(crate) type ValidationTraceCallback = dyn Fn(usize,ValidationTraceMessage);

struct ValidationTraceReporter<'callback> {
    report: Option<&'callback ValidationTraceCallback>,
    level: usize
}

impl ValidationTraceReporter<'_> {

    fn start(&mut self, location: Location<'static>, position: usize, event: ValidationTraceStart, explanation: &mut Vec<ValidWordElement>) {
        explanation.push(ValidWordElement {
            index: position,
            pattern_source: location,
            event: ValidWordEvent::Start(event.clone()),
        });
        if let Some(report) = self.report {
            report(self.level,ValidationTraceMessage::Start(location, position, event));
            self.level += 1;

        }
    }

    fn success(&mut self, location: Location<'static>, position: usize, event: ValidationTraceEnd, explanation: &mut Vec<ValidWordElement>) {
        explanation.push(ValidWordElement {
            index: position,
            pattern_source: location,
            event: ValidWordEvent::End(event.clone()),
        });
        if let Some(report) = self.report {
            if !matches!(event,ValidationTraceEnd::PhonemeFound(_) | ValidationTraceEnd::PhonemeNotFound | ValidationTraceEnd::Terminate | ValidationTraceEnd::Word) {
                // The above don't have a corresponding start, so the level doesn't get changed.
                self.level -= 1;
            }
            report(self.level,ValidationTraceMessage::Success(location, position, event));
        }
    }

    #[allow(clippy::needless_pass_by_value,reason="Clippy is wrong, the paramter error is consumed in the call to report")]
    fn failure(&mut self, location: Location<'static>, position: usize, event: ValidationTraceEnd, error: ValidationFailure) {
        if let Some(report) = self.report {
            if !matches!(event,ValidationTraceEnd::PhonemeFound(_) | ValidationTraceEnd::PhonemeNotFound | ValidationTraceEnd::Terminate | ValidationTraceEnd::Word) {
                // The above don't have a corresponding start, so the level doesn't get changed.
                self.level -= 1;
            }
            report(self.level,ValidationTraceMessage::Failure(location, position, event, error));
        }
    }
}

#[derive(Clone)]
enum ValidWordEvent {
    Start(ValidationTraceStart),
    End(ValidationTraceEnd)
}

impl Display for ValidWordEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start(event) => write!(f,"{event}"),
            Self::End(event) => write!(f,"{event}"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ValidWordElement {
    index: usize,
    pattern_source: Location<'static>,
    event: ValidWordEvent
}

impl Display for ValidWordElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            index,
            pattern_source,
            event,
        } = self;
        write!(f,"[{pattern_source}; phoneme {index}]: {event}")
    }
}

trait ValidateWord {

    // NOTE: See PatternSet::validate_word for why this doesn't return any error information.
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(),()>,ElbieError>;

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
pub(crate) struct Sequence {
    patterns: Vec<Pattern>,
    defined_at: Location<'static>
}

impl GenerateWord for Sequence {
    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        for pattern in &self.patterns {
            pattern.extend_word(language, rng, is_complete, result)?;
        }
        Ok(())
    }
}

impl ValidateWord for Sequence {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::Sequence,explanation);
        for (index,pattern) in self.patterns.iter().enumerate() {
            if pattern.validate_word(language, word, trace, explanation)?.is_err() {
                trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::Sequence,ValidationFailure::InnerSequencePatternFailed{ index });
                return Ok(Err(()))
            }
        }
        trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Sequence,explanation);
        Ok(Ok(()))
    }
}

#[derive(Debug)]
pub(crate) struct Series {
    pattern: Pattern,
    probability: u8,
    minimum: usize,
    maximum: Option<usize>,
    defined_at: Location<'static>
}


impl GenerateWord for Series {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        for _ in 0..self.minimum {
            self.pattern.extend_word(language, rng, is_complete, result)?;
        }
        let mut i = self.minimum;
        while (!*is_complete) && Pattern::is_probable(self.probability, rng) && self.maximum.is_none_or(|max| i < max) {
            self.pattern.extend_word(language, rng, is_complete, result)?;
            i += 1;
        }
        Ok(())
    }

}

impl ValidateWord for Series {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(),()>,ElbieError> {
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::Series,explanation);
        let mut count = 0;
        // clone the word so that if we have a failure, but it's above the minimum required, we can still return the original.
        let mut working_word = word.clone();
        let mut working_explanation = explanation.clone();
        #[allow(clippy::assigning_clones,reason="clone_from would require an immutable borrow")]
        while self.pattern.validate_word(language, &mut working_word, trace, &mut working_explanation)?.is_ok() {
            count += 1;
            working_word = working_word.clone();
            working_explanation = working_explanation.clone();
            if let Some(maximum) = self.maximum && count > maximum {
                // rather than returning an error, just stop validating. It's possible the remainder of the word is still valid in some other way.
                break;
            }
        };

        if count >= self.minimum {
            // reset to the last successfully matched working copy.
            *word = working_word;
            *explanation = working_explanation;
            trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Series(count),explanation);
            Ok(Ok(()))
        } else {
            trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::Series(count),ValidationFailure::SeriesFailedToReachMinimumCount {
                minimum: self.minimum,
                count,
            });
            Ok(Err(()))
        }
    }
}

#[derive(Debug)]
pub(crate) struct Optional {
    pattern: Pattern,
    probability: u8,
    defined_at: Location<'static>
}

impl GenerateWord for Optional {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        if (!*is_complete) && Pattern::is_probable(self.probability, rng)  {
            self.pattern.extend_word(language, rng, is_complete, result)
        } else {
            Ok(())
        }
    }

}

impl ValidateWord for Optional {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::Option,explanation);
        // clone the word so that if we have a failure, but it's above the minimum required, we can still return the original.
        let mut working_word = word.clone();
        let mut working_explanation = explanation.clone();
        if self.pattern.validate_word(language, &mut working_word, trace, &mut working_explanation)?.is_ok() {
            *word = working_word;
            *explanation = working_explanation;
            trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Option(true),explanation);
        } else {
            // failing to match an option isn't an error. So there really is no way for an option to fail.
            trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Option(false),explanation);
        }

        Ok(Ok(()))
    }

}


#[derive(Debug)]
struct ChoiceBranch {
    body: Pattern
}


#[derive(Debug)]
pub(crate) struct Choice {
    branches: WeightedVec<ChoiceBranch>,
    defined_at: Location<'static>
}


impl GenerateWord for Choice {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let branch = self.branches.choose(rng).ok_or(ElbieError::NoChoiceChoices(self.defined_at))?;
        branch.body.extend_word(language, rng, is_complete, result)
    }

}

impl ValidateWord for Choice {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::Choice,explanation);
        for (branch_idx,(branch,_weight)) in self.branches.items().iter().enumerate() {
            let mut working_word = word.clone();
            let mut working_explanation = explanation.clone();
            if branch.body.validate_word(language, &mut working_word, trace, &mut working_explanation)?.is_ok() {
                *word = working_word;
                *explanation = working_explanation;
                trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Choice(branch_idx),explanation);
                return Ok(Ok(()))
            }
        }
        // none of the choices matched
        trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::Choice(self.branches.items().len()),ValidationFailure::NoChoiceBranchesMatched);

        Ok(Err(()))
    }
}


#[derive(Debug)]
struct TreeBranch {
    head: Pattern,
    tail: Pattern
}


#[derive(Debug)]
pub(crate) struct Tree {
    branches: WeightedVec<TreeBranch>,
    defined_at: Location<'static>
}


impl GenerateWord for Tree {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let branch = self.branches.choose(rng).ok_or(ElbieError::NoTreeChoices(self.defined_at))?;
        branch.head.extend_word(language, rng, is_complete, result)?;
        branch.tail.extend_word(language, rng, is_complete, result)
    }

}

impl ValidateWord for Tree {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::Tree,explanation);
        for (branch_idx,(branch,_weight)) in self.branches.items().iter().enumerate() {
            let mut working_word = word.clone();
            let mut working_explanation = explanation.clone();
            if branch.head.validate_word(language, &mut working_word, trace, &mut working_explanation)?.is_ok() {
                *word = working_word;
                *explanation = working_explanation;
                if branch.tail.validate_word(language, word, trace, explanation)?.is_err() {
                    trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::Tree(branch_idx),ValidationFailure::BranchTailPatternFailed {
                        index: branch_idx
                    });
                    return Ok(Err(()))
                }
                trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Tree(branch_idx),explanation);
                return Ok(Ok(()))
            }
        }
        // none of the choices matched
        trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::Tree(self.branches.items().len()),ValidationFailure::NoTreeBranchesMatched);

        Ok(Err(()))
    }
}


#[derive(Debug)]
pub(crate) struct AddPhoneme {
    name: &'static str,
    avoid_duplicates: bool,
    defined_at: Location<'static>
}


impl AddPhoneme {

    fn extend_with_phoneme(&self,language: &Language, rng: &mut ThreadRng, is_complete: bool, result: &mut Word) -> Result<Rc<Phoneme>,ElbieError> {
        if is_complete {
            return Err(ElbieError::PhonemeAfterTerminate)
        }
        let phoneme = if self.avoid_duplicates && let Some(phoneme) = result.last() {
            language.inventory().choose_except(self.name,&[phoneme],rng)?
        } else {
            language.inventory().choose(self.name,rng)?
        };

        result.push(phoneme.clone());
        Ok(phoneme)
    }

    fn validate_with_phoneme(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<Rc<Phoneme>,()>,ElbieError> {
        if let Some((position,phoneme)) = word.next() {
            if language.inventory().phoneme_is(phoneme, self.name)? {
                trace.success(self.defined_at,position,ValidationTraceEnd::PhonemeFound(phoneme.clone()),explanation);
                Ok(Ok(phoneme.clone()))
            } else {
                trace.failure(self.defined_at,position,ValidationTraceEnd::PhonemeFound(phoneme.clone()),ValidationFailure::InvalidPhoneme {
                    expected: self.name,
                    found: phoneme.clone(),
                });
                Ok(Err(()))
            }
        } else {
            trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::PhonemeNotFound,ValidationFailure::UnexpectedEnd {
                expected: self.name,
            });
            Ok(Err(()))
        }
    }


}

impl GenerateWord for AddPhoneme {
    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        _ = self.extend_with_phoneme(language, rng, *is_complete, result)?;
        Ok(())
    }
}

impl ValidateWord for AddPhoneme {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        match self.validate_with_phoneme(language, word, trace, explanation)? {
            Ok(_) => Ok(Ok(())),
            Err(()) => Ok(Err(())),
        }
    }
}



#[derive(Debug)]
struct CaseEnvironmentBranch {
    condition_set: &'static str,
    body: Pattern,
}

#[derive(Debug)]
struct CaseEnvironment {
    branches: Vec<CaseEnvironmentBranch>,
    else_: Pattern,
    defined_at: Location<'static>
}

impl CaseEnvironment {
    // not a GeneratePattern trait because it requires the phoneme information that was just added.
    fn extend_word(&self, phoneme: &Rc<Phoneme>, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        for branch in &self.branches {
            if language.inventory().phoneme_is(phoneme, branch.condition_set)? {
                return branch.body.extend_word(language, rng, is_complete, result)
            }
        }
        self.else_.extend_word(language, rng, is_complete, result)
    }

    // not a ValidatePattern trait because it requires the phoneme information from the previous pattern.
    fn validate_word(&self, phoneme: &Rc<Phoneme>, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(),()>,ElbieError> {
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::CaseEnvironment,explanation);

        for (index,branch) in self.branches.iter().enumerate() {
            if language.inventory().phoneme_is(phoneme, branch.condition_set)? {
                match branch.body.validate_word(language, word, trace, explanation)? {
                    Ok(()) => {
                        trace.success(self.defined_at, word.next_index(), ValidationTraceEnd::CaseEnvironment(Some(index)),explanation);
                        return Ok(Ok(()))
                    },
                    Err(err) => {
                        trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::CaseEnvironment(Some(index)),ValidationFailure::CaseBranchBodyFailed {
                            index
                        });
                        return Ok(Err(err))
                    },
                }
            }
        }

        if self.else_.validate_word(language, word, trace, explanation)?.is_ok() {
            trace.success(self.defined_at, word.next_index(), ValidationTraceEnd::CaseEnvironment(None),explanation);
            Ok(Ok(()))
        } else {
            trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::CaseEnvironment(None),ValidationFailure::NoCaseBranchesMatched);
            Ok(Err(()))
        }
    }
}

#[derive(Debug)]
enum NamedOrInlineEnvironment {
    Environment(CaseEnvironment),
    Named(&'static str)
}

#[derive(Debug)]
pub(crate) struct Case {
    initial: AddPhoneme,
    environment: NamedOrInlineEnvironment,
    defined_at: Location<'static>
}


impl GenerateWord for Case {
    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let phoneme = self.initial.extend_with_phoneme(language, rng, *is_complete, result)?;
        let environment = match &self.environment {
            NamedOrInlineEnvironment::Environment(environment) => environment,
            NamedOrInlineEnvironment::Named(name) => language.patterns().get_case_environment(name)?,
        };
        environment.extend_word(&phoneme, language, rng, is_complete, result)
    }
}

impl ValidateWord for Case {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        let (environment,name) = match &self.environment {
            NamedOrInlineEnvironment::Environment(environment) => (environment,None),
            NamedOrInlineEnvironment::Named(name) => (language.patterns().get_case_environment(name)?,Some(*name)),
        };
        trace.start(self.defined_at,word.next_index(),ValidationTraceStart::Case(name),explanation);
        if let Ok(phoneme) = self.initial.validate_with_phoneme(language, word, trace, explanation)? {

            if environment.validate_word(&phoneme, language, word, trace, explanation)?.is_ok() {
                trace.success(self.defined_at, word.next_index(), ValidationTraceEnd::Case(name),explanation);
                Ok(Ok(()))
            } else {
                trace.failure(self.defined_at, word.next_index(), ValidationTraceEnd::Case(name), ValidationFailure::CaseEnvironmentFailed);
                Ok(Err(()))
            }
        } else {
            trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::Case(name), ValidationFailure::CaseConditionFailed);
            Ok(Err(()))
        }
    }
}

#[derive(Debug)]
pub(crate) struct TerminateWord {
    defined_at: Location<'static>
}

impl GenerateWord for TerminateWord {
    fn extend_word(&self, _: &Language, _: &mut ThreadRng, is_complete: &mut bool, _: &mut Word) -> Result<(),ElbieError> {
        *is_complete = true;
        Ok(())
    }
}

impl ValidateWord for TerminateWord {
    fn validate_word(&self, _: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        if let Some((index,phoneme)) = word.next() {
            trace.failure(self.defined_at,index,ValidationTraceEnd::Terminate,ValidationFailure::UnexpectedPhoneme {
                found: phoneme.clone(),
            });
            Ok(Err(()))
        } else {
            trace.success(self.defined_at,word.next_index(),ValidationTraceEnd::Terminate,explanation);
            Ok(Ok(()))
        }
    }
}

#[derive(Debug)]
pub(crate) struct RuleReference {
    name: &'static str,
    defined_at: Location<'static>
}

impl GenerateWord for RuleReference {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let pattern = language.patterns().get(self.name)?;
        pattern.extend_word(language, rng, is_complete, result)
    }
}

impl ValidateWord for RuleReference {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        trace.start(self.defined_at, word.next_index(), ValidationTraceStart::RuleReference(self.name),explanation);
        let pattern = language.patterns().get(self.name)?;
        if pattern.validate_word(language, word, trace, explanation)?.is_ok() {
            trace.success(self.defined_at, word.next_index(), ValidationTraceEnd::RuleReference(self.name),explanation);
            Ok(Ok(()))
        } else {
            trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::RuleReference(self.name), ValidationFailure::ReferencedRuleFailed{
                name: self.name
            });
            Ok(Err(()))
        }

    }
}

#[derive(Debug)]
pub(crate) enum Pattern {
    Sequence(Sequence),
    Series(Box<Series>),
    Option(Box<Optional>),
    Choice(Choice),
    Tree(Tree),
    Case(Box<Case>),
    RuleReference(RuleReference),
    Set(AddPhoneme),
    // This can be used to force completion in certain situations, such as not allowing a series to continue, or disallowing an option.
    // If used in a pattern before a non-optional pattern with phonemes, it will fail.
    Terminate(TerminateWord)
}

impl Pattern {

    fn is_probable(probability: u8, rng: &mut ThreadRng) -> bool {
        rng.random::<u8>() <= probability
    }

    fn defined_at(&self) -> Location<'static> {
        match self {
            Self::Sequence(sequence) => sequence.defined_at,
            Self::Series(series) => series.defined_at,
            Self::Option(optional) => optional.defined_at,
            Self::Choice(choice) => choice.defined_at,
            Self::Tree(tree) => tree.defined_at,
            Self::Case(case) => case.defined_at,
            Self::RuleReference(reference) => reference.defined_at,
            Self::Set(add_phoneme) => add_phoneme.defined_at,
            Self::Terminate(terminate_word) => terminate_word.defined_at,
        }
    }

}

impl GenerateWord for Pattern {

    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        match self {
            Self::Sequence(sequence) => sequence.extend_word(language, rng, is_complete, result),
            Self::Series(series) => series.extend_word(language, rng,is_complete,result),
            Self::Option(optional) => optional.extend_word(language, rng, is_complete, result),
            Self::Choice(choice) => choice.extend_word(language, rng, is_complete, result),
            Self::Tree(tree) => tree.extend_word(language, rng, is_complete, result),
            Self::Case(switch) => switch.extend_word(language, rng, is_complete, result),
            Self::RuleReference(reference) => reference.extend_word(language, rng, is_complete, result),
            Self::Set(set) => set.extend_word(language, rng, is_complete, result),
            Self::Terminate(terminate) => terminate.extend_word(language, rng, is_complete, result),
        }
    }


}

impl ValidateWord for Pattern {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        match self {
            Self::Sequence(sequence) => sequence.validate_word(language, word, trace, explanation),
            Self::Series(series) => series.validate_word(language, word, trace, explanation),
            Self::Option(optional) => optional.validate_word(language, word, trace, explanation),
            Self::Choice(choice) => choice.validate_word(language, word, trace, explanation),
            Self::Tree(tree) => tree.validate_word(language, word, trace, explanation),
            Self::Case(switch) => switch.validate_word(language, word, trace, explanation),
            Self::RuleReference(reference) => reference.validate_word(language, word, trace, explanation),
            Self::Set(set) => set.validate_word(language, word, trace, explanation),
            Self::Terminate(terminate) => terminate.validate_word(language, word, trace, explanation),
        }
    }

}

pub struct PhonoPatternBuilder {
    patterns: Vec<Pattern>
}

impl PhonoPatternBuilder {

    const fn new() -> Self {
        Self {
            patterns: Vec::new()
        }
    }

    fn flatten(mut self, defined_at: Location<'static>) -> Pattern {
        let len = self.patterns.len();
        if len == 1 {
            self.patterns.remove(0)
        } else {
            Pattern::Sequence(Sequence {
                patterns: self.patterns,
                defined_at
            })
        }
    }

    pub fn seq<PatternCallback: Fn(&mut Self)>(&mut self, callback: PatternCallback) {
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
    pub fn ser_min_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.series(probability, callback, minimum, Some(maximum));
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize) {
        self.series(probability, callback, minimum, None);
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, maximum: usize) {
        self.series(probability, callback, 0, Some(maximum));
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback) {
        self.series(probability, callback, 0, None);
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback) {
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
    /**
    # Panics

    Panics if no choices are added in the callback.
    */
    pub fn choice<BranchCallback: Fn(&mut ChoiceBuilder)>(&mut self, callback: BranchCallback) {
        let defined_at = *Location::caller();
        let mut builder = ChoiceBuilder::new();
        callback(&mut builder);
        if builder.branches.items().is_empty() {
            panic!("Branches are empty.")
        }
        self.patterns.push(Pattern::Choice(Choice {
            branches: builder.branches,
            defined_at
        }));
    }

    /**
    A tree is a branching pattern, where each branch consists of a head, tail and a weight. If the branch is chosen, the head pattern is generated, followed by the tail. However, when validating, only the head is matched to determine if it's the correct branch. If the head fails to match, the branch is assumed to be wrong and the validation goes onto the next branch. If the head matches, but the tail doesn't, then the whole tree doesn't match. If none of the heads match, then the tree also fails to match. If you don't need the tail, just add an empty pattern.

    # Panics

    Panics if no branches are added to the tree in the callback.
    */
    #[track_caller]
    pub fn tree<BranchCallback: Fn(&mut TreeBuilder)>(&mut self, callback: BranchCallback) {
        let defined_at = *Location::caller();
        let mut builder = TreeBuilder::new();
        callback(&mut builder);
        if builder.branches.items().is_empty() {
            panic!("Branches are empty.")
        }
        self.patterns.push(Pattern::Tree(Tree {
            branches: builder.branches,
            defined_at
        }));
    }

    fn case_opt<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, callback: BranchCallback) {
        let environment = NamedOrInlineEnvironment::Environment(CaseEnvironmentBuilder::build(defined_at, callback));

        self.patterns.push(Pattern::Case(Box::new(Case {
            initial: AddPhoneme {
                name: initial_phoneme,
                avoid_duplicates,
                defined_at
            },
            environment,
            defined_at,
        })));
    }

    #[track_caller]
    pub fn case<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        let defined_at = *Location::caller();
        self.case_opt(defined_at, initial_phoneme, false, callback);
    }

    #[track_caller]
    pub fn case_nodup<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        let defined_at = *Location::caller();
        self.case_opt(defined_at, initial_phoneme, true, callback);
    }

    fn case_env_opt(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, environment: &'static str) {
        let environment = NamedOrInlineEnvironment::Named(environment);

        self.patterns.push(Pattern::Case(Box::new(Case {
            initial: AddPhoneme {
                name: initial_phoneme,
                avoid_duplicates,
                defined_at
            },
            environment,
            defined_at,
        })));
    }

    #[track_caller]
    #[deprecated(since="0.4.0", note="Named environments will probably go away unless there's a good use for them. They exist only to facilitate the deprecated Phonotactic environments.")]
    pub fn case_env(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        let defined_at = *Location::caller();
        self.case_env_opt(defined_at, initial_phoneme, false, environment);
    }

    #[track_caller]
    #[deprecated(since="0.4.0", note="Named environments will probably go away unless there's a good use for them. They exist only to facilitate the deprecated Phonotactic environments.")]
    pub fn case_env_nodup(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        let defined_at = *Location::caller();
        self.case_env_opt(defined_at, initial_phoneme, true, environment);
    }

    #[track_caller]
    pub fn rule(&mut self, name: &'static str) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::RuleReference(RuleReference {
            name,
            defined_at,
        }));
    }

    #[track_caller]
    pub fn set(&mut self, name: &'static str) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::Set(AddPhoneme {
            name,
            avoid_duplicates: false,
            defined_at,
        }));
    }

    #[track_caller]
    pub fn set_nodup(&mut self, name: &'static str) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::Set(AddPhoneme {
            name,
            avoid_duplicates: true,
            defined_at,
        }));
    }

    #[track_caller]
    pub fn done(&mut self) {
        let defined_at = *Location::caller();
        self.patterns.push(Pattern::Terminate(TerminateWord {
            defined_at
        }));
    }

}

pub struct ChoiceBuilder {
    branches: WeightedVec<ChoiceBranch>
}

impl ChoiceBuilder {

    const fn new() -> Self {
        Self {
            branches: WeightedVec::new()
        }
    }

    #[track_caller]
    /// see PhonoPatternBuilder::tree for an explanation of the head and the tail patterns.
    pub fn add<BodyCallback: Fn(&mut PhonoPatternBuilder)>(&mut self, weight: usize, body_cb: BodyCallback) {
        let defined_at = *Location::caller();

        let mut head = PhonoPatternBuilder::new();
        body_cb(&mut head);
        self.branches.push(ChoiceBranch {
            body: head.flatten(defined_at)
            //defined_at
        },weight)
    }
}

pub struct TreeBuilder {
    branches: WeightedVec<TreeBranch>
}

impl TreeBuilder {

    const fn new() -> Self {
        Self {
            branches: WeightedVec::new()
        }
    }

    #[track_caller]
    /// see PhonoPatternBuilder::tree for an explanation of the head and the tail patterns.
    pub fn add<HeadCallback: Fn(&mut PhonoPatternBuilder), TailCallback: Fn(&mut PhonoPatternBuilder)>(&mut self, weight: usize, head_cb: HeadCallback, tail_cb: TailCallback) {
        let defined_at = *Location::caller();

        let mut head = PhonoPatternBuilder::new();
        head_cb(&mut head);
        let mut tail = PhonoPatternBuilder::new();
        tail_cb(&mut tail);
        self.branches.push(TreeBranch {
            head: head.flatten(defined_at),
            tail: tail.flatten(defined_at),
            //defined_at
        },weight)
    }
}


pub struct CaseEnvironmentBuilder {
    branches: Vec<CaseEnvironmentBranch>,
    else_: Option<Pattern>
}

impl CaseEnvironmentBuilder {

    fn build<BranchCallback: Fn(&mut Self)>(defined_at: Location<'static>, callback: BranchCallback) -> CaseEnvironment {
        let mut builder = Self {
            branches: Vec::new(),
            else_: None
        };
        callback(&mut builder);

        CaseEnvironment {
            branches: builder.branches,
            else_: builder.else_.expect("The environment at least needs an else."),
            defined_at
        }

    }

    #[track_caller]
    pub fn branch<Callback: Fn(&mut PhonoPatternBuilder)>(&mut self, condition_set: &'static str, body_cb: Callback) {
        let defined_at = *Location::caller();

        let mut body = PhonoPatternBuilder::new();
        body_cb(&mut body);
        self.branches.push(CaseEnvironmentBranch {
            condition_set,
            body: body.flatten(defined_at),
            //defined_at,
        })
    }

    #[track_caller]
    pub fn else_<Callback: Fn(&mut PhonoPatternBuilder)>(&mut self, body_cb: Callback) {
        let defined_at = *Location::caller();

        let mut body = PhonoPatternBuilder::new();
        body_cb(&mut body);
        self.else_ = Some(body.flatten(defined_at))
    }

}

#[derive(Debug)]
pub(crate) struct PhonoPatternSet {
    patterns: HashMap<String,Pattern>,
    case_environments: HashMap<String,CaseEnvironment>,
    initial: Pattern
}

impl PhonoPatternSet {

    #[track_caller]
    pub(crate) fn new<Callback: Fn(&mut PhonoPatternBuilder)>(initial_cb: Callback) -> Self {
        let mut builder = PhonoPatternBuilder::new();
        initial_cb(&mut builder);
        let initial = builder.flatten(*Location::caller());
        Self {
            patterns: HashMap::new(),
            case_environments: HashMap::new(),
            initial
        }

    }

    #[track_caller]
    pub(crate) fn pattern<Callback: Fn(&mut PhonoPatternBuilder)>(&mut self, name: &'static str, callback: Callback) {
        let mut builder = PhonoPatternBuilder::new();
        callback(&mut builder);
        let pattern = builder.flatten(*Location::caller());
        match self.patterns.entry(name.to_owned()) {
            Entry::Occupied(_) => panic!("A pattern named '{name}' already exists."),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(pattern),
        }
    }

    pub(crate) fn get(&self, name: &'static str) -> Result<&Pattern,ElbieError> {
        if let Some(pattern) = self.patterns.get(name) {
            Ok(pattern)
        } else {
            Err(ElbieError::UnknownPattern(name))
        }
    }

    #[track_caller]
    pub(crate) fn case_environment<Callback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, name: &'static str, callback: Callback) {
        let environment = CaseEnvironmentBuilder::build(*Location::caller(), callback);
        match self.case_environments.entry(name.to_owned()) {
            Entry::Occupied(_) => panic!("An environment named '{name}' already exists."),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(environment),
        }
    }

    fn get_case_environment(&self, name: &'static str) -> Result<&CaseEnvironment,ElbieError> {
        if let Some(pattern) = self.case_environments.get(name) {
            Ok(pattern)
        } else {
            Err(ElbieError::UnknownEnvironment(name))
        }
    }

    pub(crate) fn generate(&self, language: &Language, rng: &mut ThreadRng) -> Result<Word,ElbieError> {
        let mut result = Word::new(&[]);

        self.initial.extend_word(language, rng, &mut false, &mut result)?;
        Ok(result)

    }

    /*
    If the word is valid, it returns a list of the success trace events that led to a word being called valid. If not, it returns nothing as an error.

    No information is returned for why the word failed to validate, because the reason is almost never a single event. A failed validation at some pattern causes a catastrophic failure to all patterns that contain it. But you can't pinpoint it to that original error, because the pattern was wrapped in a tree, a switch, a series, etc. The failure may have caused only one of the branches to fail, but other failures caused the other branches to fail. So, the real reason was because none of the branches in the tree matched. But that's not the answer either, because that is also wrapped in a conditional pattern of some sort. Usually, the final error returned would be just the big switch that is used for the onset of the word failed.

    Attempts to do things to simplify this cause, like returning the last error in a branch, proved futile as the information was always useless for determining the result. Even returning a list of failed events to counteract the successful events for a valid word, wouldn't be any better than just reading the trace.
    */
    pub(crate) fn validate(&self, language: &Language, word: &Word, trace: Option<&ValidationTraceCallback>) -> Result<Result<Vec<ValidWordElement>,()>,ElbieError> {
        let mut word = EnumerateCount::new(word.phonemes().iter());
        let mut explanation = Vec::new();
        let mut trace = ValidationTraceReporter {
            report: trace,
            level: 0,
        };
        if self.initial.validate_word(language,&mut word, &mut trace, &mut explanation)?.is_err() {
            trace.failure(self.initial.defined_at(), word.next_index(), ValidationTraceEnd::Word, ValidationFailure::InitialPatternFailed);
            Ok(Err(()))
        } else if let Some((position,phoneme)) = word.next() {
            trace.failure(self.initial.defined_at(), position, ValidationTraceEnd::Word, ValidationFailure::UnexpectedPhonemeAfterPattern { found: phoneme.clone() });
            Ok(Err(()))
        } else {
            Ok(Ok(explanation))
        }
    }


}
