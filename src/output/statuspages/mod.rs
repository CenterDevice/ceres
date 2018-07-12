use chrono::prelude::*;
use chrono_humanize::HumanTime;
use prettytable::cell::Cell;
use prettytable::row::Row;
use prettytable::{color, format, Attr, Table};
use serde_json;
use std::io::Write;

use modules::statuspages::{Indicator, PageStatusResult};
use output::*;

pub trait OutputPageStatusResult {
    fn output<T: Write>(&self, writer: &mut T, results: &[PageStatusResult]) -> Result<()>;
}

pub struct JsonOutputPageStatusResult;

impl OutputPageStatusResult for JsonOutputPageStatusResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[PageStatusResult]) -> Result<()> {
        serde_json::to_writer_pretty(writer, result).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputPageStatusResult;

impl OutputPageStatusResult for PlainOutputPageStatusResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[PageStatusResult]) -> Result<()> {
        for r in result {
            let line = format!(
                "{} {} {} {} {} {}\n",
                r.name,
                r.page_status.status.indicator,
                r.page_status.status.description,
                r.page_status.page.updated_at,
                r.page_status.page.time_zone,
                r.page_status.page.url
            );
            let _ = writer.write(line.as_bytes());
        }
        Ok(())
    }
}

pub struct TableOutputPageStatusResult;

impl OutputPageStatusResult for TableOutputPageStatusResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[PageStatusResult]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("Name"),
            Cell::new("Indicator"),
            Cell::new("Description"),
            Cell::new("Since"),
            Cell::new("Last Update at"),
            Cell::new("URL"),
        ]));

        for r in result {
            let row = Row::new(vec![
                Cell::new(r.name.as_ref()),
                r.page_status.status.indicator.to_colored_cell(),
                Cell::new(r.page_status.status.description.as_ref()),
                Cell::new(since(r.page_status.page.updated_at).as_ref()),
                Cell::new(r.page_status.page.updated_at.to_string().as_ref()),
                Cell::new(r.page_status.page.url.as_ref()),
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

impl Indicator {
    fn to_colored_cell(&self) -> Cell {
        let c = Cell::new(self.to_string().as_ref());
        match self {
            Indicator::None => c.with_style(Attr::ForegroundColor(color::GREEN)),
            Indicator::Minor => c.with_style(Attr::ForegroundColor(color::YELLOW)),
            Indicator::Major => c.with_style(Attr::ForegroundColor(color::YELLOW)),
            Indicator::Critical => c.with_style(Attr::ForegroundColor(color::RED)),
        }
    }
}

fn since(updated_at: DateTime<FixedOffset>) -> String {
    let dt = updated_at.signed_duration_since(Local::now());
    let ht = HumanTime::from(dt);

    format!("{}", ht)
}
