use std::fmt;
use unicode_width::UnicodeWidthStr;
use pad::PadStr;
use crate::VecHelper;

#[derive(Clone,Debug)]
pub enum ChartStyle {
    Plain, // columns separated by spaces
    Terminal, // columns separated by '|'
    Markdown, // columns separated and lines bordered by '|', header separated from rest by '===', header text enclosed in '**..**'
    LaTeX, // columns separated by '&', lines end with '\\', header text enclosed in '\textbf{..}'
    // TODO: HTML, // written out as html markup.
}


macro_rules! plain_spacer {
    () => {
        " "
    };
}

macro_rules! pipe_spacer {
    () => {
        " | "
    };
}

macro_rules! and_spacer {
    () => {
        " & "
    };
}

macro_rules! latex {
    (multicolumn) => {
        "multicolumn"
    };
    (multirow) => {
        "multirow"
    };
    (textbf) => {
        "textbf"
    };
    (ipa) => {
        "ipa"
    };
    ($command: ident $( $argument: tt )+) => {
       concat!("\\",latex!($command),$(latex!($argument)),+) 
    };
    ({ $command: ident $( $argument: tt )+ }) => {
        concat!("{{",latex!($command $( $argument )+ ),"}}")

    };
    ({ $text: ident }) => {
        concat!("{{",stringify!($text),"}}")
    };
    ({*}) => {
        "{{*}}"
    };
    ({?}) => {
        "{{{}}}"
    };
    ({}) => {
        "{{}}"
    };
    
}

impl ChartStyle {

    pub fn get_phoneme_text(&self, phoneme: String) -> String {
        match self {
            ChartStyle::Plain => phoneme,
            ChartStyle::Terminal => phoneme,
            ChartStyle::Markdown => phoneme,
            ChartStyle::LaTeX => format!(latex!(ipa{?}),phoneme),
        }.to_owned()

    }

    fn display_spacer(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => write!(f,plain_spacer!()),
            Self::Terminal => write!(f,pipe_spacer!()),
            Self::Markdown => write!(f,pipe_spacer!()),
            Self::LaTeX => write!(f,and_spacer!()),
        }
    }

    fn display_row_span_spacer(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => write!(f,plain_spacer!()),
            Self::Terminal => write!(f,pipe_spacer!()),
            Self::Markdown => write!(f,pipe_spacer!()),
            Self::LaTeX => write!(f,and_spacer!()),
        }
    }

    fn get_spacer_width(&self) -> usize {
        match self {
            Self::Plain => plain_spacer!().len(),
            Self::Terminal => pipe_spacer!().len(),
            Self::Markdown => pipe_spacer!().len(),
            Self::LaTeX => and_spacer!().len(),
        }
    }

    fn display_start_row(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => Ok(()),
            Self::Terminal => write!(f,"| "),
            Self::Markdown => write!(f,"| "),
            Self::LaTeX => Ok(())
        }
    }

    fn display_end_row(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain => Ok(()),
            Self::Terminal => write!(f," |"),
            Self::Markdown => write!(f," |"),
            Self::LaTeX => write!(f,"\\\\")
        }
    }

    fn display_header_break(&self, f: &mut fmt::Formatter<'_>, columns: &Vec<usize>) -> fmt::Result {
        match self {
            Self::Plain => Ok(()),
            Self::Terminal => Ok(()),
            Self::Markdown => {
                write!(f,"|=")?;

                let length = columns.len();

                for (i,column) in columns.iter().enumerate() {
                    write!(f,"{}","".pad_to_width_with_char(*column,'='))?;
                    if i < (length - 1) {
                        write!(f,"=|=")?;
                    }
                }

                write!(f,"=|")?;
                writeln!(f)
            },
            Self::LaTeX => Ok(())
        }
    }


}

trait ChartCell {


    fn render_text(text: &str, is_header: bool, style: &ChartStyle, col_span: usize, row_span: usize) -> String {

        match style {
            ChartStyle::Plain => text.to_owned(),
            ChartStyle::Terminal => text.to_owned(),
            ChartStyle::Markdown => if is_header && !text.is_empty() {
                "**".to_owned() + text + "**"
            } else {
                text.to_owned()
            },
            ChartStyle::LaTeX => match (col_span, row_span,text.is_empty(),is_header) {
                (2.., 2.., _, _) => panic!("Can't have both col_span and row_span greater than 1"),
                (2.., 0..=1, true, _) => format!(latex!(multicolumn{?}{l}{textbf{}}),col_span),
                (2.., 0..=1, false, true) => format!(latex!(multicolumn{?}{l}{textbf{?}}),col_span,text),
                (2.., 0..=1, false, false) => format!(latex!(multicolumn {?} {l} {?}),col_span,text),
                (0..=1, 2.., true, _) => format!(latex!(multirow{?}{*}{}),row_span),
                (0..=1, 2.., false, true) => format!(latex!(multirow{?}{*}{textbf{?}}),row_span,text),
                (0..=1, 2.., false, false) => format!(latex!(multirow{?}{*}{?}),row_span,text),
                (_, _, true, _) => format!(""),
                (_, _, false, true) => format!(latex!(textbf{?}),text),
                (_, _, false, false) => format!("{}",text),
            }

        }

    }

    fn get_text(&self) -> &str;

    fn calculate_width(&self) -> usize {
        UnicodeWidthStr::width(self.get_text())
        
    }

    fn display_text(text: &str, f: &mut fmt::Formatter<'_>, width: &usize) -> fmt::Result { 
        // NOTE: I can't use the standard padding mechanism with rust formatting because it doesn't account for unicode width.
        // The pad library uses the same unicode_width crate that I'm using.
        let text = text.pad_to_width(*width);
        //assert_eq!(UnicodeWidthStr::width(text.as_str()),*width);
        write!(f,"{}",text)
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, width: &usize) -> fmt::Result {
        Self::display_text(self.get_text(),f,width)

    }


}

struct ChartTextCell {
    text: String
}

impl ChartTextCell {

    fn new(text: &str, style: &ChartStyle) -> Self {
        if text.contains('\n') {
            panic!("Can't create grid cell with multiple lines.");
        }
        Self {
            text: Self::render_text(text, false, style, 1, 1)
        }
    }


}

impl ChartCell for ChartTextCell {

    fn get_text(&self) -> &str {
        &self.text
    }
}

struct ChartHeaderCell{
    text: String,
    span: usize
}

impl ChartHeaderCell {

    fn new(text: &str, span: usize, style: &ChartStyle, is_row_header: bool) -> Self {
        if text.contains('\n') {
            panic!("Can't create grid cell with multiple lines.");
        }
        let (col_span, row_span) = if is_row_header {
            (1, span)
        } else {
            (span, 1)
        };
        Self {
            text: Self::render_text(text, true, style, col_span, row_span),
            span
        }
    }

    fn get_span(&self) -> &usize {
        &self.span
    }

}

impl ChartCell for ChartHeaderCell {

    fn get_text(&self) -> &str {
        &self.text
    }
}

trait ChartRow {


    fn display_inside(&self, f: &mut fmt::Formatter<'_>, style: &ChartStyle, columns: &Vec<usize>, row_span: &mut usize) -> fmt::Result;



    fn display(&self, f: &mut fmt::Formatter<'_>, style: &ChartStyle, columns: &Vec<usize>, row_span: &mut usize) -> fmt::Result {

        style.display_start_row(f)?;

        self.display_inside(f, style, columns, row_span)?;

        style.display_end_row(f)?;
        writeln!(f)?;

        Ok(())
    }

}

struct ChartBodyRow {
    row_header: Option<ChartHeaderCell>,
    cells: Vec<ChartTextCell>
}

impl ChartBodyRow {

    fn new() -> Self {
        Self {
            row_header: None,
            cells: Vec::new()
        }
    }

    fn add_cell(&mut self, cell: ChartTextCell) {
        self.cells.push(cell)
    }

    fn set_row_header(&mut self, cell: ChartHeaderCell) {
        self.row_header = Some(cell);
    }

}

impl ChartRow for ChartBodyRow {


    fn display_inside(&self, f: &mut fmt::Formatter<'_>, style: &ChartStyle, columns: &Vec<usize>, row_span: &mut usize) -> fmt::Result {

        let cells_length = self.cells.len();
        let mut j = 0;
        if let Some(cell) = &self.row_header {
            // row headers can't colspan
            cell.display(f, &columns[j])?;
            if cells_length > 0 {
                style.display_spacer(f)?;
            }
            j += 1;
            *row_span = cell.get_span() - 1;

        } else if *row_span > 0 {
            // the previous header had a larger row_span, so shift things over.
            *row_span -= 1;
            // display blank content here...
            ChartTextCell::display_text("", f, &columns[j])?;
            if cells_length > 0 {
                style.display_row_span_spacer(f)?;
            }
            j += 1;

        }

        for (i,cell) in self.cells.iter().enumerate() {
            let col_width = columns[j];
            cell.display(f, &col_width)?;
            if i < (cells_length - 1) {
                style.display_spacer(f)?;
            }
            j += 1;

        }

        Ok(())
    }

}

struct ChartHeaderRow {
    row_header: bool,
    cells: Vec<ChartHeaderCell>
}

impl ChartHeaderRow {

    fn new() -> Self {
        Self {
            row_header: false,
            cells: Vec::new()
        }
    }

    fn add_cell(&mut self, cell: ChartHeaderCell) {
        self.cells.push(cell)
    }

    fn include_row_header(&mut self) {
        self.row_header = true;
    }

}

impl ChartRow for ChartHeaderRow {

    fn display_inside(&self, f: &mut fmt::Formatter<'_>, style: &ChartStyle, columns: &Vec<usize>, row_span: &mut usize) -> fmt::Result {


        let cells_length = self.cells.len();

        let mut j = 0;
        if self.row_header {
            // row headers can't colspan
            ChartHeaderCell::display_text("", f, &columns[j])?;
            if cells_length > 0 {
                style.display_spacer(f)?;
            }
            j += 1;
            *row_span = 0;

        }

        for (i,cell) in self.cells.iter().enumerate() {
            let mut col_width = columns[j];
            for k in 1..*cell.get_span() {
                col_width += style.get_spacer_width() + columns[j+k]; // FUTURE: Do I need to check this? What if they put in a colspan that went beyond the table?
            };
            cell.display(f, &col_width)?;
            if i < (cells_length - 1) {
                style.display_spacer(f)?;
            }
            j += cell.get_span();

        }

        Ok(())
    }
}



/* NOTE: This is some old code which I'm keeping around in case I broke something...
impl ChartRowOld {

    fn new() -> Self {
        Self {
            row_header: None,
            cells: Vec::new()
        }
    }

    fn add_cell(&mut self, cell: ChartCellOld) {
        self.cells.push(cell)
    }

    fn set_row_header(&mut self, cell: ChartCellOld) {
        self.row_header = Some(cell);
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, style: &ChartStyle, columns: &Vec<usize>, row_span: &mut usize) -> fmt::Result {

        let cells_length = self.cells.len();
        style.display_start_row(f)?;

        let mut j = 0;
        if let Some(cell) = &self.row_header {
            // row headers can't colspan
            cell.display(f, &columns[j])?;
            if cells_length > 0 {
                style.display_spacer(f)?;
            }
            j += 1;
            *row_span = cell.row_span() - 1;

        } else if *row_span > 0 {
            // the previous header had a larger row_span, so shift things over.
            *row_span -= 1;
            // display blank content here...
            ChartCellOld::display_text("", f, &columns[j])?;
            if cells_length > 0 {
                style.display_row_span_spacer(f)?;
            }
            j += 1;

        }

        for (i,cell) in self.cells.iter().enumerate() {
            let mut col_width = columns[j];
            for k in 1..*cell.col_span() {
                col_width += style.get_spacer_width() + columns[j+k]; // FUTURE: Do I need to check this? What if they put in a colspan that went beyond the table?
            };
            cell.display(f, &col_width)?;
            if i < (cells_length - 1) {
                style.display_spacer(f)?;
            }
            j += cell.col_span();

        }

        style.display_end_row(f)?;
        writeln!(f)?;

        Ok(())
    }


}
*/


pub struct Chart {
    style: ChartStyle,
    col_headers: Option<ChartHeaderRow>,
    children: Vec<ChartBodyRow>
}

impl Chart {

    pub fn new(style: ChartStyle) -> Self {
        Self {
            style,
            col_headers: None,
            children: Vec::new()
        }
    }

    pub fn add_col_header_cell(&mut self, text: &str, col_span: usize) {
        let cell = ChartHeaderCell::new(text, col_span, &self.style,false);

        let row = self.col_headers.get_or_insert_with(|| ChartHeaderRow::new());
        row.add_cell(cell);
    }

    pub fn add_cell(&mut self, text: &str) {
        let cell = ChartTextCell::new(text,&self.style);
        if let Some(row) = self.children.last_mut() {
            row.add_cell(cell)
        } else {
            let mut row = ChartBodyRow::new();
            row.add_cell(cell);
            self.children.push(row)
        }
    }

    fn add_row_header_cell_at_maybe(&mut self, index: Option<usize>, text: &str, row_span: usize) {
        if let Some(row) = self.col_headers.as_mut() {
            row.include_row_header()
        };
        let cell = ChartHeaderCell::new(text, row_span, &self.style,true);
        if let Some(index) = index {
            if let Some(row) = self.children.get_mut(index) {
                row.set_row_header(cell)
            } else {
                panic!("Can't set a row header at index {}",index);
            }
        } else if let Some(row) = self.children.last_mut() {
            row.set_row_header(cell)
        } else {
            let mut row = ChartBodyRow::new();
            row.set_row_header(cell);
            self.children.push(row)
        }
    }



    pub fn add_row_header_cell(&mut self, text: &str, row_span: usize) {
        self.add_row_header_cell_at_maybe(None, text, row_span)
    }

    pub fn add_row_header_cell_at(&mut self, index: usize, text: &str, row_span: usize) {
        self.add_row_header_cell_at_maybe(Some(index), text, row_span)
    }

    pub fn add_row(&mut self) {
        self.children.push(ChartBodyRow::new())
    }

    fn calculate_columns(&self, style: &ChartStyle) -> Vec<usize> {

        let mut columns = Vec::new();

        let mut row_span = 0;

        for row in &self.children {

            let mut j = 0;

            if let Some(cell) = &row.row_header {
                let width = cell.calculate_width();
                row_span = cell.get_span() - 1;
                columns.expand_to(j+1, || 0);
                columns[j] = columns[j].max(width);
                j += 1;
            } else if row_span > 0 {
                // decrement the row_span.
                row_span -= 1;
                // shift the widths we are calculating over because there was a row_span above.
                j += 1;
            }

            for cell in &row.cells {
                let width = cell.calculate_width();
                columns.expand_to(j+1, || 0);
                columns[j] = columns[j].max(width);

                j += 1;

            }

        }

        if let Some(row) = &self.col_headers {
            let mut j = 0;

            if row.row_header {
                // the row header for the column headers should be "blank" and should not have a col_span or row_span.
                // plus, since the width is probably less than the row columns here, it's not going to change that.
                j += 1;

            }

            for cell in &row.cells {
                let width = cell.calculate_width();
                let col_span = cell.get_span();
                if col_span > &1 {
                    columns.expand_to(j+col_span+1, || 0);
                        
                    let col_group = &mut columns[j..j+col_span];
                    let spacer_width = (col_group.len() - 1) * style.get_spacer_width();
                    let col_group_width = col_group.iter().sum::<usize>() + spacer_width; 
                    if col_group_width < width {
                        // we need to expand the columns to match. 
                        // The easiest way is to just divide up the extra between all of the columns.
                        let difference = width - col_group_width;
                        let share = difference / col_span;
                        let remainder = difference % col_span;
                        for k in 0..col_group.len() {
                            col_group[k] += share;
                        }
                        // put the rest on the last column
                        if let Some(last) = col_group.last_mut() {
                            *last += remainder;
                        }

                    } // else the header will be expanded correctly, I think.
                    
                } else {
                    columns.expand_to(j+1, || 0);
                        

                    columns[j] = columns[j].max(width);
                }

                j += col_span;


            }

        }


        columns
    }

}

impl fmt::Display for Chart {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let columns = self.calculate_columns(&self.style);
        let mut row_span = 0;

        for row in &self.col_headers {
            row.display(f,&self.style,&columns,&mut row_span)?
        }

        self.style.display_header_break(f,&columns)?;

        for row in &self.children {
            row.display(f,&self.style,&columns,&mut row_span)?
        }

        Ok(())
        
    }
}