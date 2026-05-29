use crate::weighted_vec::WeightedVec;
use core::panic::Location;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::errors::ElbieError;

/*
FUTURE: This is relatively easy, but it's also hard to wrap ones head around how this works. One option to fix this is to do the same thing here than I'm doing in transformations, turn it into functions.

My idea would be to return functions that take an implentation of a trait, say: trait Phonotactics. The trait would have methods for matching phonemes in sets, similar to the way transformations work. But it would also have more stuff, like repeats which couldn't be done with loops, and weighting of choices.

The trait would be implemented by two objects: A Generator and a Validator. That way the user still only has to write one function.

The Generator uses the commands (and their weightings) to "choose" the phonemes to output.

The Validator ignores the weightings, but uses the commands to match a word and make sure it is valid. It might make use of a rule-name argument to help with tracing.

*/

#[derive(Debug, Clone)]
#[deprecated(since = "0.4.0", note = "Please use patterns instead.")]
pub enum EnvironmentChoice {
    Done,
    Continuing(&'static str, &'static str, bool) // set to generate next phoneme from, next environment to follow, whether to allow duplicate phoneme to be generated
}

#[derive(Debug, Clone)]
#[deprecated(since = "0.4.0", note = "Please use patterns instead.")]
#[allow(deprecated)]
pub struct EnvironmentBranch(&'static str, WeightedVec<EnvironmentChoice>);

#[allow(deprecated)]
impl EnvironmentBranch {
    #[must_use]
    pub fn new(set_check: &'static str, choices: &[(EnvironmentChoice, usize)]) -> Self {
        let mut vec = WeightedVec::new();
        for choice in choices {
            vec.push(choice.0.clone(), choice.1)
        }
        Self(set_check, vec)
    }

    pub(crate) const fn set(&self) -> &'static str {
        self.0
    }

    pub(crate) const fn choices(&self) -> &WeightedVec<EnvironmentChoice> {
        &self.1
    }
}

#[derive(Debug)]
pub(crate) struct Sequence {
    pub patterns: Vec<Pattern>,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct Series {
    pub pattern: Pattern,
    pub probability: u8,
    pub minimum: usize,
    pub maximum: Option<usize>,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct Optional {
    pub pattern: Pattern,
    pub probability: u8,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct ChoiceBranch {
    pub body: Pattern
}

#[derive(Debug)]
pub(crate) struct Choice {
    pub branches: WeightedVec<ChoiceBranch>,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct AddPhoneme {
    pub name: &'static str,
    pub avoid_duplicates: bool,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct CaseEnvironmentBranch {
    pub condition_set: &'static str,
    pub body: Pattern
}

#[derive(Debug)]
pub(crate) struct CaseEnvironment {
    pub branches: Vec<CaseEnvironmentBranch>,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) enum NamedOrInlineEnvironment {
    Environment(CaseEnvironment),
    Named(&'static str)
}

#[derive(Debug)]
pub(crate) struct Case {
    pub initial: AddPhoneme,
    pub environment: NamedOrInlineEnvironment,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct TerminateWord {
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) struct RuleReference {
    pub name: &'static str,
    pub defined_at: Location<'static>
}

#[derive(Debug)]
pub(crate) enum Pattern {
    Sequence(Sequence),
    Series(Box<Series>),
    Option(Box<Optional>),
    Choice(Choice),
    Case(Box<Case>),
    RuleReference(RuleReference),
    Set(AddPhoneme),
    // This can be used to force completion in certain situations, such as not allowing a series to continue, or disallowing an option.
    // If used in a pattern before a non-optional pattern with phonemes, it will fail.
    Terminate(TerminateWord)
}

impl Pattern {
    pub(crate) fn defined_at(&self) -> Location<'static> {
        match self {
            Self::Sequence(sequence) => sequence.defined_at,
            Self::Series(series) => series.defined_at,
            Self::Option(optional) => optional.defined_at,
            Self::Choice(choice) => choice.defined_at,
            Self::Case(case) => case.defined_at,
            Self::RuleReference(reference) => reference.defined_at,
            Self::Set(add_phoneme) => add_phoneme.defined_at,
            Self::Terminate(terminate_word) => terminate_word.defined_at
        }
    }
}

struct PatternList<Extra>(Vec<(Pattern, Extra)>);

impl<Extra> PatternList<Extra> {
    fn seq<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, defined_at: Location<'static>, callback: PatternCallback, extra: Extra) {
        let mut builder = PatternBuilder::new();
        callback(&mut builder);
        self.0.push((Pattern::Sequence(Sequence { patterns: builder.patterns(),
                                                  defined_at }),
                     extra));
    }

    fn series<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, defined_at: Location<'static>, probability: u8, callback: PatternCallback, minimum: usize, maximum: Option<usize>, extra: Extra) {
        if maximum.is_some_and(|max| max < minimum) {
            // I don't want to return an error here, as that would add undue complications on the closures used to build patterns. FUTURE: reconsider?
            panic!("Maximum length of series is less than minimum.")
        }
        let mut pattern = PatternBuilder::new();
        callback(&mut pattern);
        let pattern = pattern.flatten(defined_at);
        self.0.push((Pattern::Series(Box::new(Series { pattern,
                                                       probability,
                                                       minimum,
                                                       maximum,
                                                       defined_at })),
                     extra));
    }

    fn opt<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, defined_at: Location<'static>, probability: u8, callback: PatternCallback, extra: Extra) {
        let mut pattern = PatternBuilder::new();
        callback(&mut pattern);
        let pattern = pattern.flatten(defined_at);
        self.0.push((Pattern::Option(Box::new(Optional { pattern,
                                                         probability,
                                                         defined_at })),
                     extra));
    }

    /**
    # Panics

    Panics if no choices are added in the callback.
    */
    fn choice<BranchCallback: Fn(&mut ChoiceBuilder)>(&mut self, defined_at: Location<'static>, callback: BranchCallback, extra: Extra) {
        let mut builder = ChoiceBuilder::new();
        callback(&mut builder);
        if builder.pattern_list.0.is_empty() {
            // I don't want to return an error here, as that would add undue complications on the closures used to build patterns. FUTURE: reconsider?
            panic!("Branches are empty.")
        }
        self.0.push((Pattern::Choice(Choice { branches: builder.choices(),
                                              defined_at }),
                     extra));
    }

    fn case_opt<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, callback: BranchCallback,
                                                                 extra: Extra) {
        let mut builder = CaseEnvironmentBuilder::new();
        callback(&mut builder);
        let environment = NamedOrInlineEnvironment::Environment(CaseEnvironment { branches: builder.branches(),
                                                                                  defined_at });

        self.0.push((Pattern::Case(Box::new(Case { initial: AddPhoneme { name: initial_phoneme,
                                                                         avoid_duplicates,
                                                                         defined_at },
                                                   environment,
                                                   defined_at })),
                     extra));
    }

    fn case_env_opt(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, environment: &'static str, extra: Extra) {
        let environment = NamedOrInlineEnvironment::Named(environment);

        self.0.push((Pattern::Case(Box::new(Case { initial: AddPhoneme { name: initial_phoneme,
                                                                         avoid_duplicates,
                                                                         defined_at },
                                                   environment,
                                                   defined_at })),
                     extra));
    }

    fn rule(&mut self, defined_at: Location<'static>, name: &'static str, extra: Extra) {
        self.0.push((Pattern::RuleReference(RuleReference { name,
                                                            defined_at }),
                     extra));
    }

    fn set_opt(&mut self, defined_at: Location<'static>, name: &'static str, avoid_duplicates: bool, extra: Extra) {
        self.0.push((Pattern::Set(AddPhoneme { name,
                                               avoid_duplicates,
                                               defined_at }),
                     extra));
    }

    fn done(&mut self, defined_at: Location<'static>, extra: Extra) {
        self.0.push((Pattern::Terminate(TerminateWord { defined_at }), extra));
    }
}

pub struct PatternBuilder {
    pattern_list: PatternList<()>
}

impl PatternBuilder {
    const fn new() -> Self {
        Self { pattern_list: PatternList(Vec::new()) }
    }

    fn patterns(self) -> Vec<Pattern> {
        self.pattern_list.0.into_iter().map(|(p, ())| p).collect()
    }

    fn flatten(mut self, defined_at: Location<'static>) -> Pattern {
        let len = self.pattern_list.0.len();
        if len == 1 {
            self.pattern_list.0.remove(0).0
        } else {
            Pattern::Sequence(Sequence { patterns: self.patterns(),
                                         defined_at })
        }
    }

    // NOTE: no seq on the sequence, because you want to avoid nested sequences in the long run.

    #[track_caller]
    pub fn ser_min_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, Some(maximum), ());
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, minimum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, None, ());
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, Some(maximum), ());
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, None, ());
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut Self)>(&mut self, probability: u8, callback: PatternCallback) {
        self.pattern_list.opt(*Location::caller(), probability, callback, ());
    }

    #[track_caller]
    /**
    # Panics

    Panics if no choices are added in the callback.
    */
    pub fn choice<BranchCallback: Fn(&mut ChoiceBuilder)>(&mut self, callback: BranchCallback) {
        self.pattern_list.choice(*Location::caller(), callback, ());
    }

    #[track_caller]
    pub fn case<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.case_opt(*Location::caller(), initial_phoneme, false, callback, ());
    }

    #[track_caller]
    pub fn case_nodup<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.case_opt(*Location::caller(), initial_phoneme, true, callback, ());
    }

    #[track_caller]
    pub fn case_env(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.case_env_opt(*Location::caller(), initial_phoneme, false, environment, ());
    }

    #[track_caller]
    pub fn case_env_nodup(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.case_env_opt(*Location::caller(), initial_phoneme, true, environment, ());
    }

    #[track_caller]
    pub fn rule(&mut self, name: &'static str) {
        self.pattern_list.rule(*Location::caller(), name, ());
    }

    #[track_caller]
    pub fn set(&mut self, name: &'static str) {
        self.pattern_list.set_opt(*Location::caller(), name, false, ());
    }

    #[track_caller]
    pub fn set_nodup(&mut self, name: &'static str) {
        self.pattern_list.set_opt(*Location::caller(), name, true, ());
    }

    #[track_caller]
    pub fn done(&mut self) {
        self.pattern_list.done(*Location::caller(), ());
    }
}

pub struct ChoiceBuilder {
    pattern_list: PatternList<usize>
}

impl ChoiceBuilder {
    const fn new() -> Self {
        Self { pattern_list: PatternList(Vec::new()) }
    }

    fn choices(self) -> WeightedVec<ChoiceBranch> {
        let mut result = WeightedVec::new();
        for (p, weight) in self.pattern_list.0 {
            result.push(ChoiceBranch { body: p }, weight);
        }
        result
    }

    #[track_caller]
    pub fn seq<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, callback: PatternCallback) {
        self.pattern_list.seq(*Location::caller(), callback, weight);
    }

    #[track_caller]
    pub fn ser_min_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: u8, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, Some(maximum), weight);
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: u8, callback: PatternCallback, minimum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, None, weight);
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: u8, callback: PatternCallback, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, Some(maximum), weight);
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: u8, callback: PatternCallback) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, None, weight);
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: u8, callback: PatternCallback) {
        self.pattern_list.opt(*Location::caller(), probability, callback, weight);
    }

    #[track_caller]
    /**
    # Panics

    Panics if no choices are added in the callback.
    */
    pub fn choice<BranchCallback: Fn(&mut Self)>(&mut self, weight: usize, callback: BranchCallback) {
        self.pattern_list.choice(*Location::caller(), callback, weight);
    }

    #[track_caller]
    pub fn case<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, weight: usize, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.case_opt(*Location::caller(), initial_phoneme, false, callback, weight);
    }

    #[track_caller]
    pub fn case_nodup<BranchCallback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, weight: usize, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.case_opt(*Location::caller(), initial_phoneme, true, callback, weight);
    }

    #[track_caller]
    pub fn case_env(&mut self, weight: usize, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.case_env_opt(*Location::caller(), initial_phoneme, false, environment, weight);
    }

    #[track_caller]
    pub fn case_env_nodup(&mut self, weight: usize, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.case_env_opt(*Location::caller(), initial_phoneme, true, environment, weight);
    }

    #[track_caller]
    pub fn rule(&mut self, weight: usize, name: &'static str) {
        self.pattern_list.rule(*Location::caller(), name, weight);
    }

    #[track_caller]
    pub fn set(&mut self, weight: usize, name: &'static str) {
        self.pattern_list.set_opt(*Location::caller(), name, false, weight);
    }

    #[track_caller]
    pub fn set_nodup(&mut self, weight: usize, name: &'static str) {
        self.pattern_list.set_opt(*Location::caller(), name, true, weight);
    }

    #[track_caller]
    pub fn done(&mut self, weight: usize) {
        self.pattern_list.done(*Location::caller(), weight);
    }
}

pub struct CaseEnvironmentBuilder {
    pattern_list: PatternList<&'static str>
}

impl CaseEnvironmentBuilder {
    const fn new() -> Self {
        Self { pattern_list: PatternList(Vec::new()) }
    }

    fn branches(self) -> Vec<CaseEnvironmentBranch> {
        let mut result = Vec::new();
        for (p, condition_set) in self.pattern_list.0 {
            result.push(CaseEnvironmentBranch {
                condition_set,
                body: p,
                //defined_at,
            });
        }
        result
    }

    #[track_caller]
    pub fn seq<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, callback: PatternCallback) {
        self.pattern_list.seq(*Location::caller(), callback, condition_set);
    }

    #[track_caller]
    pub fn ser_min_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: u8, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, Some(maximum), condition_set);
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: u8, callback: PatternCallback, minimum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, None, condition_set);
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: u8, callback: PatternCallback, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, Some(maximum), condition_set);
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: u8, callback: PatternCallback) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, None, condition_set);
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: u8, callback: PatternCallback) {
        self.pattern_list.opt(*Location::caller(), probability, callback, condition_set);
    }

    #[track_caller]
    /**
    # Panics

    Panics if no choices are added in the callback.
    */
    pub fn choice<BranchCallback: Fn(&mut ChoiceBuilder)>(&mut self, condition_set: &'static str, callback: BranchCallback) {
        self.pattern_list.choice(*Location::caller(), callback, condition_set);
    }

    #[track_caller]
    pub fn case<BranchCallback: Fn(&mut Self)>(&mut self, condition_set: &'static str, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.case_opt(*Location::caller(), initial_phoneme, false, callback, condition_set);
    }

    #[track_caller]
    pub fn case_nodup<BranchCallback: Fn(&mut Self)>(&mut self, condition_set: &'static str, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.case_opt(*Location::caller(), initial_phoneme, true, callback, condition_set);
    }

    #[track_caller]
    pub fn case_env(&mut self, condition_set: &'static str, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.case_env_opt(*Location::caller(), initial_phoneme, false, environment, condition_set);
    }

    #[track_caller]
    pub fn case_env_nodup(&mut self, condition_set: &'static str, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.case_env_opt(*Location::caller(), initial_phoneme, true, environment, condition_set);
    }

    #[track_caller]
    pub fn rule(&mut self, condition_set: &'static str, name: &'static str) {
        self.pattern_list.rule(*Location::caller(), name, condition_set);
    }

    #[track_caller]
    pub fn set(&mut self, condition_set: &'static str, name: &'static str) {
        self.pattern_list.set_opt(*Location::caller(), name, false, condition_set);
    }

    #[track_caller]
    pub fn set_nodup(&mut self, condition_set: &'static str, name: &'static str) {
        self.pattern_list.set_opt(*Location::caller(), name, true, condition_set);
    }

    #[track_caller]
    pub fn done(&mut self, condition_set: &'static str) {
        self.pattern_list.done(*Location::caller(), condition_set);
    }
}

#[derive(Debug)]
pub(crate) struct PatternSet {
    pub patterns: HashMap<String, Pattern>,
    pub case_environments: HashMap<String, CaseEnvironment>,
    pub initial: Pattern
}

impl PatternSet {
    #[track_caller]
    pub(crate) fn new<Callback: Fn(&mut PatternBuilder)>(initial_cb: Callback) -> Self {
        let mut builder = PatternBuilder::new();
        initial_cb(&mut builder);
        let initial = builder.flatten(*Location::caller());
        Self { patterns: HashMap::new(),
               case_environments: HashMap::new(),
               initial }
    }

    #[track_caller]
    pub(crate) fn pattern<Callback: Fn(&mut PatternBuilder)>(&mut self, name: &'static str, callback: Callback) -> Result<(), ElbieError> {
        let mut builder = PatternBuilder::new();
        callback(&mut builder);
        let pattern = builder.flatten(*Location::caller());
        match self.patterns.entry(name.to_owned()) {
            Entry::Occupied(_) => return Err(ElbieError::PatternAlreadyExists(name)),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(pattern)
        }
        Ok(())
    }

    pub(crate) fn get(&self, name: &'static str) -> Result<&Pattern, ElbieError> {
        if let Some(pattern) = self.patterns.get(name) {
            Ok(pattern)
        } else {
            Err(ElbieError::UnknownPattern(name))
        }
    }

    #[track_caller]
    pub(crate) fn case_environment<Callback: Fn(&mut CaseEnvironmentBuilder)>(&mut self, name: &'static str, callback: Callback) -> Result<(), ElbieError> {
        let mut builder = CaseEnvironmentBuilder::new();
        callback(&mut builder);
        let environment = CaseEnvironment { branches: builder.branches(),
                                            defined_at: *Location::caller() };
        match self.case_environments.entry(name.to_owned()) {
            Entry::Occupied(_) => return Err(ElbieError::EnvironmentAlreadyExists(name)),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(environment)
        }
        Ok(())
    }

    pub(crate) fn get_case_environment(&self, name: &'static str) -> Result<&CaseEnvironment, ElbieError> {
        if let Some(pattern) = self.case_environments.get(name) {
            Ok(pattern)
        } else {
            Err(ElbieError::UnknownEnvironment(name))
        }
    }
}
