use crate::phoneme::Phoneme;
use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use std::rc::Rc;
use thiserror::Error;

#[derive(Clone)]
pub struct ValidPhonemeElement {
    pub found: Rc<Phoneme>,
    pub environment: &'static str,
    pub branch_set: &'static str,
    pub choice_set: &'static str,
    pub next_environment: &'static str
}

#[derive(Clone)]
pub struct ValidInitialPhoneme {
    pub found: Rc<Phoneme>,
    pub choice_set: &'static str,
    pub next_environment: &'static str
}


#[derive(Clone)]
pub enum ValidWordElement {
  Done(usize,&'static str, &'static str), // environment, branch
  InitialPhoneme(usize,ValidInitialPhoneme),
  Phoneme(usize,ValidPhonemeElement) // found phoneme, expected set, expected environment
}

impl Display for ValidWordElement {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::Done(index,environment,branch) => write!(f,"[at {index}]: end of word, environment '{environment}', branch_set '{branch}'."),
      Self::InitialPhoneme(index,ValidInitialPhoneme {
          found,
          choice_set,
          next_environment
      }) => write!(f,"[at {index}]: phoneme ({found} for initial phoneme, choice set '{choice_set}', next environment: '{next_environment}'."),
      Self::Phoneme(index,ValidPhonemeElement {
        found,
        environment,
        branch_set,
        choice_set,
        next_environment
    }) => write!(f,"[at {index}]: phoneme ({found}) for environment '{environment}', branch set '{branch_set}', choice set '{choice_set}', next environment: '{next_environment}'."),
    }

  }
}

#[derive(Debug,Clone,Error)]
pub enum ValidationError {
    #[error("[at {0}]: environment '{2}', branch set '{3}': expected choice set '{4}', found phoneme ({1}).")]
    IncorrectPhoneme(usize, Rc<Phoneme>, &'static str, &'static str, &'static str),
    #[error("[at {0}]: initial environment: expected choice set '{2}', found phoneme ({1}).")]
    IncorrectInitialPhoneme(usize, Rc<Phoneme>, &'static str),
    #[error("[at {0}]: environment '{2}', branch set '{3}': Expected end of word, found phoneme ({1})")]
    ExpectedEndOfWord(usize, Rc<Phoneme>, &'static str, &'static str),
    #[error("[at {0}]: environment '{1}', branch set '{2}': Expected choice set '{3}', found end of word")]
    ExpectedPhonemeFoundEndOfWord(usize, &'static str, &'static str, &'static str),
    #[error("[at {0}]: environment '{2}': Phoneme ({2}) does not match any branch.")]
    NoBranchFitsPhoneme(usize, Rc<Phoneme>, &'static str)

}

pub enum ValidationTraceMessage<'lifetime> {
  FoundValid(&'lifetime ValidWordElement),
  FoundError(&'lifetime ValidationError)
}

impl Display for ValidationTraceMessage<'_> {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::FoundValid(valid) => write!(f,"Found valid: {valid}"),
      Self::FoundError(err) => write!(f,"!!!Found error: {err}"),
    }

  }
}

pub(crate) type ValidationTraceCallback = dyn Fn(usize, ValidationTraceMessage);
