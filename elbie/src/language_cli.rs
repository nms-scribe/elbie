use std::process;
use crate::errors::ElbieError;
use crate::grid::Cell;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use crate::language::Language;
use crate::validation::ValidationTraceCallback;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::grid::GridStyle;
use crate::word::WordLoader as _;
use crate::validation::WordValidator as _;

pub(crate) enum ValidateOption {
  Simple,
  Explain,
  Trace
}

pub(crate) enum Command {
    GenerateWords(usize), // number of words to generate
    ValidateWords(Vec<String>,ValidateOption), // words to validate, whether to trace
    ShowPhonemes(Option<String>), // specifies the table to show
    ShowSpelling(usize), // specifies number of columns
    ShowUsage,
    ProcessLexicon(String,usize)
}

pub(crate) struct Arguments {
    grid_style: Option<GridStyle>,
    comment: Option<String>,
    command: Command
}

pub(crate) fn parse_args<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args) -> Arguments {
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

pub(crate) fn show_usage(language: &Language) {
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

pub(crate) fn format_lexicon(grid_style: Option<GridStyle>, language: &Language, path: String, ortho_index: usize) {
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

pub(crate) fn show_spelling(grid_style: Option<GridStyle>, language: &Language, columns: usize) {
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

pub(crate) fn show_phonemes(grid_style: Option<&GridStyle>, language: &Language, table: Option<&String>) {
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

pub(crate) fn validate_words(language: &Language, words: Vec<String>, option: &ValidateOption) {
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

pub(crate) fn generate_words(grid_style: Option<&GridStyle>, language: &Language, count: usize) {
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

pub fn run<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args, language: Result<Language,ElbieError>) {
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
