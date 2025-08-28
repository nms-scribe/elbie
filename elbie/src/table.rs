/*
TODO: Rethinking Chart, instead, use this concept:

struct Table4D
  columns: [(String,[String])] -- separate item for each main column, underneath there are separate ones for each subcolumn
  rows: [(String,[String])] -- same for rows and subrows
  grid: [[[Phoneme]]] -- a two-dimensional grid, each one contains a list of phonemes to display
  TODO: How do I make this so I don't have to 'index'? Maybe...
    HashMap<(Column,SubColumn,Row,SubRow),Vec<Phoneme>>

TODO: Functionality I need is:
- define the table with columns, subcolumns, rows, subrows
  - each has both captions and set names
- iterate through columns, subcolumns, rows, and subrow set names and add in phonemes that match that set
- print the table to output in various formats, including JSON.

 */

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;
use std::fmt::Write as _;
use tabled::settings::Style;
use tabled::builder::Builder;

use crate::Phoneme;

#[derive(Debug)]
pub(crate) enum Axis {
    Column,
    Subcolumn,
    Row,
    Subrow
}


#[derive(Clone,Debug)]
pub(crate) struct HeaderDef {
    caption: &'static str,
    order: usize
}

impl HeaderDef {

    fn new(caption: &'static str, order: usize) -> Self {
        Self {
            caption,
            order
        }
    }
}

#[derive(Default,Debug)]
pub(crate) struct Table4D {
    columns_by_set: HashMap<&'static str,HeaderDef>,
    subcolumns_by_set: HashMap<&'static str,HeaderDef>,
    rows_by_set: HashMap<&'static str,HeaderDef>,
    subrows_by_set: HashMap<&'static str,HeaderDef>,
    grid: HashMap<(usize,usize,usize,usize),HashSet<Rc<Phoneme>>>
}


macro_rules! table_add_fn {
    ($name: ident,$member: ident) => {
        pub(crate) fn $name(&mut self, set: &'static str, name: &'static str) {
            let next_id = self.$member.len();
            let column = HeaderDef::new(name,next_id);
            match self.$member.entry(set) {
                Entry::Vacant(entry) => _ = entry.insert(column),
                Entry::Occupied(_) => ()
            }
        }

    };
}


impl Table4D {

    table_add_fn!(add_column, columns_by_set);
    table_add_fn!(add_row, rows_by_set);
    table_add_fn!(add_subcolumn, subcolumns_by_set);
    table_add_fn!(add_subrow, subrows_by_set);

    pub(crate) fn add_phoneme(&mut self, column: &'static str, subcolumn: &'static str, row: &'static str, subrow: &'static str, phoneme: &Rc<Phoneme>) -> Result<bool,Axis> {
        let column = self.columns_by_set.get(&column).ok_or(Axis::Column)?;
        let subcolumn = self.subcolumns_by_set.get(&subcolumn).ok_or(Axis::Subcolumn)?;
        let row = self.rows_by_set.get(&row).ok_or(Axis::Row)?;
        let subrow = self.subrows_by_set.get(&subrow).ok_or(Axis::Subrow)?;
        match self.grid.entry((column.order,subcolumn.order,row.order,subrow.order)) {
            Entry::Occupied(mut entry) => {
                Ok(entry.get_mut().insert(phoneme.clone()))
            },
            Entry::Vacant(entry) => {
                Ok(entry.insert(HashSet::new()).insert(phoneme.clone()))
            },
        }

    }

}

pub(crate) trait Cell: Sized {

    fn new_corner(colspan: usize, rowspan: usize) -> Self;

    fn new_corner_span_fill() -> Option<Self>;

    fn new_column_header(caption: String, colspan: usize) -> Self;

    fn new_column_header_span_fill() -> Option<Self>;

    fn new_row_header(caption: String, rowspan: usize) -> Self;

    fn new_row_header_span_fill() -> Option<Self>;

    fn new_content(text: String) -> Self;

}

#[derive(Debug)]
pub(crate) enum CellWithSpan {
    Corner(usize,usize),
    ColumnHeader(String,usize),
    RowHeader(String,usize),
    Content(String)
}

impl Display for CellWithSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Corner(col,row) => write!(f,"<{col}:{row}>"),
            Self::ColumnHeader(header,span) |
            Self::RowHeader(header,span) => write!(f,"<*{header}:{span}*>"),
            Self::Content(text) => write!(f,"<{text}>"),
        }
    }
}



impl Cell for CellWithSpan {
    fn new_corner(colspan: usize, rowspan: usize) -> Self {
        Self::Corner(colspan, rowspan)
    }

    fn new_corner_span_fill() -> Option<Self> {
        None
    }

    fn new_column_header(caption: String, colspan: usize) -> Self {
        Self::ColumnHeader(caption, colspan)
    }

    fn new_column_header_span_fill() -> Option<Self> {
        None
    }

    fn new_row_header(caption: String, rowspan: usize) -> Self {
        Self::RowHeader(caption, rowspan)
    }

    fn new_row_header_span_fill() -> Option<Self> {
        None
    }

    fn new_content(text: String) -> Self {
        Self::Content(text)
    }
}

#[derive(Debug)]
pub(crate) enum CellWithoutSpan {
    Corner,
    ColumnHeader(Option<String>),
    RowHeader(Option<String>),
    Content(String)
}

impl Display for CellWithoutSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Corner => write!(f,"<>"),
            Self::ColumnHeader(None) |
            Self::RowHeader(None) => write!(f,"<**>"),
            Self::ColumnHeader(Some(header)) |
            Self::RowHeader(Some(header)) => write!(f,"<*{header}*>"),
            Self::Content(text) => write!(f,"<{text}>"),
        }
    }
}

impl Cell for CellWithoutSpan {
    fn new_corner(_: usize, _: usize) -> Self {
        Self::Corner
    }

    fn new_corner_span_fill() -> Option<Self> {
        Some(Self::Corner)
    }

    fn new_column_header(caption: String, _: usize) -> Self {
        Self::ColumnHeader(Some(caption))
    }

    fn new_column_header_span_fill() -> Option<Self> {
        Some(Self::ColumnHeader(None))
    }

    fn new_row_header(caption: String, _: usize) -> Self {
        Self::RowHeader(Some(caption))
    }

    fn new_row_header_span_fill() -> Option<Self> {
        Some(Self::RowHeader(None))
    }

    fn new_content(text: String) -> Self {
        Self::Content(text)
    }
}



trait HashMapToHeaderDefs {

    fn hashmap_to_captions(&self) -> Vec<&HeaderDef> ;

    fn hashmap_to_captions_len(&self) -> (Vec<&HeaderDef> ,usize) {
        let result = self.hashmap_to_captions();
        let len = result.len();
        (result,len)
    }
}

impl HashMapToHeaderDefs for HashMap<&'static str, HeaderDef> {

    fn hashmap_to_captions(&self) -> Vec<&HeaderDef> {
        let mut values: Vec<_> = self.values().collect();
        values.sort_by_key(|c| c.order);
        values
    }
}


impl Table4D {

    // TODO: Try turning subcols, subrows, and even columns into options, and how easily does that fix this? Then this just becomes a Table struct.
    // TODO: Then, the markdown for this should output HTML, so we get access to spans and headings.
    fn build_cells<CellType: Cell>(&self) -> Vec<Vec<CellType>> {
        let columns = self.columns_by_set.hashmap_to_captions();
        let (subcolumns,subcolumns_count) = self.subcolumns_by_set.hashmap_to_captions_len();
        let rows: Vec<_> = self.rows_by_set.hashmap_to_captions();
        let (subrows,subrows_count) = self.subrows_by_set.hashmap_to_captions_len();

        let mut result = Vec::new();

        result.push(Self::build_columns_row(&columns, subcolumns_count));

        result.push(Self::build_subcolumns_row(&columns, &subcolumns));

        // rows
        for row in &rows {

            for subrow in &subrows {
                let row = self.build_grid_row(&columns, &subcolumns, row, subrows_count, subrow);
                result.push(row)

            }
        }

        result

    }

    fn build_grid_row<CellType: Cell>(&self, columns: &Vec<&HeaderDef>, subcolumns: &Vec<&HeaderDef>, row_def: &HeaderDef, subrows_count: usize, subrow_def: &HeaderDef) -> Vec<CellType> {
        let mut row = Vec::new();
        if subrow_def.order == 0 {
            row.push(CellType::new_row_header(row_def.caption.to_owned(), subrows_count));
        } else if let Some(cell) = CellType::new_row_header_span_fill() {
            row.push(cell)
        }; // else, the one above should span into this.


        let subrow_idx = subrow_def.order;
        row.push(CellType::new_row_header(subrow_def.caption.to_owned(), 1));

        for column in columns {
            let column_idx = column.order;
            {
                for subcolumn in subcolumns {
                    let content = self.build_cell(column_idx, &subcolumn.order, row_def.order, subrow_idx);

                    row.push(content);
                }
            };

        }
        row
    }

    fn build_columns_row<CellType: Cell>(columns: &Vec<&HeaderDef>, span: usize) -> Vec<CellType> {
        // main column headers:
        let mut row = Vec::new();
        row.push(CellType::new_corner(2,2));
        // TODO: If we don't have subrows, this will not happen
        if let Some(cell) = CellType::new_corner_span_fill() {
            row.push(cell)
        }
        Self::build_column_headers(columns, span, &mut row);
        row
    }

    fn build_subcolumns_row<CellType: Cell>(columns: &Vec<&HeaderDef>, subcolumns: &Vec<&HeaderDef> ) -> Vec<CellType> {
        // subcolumn headers
        let mut row = Vec::new();
        // push to fill in the corner spans
        // TODO: If we don't have subrows, this will only happen once
        for _ in 0..2 {
            if let Some(cell) = CellType::new_corner_span_fill() {
                row.push(cell)
            }
        }

        // don't push a corner, because the one above has a rowspan of two
        for _ in columns {
            Self::build_column_headers(subcolumns, 1, &mut row);
        }
        row
    }

    fn build_column_headers<CellType: Cell>(columns: &Vec<&HeaderDef>, span: usize, row: &mut Vec<CellType>) {
        for column_def in columns {
            row.push(CellType::new_column_header(column_def.caption.to_owned(), span));
            for _ in 1..span {
                if let Some(cell) = CellType::new_column_header_span_fill() {
                    row.push(cell)
                }
            }
        }
    }

    fn build_cell<CellType: Cell>(&self, column_idx: usize, subcolumn_idx: &usize, row_idx: usize, subrow_idx: usize) -> CellType {
        let subcolumn_idx = *subcolumn_idx;
        let phonemes = match self.grid.get(&(column_idx,subcolumn_idx,row_idx,subrow_idx)) {
            Some(phonemes) => {
                let mut phonemes: Vec<_> = phonemes.iter().map(|p| p.to_string()).collect();
                phonemes.sort();
                phonemes
            },
            None => Vec::new(),
        };
        let content = CellType::new_content(phonemes.join(" "));
        content
    }

    pub(crate) fn build_to_string<CellType: Cell + Display>(&self) -> String {
        let cells = self.build_cells::<CellType>();
        let mut result = String::new();

        for row in cells {
            for column in row {
                write!(result,"{column}").unwrap();
            }
            result.push('|');
            result.push('\n');
        }

        result
    }

    pub(crate) fn build_markdown_no_spans(&self) -> String {
        let cells = self.build_cells::<CellWithoutSpan>();
        let mut builder = Builder::new();
        for row in cells {
            let mut record = Vec::new();
            for cell in row {
                match cell {
                    CellWithoutSpan::Corner => record.push(String::new()),
                    CellWithoutSpan::ColumnHeader(None) |
                    CellWithoutSpan::RowHeader(None) => record.push(String::new()),
                    CellWithoutSpan::ColumnHeader(Some(header)) |
                    CellWithoutSpan::RowHeader(Some(header)) => record.push(format!("**{header}**")),
                    CellWithoutSpan::Content(content) => record.push(content),
                }
            }
            builder.push_record(record);
        }
        let mut table = builder.build();
        let table = table.with(Style::markdown());
        table.to_string()
    }

    pub(crate) fn build_plain_no_spans(&self) -> String {
        let cells = self.build_cells::<CellWithoutSpan>();
        let mut builder = Builder::new();
        for row in cells {
            let mut record = Vec::new();
            for cell in row {
                match cell {
                    CellWithoutSpan::Corner => record.push(String::new()),
                    CellWithoutSpan::ColumnHeader(None) |
                    CellWithoutSpan::RowHeader(None) => record.push(String::new()),
                    CellWithoutSpan::ColumnHeader(Some(header)) |
                    CellWithoutSpan::RowHeader(Some(header)) => record.push(header),
                    CellWithoutSpan::Content(content) => record.push(content),
                }
            }
            builder.push_record(record);
        }
        let mut table = builder.build();
        let table = table.with(Style::empty());
        table.to_string()
    }

}
