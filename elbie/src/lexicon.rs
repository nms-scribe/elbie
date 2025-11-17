use core::fmt::Write;
use crate::word::Word;
use crate::grid::GridStyle;
use crate::table_writer::TableWriter;
use crate::table_writer::HTMLTableWriter;
use crate::table_writer::JSONTableWriter;
use crate::table_writer::CSVTableWriter;


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


trait LexiconWriter<'output,Output: Write> {

    fn initialize(main_orthography: &'static str, orthographies: Vec<&'static str>, output: &'output mut Output) -> Self;

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str);

    fn finalize(self);

}

struct LexiconTableWriter<Writer> {
    orthographies: Vec<&'static str>,
    writer: Writer
}

impl<'output,Output: Write, Writer: TableWriter<'output,Output>> LexiconWriter<'output,Output> for LexiconTableWriter<Writer> {

    fn initialize(main_orthography: &'static str, orthographies: Vec<&'static str>, output: &'output mut Output) -> Self {
        let mut fields = Vec::new();
        fields.push(main_orthography.to_owned());
        fields.push("word".to_owned());
        for orthography in &orthographies {
            fields.push((*orthography).to_owned());
        }
        fields.push("definition".to_owned());

        let writer = Writer::initialize(fields, output);

        Self {
            orthographies,
            writer
        }
    }

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str) {
        let mut record = Vec::new();
        record.push(main_spelling.to_owned());
        let word = word.to_string();
        record.push(word);
        for spelling in other_spellings.iter().take(self.orthographies.len()) {
            record.push((*spelling).to_owned());
        }
        record.push(definition.to_owned());
        self.writer.write_record(record);
    }

    fn finalize(self) {
        self.writer.finalize();
    }
}

struct PlainLexiconWriter<'output,Output: Write> {
    orthographies: Vec<&'static str>,
    output: &'output mut Output
}

impl<'output,Output: Write> LexiconWriter<'output,Output> for PlainLexiconWriter<'output,Output> {

    fn initialize(_: &'static str, orthographies: Vec<&'static str>, output: &'output mut Output) -> Self {
        Self {
            orthographies,
            output
        }
    }

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str) {
        write!(self.output,"{main_spelling} ({word}").expect("Could not write orthography");
        for (orthography,spelling) in self.orthographies.iter().zip(other_spellings) {
            write!(self.output,"; {orthography}: {spelling}").expect("Could not write orthography");
        }
        writeln!(self.output,"): {definition}").expect("Could not write orthography");
    }

    fn finalize(self) {
    }



}

struct MarkdownLexiconWriter<'output,Output: Write> {
    orthographies: Vec<&'static str>,
    output: &'output mut Output
}

impl<'output,Output: Write> LexiconWriter<'output,Output> for MarkdownLexiconWriter<'output,Output> {
    fn initialize(_: &'static str, orthographies: Vec<&'static str>, output: &'output mut Output) -> Self {
        Self {
            orthographies,
            output
        }
    }

    fn write_entry(&mut self, main_spelling: &str, other_spellings: &[&str], word: &Word, definition: &str) {
        write!(self.output,"**{main_spelling}**. ({word}").expect("Could not write orthography");
        for (orthography,spelling) in self.orthographies.iter().zip(other_spellings) {
            write!(self.output,"; {orthography}: *{spelling}*").expect("Could not write orthography");
        }
        writeln!(self.output,"): {definition}").expect("Could not write orthography");
    }

    fn finalize(self) {
    }


}

pub(crate) struct Lexicon {
    primary_orthography: usize,
    orthographies: Vec<&'static str>,
    entries: Vec<LexiconEntry>
}

impl Lexicon {

    pub(crate) const fn new(orthographies: Vec<&'static str>, primary_orthography: usize) -> Self {
        Self {
            primary_orthography,
            orthographies,
            entries: Vec::new()
        }
    }

    pub(crate) fn push(&mut self, entry: LexiconEntry) {
        if entry.spelling.len() != self.orthographies.len() {
            panic!("LexiconEntry does not have the same number of spellings as the lexicon")
        }
        self.entries.push(entry);
    }


    fn into_output<'output,Output: Write, Writer: LexiconWriter<'output,Output>>(self, result: &'output mut Output) {

        let mut main_orthography = "";
        let mut other_orthographies = Vec::new();

        for (i,orthography) in self.orthographies.iter().enumerate() {
            if i == self.primary_orthography {
                main_orthography = orthography
            } else {
                other_orthographies.push(*orthography);
            }
        }
        let mut writer = Writer::initialize(main_orthography, other_orthographies, result);

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

            writer.write_entry(main_spelling, &other_spellings, &entry.word, &entry.definition);

        }

        writer.finalize();
    }

    /// # Panics
    /// panics if the `self.primary_orthography' is not a valid index in the supplied orthographies.
    #[must_use]
    pub(crate) fn into_string(self, style: &GridStyle) -> String {

        let mut result = String::new();

        match style {
            // NOTE: The Plain, Terminal, and Markdown lexicons should be in paragraph format, so I'm not using a TableWriter for those.
            // (The HTML output can be styled to look like paragraph format if you really want it that way)
            GridStyle::Plain |
            GridStyle::Terminal { .. } => self.into_output::<_,PlainLexiconWriter<_>>(&mut result),
            GridStyle::Markdown => self.into_output::<_,MarkdownLexiconWriter<_>>(&mut result),
            GridStyle::HTML { .. } => self.into_output::<_,LexiconTableWriter<HTMLTableWriter<_>>>(&mut result),
            GridStyle::JSON => self.into_output::<_,LexiconTableWriter<JSONTableWriter<_>>>(&mut result),
            GridStyle::CSV => self.into_output::<_,LexiconTableWriter<CSVTableWriter<_>>>(&mut result),
        }

        result

    }
}
