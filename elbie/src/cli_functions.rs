use crate::grid::GridStyle;
use crate::language::Language;
use crate::grid::Grid;
use crate::grid::TableClass;
use crate::grid::GridRow;
use crate::grid::TRBodyClass;
use crate::grid::Cell;
use std::process;
use crate::validation::ValidationTraceCallback;
use crate::word::Word;
use crate::transformation::Transformation;
use crate::transformation::TransformationTraceCallback;

pub(crate) enum ValidateOption {
  Simple,
  Explain,
  Trace
}


pub(crate) enum TransformationOption {
  Simple,
  Explain,
  Trace
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
                if let Err(err) = language.check_word(&word,None /* eat message, no need to report */) {
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


pub(crate) fn validate_word(language: &Language, word: &Word, explain: bool, trace_cb: Option<&ValidationTraceCallback>) -> bool {
    match language.check_word(&word,trace_cb) {
        Err(err) => {
          if trace_cb.is_some() {
            println!("!!!! invalid word (see trace)");
          } else {
            println!("{err}");
          }
          false
        },
        Ok(validated) => {
          if explain {
            for valid in validated {
              println!("{valid}")
            }
          }

          for orthography in 0..language.orthographies().len() {
            print!("{} ",language.spell_word(&word,orthography));
          }
          println!("{word}");
          true
        }
    }
}

pub(crate) fn validate_words(language: &Language, words: Vec<String>, option: &ValidateOption) {
    let mut invalid_found = false;
    let trace_cb: Option<&ValidationTraceCallback> = if matches!(option,ValidateOption::Trace) {
      Some(&|level,message| {
        /* eat message, no need to report */
        println!("{}{}",str::repeat(" ",level*2),message);
      })
    } else {
      None
    };

    for word in words {
        match language.read_word(&word) {
            Ok(word) => {
                if !validate_word(language, &word, matches!(option,ValidateOption::Explain), trace_cb) {
                    invalid_found = true;
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



pub(crate) fn transform_words(transformation: &Transformation, from: &Language, validator: Option<&Language>, words: Vec<String>, option: &TransformationOption) {
    let mut invalid_found = false;


    let validation_trace_cb: Option<&ValidationTraceCallback> = if matches!(option,TransformationOption::Trace) {
      Some(&|level,message| {
        /* eat message, no need to report */
        println!("{}{}",str::repeat(" ",level*2),message);
      })
    } else {
        None
    };

    let transformation_trace_cb: Option<&TransformationTraceCallback> = if matches!(option,TransformationOption::Trace) {
        Some(&|message| {
            println!("{message}")
        })
    } else {
        None
    };


    for word in words {
        match from.read_word(&word) {
            Ok(word) => {
                match transformation.transform(word, transformation_trace_cb) {
                    Ok(word) => {
                        if let Some(validator) = validator {
                            if !validate_word(validator, &word, matches!(option,TransformationOption::Explain), validation_trace_cb) {
                                invalid_found = true;
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
