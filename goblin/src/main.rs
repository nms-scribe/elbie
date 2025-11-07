use std::env;
use elbie::cli;
use elbie::family::Family;
use crate::goblin::GOBLIN;
use crate::goblin::create_goblin_language;



mod phonemes;
mod goblin;

fn main() {
  // TODO: language_cli::run(&mut env::args(),create_goblin_language());
  cli::run_family(&env::args().skip(1).collect::<Vec<_>>(),|| {
      let mut family = Family::default();
      family.default_language(GOBLIN, create_goblin_language)?;
      Ok(family)
  })
}
