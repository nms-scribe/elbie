use crate::analysis::AnalysisConfig;
use crate::errors::ElbieError;
use crate::format::Format;
use crate::grid::Cell;
use crate::grid::Grid;
use crate::grid::GridRow;
use crate::grid::TRBodyClass;
use crate::grid::TableClass;
use crate::language::Language;
use crate::lexicon::LexiconStyle;
use crate::transformation::PreparedTransformation;
use crate::transformation::Transformation;
use crate::transformation::TransformationTraceCallback;
use crate::validation::ValidationTraceCallback;
use crate::word::Word;
use crate::word_table::WordTable;
use core::error::Error;
use core::num::ParseIntError;
use core::str::FromStr;
use std::io;
use std::io::Write;
use std::process;

pub(crate) enum ValidateOption {
    Simple,
    Explain,
    Trace,
    ExplainAndTrace
}

pub(crate) enum TransformationOption {
    Simple,
    Explain,
    Trace,
    ExplainAndTrace
}

pub(crate) fn generate_words(grid_style: Option<&Format>, language: &Language, count: usize, output: &mut impl Write) -> Result<(), io::Error> {
    let mut grid = Grid::new(TableClass::ElbieWords, format!("Generated {count} words for {}", language.name()));

    // FUTURE: Should I have a header?

    for _ in 0..count {
        let mut row = GridRow::new(TRBodyClass::BodyRow);

        match language.make_word() {
            Ok(word) => {
                for orthography in 0..language.orthographies().len() {
                    row.push_cell(Cell::content(language.spell_word(&word, orthography), None));
                }
                row.push_cell(Cell::content(format!("{word}"), None));

                // the following is a sanity check. It might catch some logic errors, but really it's just GIGO.
                if let Err(err) = language.check_word(&word, None /* eat message, no need to report */) {
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
    grid.into_output(grid_style.unwrap_or(&Format::Plain)).print(output)
}

pub(crate) fn validate_word(language: &Language, word: &Word, explain: bool, trace_cb: Option<&ValidationTraceCallback>) -> Result<Result<(), ()>, ElbieError> {
    match language.check_word(word, trace_cb)? {
        Err(()) => Ok(Err(())),
        Ok(validated) => {
            if explain {
                eprintln!("Explain: {word}");
                for valid in validated {
                    eprintln!("{valid}")
                }
            }

            Ok(Ok(()))
        }
    }
}

pub(crate) fn validate_words(language: &Language, mut words: WordTable, option: &ValidateOption, output_format: &Format, output: &mut impl Write) -> Result<(), io::Error> {
    const VALIDATED_ATTR: &str = "Validated";

    let mut invalid_count = 0;
    let trace_cb: Option<&ValidationTraceCallback> = if matches!(option, ValidateOption::Trace | ValidateOption::ExplainAndTrace) {
        Some(&|level, message| {
            eprintln!("{}{}", str::repeat(" ", level * 2), message);
        })
    } else {
        None
    };

    for orthography in language.orthographies() {
        words.add_attribute((*orthography).to_owned());
    }
    words.add_attribute(VALIDATED_ATTR.to_owned());

    for entry in &mut words.entries_mut() {
        match language.read_word(entry.word()) {
            Ok(word) => {
                // Make sure word is in phonemic format
                entry.replace_word(None, word.to_string());
                match validate_word(language, &word, matches!(option, ValidateOption::Explain | ValidateOption::ExplainAndTrace), trace_cb) {
                    Ok(Ok(())) => {
                        entry.set_attribute(VALIDATED_ATTR.to_owned(), "Valid".to_owned());
                        for (i, orthography) in language.orthographies().iter().enumerate() {
                            entry.set_attribute((*orthography).to_owned(), language.spell_word(&word, i));
                        }
                    },
                    Ok(Err(())) => {
                        entry.set_attribute(VALIDATED_ATTR.to_owned(), "!! Invalid".to_owned());
                        invalid_count += 1;
                    },
                    Err(err) => {
                        eprintln!("!!!! Can't validate word: {err}");
                        process::exit(1)
                    }
                }
            },
            Err(err) => {
                eprintln!("!!!! Can't read word: {err}");
                process::exit(1);
            }
        }
    }

    words.print(output_format, output)?;

    if invalid_count > 0 {
        eprintln!("!!!! {invalid_count} invalid words found");
        process::exit(1);
    }

    Ok(())
}

pub(crate) fn show_phonemes(grid_style: Option<&Format>, language: &Language, table: Option<&String>, output: &mut impl Write) -> Result<(), io::Error> {
    let style = grid_style.unwrap_or(&Format::Terminal { spans: true });
    let result = match table {
        Some(table) => match language.build_phoneme_table(table) {
            Ok(Some(grid)) => {
                grid.into_output(style).print(output)?;
                Ok(())
            },
            Ok(None) => {
                eprintln!("No phoneme table named {table}. Try singular or lower-case?");
                Ok(())
            },
            Err(err) => Err(err)
        },
        None => match language.build_all_phoneme_tables() {
            Ok(grids) => {
                for grid in grids {
                    writeln!(output, "{}", grid.1.caption())?;
                    grid.1.into_output(style).print(output)?;
                    writeln!(output)?;
                }

                Ok(())
            },
            Err(err) => Err(err)
        }
    };

    if let Err(err) = result {
        eprintln!("!!! Couldn't display phonemes: {err}");
        process::exit(1)
    }

    Ok(())
}

pub(crate) fn show_spelling(grid_style: Option<&Format>, language: &Language, columns: usize, output: &mut impl Write) -> Result<(), io::Error> {
    match language.display_spelling(columns) {
        Ok(grid) => grid.into_output(grid_style.unwrap_or(&Format::Terminal { spans: false })).print(output),
        Err(err) => {
            eprintln!("!!! Couldn't display spelling: {err}");
            process::exit(1)
        }
    }
}

pub(crate) fn format_lexicon(format: &Format, style: &LexiconStyle, language: &Language, path: &WordTable, ortho_index: usize, output: &mut impl Write) -> Result<(), io::Error> {
    if ortho_index >= language.orthographies().len() {
        panic!("Language only has {} orthographies.", language.orthographies().len())
    }

    match language.load_lexicon(path, ortho_index, style) {
        Ok(lexicon) => lexicon.print(format, output),
        Err(err) => {
            eprintln!("!!! Couldn't process lexicon: {err}");
            process::exit(1)
        }
    }
}

pub(crate) fn transform_and_validate_word(word: &Word, transformation: &Transformation, validator: Option<&Language>, explain: bool, transformation_trace_cb: Option<&TransformationTraceCallback>,
                                          validation_trace_cb: Option<&ValidationTraceCallback>)
                                          -> Result<(Word, Option<bool>), ElbieError> {
    let transformed = transformation.transform(word, transformation_trace_cb)?;

    if let Some(validator) = validator {
        let valid = validate_word(validator, &transformed, explain, validation_trace_cb)?.is_ok();
        Ok((transformed, Some(valid)))
    } else {
        Ok((transformed, None))
    }
}

pub(crate) enum OrthographyIndex {
    Index(usize),
    All
}

impl FromStr for OrthographyIndex {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(index) => Ok(Self::Index(index)),
            Err(_) if s == "all" => Ok(Self::All),
            Err(err) => Err(err)
        }
    }
}

impl OrthographyIndex {
    fn calculate_orthographies(spellings: &[Self], validator: &Language) -> Result<Vec<(usize, &'static str)>, ElbieError> {
        let mut spelling_indexes = Vec::new();
        let orthographies = validator.orthographies();
        for orthography_index in spellings {
            match orthography_index {
                Self::Index(index) => spelling_indexes.push(*index),
                Self::All => {
                    spelling_indexes = orthographies.iter().enumerate().map(|(i, _)| i).collect();
                    break;
                }
            }
        }

        spelling_indexes.into_iter().try_fold(Vec::new(), |mut result, i| {
                                        let name = orthographies.get(i).copied().ok_or(ElbieError::UnknownOrthography(i))?;
                                        result.push((i, name));
                                        Ok(result)
                                    })
    }
}

/// replace_word: if this is true, and there is only one transformation, the original word will be moved into a new attribute, and the transformation creates the word for the word entry. Otherwise, each transformation is added as an attribute and the original word is kept. If there is not exactly one transformation, replace_word will be set to false no matter what the input value is.
pub(crate) fn transform_words(from: &Language, transformations: &[PreparedTransformation], mut words: WordTable, replace_word: bool, spellings: &[OrthographyIndex], option: &TransformationOption,
                              output_format: &Format, output: &mut impl Write)
                              -> Result<(), Box<dyn Error>> {
    const ERROR_ATTR: &str = "Error";

    let mut invalid_found = false;

    let validation_trace_cb: Option<&ValidationTraceCallback> = if matches!(option, TransformationOption::Trace | TransformationOption::ExplainAndTrace) {
        Some(&|level, message| {
            eprintln!("{}{}", str::repeat(" ", level * 2), message);
        })
    } else {
        None
    };

    let transformation_trace_cb: Option<&TransformationTraceCallback> = if matches!(option, TransformationOption::Trace | TransformationOption::ExplainAndTrace) {
        Some(&|message| eprintln!("{message}"))
    } else {
        None
    };

    let original_word_attr = from.name();

    // we can't replace the word with sets, so only allow that option if it's not a set
    let replace_word = if replace_word && transformations.len() == 1 {
        // also add an attribute for the original word
        words.add_attribute(original_word_attr.to_owned());
        true
    } else {
        // add attributes for the transformations
        for item in transformations {
            words.add_attribute(item.name.clone());
        }
        // override the value of replace_word, so we don't ever do that again
        false
    };

    // if spellings are included, this only works if we have a language we are validating to.
    let orthographies = if transformations.len() == 1
                           && let Some(transformation) = transformations.first()
                           && let Some(validator) = transformation.validator
    {
        let orthographies = OrthographyIndex::calculate_orthographies(spellings, validator)?;
        for (_, orthography) in &orthographies {
            words.add_attribute((*orthography).to_owned());
        }
        Some(orthographies)
    } else if spellings.is_empty() {
        None
    } else {
        eprintln!("!!! Spellings were requested, but the transformation does not validate, or there are too many transformations in the set.");
        None
    };

    words.add_attribute(ERROR_ATTR.to_owned());

    for entry in &mut words.entries_mut() {
        let error = match from.read_word(entry.word()) {
            Ok(word) => {
                // The original word might not be in phonemic notation, make sure it is now for consistency...
                entry.replace_word(None, word.to_string());

                // only the last error will be returned if this is a set...
                let mut last_failure = None;

                for item in transformations {
                    match transform_and_validate_word(&word,
                                                      item.transformation,
                                                      item.validator,
                                                      matches!(option, TransformationOption::Explain | TransformationOption::ExplainAndTrace),
                                                      transformation_trace_cb,
                                                      validation_trace_cb)
                    {
                        Ok((transformed, validated)) => {
                            if replace_word {
                                // replace that word with the transformed and move the original to a new attribute
                                entry.replace_word(Some(original_word_attr.to_owned()), transformed.to_string());
                            } else {
                                entry.set_attribute(item.name.clone(), transformed.to_string());
                            }

                            if let Some(validator) = item.validator
                               && let Some(orthographies) = &orthographies
                            {
                                for (i, orthography) in orthographies {
                                    let spelled = validator.spell_word(&transformed, *i);
                                    entry.set_attribute((*orthography).to_owned(), spelled);
                                }
                            }

                            last_failure = validated.and_then(|valid| (!valid).then(|| "Word was invalid (see trace)".to_owned()))
                        },
                        Err(err) => {
                            // these should be errors in programming the transformation and validator, not just an invalid word.
                            eprintln!("!!! Error transforming and validating word {word}: {err}");
                            process::exit(1)
                        }
                    }
                }

                last_failure
            },
            Err(err) => {
                // this is an error in the data, I don't want to stop the whole batch, that could be a problem later.
                if replace_word {
                    // replace that word with an empty value and move the original to a new attribute
                    entry.replace_word(Some(original_word_attr.to_owned()), String::new());
                } // else leave the transformed attribute blank.

                Some(format!("Can't read word: {err}"))
            }
        };

        if let Some(error) = error {
            entry.set_attribute(ERROR_ATTR.to_owned(), error);
            invalid_found = true;
        }
    }

    if !invalid_found {
        words.remove_attribute(ERROR_ATTR);
    }

    words.print(output_format, output)?;

    if invalid_found {
        eprintln!("Look for errors in Error column.");
        process::exit(1)
    }

    Ok(())
}

pub(crate) fn analyze_words(from: &Language, words: &WordTable, output: &mut impl Write) -> Result<(), io::Error> {
    // TODO: Should be able to set up custom analysis stuff on the language itself.
    let config = AnalysisConfig::from_language(from);

    if let Err(err) = config.validate(from) {
        eprintln!("{err}");
        process::exit(1)
    }

    // TODO: Analyze and build a vector of clusters, along with: cluster_set, length, index in word (by cluster set), if it's final, and if it's initial.
    // - a cluster is built if the phoneme is in the cluster set. If it's not, the cluster ends and a new cluster begins.

    let analysis = match config.analyze(words) {
        Ok(analysis) => analysis,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    writeln!(output, "{analysis}")
    // TODO: Display the analysis somehow
}
