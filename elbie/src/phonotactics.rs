use crate::weighted_vec::WeightedVec;
use std::panic::Location;
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

#[derive(Debug,Clone)]
#[deprecated(since="0.4.0",note="Please use patterns instead.")]
pub enum EnvironmentChoice {
  Done,
  Continuing(&'static str,&'static str,bool),// set to generate next phoneme from, next environment to follow, whether to allow duplicate phoneme to be generated
}

#[derive(Debug,Clone)]
#[deprecated(since="0.4.0",note="Please use patterns instead.")]
#[allow(deprecated)]
pub struct EnvironmentBranch(&'static str, WeightedVec<EnvironmentChoice>);

#[allow(deprecated)]
impl EnvironmentBranch {

  #[must_use]
  pub fn new(set_check: &'static str, choices: &[(EnvironmentChoice,usize)]) -> Self {
    let mut vec = WeightedVec::new();
    for choice in choices {
      vec.push(choice.0.clone(),choice.1)
    };
    Self(set_check,vec)

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
pub(crate) struct TreeBranch {
    pub head: Pattern,
    pub tail: Pattern
}


#[derive(Debug)]
pub(crate) struct Tree {
    pub branches: WeightedVec<TreeBranch>,
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
    pub body: Pattern,
}

#[derive(Debug)]
pub(crate) struct CaseEnvironment {
    pub branches: Vec<CaseEnvironmentBranch>,
    pub else_: Pattern,
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
    Tree(Tree),
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
            Self::Tree(tree) => tree.defined_at,
            Self::Case(case) => case.defined_at,
            Self::RuleReference(reference) => reference.defined_at,
            Self::Set(add_phoneme) => add_phoneme.defined_at,
            Self::Terminate(terminate_word) => terminate_word.defined_at,
        }
    }

}



pub struct PatternBuilder {
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
    pub fn add<BodyCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, body_cb: BodyCallback) {
        let defined_at = *Location::caller();

        let mut head = PatternBuilder::new();
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
    pub fn add<HeadCallback: Fn(&mut PatternBuilder), TailCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, head_cb: HeadCallback, tail_cb: TailCallback) {
        let defined_at = *Location::caller();

        let mut head = PatternBuilder::new();
        head_cb(&mut head);
        let mut tail = PatternBuilder::new();
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
    pub fn branch<Callback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, body_cb: Callback) {
        let defined_at = *Location::caller();

        let mut body = PatternBuilder::new();
        body_cb(&mut body);
        self.branches.push(CaseEnvironmentBranch {
            condition_set,
            body: body.flatten(defined_at),
            //defined_at,
        })
    }

    #[track_caller]
    pub fn else_<Callback: Fn(&mut PatternBuilder)>(&mut self, body_cb: Callback) {
        let defined_at = *Location::caller();

        let mut body = PatternBuilder::new();
        body_cb(&mut body);
        self.else_ = Some(body.flatten(defined_at))
    }

}

#[derive(Debug)]
pub(crate) struct PatternSet {
    pub patterns: HashMap<String,Pattern>,
    pub case_environments: HashMap<String,CaseEnvironment>,
    pub initial: Pattern
}

impl PatternSet {

    #[track_caller]
    pub(crate) fn new<Callback: Fn(&mut PatternBuilder)>(initial_cb: Callback) -> Self {
        let mut builder = PatternBuilder::new();
        initial_cb(&mut builder);
        let initial = builder.flatten(*Location::caller());
        Self {
            patterns: HashMap::new(),
            case_environments: HashMap::new(),
            initial
        }

    }

    #[track_caller]
    pub(crate) fn pattern<Callback: Fn(&mut PatternBuilder)>(&mut self, name: &'static str, callback: Callback) {
        let mut builder = PatternBuilder::new();
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

    pub(crate) fn get_case_environment(&self, name: &'static str) -> Result<&CaseEnvironment,ElbieError> {
        if let Some(pattern) = self.case_environments.get(name) {
            Ok(pattern)
        } else {
            Err(ElbieError::UnknownEnvironment(name))
        }
    }

}
