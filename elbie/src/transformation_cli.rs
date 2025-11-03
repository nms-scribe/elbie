use std::collections::HashMap;
use std::process;
use crate::errors::LanguageError;
use crate::transformation::Transformation;
use crate::validation::ValidationTraceCallback;
use crate::word::WordLoader;
use crate::validation::WordValidator;
use crate::transformation::TransformationTraceCallback;

pub(crate) enum TransformationOption {
  Simple,
  Explain,
  Trace
}

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

  while let Some(arg) = args.next() {
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

pub(crate) fn show_usage(environment: TransformationEnvironment) {
    println!("usage: transform [options] <from_language> <to_language> <words>...");
    println!();
    println!("available transformations:");
    for (from,to) in environment.transformers.keys() {
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

fn transform_words(transformation: &Transformation, loader: &impl WordLoader, validator: Option<&impl WordValidator>, words: Vec<String>, option: &TransformationOption) {
    let mut invalid_found = false;


    let validation_trace_cb: Box<ValidationTraceCallback> = if matches!(option,TransformationOption::Trace) {
      Box::new(|level,message| {
        /* eat message, no need to report */
        println!("{}{}",str::repeat(" ",level*2),message);
      })
    } else {
      Box::new(|_,_| {})
    };

    let transformation_trace_cb: Box<TransformationTraceCallback> = if matches!(option,TransformationOption::Trace) {
        Box::new(|message| {
            println!("{message}")
        })
    } else {
        Box::new(|_| {})
    };


    for word in words {
        match loader.read_word(&word) {
            Ok(word) => {
                match transformation.transform(word, &transformation_trace_cb) {
                    Ok(word) => {
                        if let Some(validator) = validator {
                            match validator.check_word(&word, &validation_trace_cb) {
                                Ok(validated) => {
                                  if matches!(option,TransformationOption::Explain) {
                                    for valid in validated {
                                      println!("{valid}")
                                    }
                                  }

                                },
                                Err(err) => {
                                    invalid_found = true;
                                    if matches!(option,TransformationOption::Trace) {
                                        println!("!!!! invalid word (see trace)")
                                    } else {
                                        eprintln!("{err}")
                                    }
                                },
                            }

                        }
                        println!("{word}");
                    },
                    Err(err) => {
                        invalid_found = true;
                        eprintln!("!!!! transformation error: {err}")
                    },
                }
            },
            Err(err) => {
                invalid_found = true;
                eprintln!("!!!! can't read word: {err}")
            },
        }
    }
    if invalid_found {
        process::exit(1)
    }


}

#[derive(Default)]
pub struct TransformationEnvironment {
    transformers: HashMap<(&'static str, &'static str),Box<dyn Fn() -> Result<Transformation,LanguageError>>>,
    word_loaders: HashMap<&'static str,Box<dyn Fn() -> Result<Box<dyn WordLoader>,LanguageError>>>,
    word_validators: HashMap<&'static str,Box<dyn Fn() -> Result<Box<dyn WordValidator>,LanguageError>>>,
}

impl TransformationEnvironment {

    pub fn transformer(&mut self, from: &'static str, to: &'static str, transformer: impl Fn() -> Result<Transformation,LanguageError> + 'static) -> Option<Box<dyn Fn() -> Result<Transformation, LanguageError> + 'static>> {
        self.transformers.insert((from,to), Box::new(transformer))
    }

    pub fn word_loader(&mut self, language: &'static str, loader: impl Fn() -> Result<Box<dyn WordLoader>,LanguageError> + 'static) -> Option<Box<dyn Fn() -> Result<Box<dyn WordLoader>, LanguageError> + 'static>> {
        self.word_loaders.insert(language, Box::new(loader))
    }

    pub fn word_validator(&mut self, language: &'static str, loader: impl Fn() -> Result<Box<dyn WordValidator>,LanguageError> + 'static) -> Option<Box<dyn Fn() -> Result<Box<dyn WordValidator>, LanguageError> + 'static>> {
        self.word_validators.insert(language, Box::new(loader))
    }

    fn expect_transformer(&self, from: &str, to: &str) -> Result<(Transformation, Box<dyn WordLoader>, Option<Box<dyn WordValidator>>),LanguageError> {
        let transformer = (self.transformers.get(&(from,to)).expect("There is no known transformation for '{from}' to '{to}'"))()?;
        let loader = (self.word_loaders.get(from).expect("There is no word loader for '{from}'"))()?;
        let validator = self.word_validators.get(to).map(|v| v()).transpose()?;
        Ok((transformer,loader,validator))
    }
}

pub fn run<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>, const ORTHOGRAPHIES: usize>(args: &mut Args, environment: TransformationEnvironment) {
  let arguments = parse_args(&mut args.skip(1));

  match arguments.command {
      Command::TransformWords(from, to, words, options) => {
         match environment.expect_transformer(&from,&to) {
             Ok((transformer,loader,validator)) => transform_words(&transformer,&loader,validator.as_ref(),words,&options),
             Err(err) => eprintln!("!!! Couldn't initialize transformation: {err}"),
        }
      },
      Command::ShowUsage => show_usage(environment),
  }

}
