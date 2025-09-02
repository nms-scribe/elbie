use core::fmt::Write as _;
use std::fmt;
use core::fmt::Display;
use std::io;
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

pub enum TableClass {
    ElbiePhonemes,
    ElbieWords,
    ElbieOrthography
}

impl Display for TableClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ElbiePhonemes => write!(f,"elbie phonemes"),
            Self::ElbieWords => write!(f,"elbie generated-words"),
            Self::ElbieOrthography => write!(f,"elbie orthography"),
        }
    }
}

#[derive(Clone)]
pub enum TRHeadClass {
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
pub enum TRBodyClass {
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
pub enum THColumnClass {
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
pub enum THRowClass {
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

pub enum TDClass {
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

pub enum TextStyle {
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

// FUTURE: How about a CSV style? I've already got a CSV reader. NOTE: The prettytable plugin has CSV output available, but it doesn't quote the strings, so don't use that.
pub enum GridStyle {
    Plain,
    Terminal{ spans: bool },
    Markdown,
    HTML{ spans: bool },
    JSON
}


pub enum TableOutput {
    Pretty(PrettyTable),
    // NOTE: PrettyTable has a 'write_html' function, but it uses style attributes and I can't control the class attributes
    HTML(html_builder::Buffer),
    JSON(json::JsonValue),
    Text(String)
}

impl TableOutput {

    #[must_use]
    pub fn into_string(self) -> String {
        match self {
            Self::Pretty(table) => table.to_string(),
            Self::HTML(buffer) => buffer.finish(),
            Self::JSON(json_value) => json_value.pretty(2),
            Self::Text(text) => text
        }
    }

    pub fn print_to_stdout(self) -> Result<(),io::Error> {
        match self {
            Self::Pretty(table) => table.printstd(),
            Self::HTML(buffer) => println!("{}",buffer.finish()),
            Self::JSON(json_value) => println!("{}",json_value.pretty(2)),
            Self::Text(text) => println!("{text}")
        }

        Ok(())
    }
}


trait ColumnHeaderCell {

    fn into_pretty(self, with_spans: bool, text_style: &TextStyle, row: &mut PrettyRow);

    fn into_html_with_span(self, tr: &mut html_builder::Node<'_>);

    fn into_html_without_span(self, tr: &mut html_builder::Node<'_>);


}

#[derive(Clone,Debug)]
pub struct ColumnHeader {
    text: String,
    colspan: NonZeroUsize
}

impl ColumnHeader {


    /// # Panics
    /// Panics if 1 is somehow equal to zero or less
    #[must_use]
    pub fn new(text: String, colspan: usize) -> Self {
        Self {
            text,
            colspan: NonZeroUsize::new(colspan).unwrap_or(NonZeroUsize::new(1).unwrap())
        }
    }

}

impl ColumnHeaderCell for ColumnHeader {

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


#[derive(Debug)]
pub enum RowHeader {
    RowHeader{
        text: String,
        rowspan: NonZeroUsize,
        class: THRowClass
    },
    RowHeaderSpan
}

impl RowHeader {


    /// # Panics
    /// Panics if 1 is somehow equal to 0 or less.
    #[must_use]
    pub fn new(text: String, rowspan: usize, class: THRowClass) -> Self {
        Self::RowHeader{
            text,
            rowspan: NonZeroUsize::new(rowspan).unwrap_or(NonZeroUsize::new(1).unwrap()),
            class
        }
    }

    /// This is required when adding a new row where the previous cell had a rowspan > 1, to "line up" row headers that span multiple rows.
    ///
    /// This is simpler than trying to maintain row state and track the missing cells for the rowspan when building the final grid.
    ///
    #[must_use]
    pub const fn row_header_span() -> Self {
        Self::RowHeaderSpan
    }


}


#[derive(Debug)]
pub struct Cell {
    text: String
}

impl Cell {


    /// # Panics
    /// Panics if any of the grid cells contain newlines
    #[must_use]
    pub fn content(text: String) -> Self {
        assert!(!text.contains('\n'),"Grid cells must not contain newlines");
        Self{
            text
        }
    }




}


// TODO: Headers should be an Option<RowHeader,Option<SubrowHeader>>, similar to how columns and subcolumns are added.
// TODO: And SubrowHeader can't be a span.
#[derive(Debug)]
pub struct GridRow {
    class: TRBodyClass,
    headers: Vec<RowHeader>,
    cells: Vec<Cell>
}

impl GridRow {

    #[must_use]
    pub const fn new(class: TRBodyClass) -> Self {
        Self {
            class,
            headers: Vec::new(),
            cells: Vec::new()
        }
    }

    pub fn push_header(&mut self, header: RowHeader) {
        self.headers.push(header);
    }

    pub fn push_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }
}

trait GridHeadRow: Sized {

    type CellType: ColumnHeaderCell;

    fn into_cells(self) -> impl Iterator<Item = Self::CellType>;

    fn get_class() -> TRHeadClass;

    fn into_html(self, row_header_offset: usize, column_header_offset: usize, with_span: bool, add_corner: bool, thead: &mut html_builder::Node<'_>) {
        let mut tr = thead.tr().attr(&format!("class=\"{}\"",Self::get_class()));
        if with_span {
            if add_corner {
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
pub struct SubcolumnHeader {
    text: String
}

impl SubcolumnHeader {

    #[must_use]
    pub const fn new(text: String) -> Self {
        Self {
            text
        }
    }

}

impl ColumnHeaderCell for SubcolumnHeader {

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



impl GridHeadRow for Vec<SubcolumnHeader> {

    type CellType = SubcolumnHeader;

    fn into_cells(self) -> impl Iterator<Item = Self::CellType> {
        self.into_iter()
    }

    fn get_class() -> TRHeadClass {
        TRHeadClass::SubColumnHead
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

pub struct Grid {
    class: TableClass,
    caption: String,
    heads: Option<(Vec<ColumnHeader>,Option<Vec<SubcolumnHeader>>)>,
    body: Vec<GridRow>
}

impl Grid {

    #[must_use]
    pub const fn new(class: TableClass, caption: String) -> Self {
        Self {
            caption,
            class,
            heads: None,
            body: Vec::new()
        }
    }

    #[must_use]
    pub fn caption(&self) -> &str {
        &self.caption
    }

    /// # Panics
    /// Function panics if column headers are already set
    pub fn set_headers(&mut self, row: Vec<ColumnHeader>) {
        assert!(self.heads.is_none(),"Column headers are already set");
        self.heads = Some((row,None))

    }

    /// # Panics
    /// Function panics if column headers are not set yet, if subheaders are already set, or the length of the subheaders does not match the length of the column headers (including colspan)
    pub fn set_subheaders(&mut self, row: Vec<SubcolumnHeader>) {
        let Some((head,subhead)) = &mut self.heads else {
            panic!("Column headers must be set before subheaders")
        };

        assert!(subhead.is_none(),"Subcolumn headers are already set.");
        assert_eq!(head.iter().map(|c| c.colspan.get()).sum::<usize>(),row.len(),"Subcolumn headers must match length of headers (including colspan).");

        *subhead = Some(row)

    }

    /// # Panics
    /// Function panics if the number of cells in the row does not match the header cells, or if the row header and cell lengths do not match previously added rows.
    pub fn push_body_row(&mut self, row: GridRow) {
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
    pub fn into_html(mut self,with_span: bool) -> html_builder::Buffer {

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
            for header in row.headers {
                match header {
                    RowHeader::RowHeader { text, rowspan, class } => {
                        let th = tr.th().attr(&format!("class=\"{class}\""));
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
            }

            for Cell { text } in row.cells {
                write!(tr.td(),"{text}").expect("Could not write to html node.")
            }

        }


        buffer
    }

    #[must_use]
    pub fn into_json(self) -> json::JsonValue {


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
                    "headers": row.headers.iter().map(|header| {
                        match header {
                            RowHeader::RowHeader { text, rowspan, class } => json::object! {
                                "type": "row-header",
                                "class": class.to_string(),
                                "rowspan": rowspan.get(),
                                "text": text.as_str()
                            },
                            RowHeader::RowHeaderSpan => json::object! {
                                "type": "row-header-span"
                            },
                        }
                    }).collect::<Vec<_>>(),
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
                            if let Some(Cell{ text }) = cells_iter.next() {
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
                    let lines = table.lines().map(|line| Cell::content(line.to_owned()));

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
    pub fn into_pretty(self, with_spans: bool, text_style: &TextStyle, format: PrettyTableFormat) -> PrettyTable {

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

            for header in &body_row.headers {
                // FUTURE: PrettyTable does not support rowspanning, but maybe I can figure out how to "hide" the lower border?
                match header {
                    RowHeader::RowHeader { text, rowspan: _, class: _ } => {
                        let text = text_style.row_header(text);
                        row.add_cell(PrettyCell::new(&text).with_style(prettytable::Attr::Bold))
                    },
                    RowHeader::RowHeaderSpan => {
                        row.add_cell(PrettyCell::new(""))
                    }
                }
            }

            for Cell { text } in &body_row.cells {
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
    pub fn into_output(self, style: &GridStyle) -> TableOutput {

        match style {
            GridStyle::Plain => {
                // yes span the plain style, it makes it look much cleaner.
                let me = self.blend_column_groups();
                let table = me.into_pretty(true,&TextStyle::Plain, *FORMAT_CLEAN);
                TableOutput::Pretty(table)
            },
            GridStyle::Markdown => {
                let me = self.blend_column_groups();
                let table = me.into_pretty(false,&TextStyle::Markdown, Self::pretty_table_markdown());
                TableOutput::Pretty(table)
            },
            GridStyle::Terminal { spans } => {
                let me = self.blend_column_groups();
                let table = me.into_pretty(*spans,&TextStyle::Terminal, *FORMAT_BOX_CHARS);
                TableOutput::Pretty(table)
            },
            GridStyle::HTML { spans } => TableOutput::HTML(self.into_html(*spans)),
            GridStyle::JSON => TableOutput::JSON(self.into_json())
        }
    }



}
