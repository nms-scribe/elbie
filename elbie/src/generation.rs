use crate::language::Language;
use crate::word::Word;
use rand::rngs::ThreadRng;
use std::rc::Rc;
use crate::phoneme::Phoneme;
use crate::errors::ElbieError;
use crate::phonotactics::Sequence;
use crate::phonotactics::Series;
use crate::phonotactics::Optional;
use crate::phonotactics::Pattern;
use rand::Rng as _;
use crate::phonotactics::Choice;
use crate::phonotactics::Tree;
use crate::phonotactics::AddPhoneme;
use crate::phonotactics::CaseEnvironment;
use crate::phonotactics::Case;
use crate::phonotactics::NamedOrInlineEnvironment;
use crate::phonotactics::TerminateWord;
use crate::phonotactics::RuleReference;
use crate::phonotactics::PatternSet;

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
// [X] Separate this into three modules: patterns.rs, generation.rs, and validation.rs
// [ ] Start converting goblin to making use of the new API, and possibly even some of it's new features.
// [ ] Time to set up rustfmt rules so that I can get this thing easier to contribute to.

// TODO: Get rid of panics and replace with ElbieErrors.

/* NOTE:

Probabilities in the patterns below are marked by u8 instead of f64. To check a probability, a random u8 is generated, and if the value is <= the probability, then it is true. This is slightly more efficient since I'm not bogged down by floating point precission issues, and I don't need to worry about someone adding in values higher than 1.0.

Weight in the patterns below are found in collections of choices, branches, etc. All weights in the patterns must add up to u8::MAX. When an item is chosen, a random number is generated and the items are evaluated in order, adding up their weights, until one's running weight is that value or higher.
*/

fn is_probable(probability: u8, rng: &mut ThreadRng) -> bool {
    rng.random::<u8>() <= probability
}


trait GenerateWord {

    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError>;

}




impl GenerateWord for Sequence {
    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        for pattern in &self.patterns {
            pattern.extend_word(language, rng, is_complete, result)?;
        }
        Ok(())
    }
}


impl GenerateWord for Series {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
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

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        if (!*is_complete) && is_probable(self.probability, rng)  {
            self.pattern.extend_word(language, rng, is_complete, result)
        } else {
            Ok(())
        }
    }

}


impl GenerateWord for Choice {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let branch = self.branches.choose(rng).ok_or(ElbieError::NoChoiceChoices(self.defined_at))?;
        branch.body.extend_word(language, rng, is_complete, result)
    }

}


impl GenerateWord for Tree {

    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let branch = self.branches.choose(rng).ok_or(ElbieError::NoTreeChoices(self.defined_at))?;
        branch.head.extend_word(language, rng, is_complete, result)?;
        branch.tail.extend_word(language, rng, is_complete, result)
    }

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

}

impl GenerateWord for AddPhoneme {
    fn extend_word(&self,language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        _ = self.extend_with_phoneme(language, rng, *is_complete, result)?;
        Ok(())
    }
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

impl GenerateWord for TerminateWord {
    fn extend_word(&self, _: &Language, _: &mut ThreadRng, is_complete: &mut bool, _: &mut Word) -> Result<(),ElbieError> {
        *is_complete = true;
        Ok(())
    }
}

impl GenerateWord for RuleReference {
    fn extend_word(&self, language: &Language, rng: &mut ThreadRng, is_complete: &mut bool, result: &mut Word) -> Result<(),ElbieError> {
        let pattern = language.patterns().get(self.name)?;
        pattern.extend_word(language, rng, is_complete, result)
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

impl PatternSet {

    pub(crate) fn generate(&self, language: &Language, rng: &mut ThreadRng) -> Result<Word,ElbieError> {
        let mut result = Word::new(&[]);

        self.initial.extend_word(language, rng, &mut false, &mut result)?;
        Ok(result)

    }


}
