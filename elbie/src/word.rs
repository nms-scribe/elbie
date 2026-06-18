use crate::phoneme::Phoneme;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Word {
    phonemes: Vec<Rc<Phoneme>>
}

impl Word {
    pub(crate) fn new(phonemes: &[Rc<Phoneme>]) -> Self {
        let phonemes = phonemes.to_vec();
        Self { phonemes }
    }

    #[must_use]
    pub const fn phonemes(&self) -> &Vec<Rc<Phoneme>> {
        &self.phonemes
    }

    pub(crate) fn push(&mut self, phoneme: Rc<Phoneme>) {
        self.phonemes.push(phoneme)
    }

    pub(crate) fn last(&self) -> Option<&Rc<Phoneme>> {
        self.phonemes.last()
    }

    #[must_use]
    pub fn to_raw_string(&self) -> String {
        format!("{}", RawWord(self))
    }

    #[must_use]
    pub const fn to_raw_display(&'_ self) -> RawWord<'_> {
        RawWord(self)
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "/")?;
        write!(f, "{}", RawWord(self))?;
        write!(f, "/")?;
        Ok(())
    }
}

pub struct RawWord<'word>(&'word Word);

impl Display for RawWord<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for phoneme in &self.0.phonemes {
            write!(f, "{}", phoneme.name)?
        }
        Ok(())
    }
}

impl From<Vec<Rc<Phoneme>>> for Word {
    fn from(phonemes: Vec<Rc<Phoneme>>) -> Self {
        Self { phonemes }
    }
}
