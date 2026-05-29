use std::env;
use elbie::cli;
use elbie::family::Family;
use crate::goblin::GOBLIN;
use crate::goblin::create_goblin_language;
use crate::hobgoblin::HOBGOBLIN;
use crate::goblin::to_hobgoblin::create_goblin_to_hobgoblin;
use crate::goblin::to_hobgoblin::PLURAL;
use crate::goblin::to_hobgoblin::create_goblin_plural;

mod goblin;
mod hobgoblin;

fn main() {
    cli::run_family(&env::args().skip(1).collect::<Vec<_>>(), || {
        let mut family = Family::default();
        family.default_language(GOBLIN, create_goblin_language)?;
        family.transformation(GOBLIN, HOBGOBLIN, create_goblin_to_hobgoblin)?;
        family.transformation(GOBLIN, PLURAL, create_goblin_plural)?;
        family.transformation_set(GOBLIN, "all", &[PLURAL, HOBGOBLIN])?;
        Ok(family)
    })
}
