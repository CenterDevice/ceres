use prettytable::{
    cell::Cell,
    format,
    row::Row,
    Table,
};
use serde_json;
use std::io::Write;

use output::*;

pub trait OutputUploadId {
    fn output<T: Write>(&self, writer: &mut T, id: &str) -> Result<()>;
}

pub struct JsonOutputUploadId;

impl OutputUploadId for JsonOutputUploadId {
    fn output<T: Write>(&self, writer: &mut T, id: &str) -> Result<()> {
        #[derive(Serialize)]
        struct JsonID<'a> {
            id: &'a str
        }
        let id = JsonID { id: &id };

        serde_json::to_writer_pretty(writer, &id).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputUploadId;

impl OutputUploadId for PlainOutputUploadId {
    fn output<T: Write>(&self, writer: &mut T, id: &str) -> Result<()> {
        writer.write(id.to_string().as_bytes()).chain_err(|| ErrorKind::OutputFailed)?;

        Ok(())
    }
}

pub struct TableOutputUploadId;

impl OutputUploadId for TableOutputUploadId {
    fn output<T: Write>(&self, writer: &mut T, id: &str) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![Cell::new("Document-ID")]));
        table.add_row(Row::new(vec![Cell::new(id)]));

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}
