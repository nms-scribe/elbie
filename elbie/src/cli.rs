use crate::errors::ElbieError;
use crate::grid::GridStyle;
use crate::cli_functions::generate_words;
use crate::cli_functions::validate_words;
use crate::cli_functions::show_phonemes;
use crate::cli_functions::show_spelling;
use crate::cli_functions::format_lexicon;
use crate::cli_functions::ValidateOption;
use std::path::Path;
use std::ffi::OsStr;
use gumdrop::Options;
use std::process;
use crate::family::Family;
use crate::language::Language;
use crate::cli_functions::transform_words;
use crate::cli_functions::TransformationOption;

// TODO: Test transformations with a new goblin to hobgoblin language transformation

// Gumdrop kind of makes showing usage difficult. The only way it works is if you have a --help flag on each command, and then only if it's discovered in `parse_args_or_exit`. And I'm not calling that because I want to be able to supply my own arguments. I would prefer to have a help command that takes an optional command name parameter anyway.
fn show_usage<Command: Options>(program: &str, selected_command: Option<&str>) {
    let sub_commands = Command::command_list();
    match (selected_command,sub_commands) {
        (None, None) => println!("usage: {program} [ARGUMENTS]"),
        (None, Some(_)) => println!("usage: {program} [ARGUMENTS] [COMMAND]"),
        (Some(subcommand), None) => println!("usage: {program} {subcommand} [ARGUMENTS]"),
        (Some(subcommand), Some(_)) => println!("usage: {program} {subcommand} [ARGUMENTS] [COMMAND]"),
    }
    println!();
    // FUTURE: It would be nice to have some way to control this formatting as well.
    // FUTURE: Also, to be able to tell if the Options have positional, required, or optional arguments so I can change the usage header above.
    println!("{}",Command::usage());
    println!();
    if let Some(commands) = sub_commands {
        println!("Available commands:");
        println!("{commands}")
    }

}

trait DoIt {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError>;
}

#[derive(Options)]
/// Generates words for a language. Options allow specifying the count and the output format.
pub struct GenerateWords {

    #[options(default="1")]
    /// The number of words to generate.
    count: usize,

    #[options(default="plain")]
    #[options(no_short)]
    /// Changes the format of grid output.
    format: GridStyle,

    #[options(no_short)]
    /// Turns off column and row spanning in headers of grid output.
    no_spans: bool,

}

impl DoIt for GenerateWords {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError>  {
        let grid_style = if self.no_spans {
            &self.format.with_no_spans()
        } else {
            &self.format
        };

        let mut family = family()?;

        family.load_language_or_default(language.as_deref())?;

        let language = family.get_language_or_default(language.as_deref())?;

        generate_words(Some(&grid_style), &language, self.count);

        Ok(())
    }
}

#[derive(Options)]
/// Validate a list of words for a language, verifying that it would be possible to generate them. Pass a list of words to process at the end of the command. Options allow getting more detail about the validation.
pub struct ValidateWords {

    #[options(no_short)]
    /// Traces the validation through all phonotactic branches
    trace: bool,

    #[options(no_short)]
    /// Provides detailed explanation of valid phonemes on success.
    explain: bool,

    #[options(free)]
    /// Words to validate
    words: Vec<String>

}

impl DoIt for ValidateWords {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError>  {

        let mut family = family()?;

        family.load_language_or_default(language.as_deref())?;

        let language = family.get_language_or_default(language.as_deref())?;


        validate_words(language, &self.words, &match (self.explain,self.trace)  {
            (true, true) => ValidateOption::ExplainAndTrace,
            (true, false) => ValidateOption::Explain,
            (false, true) => ValidateOption::Trace,
            (false, false) => ValidateOption::Simple,
        });

        Ok(())
    }

}

#[derive(Options)]
/// Prints out tables of phonemes for a language. Options allow limiting to one table, and controlling the output format.
pub struct ShowPhonemes {

    #[options(no_short)]
    /// A specific phoneme table to show. If not specified all tables will be shown.
    table: Option<String>,

    #[options(default="terminal")]
    #[options(no_short)]
    /// Changes the format of grid output.
    format: GridStyle,

    #[options(no_short)]
    /// Turns off column and row spanning in headers of grid output.
    no_spans: bool,

}

impl DoIt for ShowPhonemes {


    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError>  {
        let grid_style = if self.no_spans {
            &self.format.with_no_spans()
        } else {
            &self.format
        };

        let mut family = family()?;

        family.load_language_or_default(language.as_deref())?;

        let language = family.get_language_or_default(language.as_deref())?;

        show_phonemes(Some(&grid_style), &language, self.table.as_ref());

        Ok(())
    }
}

#[derive(Options)]
/// Prints out a table of orthographies for a language. Options include controlling the output format and breaking the table into extra columns.
pub struct ShowSpelling {

    #[options(no_short)]
    #[options(default="1")]
    /// If more than 1, breaks table and splits it across the specified number of column groups for easier formatting.
    columns: usize,

    #[options(default="terminal")]
    #[options(no_short)]
    /// Changes the format of grid output.
    format: GridStyle,

    #[options(no_short)]
    /// Turns off column and row spanning in headers of grid output.
    no_spans: bool,

}

impl DoIt for ShowSpelling {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError>  {
        let grid_style = if self.no_spans {
            &self.format.with_no_spans()
        } else {
            &self.format
        };

        let mut family = family()?;

        family.load_language_or_default(language.as_deref())?;

        let language = family.get_language_or_default(language.as_deref())?;

        // TODO: These things should now be able to throw errors, maybe?
        show_spelling(Some(&grid_style), &language, self.columns);

        Ok(())
    }
}

#[derive(Options)]
/// Loads a lexicon of words in CSV format for a language, validates them and prints out a formatted listing. Requires a file name to load and an orthography index to spell the main entries. Options allow controlling the output format.
pub struct FormatLexicon {


    #[options(required)]
    /// File to load lexicon from (CSV format).
    file: String,

    #[options(required)]
    /// Orthography index (0-based) to use for generating main entries.
    spelling: usize,

    /// Changes the format of grid output.
    #[options(default="plain")]
    #[options(no_short)]
    format: GridStyle,

    #[options(no_short)]
    /// Turns off column and row spanning in headers of grid output.
    no_spans: bool,
}

impl DoIt for FormatLexicon {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError> {
        let grid_style = if self.no_spans {
            &self.format.with_no_spans()
        } else {
            &self.format
        };

        let mut family = family()?;

        family.load_language_or_default(language.as_deref())?;

        let language = family.get_language_or_default(language.as_deref())?;

        format_lexicon(Some(grid_style), language, &self.file, self.spelling);

        Ok(())

    }


}

#[derive(Options)]
pub struct Transform {

    #[options(required)]
    /// The target language for transformation. Used to lookup the transformation even if dont_validate is true.
    target: String,

    #[options(no_short)]
    /// Requests that the words not be validated after transformation.
    dont_validate: bool,

    #[options(no_short)]
    /// Traces the validation through all phonotactic branches
    trace: bool,

    #[options(no_short)]
    /// Provides detailed explanation of valid phonemes on success.
    explain: bool,

    #[options(free)]
    /// Words to validate
    words: Vec<String>

}

impl DoIt for Transform {
    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError> {

        let mut family = family()?;


        let source_language = language.or_else(|| family.default_language_name().map(ToOwned::to_owned)).ok_or_else(|| ElbieError::NoDefaultLanguage)?;

        // in theory the transformation should take care of loading any languages it needs to get phonemes from. If it doesn't, they'll get an error pretty quickly.
        family.load_transformation(&source_language, &self.target)?;

        let source_language = family.get_language(&source_language)?;

        let transformation = family.get_transformation(source_language.name(),&self.target)?;

        let target_language = if self.dont_validate || !transformation.validate() {
            None
        } else {
            Some(family.get_language(&self.target)?)
        };

        transform_words(&transformation,source_language,target_language,&self.words,&match (self.explain,self.trace)  {
            (true, true) => TransformationOption::ExplainAndTrace,
            (true, false) => TransformationOption::Explain,
            (false, true) => TransformationOption::Trace,
            (false, false) => TransformationOption::Simple,
        });

        Ok(())



    }
}

#[derive(Options)]
/// Print the languages and transformations available in the tool.
pub struct ShowInformation {
}

impl DoIt for ShowInformation {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, _: Option<String>) -> Result<(),ElbieError> {

        let family = family()?;


        let mut languages = family.language_keys();
        if !languages.is_empty() {
            languages.sort();
            let default = if languages.len() > 1 {
                family.default_language_name()
            } else {
                // we might have a default name, but there's no reason to mark anything as default if there's only one.
                None
            };
            if default.is_some() {
                println!("LANGUAGES: (* = default)")
            } else {
                println!("LANGUAGES:");
            }
            for language in &languages {
                if Some(language.as_str()) == default {
                    println!("{language} *")
                } else {
                    println!("{language}")
                }
            }
        }

        let mut transformations = family.transformation_keys();
        if !transformations.is_empty() {
            transformations.sort();
            if !languages.is_empty() {
                // need a space
                println!();
            }
            println!("TRANSFORMATIONS:");
            for (from,to) in transformations {
                println!("{from} 🡺 {to}")
            }
        }


        Ok(())

    }


}

#[derive(Options)]
/// Print out this information. Use 'help COMMAND' to get help on a specific command.
pub struct FamilyShowUsage {

    #[options(free)]
    /// A command to display specific help for
    command: Option<String>,

}

impl DoIt for FamilyShowUsage {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, _: FamilyCreator, _: Option<String>) -> Result<(),ElbieError> {
        let exe_name = std::env::current_exe().ok().as_deref().and_then(Path::file_name).map(OsStr::display).as_ref().map(ToString::to_string);
        let program = exe_name.as_deref().unwrap_or("elbie");

        let selected_command = self.command.as_deref();
        if let Some(command) = selected_command {
            match command {
                "generate" => show_usage::<GenerateWords>(program, Some(&command)),
                "validate" => show_usage::<ValidateWords>(program, Some(&command)),
                "phonemes" => show_usage::<ShowPhonemes>(program, Some(&command)),
                "spelling" => show_usage::<ShowSpelling>(program, Some(&command)),
                "lexicon" => show_usage::<FormatLexicon>(program, Some(&command)),
                "transform" => show_usage::<Transform>(program, Some(&command)),
                "information" => show_usage::<ShowInformation>(program, Some(&command)),
                "help" => show_usage::<FamilyShowUsage>(program, Some(&command)),
                "default" => show_usage::<ShowInformation>(program, Some(&command)),
                command => {
                    eprintln!("Unknown command '{command}'");
                    eprintln!();
                    show_usage::<FamilyArguments>(program, None);
                    process::exit(1);
                }
            }
        } else {
            show_usage::<FamilyArguments>(program, None);
        };

        Ok(())
    }
}

#[derive(Options)]
pub enum FamilyCommand {
    /// Generates a words for a language.
    Generate(GenerateWords),
    /// Validate a list of words for a language, verifying that it would be possible to generate them.
    Validate(ValidateWords),
    /// Prints out tables of phonemes for a language.
    Phonemes(ShowPhonemes),
    /// Prints out a table of orthographies for a language.
    Spelling(ShowSpelling),
    /// Loads a lexicon of words in CSV format for a language, validates them and prints out a formatted listing.
    Lexicon(FormatLexicon),
    /// Transforms words from a source language to another.
    Transform(Transform),
    /// Print the information about the available languages.
    Information(ShowInformation),
    /// Print out this information. Use 'help COMMAND' to get help on a specific command.
    Help(FamilyShowUsage),
}


impl Default for FamilyCommand {
    fn default() -> Self {
        Self::Help(FamilyShowUsage { command: None })
    }
}

impl DoIt for FamilyCommand {
    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError> {

        match self {
            Self::Generate(command) => command.doit(family,language),
            Self::Validate(command) => command.doit(family,language),
            Self::Phonemes(command) => command.doit(family,language),
            Self::Spelling(command) => command.doit(family,language),
            Self::Lexicon(command) => command.doit(family,language),
            Self::Transform(command) => command.doit(family,language),
            Self::Information(command) => command.doit(family,language),
            Self::Help(command) => command.doit(family,language),
        }

    }
}


#[derive(Options)]
#[options(no_help_flag)]
pub struct FamilyArguments {

    /// prints out the text '<!-- Content auto-generated by Elbie -->' before any output. To change the name of the generator, use `--creator`.
    comment: bool,
    /// prints out the text '<!-- Content auto-generated by {{0}} -->' before any output, where '{{0}}' is replaced by the specified text. Also see `--comment`.
    creator: Option<String>,

    /// The source language for commands. If the tool has more than one language, and does not have a default language (see 'information' command), this option will be required for some commands.
    // NOTE: this is not under the commands, so that I can reuse the commands for the LanguageArguments object, where the language doesn't need to be specified.
    // FUTURE: If there were some way of "requiring" this based on the command chosen, that would be great. Or adding the required commands to the option based on generic type parameters or something. At best I could do macros now, but that gets icky quickly for lots of commands with different gumdrop options.
    language: Option<String>,

    #[options(command)]
    command: Option<FamilyCommand>

}


#[derive(Options)]
/// Print out this information. Use 'help COMMAND' to get help on a specific command.
pub struct LanguageShowUsage {

    #[options(free)]
    /// A command to display specific help for
    command: Option<String>,

}

impl DoIt for LanguageShowUsage {

    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, _: FamilyCreator, _: Option<String>) -> Result<(),ElbieError> {
        let exe_name = std::env::current_exe().ok().as_deref().and_then(Path::file_name).map(OsStr::display).as_ref().map(ToString::to_string);
        let program = exe_name.as_deref().unwrap_or("elbie");

        let selected_command = self.command.as_deref();
        if let Some(command) = selected_command {
            match command {
                "generate" => show_usage::<GenerateWords>(program, Some(&command)),
                "validate" => show_usage::<ValidateWords>(program, Some(&command)),
                "phonemes" => show_usage::<ShowPhonemes>(program, Some(&command)),
                "spelling" => show_usage::<ShowSpelling>(program, Some(&command)),
                "lexicon" => show_usage::<FormatLexicon>(program, Some(&command)),
                "help" => show_usage::<LanguageShowUsage>(program, Some(&command)),
                command => {
                    eprintln!("Unknown command '{command}'");
                    eprintln!();
                    show_usage::<LanguageArguments>(program, None);
                    process::exit(1);
                }
            }
        } else {
            show_usage::<LanguageArguments>(program, None);
        };

        Ok(())
    }
}

#[derive(Options)]
pub enum LanguageCommand {
    /// Generates words for a language.
    Generate(GenerateWords),
    /// Validate a list of words for a language, verifying that it would be possible to generate them.
    Validate(ValidateWords),
    /// Prints out tables of phonemes for a language.
    Phonemes(ShowPhonemes),
    /// Prints out a table of orthographies for a language.
    Spelling(ShowSpelling),
    /// Loads a lexicon of words in CSV format for a language, validates them and prints out a formatted listing.
    Lexicon(FormatLexicon),
    /// Print out this information. Use 'help COMMAND' to get help on a specific command.
    Help(LanguageShowUsage),
}

impl Default for LanguageCommand {
    fn default() -> Self {
        Self::Help(LanguageShowUsage { command: None })
    }
}

impl DoIt for LanguageCommand {
    fn doit<FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(&self, family: FamilyCreator, language: Option<String>) -> Result<(),ElbieError> {
        match self {
            Self::Generate(command) => command.doit(family,language),
            Self::Validate(command) => command.doit(family,language),
            Self::Phonemes(command) => command.doit(family,language),
            Self::Spelling(command) => command.doit(family,language),
            Self::Lexicon(command) => command.doit(family,language),
            Self::Help(command) => command.doit(family,language),

        }

    }
}

#[derive(Options)]
#[options(no_help_flag)]
pub struct LanguageArguments {

    /// prints out the text '<!-- Content auto-generated by Elbie -->' before any output. To change the name of the generator, use `--creator`.
    comment: bool,
    /// prints out the text '<!-- Content auto-generated by {{0}} -->' before any output, where '{{0}}' is replaced by the specified text. Also see `--comment`.
    creator: Option<String>,

    #[options(command)]
    command: Option<LanguageCommand>

}


fn run_command<FamilyCreator: FnOnce() -> Result<Family,ElbieError>, Command: DoIt + Default>(comment: Option<String>, language: Option<String>, command: Option<Command>, family: FamilyCreator) {
    if let Some(comment) = comment {
        println!("<!-- Content auto-generated by {comment} -->")
    }

    let result = command.unwrap_or_default().doit(family, language);

    if let Err(err) = result {
        eprintln!("!!! While running command: {err}");
        process::exit(1)
    }
}

/// The first argument (program name) should not be included.
pub fn run_family<S: AsRef<str>, FamilyCreator: FnOnce() -> Result<Family,ElbieError>>(args: &[S], family: FamilyCreator) {

    match FamilyArguments::parse_args_default(args) {
        Ok(arguments) => {

            run_command(arguments.creator.or_else(|| arguments.comment.then(|| "Elbie".to_owned())), arguments.language, arguments.command, family);

        },
        Err(err) => {
            eprintln!("{err}");
            process::exit(1)
        },
    }
}



/// Use this to run a command line that only works with one language. The arguments are the same as the usual, except that there is no language option, and the transform command is not available.
pub fn run_language<S: AsRef<str>, Creator: FnOnce() -> Result<Language,ElbieError> + 'static>(args: &[S], name: &'static str, language: Creator) {

    match LanguageArguments::parse_args_default(args) {
        Ok(arguments) => {

            run_command(
                arguments.creator.or_else(|| arguments.comment.then(|| "Elbie".to_owned())),
                Some(name.to_owned()),
                arguments.command,
                move || {
                    let mut family = Family::default();
                    family.default_language(name, language)?;
                    Ok(family)
                }
            );

        },
        Err(err) => {
            eprintln!("{err}");
            process::exit(1)
        },
    }
}
