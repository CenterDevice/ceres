use prettytable::Table;
use prettytable::cell::Cell;
use prettytable::format;
use prettytable::row::Row;
use serde_json;
use std::collections::HashMap;
use std::io::Write;

use modules::infrastructure::asp::Asp;
use modules::infrastructure::images::Resource;
use output::*;

pub trait OutputAspListResult {
    fn output<T: Write>(&self, writer: &mut T, results: &[Asp]) -> Result<()>;
}

pub struct JsonOutputAspListResult;

impl OutputAspListResult for JsonOutputAspListResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Asp]) -> Result<()> {
        let by_project = asps_by_project(result);
        serde_json::to_writer_pretty(writer, &by_project).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputAspListResult;

impl OutputAspListResult for PlainOutputAspListResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Asp]) -> Result<()> {
        for asp in result {
            let line = format!("{} {}\n", asp.project, asp.resource);
            let _ = writer.write(line.as_bytes());
        }
        Ok(())
    }
}

pub struct TableOutputAspListResult;

impl OutputAspListResult for TableOutputAspListResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Asp]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("Project"), Cell::new("Resource")
        ]));

        for asp in result {
            let row = Row::new(vec![
                Cell::new(asp.project.as_ref()), Cell::new(asp.resource.as_ref())
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn asps_by_project(asps: &[Asp]) -> HashMap<&str, Vec<&str>> {
    let mut map = HashMap::new();

    for asp in asps {
        let mut v = map.entry(asp.project.as_ref()).or_insert(Vec::new());
        v.push(asp.resource.as_ref());
    }

    map
}

pub trait OutputResourceListResult {
    fn output<T: Write>(&self, writer: &mut T, results: &[Resource]) -> Result<()>;
}

pub struct JsonOutputResourceListResult;

impl OutputResourceListResult for JsonOutputResourceListResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Resource]) -> Result<()> {
        let by_project = resources_by_project(result);
        serde_json::to_writer_pretty(writer, &by_project).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputResourceListResult;

impl OutputResourceListResult for PlainOutputResourceListResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Resource]) -> Result<()> {
        for resource in result {
            let line = format!("{} {}\n", resource.project, resource.name);
            let _ = writer.write(line.as_bytes());
        }
        Ok(())
    }
}

pub struct TableOutputResourceListResult;

impl OutputResourceListResult for TableOutputResourceListResult {
    fn output<T: Write>(&self, writer: &mut T, result: &[Resource]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("Project"), Cell::new("Resource")
        ]));

        for resource in result {
            let row = Row::new(vec![
                Cell::new(resource.project.as_ref()), Cell::new(resource.name.as_ref())
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn resources_by_project(resources: &[Resource]) -> HashMap<&str, Vec<&str>> {
    let mut map = HashMap::new();

    for resource in resources {
        let mut v = map.entry(resource.project.as_ref()).or_insert(Vec::new());
        v.push(resource.name.as_ref());
    }

    map
}

