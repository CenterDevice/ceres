use centerdevice::client::users::User;
use prettytable::{
    Table,
    cell::Cell,
    row::Row,
    format::{self, Alignment},
};
use serde_json;
use std::io::Write;

use output::*;

pub trait OutputUsers {
    fn output<T: Write>(&self, writer: &mut T, results: &[User]) -> Result<()>;
}

pub struct JsonOutputUsers;

impl OutputUsers for JsonOutputUsers {
    fn output<T: Write>(&self, writer: &mut T, result: &[User]) -> Result<()> {
        serde_json::to_writer_pretty(writer, result).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputUsers;

impl OutputUsers for PlainOutputUsers {
    fn output<T: Write>(&self, writer: &mut T, result: &[User]) -> Result<()> {
        for u in result {
            let line = format!(
                "{} {} {} {} {:?} {:?} {}\n",
                u.id,
                u.first_name,
                u.last_name,
                u.email,
                u.status,
                u.role,
                u.technical_user.unwrap_or_else(|| false),
            );
            let _ = writer.write(line.as_bytes());
        }

        Ok(())
    }
}

pub struct TableOutputUsers;

impl OutputUsers for TableOutputUsers {
    fn output<T: Write>(&self, writer: &mut T, result: &[User]) -> Result<()> {
        if result.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(Row::new(vec![
            Cell::new("#"),
            Cell::new("User-ID"),
            Cell::new("Name"),
            Cell::new("E-Mail-Address"),
            Cell::new("Status"),
            Cell::new("Role"),
            Cell::new("Technical User"),
        ]));

        for (i, u) in result.iter().enumerate() {
            let row = Row::new(vec![
                Cell::new_align(i.to_string().as_ref(), Alignment::RIGHT),
                Cell::new(u.id.as_ref()),
                Cell::new(format!("{} {}", u.first_name, u.last_name).as_ref()),
                Cell::new(u.email.as_ref()),
                Cell::new(format!("{:?}", u.status).as_ref()),
                Cell::new(format!("{:?}", u.role).as_ref()),
                Cell::new(format!("{:?}", u.technical_user).as_ref()),
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}
