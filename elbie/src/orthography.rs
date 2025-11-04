use core::fmt;
use core::fmt::Formatter;
use core::slice::Iter;
use core::iter::Peekable;
use crate::language::Language;
use crate::phoneme::Phoneme;
use std::rc::Rc;

pub(crate) type SpellingCallback = fn(&Language, &Rc<Phoneme>, &mut String, Option<&mut Peekable<Iter<Rc<Phoneme>>>>);

#[derive(Default)]
pub(crate) enum SpellingBehavior {
  #[default]
  Default, // default behavior is to spell the phoneme
  Text(&'static str),
  Callback(SpellingCallback)
}

impl fmt::Debug for SpellingBehavior {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    write!(f,"PhonemeBehavior::")?;
    match self {
      Self::Default => write!(f,"Default"),
      Self::Text(text) => write!(f,"Text({text})"),
      Self::Callback(_) => write!(f,"Callback(<...>)"),
    }

  }
}
