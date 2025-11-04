use crate::errors::ElbieError;
use crate::phoneme::Phoneme;
use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use std::rc::Rc;
use crate::word::Word;

#[derive(Clone)]
pub enum ValidWordElement {
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

pub trait WordValidator {

    fn check_word(&self,word: &Word, trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,ElbieError>;

}

impl WordValidator for Box<dyn WordValidator> {
    fn check_word(&self,word: &Word, trace: &ValidationTraceCallback) -> Result<Vec<ValidWordElement>,ElbieError> {
        self.as_ref().check_word(word, trace)
    }
}
