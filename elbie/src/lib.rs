pub use crate::errors::LanguageError;
pub use crate::language::Language;
pub use crate::phoneme::Phoneme;
pub use crate::phoneme_table::TableOption;
pub use crate::phoneme_table::Axis;
pub use crate::phoneme_table::HeaderDef;
pub use crate::phoneme_table_builder::TableBuilder;
pub use crate::phonotactics::EnvironmentBranch;
pub use crate::phonotactics::EnvironmentChoice;
pub use crate::language::EMPTY;
pub use crate::language::PHONEME;
pub use crate::word::Word;
use crate::grid::Cell;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::grid::GridStyle;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use crate::phoneme_table::Table0D;
use crate::phoneme_table::Table0DDef;
use crate::phoneme_table::Table1D;
use crate::phoneme_table::Table2D;
use crate::phoneme_table::Table3D;
use crate::phoneme_table::Table4D;
use crate::phoneme_table::Table4DDef;
use crate::phoneme_table::TableDef;
use crate::validation::ValidationTraceCallback;
use std::process;

mod phoneme_table;
mod grid;
mod lexicon;
mod errors;
mod bag;
mod weighted_vec;
mod phoneme;
mod orthography;
mod phoneme_behavior;
mod word;
mod phonotactics;
mod validation;
mod phoneme_table_builder;
mod language;
#[cfg(test)] mod test;




/*
Elbie = LB, Language Builder, and is a bunch of tools for building a constructed language.
*/

/* TODO:

TODO: First, clean up cargo clippy, then publish.

TODO: This will be easier if I can separate the phoneme/set stuff from the language, as I will need a similar non-language structure to do that.

Transformations, possibly:

A LanguageTransformer basically looks like this:

struct Transformer
  phonemes: HashMap<str,Rc<Phoneme>>
  sets: HashMap<str,Bag<Rc<Phoneme>>>
  rules: Vec<(Vec<Match>,Vec<Replace>)>

Transformer::from_language(Language) -> Self
-- copies sets and phonemes from the specified language and inserts them into Namespaced entities. (str,str) where the first is the language and the second is the value.

Transformer::add_target_language(&mut self, Language)
-- copies sets and phonemes from the target language, into namespaces
-- Note that this is optional, as Transformers could be used to implement orthography instead
-- more than one language can also be used, which allows the user to transform *through* another language.

Transformer::add_phoneme(&mut self, Rc<Phoneme>, namespace: str, sets: [(str)])
-- adds a specified phoneme to a temporary namespace with the specified sets (the namespace can not be the same as one of the languages)

Transformer::build_*
-- similar to all of the build_* sets in the languages.

Transformer::add_rule(Match)
--- see below for Matches and Replaces
--- adds a "transformation" rule.

Transformer::transform(Word,trace) -> Word
-- applies each rule, one at a time and in order, to the word, transforming it as it goes, and returns the specified word
-- if the pattern matches, any Captures in the rule are replaced with the Replace expression. There must be one Replace expression for each Capture, all phonemes in the replace will be inserted where the capture was found.

struct Match
  initial: indicates that the match must occur at the beginning of the word. The matcher will not continue on to see other phonems in the word if the first phoneme doesn't match the first pattern
  final: indicates that the match must occur at the end of the word. The matcher will not match if the end of the word is not reached when the pattern is matched.
  patterns: Vec<Pattern>

enum Pattern
  Match(name,Usize,NonZeroUsize) -- matches from x to y phonemes of the specified set or phoneme name if the set doesn't exist, a match can match 0 phonemes.
  Replace(name,Usize,NonZeroUsize,Replace) -- "captures" from x to y phonemes of the specified set or phoneme name, if a capture has a min of 0, then it will always capture there, and any replace expression is inserted in it's place.
  -- repeats are built into the enums, so I don't need anything special
  -- sequences are part of the pattern
  -- choices can be handled two ways:
     -- a choice of a single phoneme can be a set
     -- a choice of two different sequences can be a new rule.

enum Replace
  Sequence(Vec<Rc<Phoneme>>) -- replaces with specified phoneme
  ConvertSet(Vec<str>,Vec<str>) -- Extracts the sets from the phoneme, removes the specified sets, and then adds the new sets. It then looks for a single phoneme in the new set and replaces the phoneme with that one. Error if not exactly 1 phoneme is returned, or if the original phoneme doesn't have the source sets. Note that sets can be empty in source or target.
  SwitchSet(str,str) -- switches a "set" from one to the other
  SetLanguage(str) -- switches the phoneme from it's source to the same phoneme in the target language, error if it doesn't exist in the new language


*/

/*
FUTURE: Implementing syllable breaks, stress, etc, Simple Solution:
- a "word" is sequence of syllables, not phonemes. A syllable is a sequence of phonemes. I don't think we need to support onset/rhyme structure, since that could be analyzed differently. In fact, some languages might not be able to analyze syllables, in which case each word would have to be one big syllable.
- A syllable can also have stress, tone, etc.
- spelling callbacks are the hardest part to deal with, but I'm not sure these are great anyway. Spelling might be a type of transformation.
- Another difficulty is "converting" old words, which won't have the syllable breaks and stress indicators. The best thing I can think of is to have the validators guess when a syllable break is missing, and warn about modifiers missing without stopping the process.

There have been arguments against syllables being a real thing, but I feel like their usage in analysis is big enough that I can still use them.
https://web.archive.org/web/20150923211920/http://www.cunyphonologyforum.net/syllable.php
https://web.archive.org/web/20150918220252/http://cunyphonologyforum.wikifoundry.com/page/Paraphonological+Phenomena

FUTURE: Implement transformations:
* regular sound change for building lexicons of daughter languages
* regular sound changes for loan words from other languages (I don't expect this to be common)
* orthography -- the same pattern matching of sound change could potentially be used to create more realistic orthography
- This is mostly something very similar to regular expressions, searching for patterns in a word, possibly capturing some patterns, and replacing them with other patterns. The final test, however, would require validation to a new language, or something like that.

FUTURE: Is there some way to use types or something else to make languages easier to create?
- One issue is the use of string constants to identify environments, sets, phonemes, etc.
  - There is a small possibility that I could repeat the string name under two different constant names, which could cause some hard to debug issues.
  - The use of a string constant removes some useful type-checking: if I specify an environment name instead of a set name, I don't know until run-time.
  - It would be nice if I could just have "phoneme" and "phoneme_set" objects and the like that can be reference by variable, and have internal access to the language they are associated with. (For example, "fricative.intersect_with(glottal)" should work without having to retrieve things off of the language, or even without having a string name)
- Constant type parameters are now possible in rust, there might be something I could use out of that.

// FUTURE: Is there some way I can do the environments and sets as types? Maybe phonemes, sets and environments are traits instead that you implement in structs. I might be able to use generic constant parameters to help with that.
// I could use macros to make those implementations easier to code. Phonemes should really be enumerations. This would require the language to be generic
// and base itself off of phonemes. --- I think the hardest part is implementing a set that describes which phonemes can be chosen, and then to choose such a
// type randomly?


*/




enum ValidateOption {
  Simple,
  Explain,
  Trace
}

enum Command {
    GenerateWords(usize), // number of words to generate
    ValidateWords(Vec<String>,ValidateOption), // words to validate, whether to trace
    ShowPhonemes(Option<String>), // specifies the table to show
    ShowSpelling(usize), // specifies number of columns
    ShowUsage,
    ProcessLexicon(String,usize)
}

struct Arguments {
    grid_style: Option<GridStyle>,
    comment: Option<String>,
    command: Command
}

fn parse_args<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args) -> Arguments {
  let mut command = None;
  let mut grid_style = None;
  let mut spanning = true;
  let mut comment = None;

  macro_rules! set_grid_style {
      ($style: expr) => {
        if grid_style.is_some() {
          panic!("Too many grid styles");
        } else {
          grid_style = Some($style);
        }
      };
  }

  macro_rules! set_command {
      ($command: expr) => {
        if command.is_some() {
          panic!("Too many commands");
        } else {
          command = Some($command);
        }
      };
  }

  while let Some(arg) = args.next() {
    match arg.as_ref() {
      "--format=plain" => set_grid_style!(GridStyle::Plain),
      "--format=terminal" => set_grid_style!(GridStyle::Terminal{ spans: spanning }),
      "--format=markdown" => set_grid_style!(GridStyle::Markdown),
      "--format=html" => set_grid_style!(GridStyle::HTML { spans: spanning }),
      "--format=json" => set_grid_style!(GridStyle::JSON),
      "--no-spans" => if let Some(style) = &mut grid_style {
          match style {
            GridStyle::Plain |
            GridStyle::JSON |
            GridStyle::Markdown => (),
            GridStyle::Terminal { spans } |
            GridStyle::HTML { spans } => if *spans {
                *spans = false
            } else {
                panic!("--no-spans specified twice")
            }
        }
      } else {
          spanning = false
      },
      "--comment" => {
          if let Some(text) = args.next() {
              comment = Some(text.as_ref().to_owned())
          } else {
              comment = Some("Elbie".to_owned())
          }
      },
      "--generate" => set_command!(Command::GenerateWords(args.next().expect("Generate count required").as_ref().parse().expect("Argument should be a number"))),
      "--validate" => {
        let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
        words.extend(args.map(|x| x.as_ref().to_owned()));
        set_command!(Command::ValidateWords(words,ValidateOption::Simple));
      },
      "--validate=explain" => {
        let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
        words.extend(args.map(|x| x.as_ref().to_owned()));
        set_command!(Command::ValidateWords(words,ValidateOption::Explain));
      },
      "--validate=trace" => {
        let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
        words.extend(args.map(|x| x.as_ref().to_owned()));
        set_command!(Command::ValidateWords(words,ValidateOption::Trace));
      },
      "--phonemes" => set_command!(Command::ShowPhonemes(None)),
      a if a.starts_with("--phonemes=") => set_command!(Command::ShowPhonemes(Some(a.trim_start_matches("--phonemes=").to_owned()))),
      "--spelling" => set_command!(Command::ShowSpelling(1)),
      a if a.starts_with("--spelling=") => set_command!(Command::ShowSpelling(a.trim_start_matches("--spelling=").parse::<usize>().expect("Parameter should be a number").clamp(1,usize::MAX))),
      "--lexicon" => {
        let path = args.next().expect("No lexicon filename given").as_ref().to_owned();
        let spelling_index = args.next().expect("No orthography index given").as_ref().parse().expect("orthography index must be a number");
        set_command!(Command::ProcessLexicon(path,spelling_index))
      },
      "--help" => set_command!(Command::ShowUsage),
      _ => panic!("Unknown command {}",arg.as_ref())

    }
  }

  Arguments {
      grid_style,
      comment,
      command: command.unwrap_or(Command::GenerateWords(1))
  }

}


fn show_usage<const ORTHOGRAPHIES: usize>(language: &Language<ORTHOGRAPHIES>) {
    println!("usage: {} [options] <command>",language.name());
    println!("default command: --generate 1");
    println!("options:");
    println!("   --format=<plain | terminal | markdown | html | json | csv>");
    println!("      changes the format of grid output. Default is \"plain\" for generate and lexicon, and \"terminal\" for phonemes and spelling.");
    println!("commands:");
    println!("   --no-spans");
    println!("      turns off column and row spanning in headers of grid output.");
    println!("   --generate <integer>");
    println!("      generates the specified number of words.");
    println!("   --validate <words>...");
    println!("      validates the specified words (verifies that it is possible to generate them).");
    println!("   --validate=trace <words>...");
    println!("      same as --validate, but traces the validation through all environment branches.");
    println!("   --validate=explain <words>...");
    println!("      same as --validate, but provides detailed explanation of valid phonemes on success.");
    println!("   --phonemes");
    println!("      prints out the phonemes of the language.");
    println!("   --phonemes=<table>");
    println!("      prints out one table of phonemes of the language.");
    println!("   --spelling");
    println!("      prints out the orthographies of the language.");
    println!("   --spelling=<2..>");
    println!("      prints spelling table in multiple columns.");
    println!("   --lexicon <path>");
    println!("      validates lexicon and outputs into a LaTeX file.");
    println!("   --comment [String]");
    println!("      prints out the text '<!-- Content auto-generated by {{0}} -->' before any output, where '{{0}}' is replaced by the specified text, or the word 'Elbie'");
    println!("   --help");
    println!("      display this information.");
}

fn format_lexicon<const ORTHOGRAPHIES: usize>(grid_style: Option<GridStyle>, language: &Language<ORTHOGRAPHIES>, path: String, ortho_index: usize) {
  if ortho_index >= language.orthographies().len() {
        panic!("Language only has {} orthographies.",language.orthographies().len())
  }

  let grid_style = grid_style.unwrap_or(GridStyle::Plain);

  match language.load_lexicon(path,ortho_index) {
    Ok(lexicon) => {
        let result = lexicon.into_string(&grid_style);
        println!("{result}")

    },
    Err(err) => {
      eprintln!("!!! Couldn't process lexicon: {err}");
      process::exit(1)
    }
  }
}

fn show_spelling<const ORTHOGRAPHIES: usize>(grid_style: Option<GridStyle>, language: &Language<ORTHOGRAPHIES>, columns: usize) {
    match language.display_spelling(columns) {
        Ok(grid) => {
            grid.into_output(&grid_style.unwrap_or(GridStyle::Terminal { spans: false })).print_to_stdout();
        },
        Err(err) => {
            eprintln!("!!! Couldn't display spelling: {err}");
            process::exit(1)
        }
    }
}

fn show_phonemes<const ORTHOGRAPHIES: usize>(grid_style: Option<&GridStyle>, language: &Language<ORTHOGRAPHIES>, table: Option<&String>) {
    let style = grid_style.unwrap_or(&GridStyle::Terminal{ spans: true });
    let result = match table {
        Some(table) => match language.build_phoneme_table(table) {
            Ok(Some(grid)) => {
                grid.into_output(style).print_to_stdout();
                Ok(())
            },
            Ok(None) => {
                eprintln!("No phoneme table named {table}. Try singular?");
                Ok(())
            }
            Err(err) => Err(err),
        },
        None => match language.build_all_phoneme_tables() {
            Ok(grids) => {
                for grid in grids {
                    println!("{}",grid.1.caption());
                    grid.1.into_output(style).print_to_stdout();
                    println!();

                }

                Ok(())
            },
            Err(err) => Err(err),
        },
    };

    if let Err(err) = result {
        eprintln!("!!! Couldn't display phonemes: {err}");
        process::exit(1)

    }
}

fn validate_words<const ORTHOGRAPHIES: usize>(language: &Language<ORTHOGRAPHIES>, words: Vec<String>, option: &ValidateOption) {
    let mut invalid_found = false;
    for word in words {
        match language.read_word(&word) {
            Ok(word) => {
                let trace_cb: Box<ValidationTraceCallback> = if matches!(option,ValidateOption::Trace) {
                  Box::new(|level,message| {
                    /* eat message, no need to report */
                    println!("{}{}",str::repeat(" ",level*2),message);
                   })
                } else {
                  Box::new(|_,_| {})
                };
                match language.check_word(&word,&trace_cb) {
                    Err(err) => {
                      invalid_found = true;
                      if matches!(option,ValidateOption::Trace) {
                        println!("!!!! invalid word (see trace)");
                      } else {
                        println!("{err}");
                      }
                    },
                    Ok(validated) => {
                      if matches!(option,ValidateOption::Explain) {
                        for valid in validated {
                          println!("{valid}")
                        }
                      }

                      for orthography in 0..language.orthographies().len() {
                        print!("{} ",language.spell_word(&word,orthography));
                      }
                      println!("{word}");
                    }
                }
            },
            Err(err) => {
                eprintln!("!!!! Can't read word: {err}");
                process::exit(1);
            }
        }
    }
    if invalid_found {
      process::exit(1);
    }
}

fn generate_words<const ORTHOGRAPHIES: usize>(grid_style: Option<&GridStyle>, language: &Language<ORTHOGRAPHIES>, count: usize) {
    let mut grid = Grid::new(TableClass::ElbieWords, format!("Generated {count} words for {}",language.name()));

    // FUTURE: Should I have a header?

    for _ in 0..count {
        let mut row = GridRow::new(TRBodyClass::BodyRow);

        match language.make_word() {
            Ok(word) => {
                for orthography in 0..language.orthographies().len() {
                    row.push_cell(Cell::content(language.spell_word(&word,orthography),None));
                }
                row.push_cell(Cell::content(format!("{word}"),None));

                // the following is a sanity check. It might catch some logic errors, but really it's just GIGO.
                if let Err(err) = language.check_word(&word,&|_,_| { /* eat message, no need to report */}) {
                    eprintln!("-- !!!! invalid word: {err}");
                    process::exit(1);
                }
            },
            Err(err) => {
                eprintln!("!!! Couldn't make word: {err}");
                process::exit(1);
            }
        }

        grid.push_body_row(row);
    }
    grid.into_output(grid_style.unwrap_or(&GridStyle::Plain)).print_to_stdout();
}


pub fn run_main<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, language: Result<Language<ORTHOGRAPHIES>,LanguageError>) {
  let arguments = parse_args(&mut args.skip(1));

  if let Some(comment) = arguments.comment {
      println!("<!-- Content auto-generated by {comment} -->")
  }

  match language {
      Ok(language) => {

        match arguments.command {
            Command::GenerateWords(count) => generate_words(arguments.grid_style.as_ref(), &language, count),
            Command::ValidateWords(words,option) => validate_words(&language, words, &option),
            Command::ShowPhonemes(table) => show_phonemes(arguments.grid_style.as_ref(), &language, table.as_ref()),
            Command::ShowSpelling(columns) => show_spelling(arguments.grid_style, &language, columns),
            Command::ProcessLexicon(path,ortho_index) => format_lexicon(arguments.grid_style, &language, path, ortho_index),
            Command::ShowUsage => show_usage(&language),
        }

      },
      Err(err) => eprintln!("!!! Language Incomplete: {err}")
    }


}
