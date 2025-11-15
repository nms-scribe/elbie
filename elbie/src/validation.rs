use crate::errors::ElbieError;
use crate::phoneme::Phoneme;
use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use std::rc::Rc;

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

pub enum ValidationTraceMessage<'lifetime> {
  FoundValid(&'lifetime ValidWordElement),
  FoundError(&'lifetime ElbieError)
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
