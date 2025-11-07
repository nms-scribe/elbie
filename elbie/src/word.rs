use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use crate::phoneme::Phoneme;
use std::rc::Rc;

#[derive(Debug,Clone)]
pub struct Word {
  phonemes: Vec<Rc<Phoneme>>
}

impl Word {
  pub(crate) fn new(phonemes: &[Rc<Phoneme>]) -> Self {
    let phonemes = phonemes.to_vec();
    Self{phonemes}
  }

  pub(crate) const fn phonemes(&self) -> &Vec<Rc<Phoneme>> {
      &self.phonemes
  }


  pub(crate) fn push(&mut self,phoneme: Rc<Phoneme>) {
    self.phonemes.push(phoneme)
  }

  pub(crate) fn _last(&self) -> Option<&Rc<Phoneme>> {
    self.phonemes.last()
  }

}

impl Display for Word {
  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    write!(f,"/")?;
    for phoneme in &self.phonemes {
      write!(f,"{}",phoneme.name)?
    }
    write!(f,"/")?;
    Ok(())
  }

}

impl From<Vec<Rc<Phoneme>>> for Word {
    fn from(phonemes: Vec<Rc<Phoneme>>) -> Self {
        Self {
            phonemes
        }
    }
}
