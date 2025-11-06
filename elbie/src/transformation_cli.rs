use std::collections::HashMap;
use crate::errors::ElbieError;
use crate::transformation::Transformation;
use crate::language::Language;
use crate::cli_functions::TransformationOption;
use crate::cli_functions::transform_words;
use std::path::Path;
use std::ffi::OsStr;

pub(crate) enum Command {
    TransformWords(String,String,Vec<String>,TransformationOption), // words to validate, whether to trace
    ShowUsage
}

pub(crate) struct Arguments {
    command: Command
}

pub(crate) fn parse_args<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args) -> Arguments {
  let mut command = None;
  let mut from_language = None;
  let mut to_language = None;
  let mut transformation_option = TransformationOption::Simple;
  let mut words = Vec::new();


  macro_rules! set_command {
      ($command: expr) => {
        if command.is_some() {
          panic!("Too many commands");
        } else {
          command = Some($command);
        }
      };
  }

  for arg in args {
    match arg.as_ref() {
      "--explain" => {
        transformation_option = TransformationOption::Explain;
      },
      "--trace" => {
        transformation_option = TransformationOption::Trace;
      },
      "--help" => set_command!(Command::ShowUsage),
      from if from_language.is_none() => {
          from_language = Some(from.to_owned());
      },
      to if to_language.is_none() => {
          to_language = Some(to.to_owned())
      },
      words_arg => {
          words.push(words_arg.to_owned());
      }
      //_ => panic!("Unknown command {}",arg.as_ref())

    }
  }

  let command = if let Some(command) = command {
      command
  } else {
      Command::TransformWords(
          from_language.expect("A source language is required for transformation"),
          to_language.expect("A target is required for transformation"),
          words,
          transformation_option
      )
  };

  Arguments {
      command
  }

}

pub(crate) fn show_usage(program: &str, environment: &TransformationEnvironment) {
    println!("usage: {program} [options] <from_language> <to_language> <words>...");
    println!();
    println!("available transformations:");
    let mut keys = environment.transformers.keys().collect::<Vec<_>>();
    keys.sort();
    for (from,to) in keys {
        println!("   {from} -> {to}");
    }
    println!();
    println!("options:");
    println!("   --trace");
    println!("      traces transformation through all rules, and validation through all environment branches.");
    println!("   --explain");
    println!("      same as --trace, but provides detailed explanation of valid phonemes on success, assuming the transformer has a validator.");
    println!("   --help");
    println!("      display this information.");
}

type LanguageCreator = Box<dyn Fn() -> Result<Language, ElbieError>>;
type TransformerCreator = Box<dyn Fn() -> Result<Transformation, ElbieError>>;

#[derive(Default)]
pub struct TransformationEnvironment {
    transformers: HashMap<(&'static str, &'static str),(TransformerCreator,bool)>,
    languages: HashMap<&'static str,LanguageCreator>
}

impl TransformationEnvironment {

    pub fn transformer(&mut self, from: &'static str, to: &'static str, transformer: impl Fn() -> Result<Transformation,ElbieError> + 'static, validate_result: bool) -> Option<(TransformerCreator,bool)> {
        self.transformers.insert((from,to), (Box::new(transformer),validate_result))
    }

    pub fn language(&mut self, language: &'static str, loader: impl Fn() -> Result<Language,ElbieError> + 'static) -> Option<LanguageCreator> {
        self.languages.insert(language, Box::new(loader))
    }

    fn transform_words(&self, from: &str, to: &str, words: Vec<String>, option: &TransformationOption) -> Result<(),ElbieError> {
        let (creator,validate_result) = self.transformers.get(&(from,to)).ok_or_else(|| ElbieError::UnknownTransformation(from.to_owned(),to.to_owned()))?;
        let transformer = (creator)()?;
        let loader = (self.languages.get(from).ok_or_else(|| ElbieError::UnknownLanguage(from.to_owned()))?)()?;
        // validators don't have to be defined.
        let validator = if *validate_result {
            self.languages.get(to).map(|v| v()).transpose()?
        } else {
            None
        };
        transform_words(&transformer, &loader, validator.as_ref(), &words, option);
        Ok(())

    }
}

pub fn run<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, environment: &TransformationEnvironment) {
  let arguments = parse_args(&mut args.skip(1));

  match arguments.command {
      Command::TransformWords(from, to, words, options) => {
         match environment.transform_words(&from, &to, words, &options) {
             Ok(()) => (),
             Err(err) => eprintln!("!!!! {err}"),
        }
      },
      Command::ShowUsage => {
          let exe_name = std::env::current_exe().ok().as_deref().and_then(Path::file_name).map(OsStr::display).as_ref().map(ToString::to_string);
          let program = exe_name.as_deref().unwrap_or("transform");
          show_usage(program,environment)
    },
  }

}
