use crate::errors::ElbieError;
use crate::language::Language;
use crate::phoneme::Phoneme;
use crate::phonotactics::AddPhoneme;
use crate::phonotactics::Choice;
use crate::phonotactics::NamedOrInlineBranches;
use crate::phonotactics::Optional;
use crate::phonotactics::Pattern;
use crate::phonotactics::PatternSet;
use crate::phonotactics::RuleReference;
use crate::phonotactics::Sequence;
use crate::phonotactics::Series;
use crate::phonotactics::TerminateWord;
use crate::phonotactics::Tree;
use crate::phonotactics::TreeBranches;
use crate::word::Word;
use rand::Rng as _;
use rand::rngs::ThreadRng;
use std::rc::Rc;

// TODO: Time to set up rustfmt so that I can make it easier to contribute to. As long as I can check the config into git to force users to use the same. And also, find some way to force it to run before a git commit, but not on every save. (Although, would it really be bad to do on every save? As long as rustfmt isn't using AI, right?)

/* NOTE:

Probabilities in the patterns below are marked by u8 instead of f64. To check a probability, a random u8 is generated, and if the value is <= the probability, then it is true. This is slightly more efficient since I'm not bogged down by floating point precission issues, and I don't need to worry about someone adding in values higher than 1.0.

*/

fn is_probable(probability: f32, rng: &mut ThreadRng) -> bool {
    rng.random_range(0.0..1.0) <= probability
    //rng.random::<u8>() <= probability
}

trait GenerateWord {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError>;
}

impl GenerateWord for Sequence {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        for pattern in &self.patterns {
            pattern.extend_word(language, rng, is_complete, result)?;
        }
        Ok(())
    }
}

impl GenerateWord for Series {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        for _ in 0..self.minimum {
            self.pattern.extend_word(language, rng, is_complete, result)?;
        }
        let mut i = self.minimum;
        while (!*is_complete) && is_probable(self.probability, rng) && self.maximum.is_none_or(|max| i < max) {
            self.pattern.extend_word(language, rng, is_complete, result)?;
            i += 1;
        }
        Ok(())
    }
}

impl GenerateWord for Optional {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        if (!*is_complete) && is_probable(self.probability, rng) {
            self.pattern.extend_word(language, rng, is_complete, result)
        } else {
            Ok(())
        }
    }
}

impl GenerateWord for Choice {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        let branch = self.branches.choose(rng).ok_or(ElbieError::NoChoiceChoices(self.defined_at))?;
        branch.body.extend_word(language, rng, is_complete, result)
    }
}

impl AddPhoneme {
    fn extend_with_phoneme(&self, language: &Language, rng: &mut ThreadRng, is_complete: bool, result: &mut Word) -> Result<Rc<Phoneme>, ElbieError> {
        if is_complete {
            return Err(ElbieError::PhonemeAfterTerminate);
        }
        let phoneme = if self.avoid_duplicates
                         && let Some(phoneme) = result.last()
        {
            language.inventory().choose_except(self.name, &[phoneme], rng)?
        } else {
            language.inventory().choose(self.name, rng)?
        };

        result.push(phoneme.clone());
        Ok(phoneme)
    }
}

impl GenerateWord for AddPhoneme {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        _ = self.extend_with_phoneme(language, rng, *is_complete, result)?;
        Ok(())
    }
}

impl TreeBranches {
    // not a GeneratePattern trait because it requires the phoneme information that was just added.
    fn extend_word(&self, phoneme: &Rc<Phoneme>, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        for branch in &self.branches {
            if language.inventory().phoneme_is(phoneme, branch.condition_set)? {
                return branch.body.extend_word(language, rng, is_complete, result);
            }
        }
        Err(ElbieError::NoCatchAllInEnvironment(self.defined_at))
    }
}

impl GenerateWord for Tree {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        let phoneme = self.initial.extend_with_phoneme(language, rng, *is_complete, result)?;
        let environment = match &self.environment {
            NamedOrInlineBranches::Inline(environment) => environment,
            NamedOrInlineBranches::Named(name) => language.patterns().get_named_branches(name)?
        };
        environment.extend_word(&phoneme, language, rng, is_complete, result)
    }
}

impl GenerateWord for TerminateWord {
    fn extend_word(&self, _: &Language, _: &mut ThreadRng, is_complete: &mut bool, _: &mut Word) -> Result<(), ElbieError> {
        *is_complete = true;
        Ok(())
    }
}

impl GenerateWord for RuleReference {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        let pattern = language.patterns().get(self.name)?;
        pattern.extend_word(language, rng, is_complete, result)
    }
}

impl GenerateWord for Pattern {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(), ElbieError> {
        match self {
            Self::Sequence(sequence) => sequence.extend_word(language, rng, is_complete, result),
            Self::Series(series) => series.extend_word(language, rng, is_complete, result),
            Self::Option(optional) => optional.extend_word(language, rng, is_complete, result),
            Self::Choice(choice) => choice.extend_word(language, rng, is_complete, result),
            Self::Tree(switch) => switch.extend_word(language, rng, is_complete, result),
            Self::RuleReference(reference) => reference.extend_word(language, rng, is_complete, result),
            Self::Set(set) => set.extend_word(language, rng, is_complete, result),
            Self::Terminate(terminate) => terminate.extend_word(language, rng, is_complete, result)
        }
    }
}

#[allow(clippy::multiple_inherent_impl, reason = "I want to separate validation and generation from the patterns")]
impl PatternSet {
    pub(crate) fn generate(&self, language: &Language, rng: &mut ThreadRng) -> Result<Word, ElbieError> {
        let mut result = Word::new(&[]);

        self.initial.extend_word(language, rng, &mut false, &mut result)?;
        Ok(result)
    }
}
