use core::fmt::Write as _;
use std::fmt;
use core::fmt::Display;
use core::num::NonZeroUsize;
use html_builder::Html5 as _;
use prettytable::format::consts::FORMAT_BOX_CHARS;
use prettytable::format::consts::FORMAT_CLEAN;
use prettytable::format::FormatBuilder as PrettyFormatBuilder;
use prettytable::format::LinePosition;
use prettytable::format::LineSeparator;
use prettytable::format::TableFormat as PrettyTableFormat;
use prettytable::Table as PrettyTable;
use prettytable::Row as PrettyRow;
use prettytable::Cell as PrettyCell;
use std::io;
use crate::format::Format;

/*
NOTE: On my decision for text output tables:

This is what I needed:
- column spanning
- row spanning
- ability to turn off borders between two cells
- several different styles of table, including plain and markdown

- tabled https://crates.io/crates/tabled
  - CON: The architecture is oddly typed and makes this difficult to work with
    - Instead of just having methods that are specific to what you want to do, they have methods like 'with' that take traits that could change anything from styling to borders to specific cell options.
    - The builder pattern they use depends on methods that return &mut Self instead of Self, which is really difficult to work with:
      - I can't put them all in one line because the first builder method returns a reference that is immediately dropped.
      - Every once in a while, if you're not careful, you run into errors because you're trying to borrow mutably more than once.
    - The Border and Style types are generically typed based on whether borders (left, right, top, bottom) are turned on and off, among other things. This makes it more difficult to apply an arbitrary border style, as well as make it difficult to override the styles for the cells.
  - CON: The cell-specific options are specified as options separately from the cells. Meaning that when I want to span, I have to track columns and rows in an array, and add the spanning later, instead of having a 'span' option on the cell itself.
  - CON: Attempting to remove the left border to merge cells somehow completely changes the upper-left and lower-left border to something else, and I have to do a lot of other unexpected tweaking because of that.
- prettytable https://crates.io/crates/prettytable-rs/0.10.0
  - CON: Appears to be several years old, not sure how well it's maintained
  - CON: Can't support turning off borders for specific cells
  - PRO: Very simple architecture with few dependencies.
- comfytable https://crates.io/crates/comfy-table
  - CON: Spans are not supported *at all*, and documentation says that it will never be
- cli-table https://crates.io/crates/cli-table
  - CON: spans are not supported
- text-tables https://crates.io/crates/text-tables
  - CON: Does not support any sort of customization (it just renders a Vec<Vec<Str>> to a writer)
  - CON: Repository is gone
  - PRO: It's 153 lines of text, so I could just grab it.

Decision: I ended up going with pretty table, as it required the least customization. I can live without rowspans. And I found a way to remove borders, it just takes a bit of convoluted pre-processing. FUTURE: Consider taking the code and rewriting it to allow rowspanning and cell-specific border customization.

FUTURE: I could potentially pre-process the rows in the same way that I do for multi-columns, assuming that prettytable handles multi-line cells.
*/

#[expect(clippy::enum_variant_names,reason="They all start with the same text. But, I'm trying to represent the HTML class names, which won't be in a namespace.")]
pub(crate) enum TableClass {
    ElbiePhonemes,
    ElbieWords,
    ElbieOrthography,
    ElbieLexicon
}

impl Display for TableClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ElbiePhonemes => write!(f,"elbie phonemes"),
            Self::ElbieWords => write!(f,"elbie generated-words"),
            Self::ElbieOrthography => write!(f,"elbie orthography"),
            Self::ElbieLexicon => write!(f,"elbie lexicon"),
        }
    }
}

#[derive(Clone)]
pub(crate) enum TRHeadClass {
    ColumnHead,
    SubColumnHead
}

impl Display for TRHeadClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnHead => write!(f,"column-head"),
            Self::SubColumnHead => write!(f,"subcolumn-head"),
        }
    }
}

#[derive(Debug)]
#[expect(clippy::enum_variant_names,reason="They all start with the same text. But, I'm trying to represent the HTML class names, which won't be in a namespace.")]
pub(crate) enum TRBodyClass {
    BodyRow,
    BodyRowGroupStart,
    BodyRowGroupMiddle,
    BodyRowGroupEnd,
}

impl Display for TRBodyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyRow => write!(f,"body-row"),
            Self::BodyRowGroupStart => write!(f,"body-row row-group-start"),
            Self::BodyRowGroupMiddle => write!(f,"body-row row-group-middle"),
            Self::BodyRowGroupEnd => write!(f,"body-row row-group-end"),
        }
    }
}

#[derive(Clone)]
pub(crate) enum THColumnClass {
    ColumnHeader,
    SubColumnHeader
}

impl Display for THColumnClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnHeader => write!(f,"column-header"),
            Self::SubColumnHeader => write!(f,"subcolumn-header"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum THRowClass {
    RowHeader,
    SubrowHeader
}

impl Display for THRowClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RowHeader => write!(f,"row-header"),
            Self::SubrowHeader => write!(f,"subrow-header"),
        }
    }
}

#[derive(Debug)]
#[expect(clippy::enum_variant_names,reason="They all start with the same text. But, I'm trying to represent the HTML class names, which won't be in a namespace.")]
pub(crate) enum TDClass {
    ColumnGroupStart,
    ColumnGroupMiddle,
    ColumnGroupEnd
}

impl Display for TDClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnGroupStart => write!(f,"column-group-start"),
            Self::ColumnGroupMiddle => write!(f,"column-group-middle"),
            Self::ColumnGroupEnd => write!(f,"column-group-end"),
        }
    }
}

trait ZeroOneOrTwo {

    fn len(&self) -> usize;
}

impl<FirstType,SecondType> ZeroOneOrTwo for Option<(FirstType,Option<SecondType>)> {
    fn len(&self) -> usize {
        match self {
            Some((_,Some(_))) => 2,
            Some((_,None)) => 1,
            None => 0,
        }
    }
}



pub(crate) enum TextStyle {
    Plain,
    Terminal,
    Markdown
}

impl TextStyle {

    fn column_header(&self, text: &str) -> String {
        match self {
            Self::Plain |
            // FUTURE: I reserve the right to make this look different (perhaps use ansi codes to bold it?)
            Self::Terminal => text.to_owned(),
            Self::Markdown => format!("**{text}**"),
        }
    }

    fn row_header(&self, text: &str) -> String {
        match self {
            Self::Plain |
            // FUTURE: I reserve the right to make this look different (perhaps use ansi codes to bold it?)
            Self::Terminal => text.to_owned(),
            Self::Markdown => format!("**{text}**"),
        }
    }

}

pub(crate) enum TableOutput {
    Pretty(PrettyTable),
    // NOTE: PrettyTable has a 'write_html' function, but it uses style attributes and I can't control the class attributes
    HTML(html_builder::Buffer),
    JSON(json::JsonValue),
    CSV(Vec<csv::StringRecord>)
}

impl TableOutput {

    //#[must_use]
    //pub(crate) fn into_string(self) -> String {
    //    match self {
    //        Self::Pretty(table) => table.to_string(),
    //        Self::HTML(buffer) => buffer.finish(),
    //        Self::JSON(json_value) => json_value.pretty(2),
    //        Self::Text(text) => text
    //    }
    //}

    pub(crate) fn print_to_stdout(self) {
        match self {
            Self::Pretty(table) => table.printstd(),
            Self::HTML(buffer) => println!("{}",buffer.finish()),
            Self::JSON(json_value) => println!("{}",json_value.pretty(2)),
            Self::CSV(records) => {
                let mut builder = csv::WriterBuilder::new();
                _ = builder.quote_style(csv::QuoteStyle::Always);
                let mut writer = builder.from_writer(io::stdout());

                for record in records {
                    writer.write_record(&record).expect("Could not write CSV to stdout");
                }
            }
        }
    }
}


trait GridHeadCell {

    fn into_pretty(self, with_spans: bool, text_style: &TextStyle, row: &mut PrettyRow);

    fn into_html_with_span(self, tr: &mut html_builder::Node<'_>);

    fn into_html_without_span(self, tr: &mut html_builder::Node<'_>);


}

impl GridHeadCell for ColumnHeader {

    fn into_pretty(self, with_spans: bool, text_style: &TextStyle, row: &mut PrettyRow) {
        let text = text_style.column_header(&self.text);
        let cell = PrettyCell::new(&text).with_style(prettytable::Attr::Bold);
        if with_spans && let colspan @ 2.. = self.colspan.get() {
            row.add_cell(cell.with_hspan(colspan));
        } else {
            row.add_cell(cell);
            for _ in 1..self.colspan.get() {
                row.add_cell(PrettyCell::new(""));
            }
        }
    }

    fn into_html_with_span(self, tr: &mut html_builder::Node<'_>) {
        let th = tr.th().attr(&format!("class=\"{}\"",THColumnClass::ColumnHeader));
        let mut th = if self.colspan.get() > 1 {
            th.attr(&format!("colspan={}",self.colspan))
        } else {
            th
        };
        write!(th,"{}",self.text).expect("Could not write to html node");
    }

    fn into_html_without_span(self, tr: &mut html_builder::Node<'_>) {
        write!(tr.th().attr(&format!("class=\"{}\"",THColumnClass::ColumnHeader)),"{}",self.text).expect("Could not write to html node");
        for _ in 1..self.colspan.get() {
            _ = tr.td();
        }
    }


}



impl GridHeadCell for SubcolumnHeader {

    fn into_pretty(self, _with_spans: bool, text_style: &TextStyle, row: &mut PrettyRow) {
        let text = text_style.column_header(&self.text);
        let cell = PrettyCell::new(&text).with_style(prettytable::Attr::Bold);
        row.add_cell(cell);
    }

    fn into_html_with_span(self, tr: &mut html_builder::Node<'_>) {
        self.into_html_without_span(tr);
    }

    fn into_html_without_span(self, tr: &mut html_builder::Node<'_>) {
        write!(tr.th().attr(&format!("class=\"{}\"",THColumnClass::SubColumnHeader)),"{}",self.text).expect("Could not write to html node");
    }

}


trait GridHeadRow: Sized {

    type CellType: GridHeadCell;

    fn into_cells(self) -> impl Iterator<Item = Self::CellType>;

    fn get_class() -> TRHeadClass;

    fn into_html(self, row_header_offset: usize, column_header_offset: usize, with_span: bool, add_corner: bool, thead: &mut html_builder::Node<'_>) {
        let mut tr = thead.tr().attr(&format!("class=\"{}\"",Self::get_class()));
        if with_span {
            if add_corner && (row_header_offset > 0) {
                let th = tr.th();
                let th = if column_header_offset > 1 {
                    th.attr(&format!("colspan={column_header_offset}"))
                } else {
                    th
                };
                if row_header_offset > 1 {
                    _ = th.attr(&format!("rowspan={row_header_offset}"));
                }

            }
            for header in self.into_cells() {
                header.into_html_with_span(&mut tr);
            }
        } else {
            for _ in 0..row_header_offset {
                _ = tr.th();
            }
            for header in self.into_cells() {
                header.into_html_without_span(&mut tr);
            }
        }
    }

    fn into_pretty(self, with_spans: bool, text_style: &TextStyle, row_header_offset: usize) -> PrettyRow {
        let mut row = PrettyRow::empty();


        // need a corner square
        if row_header_offset > 0 {
            // FUTURE: PrettyTable does not support rowspanning, but maybe I can figure something out?
            let cell = PrettyCell::new("").with_hspan(row_header_offset);
            row.add_cell(cell);
        }

        for header in self.into_cells() {
            header.into_pretty(with_spans, text_style, &mut row);
        }
        row
    }

}


#[derive(Clone,Debug)]
pub(crate) struct ColumnHeader {
    text: String,
    colspan: NonZeroUsize
}

impl ColumnHeader {


    /// # Panics
    /// Panics if 1 is somehow equal to zero or less
    #[must_use]
    pub(crate) fn new(text: String, colspan: usize) -> Self {
        Self {
            text,
            colspan: NonZeroUsize::new(colspan).unwrap_or(NonZeroUsize::new(1).unwrap())
        }
    }

}


impl GridHeadRow for Vec<ColumnHeader> {

    type CellType = ColumnHeader;

    fn into_cells(self) -> impl Iterator<Item = Self::CellType> {
        self.into_iter()
    }

    fn get_class() -> TRHeadClass {
        TRHeadClass::ColumnHead
    }



}



#[derive(Clone)]
pub(crate) struct SubcolumnHeader {
    text: String
}

impl SubcolumnHeader {

    #[must_use]
    pub(crate) const fn new(text: String) -> Self {
        Self {
            text
        }
    }

}

impl GridHeadRow for Vec<SubcolumnHeader> {

    type CellType = SubcolumnHeader;

    fn into_cells(self) -> impl Iterator<Item = Self::CellType> {
        self.into_iter()
    }

    fn get_class() -> TRHeadClass {
        TRHeadClass::SubColumnHead
    }

}




#[derive(Debug)]
pub(crate) enum RowHeader {
    RowHeader{
        text: String,
        rowspan: NonZeroUsize
    },
    RowHeaderSpan
}

impl RowHeader {


    /// # Panics
    /// Panics if 1 is somehow equal to 0 or less.
    #[must_use]
    pub(crate) fn new(text: String, rowspan: usize) -> Self {
        Self::RowHeader{
            text,
            rowspan: NonZeroUsize::new(rowspan).unwrap_or(NonZeroUsize::new(1).unwrap())
        }
    }

    /// This is required when adding a new row where the previous cell had a rowspan > 1, to "line up" row headers that span multiple rows.
    ///
    /// This is simpler than trying to maintain row state and track the missing cells for the rowspan when building the final grid.
    ///
    #[must_use]
    pub(crate) const fn row_header_span() -> Self {
        Self::RowHeaderSpan
    }


}

#[derive(Debug)]
pub(crate) struct SubrowHeader {
    text: String
}

impl SubrowHeader {

    #[must_use]
    pub(crate) const fn new(text: String) -> Self {
        Self {
            text
        }
    }
}

#[derive(Debug)]
pub(crate) struct Cell {
    text: String,
    class: Option<TDClass>
}

impl Cell {


    /// # Panics
    /// Panics if any of the grid cells contain newlines
    #[must_use]
    pub(crate) fn content(text: String, class: Option<TDClass>) -> Self {
        assert!(!text.contains('\n'),"Grid cells must not contain newlines");
        Self{
            text,
            class
        }
    }




}


#[derive(Debug)]
pub(crate) struct GridRow {
    class: TRBodyClass,
    headers: Option<(RowHeader,Option<SubrowHeader>)>,
    cells: Vec<Cell>
}

impl GridRow {

    #[must_use]
    pub(crate) const fn new(class: TRBodyClass) -> Self {
        Self {
            class,
            headers: None,
            cells: Vec::new()
        }
    }

    /// # Panics
    /// Panics if the row header is already set.
    pub(crate) fn set_header(&mut self, header: RowHeader) {
        assert!(self.headers.is_none(),"Row header already set.");
        self.headers = Some((header,None))
    }

    /// # Panics
    /// Panics if there is no row header yet, or if the subrow header is already set.
    pub(crate) fn set_subheader(&mut self, new_subheader: SubrowHeader) {
        let Some((_,subheader)) = &mut self.headers else {
            panic!("Row headers must be set before subheaders")
        };

        assert!(subheader.is_none(),"Subrow header is already set.");

        *subheader = Some(new_subheader)

    }

    pub(crate) fn push_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }
}


pub(crate) struct Grid {
    class: TableClass,
    caption: String,
    heads: Option<(Vec<ColumnHeader>,Option<Vec<SubcolumnHeader>>)>,
    body: Vec<GridRow>
}

impl Grid {

    #[must_use]
    pub(crate) const fn new(class: TableClass, caption: String) -> Self {
        Self {
            caption,
            class,
            heads: None,
            body: Vec::new()
        }
    }

    #[must_use]
    pub(crate) fn caption(&self) -> &str {
        &self.caption
    }

    /// # Panics
    /// Function panics if column headers are already set
    pub(crate) fn set_headers(&mut self, row: Vec<ColumnHeader>) {
        assert!(self.heads.is_none(),"Column headers are already set");
        self.heads = Some((row,None))

    }

    /// # Panics
    /// Function panics if column headers are not set yet, if subheaders are already set, or the length of the subheaders does not match the length of the column headers (including colspan)
    pub(crate) fn set_subheaders(&mut self, row: Vec<SubcolumnHeader>) {
        let Some((head,subhead)) = &mut self.heads else {
            panic!("Column headers must be set before subheaders")
        };

        assert!(subhead.is_none(),"Subcolumn headers are already set.");
        assert_eq!(head.iter().map(|c| c.colspan.get()).sum::<usize>(),row.len(),"Subcolumn headers must match length of headers (including colspan).");

        *subhead = Some(row)

    }

    /// # Panics
    /// Function panics if the number of cells in the row does not match the header cells, or if the row header and cell lengths do not match previously added rows.
    pub(crate) fn push_body_row(&mut self, row: GridRow) {
        if let Some((head,_)) = &self.heads {
            assert_eq!(head.iter().map(|c| c.colspan.get()).sum::<usize>(),row.cells.len(),"Row cells must match length of headers (including colspan).");
        }
        if let Some(first_row) = self.body.first() {
            // Also, compare the body row headers to the first row added.
            assert_eq!(first_row.headers.len(),row.headers.len(),"Body row headers must have the same length.");
            assert_eq!(first_row.cells.len(),row.cells.len(),"Body row cells must have the same length.");
        }
        self.body.push(row);


    }


    #[must_use]
    fn into_html(mut self,with_span: bool) -> html_builder::Buffer {

        // need to know this for creating "corner" cell in the top-left
        let row_header_offset = self.body.first().map_or(0, |first| {
            first.headers.len()
        });
        let column_header_offset = self.heads.len();

        let mut buffer = html_builder::Buffer::new();
        let mut table = buffer.table().attr(&format!("class=\"{}\"",self.class));
        write!(table.caption(),"{}",self.caption).expect("Could not write to html node");

        let mut thead = table.thead();
        if let Some((head,subhead)) = self.heads.take() {
            head.into_html(row_header_offset, column_header_offset, with_span, true, &mut thead);
            if let Some(subhead) = subhead {
                subhead.into_html(row_header_offset, column_header_offset, with_span, false, &mut thead);
            }

        }

        let mut tbody = table.tbody();

        for row in self.body {
            let mut tr = tbody.tr().attr(&format!("class=\"{}\"",row.class));
            if let Some((header,subheader)) = row.headers {
                match header {
                    RowHeader::RowHeader { text, rowspan } => {
                        let th = tr.th().attr(&format!("class=\"{}\"",THRowClass::RowHeader));
                        let mut th = if with_span && rowspan.get() > 1 {
                            th.attr(&format!("rowspan={rowspan}"))
                        } else {
                            th
                        };
                        write!(th,"{text}").expect("Could not write to html node");
                    },
                    RowHeader::RowHeaderSpan => {
                        if !with_span {
                            // just put an empty row header here.
                            _ = tr.th();
                        }
                    },
                }

                if let Some(SubrowHeader { text }) = subheader {
                    let mut th = tr.th().attr(&format!("class=\"{}\"",THRowClass::SubrowHeader));
                    write!(th,"{text}").expect("Could not write to html node");
                }

            }

            for Cell { text, class } in row.cells {
                let mut td = if let Some(class) = class {
                    tr.td().attr(&format!("class=\"{class}\""))
                } else {
                    tr.td()
                };
                write!(&mut td,"{text}").expect("Could not write to html node.")
            }

        }


        buffer
    }

    #[must_use]
    fn into_json(self) -> json::JsonValue {


        let result = json::object!{
            "type": "elbie-grid",
            "class": self.class.to_string(),
            "head": if let Some((head,subhead)) = self.heads {
                json::object!{
                    "type": "column-head",
                    cells: head.iter().map(|header| {
                        json::object!{
                            "type": "column-header",
                            "colspan": header.colspan.get(),
                            "text": header.text.as_str()
                        }
                    }).collect::<Vec<_>>(),
                    subhead: if let Some(subhead) = subhead {
                        json::object!{
                            "type": "subcolumn-head",
                            cells: subhead.iter().map(|header| {
                                json::object!{
                                    "type": "column-header",
                                    "text": header.text.as_str()
                                }
                            }).collect::<Vec<_>>()
                        }
                    } else {
                        json::Null
                    }
                }

            } else {
                json::Null
            },
            "body": self.body.iter().map(|row| {
                json::object!{
                    "header": match &row.headers {
                        Some((header,subheader)) => {
                            let subheader = if let Some(SubrowHeader { text }) = subheader {
                                json::object! {
                                    "type": "row-header",
                                    "text": text.as_str()
                                }

                            } else {
                                json::JsonValue::Null
                            };


                            match header {
                                RowHeader::RowHeader { text, rowspan } => json::object! {
                                    "type": "row-header",
                                    "rowspan": rowspan.get(),
                                    "text": text.as_str(),
                                    "subheader": subheader
                                },
                                RowHeader::RowHeaderSpan => json::object! {
                                    "type": "row-header-span",
                                    "subheader": subheader
                                },
                            }
                        },
                        None => json::JsonValue::Null
                    },
                    "cells": row.cells.iter().map(|cell| {
                        json::object!{
                            "type": "cell",
                            "text": cell.text.as_str(),
                        }
                    }).collect::<Vec<_>>()
                }
            }).collect::<Vec<_>>()


        };

        result

    }

    #[must_use]
    fn into_csv(self) -> Vec<csv::StringRecord> {

        let row_header_offset = self.body.first().map_or(0, |first| {
            first.headers.len()
        });


        let mut output = Vec::new();

        if let Some((head,subhead)) = self.heads {

            let mut head_record = csv::StringRecord::new();
            // need a corner square
            for _ in 0..row_header_offset {
                head_record.push_field("");
            }

            for header in head {
                head_record.push_field(&header.text);
                for _ in 1..header.colspan.get() {
                    head_record.push_field("");
                }
            }

            output.push(head_record);


            if let Some(subhead) = subhead {

                let mut subhead_record = csv::StringRecord::new();

                for _ in 0..row_header_offset {
                    subhead_record.push_field("");
                }

                for header in subhead {
                    subhead_record.push_field(&header.text);
                }

                output.push(subhead_record);

            }
        }

        for row in self.body {

            let mut record = csv::StringRecord::new();
            if let Some((header,subheader)) = row.headers {

                match header {
                    RowHeader::RowHeader { text, rowspan: _ } => record.push_field(&text),
                    RowHeader::RowHeaderSpan => record.push_field(""),
                }

                if let Some(subheader) = subheader {
                    record.push_field(&subheader.text);
                }

            }

            for cell in row.cells {
                record.push_field(&cell.text);
            }

            output.push(record);

        };

        output

    }

    fn blend_column_groups(self) -> Self {


        let mut plain_table_format = *FORMAT_CLEAN;
        plain_table_format.padding(0, 0);
        plain_table_format.column_separator(' ');

        let Self {
            caption,
            class,
            heads: head,
            body
        } = self;

        match head {
            Some((head,Some(subhead))) => {
                // Don't blend if subheadings exist
                Self {
                    caption,
                    class,
                    heads: Some((head,Some(subhead))),
                    body
                }
            },
            None => {
                // Don't blend if there are no headings
                Self {
                    caption,
                    class,
                    heads: None,
                    body
                }
            },
            Some((head,None)) => {
                let mut column_groups = head.into_iter().map(|ch| {
                    (ch,Vec::new())
                }).collect::<Vec<_>>();

                let mut new_rows = Vec::new();
                for (row_idx,row) in body.into_iter().enumerate() {
                    new_rows.push(GridRow {
                        class: row.class,
                        headers: row.headers,
                        cells: Vec::new()
                    });
                    let old_cells = row.cells;

                    let mut cells_iter = old_cells.into_iter();
                    for (group_idx,(column_header,column_cells)) in column_groups.iter_mut().enumerate() {
                        let mut column_row = Vec::new();
                        for idx in 0..column_header.colspan.get() {
                            if let Some(Cell{ text, class: _ }) = cells_iter.next() {
                                column_row.push(text)
                            } else {
                                panic!("Missing grid cell #{idx} in row {row_idx}, for column header {group_idx} '{}'",column_header.text);
                            }
                        }
                        column_cells.push(column_row);
                    }

                }

                let mut new_headers = Vec::new();
                // blend the cells into single cells, and push onto the new rows.
                for (header,cells) in column_groups {
                    // create a table and format it into plain text
                    let mut table = PrettyTable::from(cells);
                    table.set_format(plain_table_format);
                    let table = table.to_string();
                    let lines = table.lines().map(|line| Cell::content(line.to_owned(), None));

                    // return the header to the headers, but with a colspan of 1.
                    new_headers.push(ColumnHeader::new(header.text, 1));

                    // assign the lines to the rows:
                    for (row,cell) in new_rows.iter_mut().zip(lines) {
                        row.cells.push(cell);
                    }

                }

                Self {
                    class,
                    caption,
                    heads: Some((new_headers,None)),
                    body: new_rows,
                }
            }
        }



    }

    fn blend_row_groups(self) -> Self {

        let Self {
            caption,
            class,
            heads,
            body
        } = self;

        let mut new_body = Vec::new();

        let mut body_iter = body.into_iter().peekable();
        while let Some(mut row) = body_iter.next() {
            if let Some((RowHeader::RowHeader { text: header_text, rowspan: _ },None)) = row.headers {
                while let Some(next_row) = body_iter.next_if(|next_row| {
                    // blend the next one if the row header is a span and there is no subheader.
                    matches!(next_row.headers,Some((RowHeader::RowHeaderSpan,None)))
                }) {
                    for (cell,next_cell) in row.cells.iter_mut().zip(next_row.cells) {
                        // blend the cell by pushing the next one down as a new line of the above.
                        // The PrettyTable stuff will take care of expanding the cell.
                        cell.text.push('\n');
                        cell.text.push_str(&next_cell.text);
                    }
                }
                // trim the extra lines off of the ends to compact the table a bit.
                for cell in &mut row.cells {
                    cell.text = cell.text.trim_end().to_owned()
                }
                new_body.push(GridRow {
                    class: row.class,
                    // set the rowspan to 1, just in case we support rowspan in the table output someday.
                    headers: Some((RowHeader::new(header_text, 1),None)),
                    cells: row.cells
                });

            } else {
                // we're not blending...
                new_body.push(row);
            }


        }


        Self {
            caption,
            class,
            heads,
            body: new_body
        }



    }

    /* NMS: Archive this, an old way of blending, depended on blended cells being a special Cell that took two strings.
    #[allow(clippy::shadow_unrelated,reason="I'm doing a lot of stuff where using the same name makes sense")]
    fn blend_multi_columns(self) -> Self {

        let mut plain_table_format = *FORMAT_CLEAN;
        plain_table_format.padding(0, 0);
        plain_table_format.column_separator(' ');

        let Self {
            caption,
            class,
            head,
            body
        } = self;

        let mut extracted_multi_cells = HashMap::new();

        let body_extracted = body.into_iter().enumerate().map(|(row_idx,row)| {
            let GridRow {
                headers,
                cells,
                class
            } = row;

            let cells = cells.into_iter().enumerate().map(|(col_idx,cell)| {
                match cell {
                    Cell::Group { cells: items, .. } => {
                        extracted_multi_cells.entry(col_idx).or_insert(Vec::new()).push((row_idx,items));
                        None
                    },
                    cell => Some(cell)
                }
            }).collect::<Vec<_>>();
            (headers,cells,class)

        }).collect::<Vec<_>>();



        // Not sure why I have to specify RandomState here, I've never had to before...
        let mut formatted_multi_cells: HashMap<_,_,RandomState> = HashMap::from_iter(extracted_multi_cells.into_iter().map(|(col_idx,indexed_cells)| {
            let (rows,cells): (Vec<_>,Vec<_>) = indexed_cells.into_iter().unzip();
            let mut table = PrettyTable::from(cells);
            table.set_format(plain_table_format);
            let table = table.to_string();
            let lines = table.lines().map(ToOwned::to_owned);
            let indexed_lines = rows.into_iter().zip(lines).collect::<HashMap<_,_>>();
            (col_idx,indexed_lines)

        }));

        #[allow(clippy::shadow_unrelated,reason="It is, in face, related")]
        let body = body_extracted.into_iter().enumerate().map(|(row_idx,(headers,some_cells,class))| {

            let cells = some_cells.into_iter().enumerate().map(|(col_idx,cell)| {
                match cell {
                    Some(cell) => cell,
                    None => match formatted_multi_cells.get_mut(&col_idx) {
                        Some(indexed_rows) => match indexed_rows.remove(&row_idx) {
                            Some(line) => Cell::content(line),
                            None => Cell::content(String::new()),
                        },
                        None => Cell::content(String::new()),
                    }
                }

            }).collect();


            GridRow {
                class,
                headers,
                cells
            }
        }).collect();


        Self {
            class,
            caption,
            head,
            body
        }

    }*/

    #[must_use]
    fn into_pretty(self, with_spans: bool, text_style: &TextStyle, format: PrettyTableFormat) -> PrettyTable {

        let row_header_offset = self.body.first().map_or(0, |first| {
            first.headers.len()
        });

        let mut table = PrettyTable::new();

        if let Some((head,subhead)) = self.heads {
            let row = head.into_pretty(with_spans, text_style, row_header_offset);
            table.set_titles(row);
            if let Some(subhead) = subhead {
                let subrow = subhead.into_pretty(with_spans, text_style, row_header_offset);
                _ = table.add_row(subrow);
            }
        }

        for body_row in &self.body {
            let mut row = PrettyRow::empty();

            if let Some((header,subheader)) = &body_row.headers {
                match header {
                    RowHeader::RowHeader { text, rowspan: _ } => {
                        // FUTURE: PrettyTable does not support rowspanning, but maybe I can figure out how to "hide" the lower border?
                        let text = text_style.row_header(text);
                        row.add_cell(PrettyCell::new(&text).with_style(prettytable::Attr::Bold))
                    },
                    RowHeader::RowHeaderSpan => {
                        row.add_cell(PrettyCell::new(""))
                    }
                }

                if let Some(SubrowHeader { text }) = subheader {
                    let text = text_style.row_header(text);
                    row.add_cell(PrettyCell::new(&text).with_style(prettytable::Attr::Bold))
                }
            }

            for Cell { text, class: _ } in &body_row.cells {
                row.add_cell(PrettyCell::new(text))
            }

            _ = table.add_row(row);

        }

        table.set_format(format);

        table

    }

    fn pretty_table_markdown() -> PrettyTableFormat {

        PrettyFormatBuilder::new().
            column_separator('|').
            separator(
                LinePosition::Title,
                LineSeparator::new('-', '|', '|', '|')
            ).
            left_border('|').
            right_border('|').
            padding(1, 1).
            build()

    }

    #[must_use]
    pub(crate) fn into_output(self, style: &Format) -> TableOutput {

        match style {
            Format::Plain => {
                // yes span the plain style, it makes it look much cleaner.
                let me = self.blend_column_groups();
                let me = me.blend_row_groups();
                let table = me.into_pretty(true,&TextStyle::Plain, *FORMAT_CLEAN);
                TableOutput::Pretty(table)
            },
            Format::Markdown => {
                let me = self.blend_column_groups();
                let me = me.blend_row_groups();
                let table = me.into_pretty(false,&TextStyle::Markdown, Self::pretty_table_markdown());
                TableOutput::Pretty(table)
            },
            Format::Terminal { spans } => {
                let me = self.blend_column_groups();
                let me = me.blend_row_groups();
                let table = me.into_pretty(*spans,&TextStyle::Terminal, *FORMAT_BOX_CHARS);
                TableOutput::Pretty(table)
            },
            Format::HTML { spans } => TableOutput::HTML(self.into_html(*spans)),
            Format::JSON => TableOutput::JSON(self.into_json()),
            // NOTE: While PrettyTable has a CSV output available, I've already got a more configurable CSV crate loaded.
            Format::CSV => TableOutput::CSV(self.into_csv())
        }
    }



}
