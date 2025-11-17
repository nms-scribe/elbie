use crate::errors::ElbieError;
use crate::language::Language;
use crate::grid::GridStyle;
use crate::cli_functions::generate_words;
use crate::cli_functions::validate_words;
use crate::cli_functions::show_phonemes;
use crate::cli_functions::show_spelling;
use crate::cli_functions::format_lexicon;
use crate::cli_functions::ValidateOption;
use std::path::Path;
use std::ffi::OsStr;
use std::env;


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
            GridStyle::Markdown |
            GridStyle::CSV => (),
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

pub(crate) fn show_usage(program: &str) {
    println!("usage: {program} [options] [command]");
    println!("default command: --generate 1");
    println!("options:");
    println!("   --format=<plain | terminal | markdown | html | json | csv>");
    println!("      changes the format of grid output. Default is \"plain\" for generate and lexicon, and \"terminal\" for phonemes and spelling.");
    println!("   --no-spans");
    println!("      turns off column and row spanning in headers of grid output.");
    println!("   --comment [String]");
    println!("      prints out the text '<!-- Content auto-generated by {{0}} -->' before any output, where '{{0}}' is replaced by the specified text, or the word 'Elbie'");
    println!("commands:");
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
    println!("   --help");
    println!("      display this information.");
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
            Command::ValidateWords(words,option) => validate_words(&language, words.into_iter(), &option),
            Command::ShowPhonemes(table) => show_phonemes(arguments.grid_style.as_ref(), &language, table.as_ref()),
            Command::ShowSpelling(columns) => show_spelling(arguments.grid_style.as_ref(), &language, columns),
            Command::ProcessLexicon(path,ortho_index) => format_lexicon(arguments.grid_style.as_ref(), &language, &path, ortho_index),
            Command::ShowUsage => {
                let exe_name = env::current_exe().ok().as_deref().and_then(Path::file_name).map(OsStr::display).as_ref().map(ToString::to_string);
                let program = exe_name.as_deref().unwrap_or_else(|| language.name());
                show_usage(program)
            },
        }

      },
      Err(err) => eprintln!("!!! Language Incomplete: {err}")
    }


}
