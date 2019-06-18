use centerdevice::client::collections::Collection;
use prettytable::{
    Table,
    cell::Cell,
    row::Row,
    format::{self, Alignment},
};
use serde_json;
use std::io::Write;
use std::collections::HashMap;

use output::*;

pub trait OutputCollections {
    fn output<T: Write>(&self, writer: &mut T, results: &[Collection]) -> Result<()>;
}

pub struct JsonOutputCollections;

impl OutputCollections for JsonOutputCollections {
    fn output<T: Write>(&self, writer: &mut T, result: &[Collection]) -> Result<()> {
        serde_json::to_writer_pretty(writer, result).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputCollections;

impl OutputCollections for PlainOutputCollections {
    fn output<T: Write>(&self, writer: &mut T, result: &[Collection]) -> Result<()> {
        for c in result {
            let line = format!(
                "{} {} {} {} {} {}\n",
                c.id,
                c.name,
                c.owner,
                c.public,
                c.auditing,
                c.archived_date.map(|x| x.to_rfc3339()).unwrap_or_else(|| "-".to_string()),
            );
            let _ = writer.write(line.as_bytes());
        }

        Ok(())
    }
}

pub struct TableOutputCollections<'a> {
    pub user_map: Option<&'a HashMap<String, String>>
}

impl<'a> OutputCollections for TableOutputCollections<'a> {
    fn output<T: Write>(&self, writer: &mut T, result: &[Collection]) -> Result<()> {
        if result.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(Row::new(vec![
            Cell::new("#"),
            Cell::new("Collection-ID"),
            Cell::new("Name"),
            Cell::new("Owner"),
            Cell::new("Public"),
            Cell::new("Auditing"),
            Cell::new("Archived"),
            Cell::new("Archived Date"),
        ]));

        let format_str = "%a, %d.%m.%Y %H:%M:%S";
        for (i, c) in result.iter().enumerate() {
            let archived_date = c.archived_date.map(|x| x.format(format_str).to_string()).unwrap_or_else(|| "-".to_string());

            let row = Row::new(vec![
                Cell::new_align(i.to_string().as_ref(), Alignment::RIGHT),
                Cell::new(c.id.as_ref()),
                Cell::new(c.name.as_ref()),
                Cell::new(self.map_user_id_to_name(c.owner.as_ref())),
                Cell::new(format!("{:?}", c.public).as_ref()),
                Cell::new(format!("{:?}", c.auditing).as_ref()),
                Cell::new(format!("{:?}", c.archived_date.is_some()).as_ref()),
                Cell::new_align(archived_date.as_ref(), Alignment::CENTER),
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

impl<'a> TableOutputCollections<'a> {
    fn map_user_id_to_name<'b: 'a>(&self, id: &'b str) -> &'a str {
        super::map_user_id_to_name(self.user_map, id)
    }
}
