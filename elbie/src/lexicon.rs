use core::fmt::Write;
use crate::word::Word;
use crate::format::Format;
use crate::grid::Grid;
use crate::grid::TableClass;
use crate::grid::ColumnHeader;
use crate::grid::GridRow;
use crate::grid::TRBodyClass;
use crate::grid::Cell;
use crate::grid::TableOutput;
use html_builder::Html5 as _;
use std::str::FromStr;


pub(crate) enum LexiconStyle {
    Table,
    List
}

impl FromStr for LexiconStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "table" => Ok(Self::Table),
            "list" => Ok(Self::List),
            name => Err(format!("Unknown lexicon style '{name}'."))
        }
    }
}


pub(crate) struct LexiconEntry {
  word: Word,
  spelling: Vec<String>,
  definition: String
}

impl LexiconEntry {

    pub(crate) const fn new(word: Word, spelling: Vec<String>, definition: String) -> Self {
        Self {
            word,
            spelling,
            definition
        }

    }


}


pub(crate) struct LexiconTable {
    grid: Grid,
    primary_orthography_idx: usize
}

impl LexiconTable {

    pub(crate) fn new(orthographies: Vec<&'static str>, primary_orthography_idx: usize) -> Self {
        let mut grid = Grid::new(TableClass::ElbieLexicon, "Lexicon".to_owned());

        let mut primary_orthography = None;
        let mut other_orthographies = Vec::new();
        for (i,orthography) in orthographies.iter().enumerate() {
            if i == primary_orthography_idx {
                primary_orthography = Some(orthography)
            } else {
                other_orthographies.push(*orthography);
            }
        }
        let primary_orthography = primary_orthography.expect("Primary orthography index was out of bounds");

        let mut headers = Vec::new();
        headers.push(ColumnHeader::new((*primary_orthography).to_owned(),1));
        headers.push(ColumnHeader::new("Word".to_owned(),1));
        for orthography in &other_orthographies {
            headers.push(ColumnHeader::new((*orthography).to_owned(),1));
        }
        headers.push(ColumnHeader::new("Definition".to_owned(),1));

        grid.set_headers(headers);

        Self {
            grid,
            primary_orthography_idx
        }
    }

    pub(crate) fn push_entry(&mut self, entry: LexiconEntry) {

        let mut primary_spelling = None;
        let mut other_spellings = Vec::new();
        for (i,spelling) in entry.spelling.iter().enumerate() {
            if i == self.primary_orthography_idx {
                primary_spelling = Some(spelling)
            } else {
                other_spellings.push(spelling);
            }
        }
        let primary_spelling = primary_spelling.expect("Primary orthography index was out of bounds");

        let mut fields = GridRow::new(TRBodyClass::BodyRow);
        fields.push_cell(Cell::content((*primary_spelling).to_owned(),None));
        fields.push_cell(Cell::content(entry.word.to_string(),None));
        for spelling in &other_spellings {
            fields.push_cell(Cell::content((*spelling).to_owned(),None));
        }
        fields.push_cell(Cell::content(entry.definition,None));

        self.grid.push_body_row(fields);


    }

    /// # Panics
    /// panics if the `self.primary_orthography' is not a valid index in the supplied orthographies.
    #[must_use]
    pub(crate) fn into_output(self, style: &Format) -> TableOutput {

        self.grid.into_output(style)

    }
}



trait LexiconWriter {

    fn initialize(main_orthography: &'static str, orthographies: Vec<&'static str>) -> Self;

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str, output: &mut String);

}


struct PlainLexiconWriter {
    orthographies: Vec<&'static str>
}

impl LexiconWriter for PlainLexiconWriter {

    fn initialize(_: &'static str, orthographies: Vec<&'static str>) -> Self {
        Self {
            orthographies
        }
    }

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str, output: &mut String) {
        write!(output,"{main_spelling} ({word}").expect("Could not write to Plain Text");
        for (orthography,spelling) in self.orthographies.iter().zip(other_spellings) {
            write!(output,"; {orthography}: {spelling}").expect("Could not write to Plain Text");
        }
        writeln!(output,"): {definition}").expect("Could not write to Plain Text");
    }

}

struct MarkdownLexiconWriter {
    orthographies: Vec<&'static str>
}

impl LexiconWriter for MarkdownLexiconWriter {
    fn initialize(_: &'static str, orthographies: Vec<&'static str>) -> Self {
        Self {
            orthographies
        }
    }

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str, output: &mut String) {
        write!(output,"**{main_spelling}**. ({word}").expect("Could not write to Markdown");
        for (orthography,spelling) in self.orthographies.iter().zip(other_spellings) {
            write!(output,"; {orthography}: *{spelling}*").expect("Could not write to Markdown");
        }
        writeln!(output,"): {definition}").expect("Could not write to Markdown");
    }

}

struct HTMLLexiconWriter {
    orthographies: Vec<&'static str>
}


impl LexiconWriter for HTMLLexiconWriter {
    fn initialize(_: &'static str, orthographies: Vec<&'static str>) -> Self {
        Self {
            orthographies
        }
    }

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str, output: &mut String) {
        let mut buffer = html_builder::Buffer::new();
        let mut p = buffer.p();
        write!(p.strong(),"{main_spelling}").expect("Could not write to HTML");
        write!(p,". ({word}").expect("Could not write to HTML");
        for (orthography,spelling) in self.orthographies.iter().zip(other_spellings) {
            write!(p,"; {orthography}: ").expect("Could not write to HTML");
            write!(p.em(),"{spelling}").expect("Could not write to HTML");
        }
        write!(p,"): {definition}").expect("Could not write orthography");
        write!(output,"{}",buffer.finish()).expect("Could not write to HTML");
    }



}



pub(crate) struct LexiconList {
    primary_orthography: usize,
    orthographies: Vec<&'static str>,
    entries: Vec<LexiconEntry>
}

impl LexiconList {

    pub(crate) const fn new(orthographies: Vec<&'static str>, primary_orthography: usize) -> Self {
        Self {
            primary_orthography,
            orthographies,
            entries: Vec::new()
        }
    }

    pub(crate) fn push_entry(&mut self, entry: LexiconEntry) {
        if entry.spelling.len() != self.orthographies.len() {
            panic!("LexiconEntry does not have the same number of spellings as the lexicon")
        }
        self.entries.push(entry);
    }


    fn into_string<Writer: LexiconWriter>(self, result: &mut String) {

        let mut main_orthography = "";
        let mut other_orthographies = Vec::new();

        for (i,orthography) in self.orthographies.iter().enumerate() {
            if i == self.primary_orthography {
                main_orthography = orthography
            } else {
                other_orthographies.push(*orthography);
            }
        }
        let mut writer = Writer::initialize(main_orthography, other_orthographies);

        for entry in self.entries {

            let mut main_spelling = "";
            let mut other_spellings = Vec::new();
            for (i,spelling) in entry.spelling.iter().enumerate() {
              if i == self.primary_orthography {
                main_spelling = spelling;
              } else {
                other_spellings.push(spelling.as_str())
              }
            }

            assert_ne!(main_spelling.len(),0,"Missing spelling for orthography {} in {}",self.primary_orthography,entry.word);

            writer.write_entry(main_spelling, &other_spellings, &entry.word, &entry.definition, result);

        }

    }

    fn into_table(self, style: &Format) -> TableOutput {
        let mut table = LexiconTable::new(self.orthographies, self.primary_orthography);

        for entry in self.entries {
            table.push_entry(entry);

        }

        table.into_output(style)
    }

    pub(crate) fn print_to_stdout(self, style: &Format) {


        match style {
            // NOTE: The Plain, Terminal, and Markdown lexicons should be in paragraph format, so I'm not using a TableWriter for those.
            // (The HTML output can be styled to look like paragraph format if you really want it that way)
            Format::Plain |
            Format::Terminal { .. } => {
                let mut result = String::new();
                self.into_string::<PlainLexiconWriter>(&mut result);
                print!("{result}")
            },
            Format::Markdown => {
                let mut result = String::new();
                self.into_string::<MarkdownLexiconWriter>(&mut result);
                print!("{result}")
            },
            Format::HTML { .. } => {
                let mut result = String::new();
                self.into_string::<HTMLLexiconWriter>(&mut result);
                print!("{result}")
            },
            Format::JSON |
            Format::CSV => {
                self.into_table(style).print_to_stdout();
            }
        }


    }
}



pub(crate) enum Lexicon {
    List(LexiconList),
    Table(LexiconTable)
}

impl Lexicon {

    pub(crate) fn new(style: &LexiconStyle, orthographies: Vec<&'static str>, primary_orthography_idx: usize) -> Self {
        match style {
            LexiconStyle::Table => Self::Table(LexiconTable::new(orthographies, primary_orthography_idx)),
            LexiconStyle::List => Self::List(LexiconList::new(orthographies, primary_orthography_idx)),
        }

    }

    pub(crate) fn push_entry(&mut self, entry: LexiconEntry) {
        match self {
            Lexicon::List(lexicon_list) => lexicon_list.push_entry(entry),
            Lexicon::Table(lexicon_table) => lexicon_table.push_entry(entry),
        }
    }

    pub(crate) fn print_to_stdout(self, style: &Format) {
        match self {
            Lexicon::List(lexicon_list) => {
                lexicon_list.print_to_stdout(style);
            },
            Lexicon::Table(lexicon_table) => {
                let output = lexicon_table.into_output(style);
                output.print_to_stdout();
            },
        }
    }


}
