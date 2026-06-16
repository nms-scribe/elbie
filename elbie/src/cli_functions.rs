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

pub(crate) fn generate_words(grid_style: Option<&Format>, language: &Language, count: usize) {
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
    grid.into_output(grid_style.unwrap_or(&Format::Plain)).print_to_stdout();
}

fn validate_word(language: &Language, word: &Word, explain: bool, trace_cb: Option<&ValidationTraceCallback>) -> Result<Result<(), ()>, ElbieError> {
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

pub(crate) fn validate_words(language: &Language, mut words: WordTable, option: &ValidateOption, output_format: &Format) {
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

    words.print_to_stdout(output_format);

    if invalid_count > 0 {
        eprintln!("!!!! {invalid_count} invalid words found");
        process::exit(1);
    }
}

pub(crate) fn show_phonemes(grid_style: Option<&Format>, language: &Language, table: Option<&String>) {
    let style = grid_style.unwrap_or(&Format::Terminal { spans: true });
    let result = match table {
        Some(table) => match language.build_phoneme_table(table) {
            Ok(Some(grid)) => {
                grid.into_output(style).print_to_stdout();
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
                    println!("{}", grid.1.caption());
                    grid.1.into_output(style).print_to_stdout();
                    println!();
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
}

pub(crate) fn show_spelling(grid_style: Option<&Format>, language: &Language, columns: usize) {
    match language.display_spelling(columns) {
        Ok(grid) => {
            grid.into_output(grid_style.unwrap_or(&Format::Terminal { spans: false })).print_to_stdout();
        },
        Err(err) => {
            eprintln!("!!! Couldn't display spelling: {err}");
            process::exit(1)
        }
    }
}

pub(crate) fn format_lexicon(format: &Format, style: &LexiconStyle, language: &Language, path: &WordTable, ortho_index: usize) {
    if ortho_index >= language.orthographies().len() {
        panic!("Language only has {} orthographies.", language.orthographies().len())
    }

    match language.load_lexicon(path, ortho_index, style) {
        Ok(lexicon) => {
            lexicon.print_to_stdout(format);
        },
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

/// replace_word: if this is true, and there is only one transformation, the original word will be moved into a new attribute, and the transformation creates the word for the word entry. Otherwise, each transformation is added as an attribute and the original word is kept. If there is not exactly one transformation, replace_word will be set to false no matter what the input value is.
pub(crate) fn transform_words(from: &Language, transformations: &[PreparedTransformation], mut words: WordTable, replace_word: bool, spellings: &[usize], option: &TransformationOption,
                              output_format: &Format) {
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

    // if spellings are included
    let orthographies = from.orthographies();
    let orthographies: Vec<_> = spellings.iter().map(|i| (*i, orthographies.get(*i).copied())).collect();
    for (i, orthography) in &orthographies {
        if let Some(orthography) = *orthography {
            words.add_attribute(orthography.to_owned());
        } else {
            eprintln!("There is no orthography for index {i}");
        }
    }

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

                            for (i, orthography) in &orthographies {
                                if let Some(orthography) = *orthography {
                                    let spelled = from.spell_word(&transformed, *i);
                                    entry.set_attribute(orthography.to_owned(), spelled);
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

    words.print_to_stdout(output_format);

    if invalid_found {
        eprintln!("Look for errors in Error column.");
        process::exit(1)
    }
}

pub(crate) fn analyze_words(from: &Language, words: &WordTable) {
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

    println!("{analysis}")
    // TODO: Display the analysis somehow
}
