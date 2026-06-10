use crate::errors::ElbieError;
use crate::weighted_vec::WeightedVec;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::panic::Location;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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

impl Display for Sequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for pattern in &self.patterns {
            if first {
                first = false;
            } else {
                write!(f, " + ")?;
            }
            write!(f, "{pattern}")?
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub(crate) struct Series {
    pub pattern: Pattern,
    pub probability: f32,
    pub minimum: usize,
    pub maximum: Option<usize>,
    pub defined_at: Location<'static>
}

impl Display for Series {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { pattern,
                   probability,
                   minimum,
                   maximum,
                   defined_at: _ } = self;
        match (minimum, maximum) {
            (0, None) => write!(f, "{pattern}[{probability}]*"),
            (1.., None) => write!(f, "{pattern}[{probability}]+"),
            (n, Some(max)) => write!(f, "{pattern}[{probability}]{{{n}..{max}}}")
        }
    }
}

#[derive(Debug)]
pub(crate) struct Optional {
    pub pattern: Pattern,
    pub probability: f32,
    pub defined_at: Location<'static>
}

impl Display for Optional {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { pattern,
                   probability,
                   defined_at: _ } = self;
        write!(f, "{pattern}[{probability}]?")
    }
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

impl Display for Choice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for (branch, weight) in self.branches.items() {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }
            write!(f, "{}[{weight}]", branch.body)?
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub(crate) struct AddPhoneme {
    pub name: &'static str,
    pub avoid_duplicates: bool,
    pub defined_at: Location<'static>
}

impl Display for AddPhoneme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.name)?;
        if self.avoid_duplicates {
            write!(f, "[nodup]")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Branch {
    pub condition_set: &'static str,
    pub body: Pattern
}

#[derive(Debug)]
pub(crate) struct TreeBranches {
    pub branches: Vec<Branch>,
    pub defined_at: Location<'static>
}

impl Display for TreeBranches {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for branch in &self.branches {
            if first {
                first = false;
            } else {
                write!(f, "; ")?;
            }
            let Branch { condition_set,
                         body } = branch;
            write!(f, "{condition_set} -> {body}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum NamedOrInlineBranches {
    Inline(TreeBranches),
    Named(&'static str)
}

#[derive(Debug)]
pub(crate) struct Tree {
    pub initial: AddPhoneme,
    pub environment: NamedOrInlineBranches,
    pub defined_at: Location<'static>
}

impl Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.initial)?;
        match &self.environment {
            NamedOrInlineBranches::Inline(tree_branches) => write!(f, "[{tree_branches}]"),
            NamedOrInlineBranches::Named(name) => write!(f, "\"{name}\"")
        }
    }
}

#[derive(Debug)]
pub(crate) struct TerminateWord {
    pub defined_at: Location<'static>
}

impl Display for TerminateWord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "!")
    }
}

#[derive(Debug)]
pub(crate) struct RuleReference {
    pub name: &'static str,
    pub defined_at: Location<'static>
}

impl Display for RuleReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.name)
    }
}

#[derive(Debug)]
pub(crate) enum Pattern {
    Sequence(Sequence),
    Series(Box<Series>),
    Option(Box<Optional>),
    Choice(Choice),
    Tree(Box<Tree>),
    RuleReference(RuleReference),
    Set(AddPhoneme),
    // This can be used to force completion in certain situations, such as not allowing a series to continue, or disallowing an option.
    // If used in a pattern before a non-optional pattern with phonemes, it will fail.
    Terminate(TerminateWord)
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sequence(sequence) => write!(f, "{sequence}"),
            Self::Series(series) => write!(f, "{series}"),
            Self::Option(optional) => write!(f, "{optional}"),
            Self::Choice(choice) => write!(f, "{choice}"),
            Self::Tree(tree) => write!(f, "{tree}"),
            Self::RuleReference(rule_reference) => write!(f, "{rule_reference}"),
            Self::Set(add_phoneme) => write!(f, "{add_phoneme}"),
            Self::Terminate(terminate_word) => write!(f, "{terminate_word}")
        }
    }
}

impl Pattern {
    pub(crate) fn defined_at(&self) -> Location<'static> {
        match self {
            Self::Sequence(sequence) => sequence.defined_at,
            Self::Series(series) => series.defined_at,
            Self::Option(optional) => optional.defined_at,
            Self::Choice(choice) => choice.defined_at,
            Self::Tree(tree) => tree.defined_at,
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

    fn series<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, defined_at: Location<'static>, probability: f32, callback: PatternCallback, minimum: usize, maximum: Option<usize>, extra: Extra) {
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

    fn opt<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, defined_at: Location<'static>, probability: f32, callback: PatternCallback, extra: Extra) {
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

    fn tree_opt<BranchCallback: Fn(&mut TreeBranchesBuilder)>(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, callback: BranchCallback, extra: Extra) {
        let mut builder = TreeBranchesBuilder::new();
        callback(&mut builder);
        let environment = NamedOrInlineBranches::Inline(TreeBranches { branches: builder.branches(),
                                                                       defined_at });

        self.0.push((Pattern::Tree(Box::new(Tree { initial: AddPhoneme { name: initial_phoneme,
                                                                         avoid_duplicates,
                                                                         defined_at },
                                                   environment,
                                                   defined_at })),
                     extra));
    }

    fn branches_opt(&mut self, defined_at: Location<'static>, initial_phoneme: &'static str, avoid_duplicates: bool, environment: &'static str, extra: Extra) {
        let environment = NamedOrInlineBranches::Named(environment);

        self.0.push((Pattern::Tree(Box::new(Tree { initial: AddPhoneme { name: initial_phoneme,
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
    pub fn ser_min_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: f32, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, Some(maximum), ());
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut Self)>(&mut self, probability: f32, callback: PatternCallback, minimum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, None, ());
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut Self)>(&mut self, probability: f32, callback: PatternCallback, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, Some(maximum), ());
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut Self)>(&mut self, probability: f32, callback: PatternCallback) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, None, ());
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut Self)>(&mut self, probability: f32, callback: PatternCallback) {
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
    pub fn tree<BranchCallback: Fn(&mut TreeBranchesBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.tree_opt(*Location::caller(), initial_phoneme, false, callback, ());
    }

    #[track_caller]
    pub fn tree_nodup<BranchCallback: Fn(&mut TreeBranchesBuilder)>(&mut self, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.tree_opt(*Location::caller(), initial_phoneme, true, callback, ());
    }

    #[track_caller]
    pub fn tree_named(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.branches_opt(*Location::caller(), initial_phoneme, false, environment, ());
    }

    #[track_caller]
    pub fn tree_named_nodup(&mut self, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.branches_opt(*Location::caller(), initial_phoneme, true, environment, ());
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
    pub fn ser_min_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: f32, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, Some(maximum), weight);
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: f32, callback: PatternCallback, minimum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, None, weight);
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: f32, callback: PatternCallback, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, Some(maximum), weight);
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: f32, callback: PatternCallback) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, None, weight);
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, weight: usize, probability: f32, callback: PatternCallback) {
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
    pub fn tree<BranchCallback: Fn(&mut TreeBranchesBuilder)>(&mut self, weight: usize, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.tree_opt(*Location::caller(), initial_phoneme, false, callback, weight);
    }

    #[track_caller]
    pub fn tree_nodup<BranchCallback: Fn(&mut TreeBranchesBuilder)>(&mut self, weight: usize, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.tree_opt(*Location::caller(), initial_phoneme, true, callback, weight);
    }

    #[track_caller]
    pub fn tree_named(&mut self, weight: usize, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.branches_opt(*Location::caller(), initial_phoneme, false, environment, weight);
    }

    #[track_caller]
    pub fn tree_named_nodup(&mut self, weight: usize, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.branches_opt(*Location::caller(), initial_phoneme, true, environment, weight);
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
    // allows quickly adding an 'empty' sequence as a choice.
    pub fn empty(&mut self, weight: usize) {
        self.pattern_list.seq(*Location::caller(), |_| {}, weight);
    }

    #[track_caller]
    pub fn done(&mut self, weight: usize) {
        self.pattern_list.done(*Location::caller(), weight);
    }
}

pub struct TreeBranchesBuilder {
    pattern_list: PatternList<&'static str>
}

impl TreeBranchesBuilder {
    const fn new() -> Self {
        Self { pattern_list: PatternList(Vec::new()) }
    }

    fn branches(self) -> Vec<Branch> {
        let mut result = Vec::new();
        for (p, condition_set) in self.pattern_list.0 {
            result.push(Branch {
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
    pub fn ser_min_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: f32, callback: PatternCallback, minimum: usize, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, Some(maximum), condition_set);
    }

    #[track_caller]
    pub fn ser_min<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: f32, callback: PatternCallback, minimum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, minimum, None, condition_set);
    }

    #[track_caller]
    pub fn ser_max<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: f32, callback: PatternCallback, maximum: usize) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, Some(maximum), condition_set);
    }

    #[track_caller]
    pub fn ser<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: f32, callback: PatternCallback) {
        self.pattern_list.series(*Location::caller(), probability, callback, 0, None, condition_set);
    }

    #[track_caller]
    pub fn opt<PatternCallback: Fn(&mut PatternBuilder)>(&mut self, condition_set: &'static str, probability: f32, callback: PatternCallback) {
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
    pub fn tree<BranchCallback: Fn(&mut Self)>(&mut self, condition_set: &'static str, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.tree_opt(*Location::caller(), initial_phoneme, false, callback, condition_set);
    }

    #[track_caller]
    pub fn tree_nodup<BranchCallback: Fn(&mut Self)>(&mut self, condition_set: &'static str, initial_phoneme: &'static str, callback: BranchCallback) {
        self.pattern_list.tree_opt(*Location::caller(), initial_phoneme, true, callback, condition_set);
    }

    #[track_caller]
    pub fn tree_named(&mut self, condition_set: &'static str, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.branches_opt(*Location::caller(), initial_phoneme, false, environment, condition_set);
    }

    #[track_caller]
    pub fn tree_named_nodup(&mut self, condition_set: &'static str, initial_phoneme: &'static str, environment: &'static str) {
        self.pattern_list.branches_opt(*Location::caller(), initial_phoneme, true, environment, condition_set);
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
    pub fn empty(&mut self, condition_set: &'static str) {
        self.pattern_list.seq(*Location::caller(), |_| {}, condition_set);
    }

    #[track_caller]
    pub fn done(&mut self, condition_set: &'static str) {
        self.pattern_list.done(*Location::caller(), condition_set);
    }
}

#[derive(Debug)]
pub(crate) struct PatternSet {
    pub patterns: HashMap<String, Pattern>,
    pub branches: HashMap<String, TreeBranches>,
    pub initial: Pattern
}

impl PatternSet {
    #[track_caller]
    pub(crate) fn new<Callback: Fn(&mut PatternBuilder)>(initial_cb: Callback) -> Self {
        let mut builder = PatternBuilder::new();
        initial_cb(&mut builder);
        let initial = builder.flatten(*Location::caller());
        Self { patterns: HashMap::new(),
               branches: HashMap::new(),
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
    pub(crate) fn named_branches<Callback: Fn(&mut TreeBranchesBuilder)>(&mut self, name: &'static str, callback: Callback) -> Result<(), ElbieError> {
        let mut builder = TreeBranchesBuilder::new();
        callback(&mut builder);
        let environment = TreeBranches { branches: builder.branches(),
                                         defined_at: *Location::caller() };
        match self.branches.entry(name.to_owned()) {
            Entry::Occupied(_) => return Err(ElbieError::EnvironmentAlreadyExists(name)),
            Entry::Vacant(vacant_entry) => _ = vacant_entry.insert(environment)
        }
        Ok(())
    }

    pub(crate) fn get_named_branches(&self, name: &'static str) -> Result<&TreeBranches, ElbieError> {
        if let Some(pattern) = self.branches.get(name) {
            Ok(pattern)
        } else {
            Err(ElbieError::UnknownEnvironment(name))
        }
    }
}
