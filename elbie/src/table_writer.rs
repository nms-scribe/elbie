use core::fmt::Write;
use html_builder::Html5 as _;

// NOTE: This is not the same as the grid. This is for much "dumber" tables, like those output in lexicons and validation.


pub(crate) trait TableWriter<'output,Output: Write> {

    fn initialize(header: Vec<String>, output: &'output mut Output) -> Self;

    fn write_record(&mut self, fields: Vec<String>);

    fn finalize(self);

}

pub(crate) struct HTMLTableWriter<'output,Output: Write> {
    output: &'output mut Output,
    header: Vec<String>,
    data: Vec<Vec<String>>,
}

impl<'output,Output: Write> TableWriter<'output,Output> for HTMLTableWriter<'output,Output> {
    fn initialize(header: Vec<String>, output: &'output mut Output) -> Self {

        let data = Vec::new();

        Self {
            output,
            header,
            data
        }
    }

    fn write_record(&mut self, fields: Vec<String>) {
        self.data.push(fields);
    }

    fn finalize(self) {
        let mut buffer = html_builder::Buffer::new();
        let mut table = buffer.table();
        {
            let mut tr = table.tr();
            for field in self.header {
                write!(tr.th(),"{field}").expect("Could not write HTML")
            }
        }
        for record in self.data {
            let mut tr = table.tr();
            for field in record {
                write!(tr.td(),"{field}").expect("Could not write HTML")
            }
        }

        write!(self.output,"{}",buffer.finish()).expect("Could not write HTML");
    }



}

pub(crate) struct JSONTableWriter<'output,Output: Write> {
    header: Vec<String>,
    output: &'output mut Output,
    array: json::Array
}

impl<'output,Output: Write> TableWriter<'output,Output> for JSONTableWriter<'output,Output> {

    fn initialize(header: Vec<String>, output: &'output mut Output) -> Self {
        Self {
            header,
            output,
            array: json::Array::new()
        }
    }

    fn write_record(&mut self, fields: Vec<String>) {
        #[expect(clippy::absolute_paths,reason="'Object' is too vague, so I'd rather leave this fully qualified")]
        let mut object = json::object::Object::new();
        for (key,value) in self.header.iter().zip(fields) {
            object.insert(key, value.into());
        }
        self.array.push(json::JsonValue::Object(object));
    }

    fn finalize(self) {
        writeln!(self.output,"{:#}",json::JsonValue::Array(self.array)).expect("Could not write json")
    }



}

pub(crate) struct CSVTableWriter<'output,Output: Write> {
    output: &'output mut Output,
    csv_writer: csv::Writer<Vec<u8>>
}

impl<'output,Output: Write> TableWriter<'output,Output> for CSVTableWriter<'output,Output> {
    fn initialize(header: Vec<String>, output: &'output mut Output) -> Self {
        let mut builder = csv::WriterBuilder::new();
        let mut csv_writer = builder.quote_style(csv::QuoteStyle::Always).from_writer(Vec::new());

        csv_writer.write_record(header).expect("Could not write CSV");

        Self {
            output,
            csv_writer
        }
    }

    fn write_record(&mut self, fields: Vec<String>) {
        self.csv_writer.write_record(fields).expect("Could not write CSV");
    }

    fn finalize(self) {
        let buffer = String::from_utf8(self.csv_writer.into_inner().expect("Could not get CSV record")).expect("CSV was not utf8");
        // it seems to put a linefeed at the end on it's own, so I don't need the 'ln' on 'write!'.
        write!(self.output,"{buffer}").expect("Could not write CSV to output");
    }


}
