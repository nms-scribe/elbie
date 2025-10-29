use crate::errors::LanguageError;
use crate::Phoneme;
use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) enum ValidWordElement {
  Done(usize,&'static str), // environment
  Phoneme(usize,Rc<Phoneme>,&'static str,&'static str) // found phoneme, expected set, expected environment
}

impl Display for ValidWordElement {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    match self {
      Self::Done(index,environment) => write!(f,"[Environment {environment} at {index}]: end of word"),
      Self::Phoneme(index,phoneme,set,environment) => write!(f,"[Environment {environment} at {index}]: phoneme ({phoneme}) from {set}."),
    }

  }
}

pub(crate) enum ValidationTraceMessage<'lifetime> {
  FoundValid(&'lifetime ValidWordElement),
  FoundError(&'lifetime LanguageError)
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
