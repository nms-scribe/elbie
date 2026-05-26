use std::rc::Rc;
use crate::phoneme::Phoneme;
use core::fmt::Display;
use core::fmt;
use core::panic::Location;
use crate::language::Language;
use crate::enumerate_with_count::EnumerateCount;
use core::slice::Iter;
use crate::errors::ElbieError;
use crate::phonotactics::Sequence;
use crate::phonotactics::Series;
use crate::phonotactics::Optional;
use crate::phonotactics::Choice;
use crate::phonotactics::AddPhoneme;
use crate::phonotactics::CaseEnvironment;
use crate::phonotactics::Case;
use crate::phonotactics::NamedOrInlineEnvironment;
use crate::phonotactics::TerminateWord;
use crate::phonotactics::RuleReference;
use crate::phonotactics::Pattern;
use crate::phonotactics::PatternSet;
use crate::word::Word;


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

pub(crate) struct ValidationTraceReporter<'callback> {
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

pub(crate) trait ValidateWord {

    // NOTE: See PatternSet::validate_word for why this doesn't return any error information.
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(),()>,ElbieError>;

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


#[allow(clippy::multiple_inherent_impl,reason="I want to separate validation and generation from the patterns")]
impl AddPhoneme {


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


impl ValidateWord for AddPhoneme {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        match self.validate_with_phoneme(language, word, trace, explanation)? {
            Ok(_) => Ok(Ok(())),
            Err(()) => Ok(Err(())),
        }
    }
}

#[allow(clippy::multiple_inherent_impl,reason="I want to separate validation and generation from the patterns")]
impl CaseEnvironment {

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

        trace.failure(self.defined_at,word.next_index(),ValidationTraceEnd::CaseEnvironment(None),ValidationFailure::NoCaseBranchesMatched);
        Ok(Err(()))
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


impl ValidateWord for Pattern {
    fn validate_word(&self, language: &Language, word: &mut EnumerateCount<Iter<Rc<Phoneme>>>, trace: &mut ValidationTraceReporter, explanation: &mut Vec<ValidWordElement>) -> Result<Result<(), ()>, ElbieError> {
        match self {
            Self::Sequence(sequence) => sequence.validate_word(language, word, trace, explanation),
            Self::Series(series) => series.validate_word(language, word, trace, explanation),
            Self::Option(optional) => optional.validate_word(language, word, trace, explanation),
            Self::Choice(choice) => choice.validate_word(language, word, trace, explanation),
            Self::Case(switch) => switch.validate_word(language, word, trace, explanation),
            Self::RuleReference(reference) => reference.validate_word(language, word, trace, explanation),
            Self::Set(set) => set.validate_word(language, word, trace, explanation),
            Self::Terminate(terminate) => terminate.validate_word(language, word, trace, explanation),
        }
    }

}

#[allow(clippy::multiple_inherent_impl,reason="I want to separate validation and generation from the patterns")]
impl PatternSet {


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
