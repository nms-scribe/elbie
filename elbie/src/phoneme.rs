use core::fmt;
use core::fmt::Formatter;
use core::fmt::Display;
use std::rc::Rc;

#[derive(Debug,Ord,PartialOrd,Eq,PartialEq,Hash)]
pub struct Phoneme {
  pub name: &'static str
}

impl Phoneme {
  pub(crate) fn new(name: &'static str) -> Rc<Self> {
    Rc::new(Self {
      name
    })
  }

}

impl Display for Phoneme {

  fn fmt(&self, f: &mut Formatter) -> Result<(),fmt::Error> {
    write!(f,"/{}/",self.name)
  }

}
