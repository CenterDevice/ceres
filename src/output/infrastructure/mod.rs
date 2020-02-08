use prettytable::{cell::Cell, format, row::Row, Table};
use serde_json;
use std::{collections::HashMap, io::Write};

use modules::infrastructure::Resource;
use output::*;

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

        table.set_titles(Row::new(vec![Cell::new("Project"), Cell::new("Resource")]));

        for resource in result {
            let row = Row::new(vec![
                Cell::new(resource.project.as_ref()),
                Cell::new(resource.name.as_ref()),
            ]);
            table.add_row(row);
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn resources_by_project(resources: &[Resource]) -> HashMap<&str, Vec<&str>> {
    let mut map = HashMap::new();

    for resource in resources {
        let v = map.entry(resource.project.as_ref()).or_insert_with(Vec::new);
        v.push(resource.name.as_ref());
    }

    map
}
