use core::fmt::Write as _;
use std::collections::HashMap;
use std::hash::RandomState;
use html_builder::Html5 as _;
use prettytable::format::FormatBuilder as PrettyFormatBuilder;
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

pub struct ColumnHeader {
    text: String,
    colspan: usize,
    class: &'static str
}


impl ColumnHeader {

    #[must_use]
    pub const fn new(text: String, colspan: usize, class: &'static str) -> Self {
        Self {
            text,
            colspan,
            class
        }
    }

}


#[derive(Debug)]
pub enum RowHeader {
    RowHeader{
        text: String,
        rowspan: usize,
        class: &'static str
    },
    RowHeaderSpan
}

impl RowHeader {


    #[must_use]
    pub const fn new(text: String, rowspan: usize, class: &'static str) -> Self {
        Self::RowHeader{
            text,
            rowspan,
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
pub enum Cell {
    Content{
        text: String,
        class: &'static str
    },
    Group{
        cells: Vec<String>,
        class: &'static str
    },
}

impl Cell {


    #[must_use]
    pub fn content(text: String, class: &'static str) -> Self {
        assert!(!text.contains('\n'),"Grid cells must not contain newlines");
        Self::Content{
            text,
            class
        }
    }

    #[must_use]
    pub fn cell_group(cells: Vec<String>, class: &'static str) -> Self {
        assert!(!cells.iter().any(|c| c.contains('\n')),"Grid cells must not contain newlines.");
        Self::Group{
            cells,
            class
        }

    }



}


pub struct GridRow {
    headers: Vec<RowHeader>,
    cells: Vec<Cell>
}

impl GridRow {

    #[must_use]
    pub const fn new() -> Self {
        Self {
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

pub enum TabledBorderRequest {
    // upper_left and lower_left must remove notches
    MainHeaderAfter,
    // lower_left must remove notches
    SubHeaderAfter,
    // lower_left must remove notch, and left must have a 'space' border
    CellGroupAfter
}

pub struct Grid {
    class: &'static str,
    headers: Vec<Vec<ColumnHeader>>,
    body: Vec<GridRow>
}

impl Grid {

    #[must_use]
    pub const fn new(class: &'static str) -> Self {
        Self {
            class,
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    /// # Panics
    /// If there is a header row in the grid already, the sum of the first row's colspan values is compared to that of this one, and if they differ this will panic.
    pub fn push_header_row(&mut self, row: Vec<ColumnHeader>) {
        if let Some(first_row) = self.headers.first() {
            let new_len: usize = row.iter().map(|c| c.colspan).sum();
            let first_len: usize = first_row.iter().map(|c| c.colspan).sum();
            assert_eq!(first_len,new_len,"Header rows must have the same length.");
        }
        self.headers.push(row);
    }

    /// # Panics
    /// If there is a row in the grid already, and the new row does not match it's dimensions, then this will panic.
    pub fn push_body_row(&mut self, row: GridRow) {
        if let Some(first_row) = self.body.first() {
            assert_eq!(first_row.headers.len(),row.headers.len(),"Body row headers must have the same length.");
            assert_eq!(first_row.cells.len(),row.cells.len(),"Body row cells must have the same length.");
        }
        self.body.push(row);
    }

}

pub enum TableOutput {
    Pretty(PrettyTable),
    // NOTE: PrettyTable has a 'write_html' function, but it uses style attributes and I can't control the class attributes
    HTML(html_builder::Buffer),
    JSON(json::JsonValue),
    Text(String)
}

impl TableOutput {

    pub fn into_string(self) -> String {
        match self {
            Self::Pretty(table) => table.to_string(),
            Self::HTML(buffer) => buffer.finish(),
            Self::JSON(json_value) => json_value.pretty(2),
            Self::Text(text) => text
        }
    }

    pub fn print_to_stdout(self) -> Result<(),std::io::Error> {
        match self {
            Self::Pretty(table) => table.printstd(),
            Self::HTML(buffer) => println!("{}",buffer.finish()),
            Self::JSON(json_value) => println!("{}",json_value.pretty(2)),
            Self::Text(text) => println!("{text}")
        }

        Ok(())
    }
}


impl Grid {

    #[must_use]
    pub fn into_html(self,with_span: bool) -> html_builder::Buffer {

        // need to know this for creating "corner" cell in the top-left
        let row_header_offset = self.body.first().map_or(0, |first| {
            first.headers.len()
        });
        let column_header_offset = self.headers.len();

        let mut buffer = html_builder::Buffer::new();
        let mut table = buffer.table().attr(&format!("class=\"{}\"",self.class));
        let mut thead = table.thead();
        for (i,headers) in self.headers.iter().enumerate() {
            let mut tr = thead.tr();
            if with_span {
                if i == 0 {
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
                for header in headers {
                    let th = tr.th().attr(&format!("class=\"{}\"",header.class));
                    let mut th = if header.colspan > 1 {
                        th.attr(&format!("colspan={}",header.colspan))
                    } else {
                        th
                    };
                    write!(th,"{}",header.text).expect("Could not write to html node");
                }
            } else {
                for _ in 0..row_header_offset {
                    _ = tr.th();
                }
                for header in headers {
                    write!(tr.th().attr(&format!("class=\"{}\"",header.class)),"{}",header.text).expect("Could not write to html node");
                    for _ in 1..header.colspan {
                        _ = tr.td();
                    }
                }
            }
        }

        let mut tbody = table.tbody();

        for row in self.body {
            let mut tr = tbody.tr();
            for header in row.headers {
                match header {
                    RowHeader::RowHeader { text, rowspan, class } => {
                        let th = tr.th().attr(&format!("class=\"{class}\""));
                        let mut th = if with_span && rowspan > 1 {
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

            for cell in row.cells {
                match cell {
                    Cell::Content{ text, class } => {
                        write!(tr.td().attr(&format!("class=\"{class}\"")),"{text}").expect("Could not write to html node.")
                    },
                    Cell::Group{ cells, class} => {
                        for cell in cells {
                            write!(tr.td().attr(&format!("class=\"{class}\"")),"{cell}").expect("Could not write to html node.")
                        }
                    },
                }
            }

        }


        buffer
    }

    #[must_use]
    pub fn into_json(self) -> json::JsonValue {


        let result = json::object!{
            "type": "elbie-grid",
            "class": self.class,
            "headers": [
                self.headers.iter().map(|header| {
                    header.iter().map(|header| {
                        json::object!{
                            "type": "column-header",
                            "class": *header.class,
                            "colspan": header.colspan,
                            "text": header.text.as_str()
                        }

                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>()
            ],
            "body": self.body.iter().map(|row| {
                json::object!{

                    "headers": row.headers.iter().map(|header| {
                        match header {
                            RowHeader::RowHeader { text, rowspan, class } => json::object! {
                                "type": "row-header",
                                "class": *class,
                                "rowspan": *rowspan,
                                "text": text.as_str()
                            },
                            RowHeader::RowHeaderSpan => json::object! {
                                "type": "row-header-span"
                            },
                        }
                    }).collect::<Vec<_>>(),
                    "cells": row.cells.iter().map(|cell| {
                        match cell {
                            Cell::Content { text, class } => json::object!{
                                "type": "cell",
                                "class": *class,
                                "text": text.as_str(),
                            },
                            Cell::Group { cells, class } => json::object!{
                                "type": "multi-cell",
                                "class": *class,
                                "cells": cells.iter().map(String::as_str).collect::<Vec<_>>()
                            },
                        }
                    }).collect::<Vec<_>>()
                }
            }).collect::<Vec<_>>()


        };

        result

    }

    fn blend_multi_columns(self) -> Self {

        let mut plain_table_format = prettytable::format::consts::FORMAT_CLEAN.clone();
        plain_table_format.padding(0, 0);
        plain_table_format.column_separator(' ');

        let Self {
            class,
            headers,
            body,
        } = self;

        let headers = headers.into_iter().map(|header| {
            header.into_iter().map(|mut cell| {
                cell.colspan = 1;
                cell
            }).collect()

        }).collect();

        let mut extracted_multi_cells = HashMap::new();

        let body_extracted = body.into_iter().enumerate().map(|(row_idx,row)| {
            let GridRow {
                headers,
                cells,
            } = row;

            let cells = cells.into_iter().enumerate().map(|(col_idx,cell)| {
                match cell {
                    Cell::Group { cells, .. } => {
                        extracted_multi_cells.entry(col_idx).or_insert(Vec::new()).push((row_idx,cells));
                        None
                    },
                    cell => Some(cell)
                }
            }).collect::<Vec<_>>();
            (headers,cells)

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

        let body = body_extracted.into_iter().enumerate().map(|(row_idx,(headers,some_cells))| {

            let cells = some_cells.into_iter().enumerate().map(|(col_idx,cell)| {
                match cell {
                    Some(cell) => cell,
                    None => match formatted_multi_cells.get_mut(&col_idx) {
                        Some(indexed_rows) => match indexed_rows.remove(&row_idx) {
                            Some(line) => Cell::content(line.to_owned(),""),
                            None => Cell::content(String::new(),""),
                        },
                        None => Cell::content(String::new(),""),
                    }
                }

            }).collect();


            GridRow {
                headers,
                cells
            }
        }).collect();


        Self {
            class,
            headers,
            body
        }

    }

    pub fn into_pretty(self, with_spans: bool, text_style: &TextStyle, format: PrettyTableFormat) -> PrettyTable {

        let row_header_offset = self.body.first().map_or(0, |first| {
            first.headers.len()
        });

        let mut table = PrettyTable::new();

        for (row_idx,header_row) in self.headers.iter().enumerate() {
            let mut row = PrettyRow::empty();


            // need a corner square
            if row_header_offset > 0 {
                // FUTURE: PrettyTable does not support rowspanning, but maybe I can figure something out?
                let cell = PrettyCell::new("").with_hspan(row_header_offset);
                row.add_cell(cell);
            }

            for header in header_row.iter() {
                let text = text_style.column_header(&header.text);
                let cell = PrettyCell::new(&text).with_style(prettytable::Attr::Bold);
                if with_spans && let colspan @ 2.. = header.colspan {
                    row.add_cell(cell.with_hspan(colspan));
                } else {
                    row.add_cell(cell);
                    for _ in 1..header.colspan {
                        row.add_cell(PrettyCell::new(""));
                    }
                }
            }

            if row_idx == 0 {
                // NOTE: This is only done so that markdown can draw the heading line.
                // Github-flavored Markdown only supports one header row, even though we provide more.
                // FUTURE: If I were ever to support other formats that support table heading lines, then:
                // 1) I will need to add both rows as titles
                // 2) Except for Markdown, where I should only output one.
                table.set_titles(row);

            } else {
                _ = table.add_row(row);

            }

        }

        for body_row in self.body.iter() {
            let mut row = PrettyRow::empty();

            for header in body_row.headers.iter() {
                // FUTURE: PrettyTable does not support rowspanning, but maybe I can figure out how to "hide" the lower border?
                match header {
                    RowHeader::RowHeader { text, rowspan: _, class: _ } => {
                        let text = text_style.row_header(&text);
                        row.add_cell(PrettyCell::new(&text).with_style(prettytable::Attr::Bold))
                    },
                    RowHeader::RowHeaderSpan => {
                        row.add_cell(PrettyCell::new(""))
                    }
                }
            }

            for cell in body_row.cells.iter() {
                match cell {
                    Cell::Content { text, class: _  } => row.add_cell(PrettyCell::new(&text)),
                    Cell::Group { cells, class: _  } => for cell in cells.iter() {
                        // FUTURE: Figure out how to hide the border between these?
                        row.add_cell(PrettyCell::new(cell));
                    },
                }
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
                prettytable::format::LinePosition::Title,
                prettytable::format::LineSeparator::new('-', '|', '|', '|')
            ).
            left_border('|').
            right_border('|').
            padding(1, 1).
            build()

    }

    pub fn into_output(self, style: &GridStyle) -> TableOutput {

        match style {
            GridStyle::Plain => {
                // yes span the plain style, it makes it look much cleaner.
                let me = self.blend_multi_columns();
                let table = me.into_pretty(true,&TextStyle::Plain, prettytable::format::consts::FORMAT_CLEAN.clone());
                TableOutput::Pretty(table)
            },
            GridStyle::Markdown => {
                let table = self.into_pretty(false,&TextStyle::Markdown, Self::pretty_table_markdown());
                TableOutput::Pretty(table)
            },
            GridStyle::Terminal { spans } => {
                let me = self.blend_multi_columns();
                let table = me.into_pretty(*spans,&TextStyle::Terminal, prettytable::format::consts::FORMAT_BOX_CHARS.clone());
                TableOutput::Pretty(table)
            },
            GridStyle::HTML { spans } => TableOutput::HTML(self.into_html(*spans)),
            GridStyle::JSON => TableOutput::JSON(self.into_json())
        }
    }



}
