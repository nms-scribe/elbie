use std::collections::HashMap;

use std::fmt::Write as _;
use html_builder::Html5 as _;
use tabled::builder::Builder;
use tabled::settings::themes::BorderCorrection;
use tabled::settings::Span;
use tabled::settings::Style;

pub enum TextStyle {
    Plain,
    Terminal,
    Markdown
}

impl TextStyle {

    fn column_header(&self, text: &str) -> String {
        match self {
            TextStyle::Plain |
            TextStyle::Terminal { .. } => text.to_owned(),
            TextStyle::Markdown { .. } => format!("**{text}**"),
        }
    }

    fn row_header(&self, text: &str) -> String {
        match self {
            TextStyle::Plain |
            TextStyle::Terminal { .. } => text.to_owned(),
            TextStyle::Markdown { .. } => format!("**{text}**"),
        }
    }

}


pub enum GridStyle {
    Plain,
    Terminal{ spans: bool },
    Markdown{ spans: bool },
    HTML{ spans: bool },
}


impl GridStyle {

    fn to_text_style(&self) -> TextStyle {
        match self {
            GridStyle::Plain => TextStyle::Plain,
            GridStyle::Terminal { .. } => TextStyle::Terminal,
            GridStyle::Markdown { .. } => TextStyle::Markdown,
            // HTML style has it's own ways of specifying this stuff, so it just returns plain.
            GridStyle::HTML { .. } => TextStyle::Plain,
        }
    }
}

pub struct ColumnHeader {
    text: String,
    colspan: usize,
    class: &'static str
}


impl ColumnHeader {

    pub fn new(text: String, colspan: usize, class: &'static str) -> Self {
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


    pub fn row_header(text: String, rowspan: usize, class: &'static str) -> Self {
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
    pub fn row_header_span() -> Self {
        Self::RowHeaderSpan
    }


}


#[derive(Debug)]
pub enum Cell {
    Content{
        text: String,
        class: &'static str
    },
    MultiColumnCell{
        cells: Vec<String>,
        class: &'static str
    },
}

impl Cell {


    pub fn content(text: String, class: &'static str) -> Self {
        Self::Content{
            text,
            class
        }
    }

    pub fn multi_col_cell(cells: Vec<String>, class: &'static str) -> Self {
        Self::MultiColumnCell{
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

    pub fn new() -> Self {
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


pub struct Grid {
    class: &'static str,
    headers: Vec<Vec<ColumnHeader>>,
    body: Vec<GridRow>
}

impl Grid {

    pub fn new(class: &'static str) -> Self {
        Self {
            class,
            headers: Vec::new(),
            body: Vec::new()
        }
    }

    pub fn push_header_row(&mut self, row: Vec<ColumnHeader>) {
        if let Some(first_row) = self.headers.first() {
            let new_len: usize = row.iter().map(|c| c.colspan).sum();
            let first_len: usize = first_row.iter().map(|c| c.colspan).sum();
            assert_eq!(first_len,new_len,"Header rows must have the same length.");
        }
        self.headers.push(row);
    }

    pub fn push_body_row(&mut self, row: GridRow) {
        if let Some(first_row) = self.body.first() {
            assert_eq!(first_row.headers.len(),row.headers.len(),"Body row headers must have the same length.");
            assert_eq!(first_row.cells.len(),row.cells.len(),"Body row cells must have the same length.");
        }
        self.body.push(row);
    }

    fn blend_columns_to_text(&mut self) {

        let mut table_index: HashMap<_, Vec<_>> = HashMap::new();
        // build an index of multi_cell values by column index
        for (row_idx,row) in self.body.iter().enumerate() {
            for (col_idx,cell) in row.cells.iter().enumerate() {
                if let Cell::MultiColumnCell{ cells, class} = cell {

                    table_index.entry(col_idx).or_default().push((row_idx,cells.clone(),class.to_owned()));

                }
            }
        }

        for (col_idx,list) in table_index {
            let mut plain_builder = Builder::new();
            let mut rows = Vec::new();
            for (row_idx,cells,class) in list {
                plain_builder.push_record(cells);
                rows.push((row_idx,class));
            }

            let mut table = plain_builder.build();
            let table = table.with(Style::blank());
            let text = table.to_string();

            for ((row_idx,class),line) in rows.into_iter().zip(text.lines()) {
                if let Some(row) = self.body.get_mut(row_idx) {
                    if let Some(cell) = row.cells.get_mut(col_idx) {
                        * cell = Cell::content(line.to_owned(), class);
                    } else {
                        panic!("While merging columns, couldn't found cell at {row_idx}:{col_idx}")
                    }
                } else {
                    panic!("While merging columns, couldn't found row {row_idx} for {col_idx}")

                }
            }

        }



    }


    // NOTE: I don't ordinarily like single-letter type names, but I'm just trying to match the Style
    pub fn into_tabled<T, B, L, R, H, V, const HSIZE: usize, const VSIZE: usize>(mut self, with_span: bool, text_style: TextStyle, table_style: Style<T, B, L, R, H, V, HSIZE, VSIZE>) -> tabled::Table {

        self.blend_columns_to_text();

        let mut builder = Builder::new();
        let mut row_spans = Vec::new();
        let mut col_spans = Vec::new();

        // need to know this for creating "corner" cell in the top-left
        let row_header_offset = self.body.first().map(|first| {
            first.headers.len()
        }).unwrap_or(0);
        let column_header_offset = self.headers.len();

        for (row_idx,row) in self.headers.iter().enumerate() {
            let mut record = Vec::new();

            // add "corner" cells
            if row_header_offset > 0 {
                for _ in 0..row_header_offset {
                    record.push(String::new())
                }
                if row_idx == 0 {
                    col_spans.push((row_idx,0,row_header_offset));
                    row_spans.push((row_idx,0,column_header_offset));
                }
            }

            for (col_idx,cell) in row.iter().enumerate() {
                let ColumnHeader {
                    text,
                    colspan,
                    class: _
                } = cell;

                record.push(text_style.column_header(text));

                if colspan > &1 {
                    let colspan = *colspan;
                    col_spans.push((row_idx,row_header_offset + col_idx,colspan));
                    // push in empty cells for the column to span over (without these, the captions overwrite each other)
                    for _ in 1..colspan {
                        record.push(String::new())
                    }
                }

            }
            builder.push_record(record);
        }

        for (row_idx,row) in self.body.iter().enumerate() {
            let mut record = Vec::new();
            for (col_idx,cell) in row.headers.iter().enumerate() {
                match cell {
                    RowHeader::RowHeader { text, rowspan, class: _ } => {
                        if rowspan > &1 {
                            row_spans.push((column_header_offset + row_idx,col_idx,*rowspan))
                        }
                        record.push(text_style.row_header(text))
                    },
                    RowHeader::RowHeaderSpan => {
                        // even if I'm going to set them to spanning later, I still need an empty cell for it to span over.
                        // (this get's written over, so I can't use this mechanism for blending stuff... )
                        record.push(String::new());
                    }
                };

            }

            for cell in row.cells.iter() {
                let text = match cell {
                    Cell::Content{ text, class: _ } => text.to_owned(),
                    Cell::MultiColumnCell{..} => {
                        // This should have been processed out by `process_multi_cells` above, so I'm going to ignore this...
                        String::new()
                    }
                };

                record.push(text)
            }
            builder.push_record(record);
        }
        let mut table = builder.build();
        if with_span {
            for (row,col,span) in row_spans {
                _ = table.modify(tabled::settings::object::Cell::new(row,col), Span::row(span as isize));
            }
            for (row,col,span) in col_spans {
                _ = table.modify(tabled::settings::object::Cell::new(row,col), Span::column(span as isize));
            }

            _ = table.with(BorderCorrection::span());

        }





        _ = table.with(table_style);

        table
    }

    pub fn into_html(self,with_span: bool) -> String {

        // need to know this for creating "corner" cell in the top-left
        let row_header_offset = self.body.first().map(|first| {
            first.headers.len()
        }).unwrap_or(0);
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
                    Cell::MultiColumnCell{ cells, class} => {
                        // HTML format should automatically turn off column blending, but just in case:
                        let text = cells.join(" ");
                        write!(tr.td().attr(&format!("class=\"{class}\"")),"{text}").expect("Could not write to html node.")
                    },
                }
            }

        }


        buffer.finish()
    }

    pub fn into_string(self, style: &GridStyle) -> String {

        let text_style = style.to_text_style();

        match style {
            GridStyle::Plain => self.into_tabled(false,text_style,Style::blank()).to_string(),
            GridStyle::Terminal { spans } => self.into_tabled(*spans,text_style,Style::modern()).to_string(),
            GridStyle::Markdown { spans } => self.into_tabled(*spans,text_style,Style::markdown()).to_string(),
            GridStyle::HTML { spans } => self.into_html(*spans).to_string(),
        }

    }


}
