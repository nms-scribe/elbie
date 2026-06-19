use crate::cli_functions::ValidateOption;
use crate::cli_functions::format_lexicon;
use crate::cli_functions::generate_words;
use crate::cli_functions::show_phonemes;
use crate::cli_functions::show_spelling;
use crate::cli_functions::validate_words;
use crate::errors::ElbieError;
use crate::format::Format;
use crate::language::Language;
use crate::lexicon::LexiconStyle;
use crate::word_table::WordTable;
use std::env;
use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;

pub(crate) enum Command {
    GenerateWords(usize),                       // number of words to generate
    ValidateWords(Vec<String>, ValidateOption), // words to validate, whether to trace
    ShowPhonemes(Option<String>),               // specifies the table to show
    ShowSpelling(usize),                        // specifies number of columns
    ShowUsage,
    ProcessLexicon(String, usize)
}

pub(crate) struct Arguments {
    grid_style: Option<Format>,
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
            "--format=plain" => set_grid_style!(Format::Plain),
            "--format=terminal" => set_grid_style!(Format::Terminal { spans: spanning }),
            "--format=markdown" => set_grid_style!(Format::Markdown),
            "--format=html" => set_grid_style!(Format::HTML { spans: spanning }),
            "--format=json" => set_grid_style!(Format::JSON),
            "--no-spans" => {
                if let Some(style) = &mut grid_style {
                    match style {
                        Format::Plain | Format::JSON | Format::Markdown | Format::CSV => (),
                        Format::Terminal { spans } | Format::HTML { spans } => {
                            if *spans {
                                *spans = false
                            } else {
                                panic!("--no-spans specified twice")
                            }
                        },
                    }
                } else {
                    spanning = false
                }
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
                set_command!(Command::ValidateWords(words, ValidateOption::Simple));
            },
            "--validate=explain" => {
                let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
                words.extend(args.map(|x| x.as_ref().to_owned()));
                set_command!(Command::ValidateWords(words, ValidateOption::Explain));
            },
            "--validate=trace" => {
                let mut words = vec![args.next().expect("No words to validate").as_ref().to_owned()];
                words.extend(args.map(|x| x.as_ref().to_owned()));
                set_command!(Command::ValidateWords(words, ValidateOption::Trace));
            },
            "--phonemes" => set_command!(Command::ShowPhonemes(None)),
            a if a.starts_with("--phonemes=") => set_command!(Command::ShowPhonemes(Some(a.trim_start_matches("--phonemes=").to_owned()))),
            "--spelling" => set_command!(Command::ShowSpelling(1)),
            a if a.starts_with("--spelling=") => set_command!(Command::ShowSpelling(a.trim_start_matches("--spelling=").parse::<usize>().expect("Parameter should be a number").clamp(1, usize::MAX))),
            "--lexicon" => {
                let path = args.next().expect("No lexicon filename given").as_ref().to_owned();
                let spelling_index = args.next().expect("No orthography index given").as_ref().parse().expect("orthography index must be a number");
                set_command!(Command::ProcessLexicon(path, spelling_index))
            },
            "--help" => set_command!(Command::ShowUsage),
            _ => panic!("Unknown command {}", arg.as_ref())
        }
    }

    Arguments { grid_style,
                comment,
                command: command.unwrap_or(Command::GenerateWords(1)) }
}

pub(crate) fn show_usage(program: &str, output: &mut impl Write) -> Result<(), io::Error> {
    writeln!(output, "usage: {program} [options] [command]")?;
    writeln!(output, "default command: --generate 1")?;
    writeln!(output, "options:")?;
    writeln!(output, "   --format=<plain | terminal | markdown | html | json | csv>")?;
    writeln!(output, "      changes the format of grid output. Default is \"plain\" for generate and lexicon, and \"terminal\" for phonemes and spelling.")?;
    writeln!(output, "   --no-spans")?;
    writeln!(output, "      turns off column and row spanning in headers of grid output.")?;
    writeln!(output, "   --comment [String]")?;
    writeln!(output, "      prints out the text '<!-- Content auto-generated by {{0}} -->' before any output, where '{{0}}' is replaced by the specified text, or the word 'Elbie'")?;
    writeln!(output, "commands:")?;
    writeln!(output, "   --generate <integer>")?;
    writeln!(output, "      generates the specified number of words.")?;
    writeln!(output, "   --validate <words>...")?;
    writeln!(output, "      validates the specified words (verifies that it is possible to generate them).")?;
    writeln!(output, "   --validate=trace <words>...")?;
    writeln!(output, "      same as --validate, but traces the validation through all environment branches.")?;
    writeln!(output, "   --validate=explain <words>...")?;
    writeln!(output, "      same as --validate, but provides detailed explanation of valid phonemes on success.")?;
    writeln!(output, "   --phonemes")?;
    writeln!(output, "      prints out the phonemes of the language.")?;
    writeln!(output, "   --phonemes=<table>")?;
    writeln!(output, "      prints out one table of phonemes of the language.")?;
    writeln!(output, "   --spelling")?;
    writeln!(output, "      prints out the orthographies of the language.")?;
    writeln!(output, "   --spelling=<2..>")?;
    writeln!(output, "      prints spelling table in multiple columns.")?;
    writeln!(output, "   --lexicon <path>")?;
    writeln!(output, "      validates lexicon and outputs into a LaTeX file.")?;
    writeln!(output, "   --help")?;
    writeln!(output, "      display this information.")
}

pub fn run<ArgItem: AsRef<str>, Args: Iterator<Item = ArgItem>>(args: &mut Args, language: Result<Language, ElbieError>, output: &mut impl Write) -> Result<(), io::Error> {
    let arguments = parse_args(&mut args.skip(1));

    if let Some(comment) = arguments.comment {
        println!("<!-- Content auto-generated by {comment} -->")
    }

    match language {
        Ok(language) => {
            match arguments.command {
                Command::GenerateWords(count) => generate_words(arguments.grid_style.as_ref(), &language, count, output),
                Command::ValidateWords(words, option) => {
                    let mut words_data = WordTable::default();
                    words_data.add_words(&words);
                    validate_words(&language, words_data, &option, &Format::Plain, output)
                },
                Command::ShowPhonemes(table) => show_phonemes(arguments.grid_style.as_ref(), &language, table.as_ref(), output),
                Command::ShowSpelling(columns) => show_spelling(arguments.grid_style.as_ref(), &language, columns, output),
                Command::ProcessLexicon(path, ortho_index) => {
                    // NOTE: I'm doing an expect here because this whole 'run' function is deprecated anyway, so I'm not going to change it's signature.
                    let Ok(words_data) = WordTable::read(&path) else {
                        eprintln!("!!! Couldn't read input lexicon");
                        process::exit(1);
                    };
                    format_lexicon(arguments.grid_style.as_ref().unwrap_or(&Format::Plain), &LexiconStyle::List, &language, &words_data, ortho_index, output)
                },
                Command::ShowUsage => {
                    let exe_name = env::current_exe().ok().as_deref().and_then(Path::file_name).map(OsStr::display).as_ref().map(ToString::to_string);
                    let program = exe_name.as_deref().unwrap_or_else(|| language.name());
                    show_usage(program, output)
                }
            }
        },
        Err(err) => {
            eprintln!("!!! Language Incomplete: {err}");
            process::exit(1)
        }
    }
}
