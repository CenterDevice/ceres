use centerdevice::client::search::Document;
use prettytable::{
    cell::Cell,
    format::{self, Alignment},
    row::Row,
    Table,
};
use serde_json;
use std::{collections::HashMap, io::Write};

use output::*;

pub trait OutputSearchResult {
    fn output<T: Write>(&self, writer: &mut T, results: &[Document]) -> Result<()>;
}

pub struct JsonOutputSearchResult;

impl OutputSearchResult for JsonOutputSearchResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Document]) -> Result<()> {
        serde_json::to_writer_pretty(writer, result).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputSearchResult;

impl OutputSearchResult for PlainOutputSearchResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Document]) -> Result<()> {
        for d in result {
            let line = format!(
                "{} {} {} {} {} {} {}\n",
                d.id,
                d.filename,
                d.document_date,
                d.upload_date,
                d.version,
                d.version_date,
                d.owner,
            );
            let _ = writer.write(line.as_bytes());
        }
        Ok(())
    }
}

pub struct TableOutputSearchResult<'a> {
    pub user_map: Option<&'a HashMap<String, String>>,
}

impl<'a> OutputSearchResult for TableOutputSearchResult<'a> {
    fn output<T: Write>(&self, writer: &mut T, result: &[Document]) -> Result<()> {
        if result.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("#"),
            Cell::new("Document-ID"),
            Cell::new("Filename"),
            Cell::new("Document Date"),
            Cell::new("Upload Date"),
            Cell::new("Version"),
            Cell::new("Version Date"),
            Cell::new("Version Owner"),
        ]));

        let format_str = "%a, %d.%m.%Y %H:%M:%S";
        for (i, d) in result.iter().enumerate() {
            let document_date = d.document_date.format(format_str).to_string();
            let upload_date = d.upload_date.format(format_str).to_string();
            let version_date = d.version_date.format(format_str).to_string();

            let row = Row::new(vec![
                Cell::new_align(i.to_string().as_ref(), Alignment::RIGHT),
                Cell::new(d.id.as_ref()),
                Cell::new(d.filename.as_ref()),
                Cell::new(&document_date),
                Cell::new(&upload_date),
                Cell::new_align(d.version.to_string().as_ref(), Alignment::RIGHT),
                Cell::new(&version_date),
                Cell::new(self.map_user_id_to_name(&d.owner)),
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

impl<'a> TableOutputSearchResult<'a> {
    fn map_user_id_to_name<'b: 'a>(&self, id: &'b str) -> &'a str {
        super::map_user_id_to_name(self.user_map, id)
    }
}
