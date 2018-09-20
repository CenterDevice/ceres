use chrono::prelude::*;
use chrono_humanize::HumanTime;
use prettytable::cell::Cell;
use prettytable::row::Row;
use prettytable::{color, format, Attr, Table};
use serde_json;
use std::io::Write;

use modules::health::check::{HealthCheck, HealthCheckResult, HealthSample};
use output::*;

const GLOBAL_NAME: &str = "global";

pub trait OutputHealthCheck {
    fn output<T: Write>(&self, writer: &mut T, health_checks: &[HealthCheck]) -> Result<()>;
}

pub struct JsonOutputHealthCheck;

impl OutputHealthCheck for JsonOutputHealthCheck {
    fn output<T: Write>(&self, writer: &mut T, health_checks: &[HealthCheck]) -> Result<()> {
        serde_json::to_writer_pretty(writer, health_checks).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct PlainOutputHealthCheck;

impl OutputHealthCheck for PlainOutputHealthCheck {
    fn output<T: Write>(&self, writer: &mut T, health_checks: &[HealthCheck]) -> Result<()> {
        for hc in health_checks {
            match hc.result {
                HealthCheckResult::Ok(ref checks) => {
                    for resource_name in checks.keys() {
                        let resource = &checks[resource_name]; // Safe, because iter over keys
                        let line = format!(
                            "{} {} {} {} {}\n",
                            hc.name,
                            resource_name,
                            resource.time_stamp.map(|x| format!("{}", x)).unwrap_or("-".to_string()),
                            resource.stampling_time.map(|x| format!("{}", x)).unwrap_or("-".to_string()),
                            resource.healthy,
                        );
                        let _ = writer.write(line.as_bytes());
                    }
                }
                HealthCheckResult::Failed(ref reason) => {
                    let line = format!("{} Failed {}\n", hc.name, reason);
                    let _ = writer.write(line.as_bytes());
                }
            }
        }
        Ok(())
    }
}

pub struct TableOutputHealthCheck;

impl OutputHealthCheck for TableOutputHealthCheck {
    fn output<T: Write>(&self, writer: &mut T, health_checks: &[HealthCheck]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("Service"),
            Cell::new("Resource"),
            Cell::new("Health"),
            Cell::new("Since *"),
            Cell::new("Last update at *"),
        ]));

        for hc in health_checks {
            let mut previous_hc_name: Option<&str> = None;
            match hc.result {
                HealthCheckResult::Ok(ref checks) => {
                    if let Some(resource) = checks.get(GLOBAL_NAME) {
                        let row = make_row(&hc.name, &previous_hc_name, GLOBAL_NAME, resource);
                        table.add_row(row);
                        previous_hc_name = Some(&hc.name);
                    }
                    for resource_name in checks.keys().filter(|x| &x[..] != GLOBAL_NAME) {
                        let resource = &checks[resource_name]; // Safe, because iter over keys
                        let row = make_row(&hc.name, &previous_hc_name, resource_name, resource);
                        table.add_row(row);
                        previous_hc_name = Some(&hc.name);
                    }
                }
                HealthCheckResult::Failed(ref reason) => {
                    let failed = format!("Failed because {}", reason);
                    let row = Row::new(vec![Cell::new(&hc.name), Cell::new(&failed)]);
                    table.add_row(row);
                }
            }
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)?;
        writeln!(writer, "* Mind that results may come from different backend servers for each call and thus, time stamps may very.")
            .chain_err(|| ErrorKind::OutputFailed)
    }
}

fn make_row(hc_name: &str, previous_hc_name: &Option<&str>, resource_name: &str, resource: &HealthSample) -> Row {
    let service_cell = match previous_hc_name {
        Some(name) if name == &hc_name => Cell::new(""),
        Some(_) | None => {
            Cell::new(hc_name)
        }
    };

    let healthy_cell = if resource.healthy {
        Cell::new("up").with_style(Attr::ForegroundColor(color::GREEN))
    } else {
        Cell::new("down").with_style(Attr::ForegroundColor(color::RED))
    };

    let updated_at: Option<DateTime<Local>> = resource.time_stamp.map(|x| {
        let naive_datetime =
            NaiveDateTime::from_timestamp(x / 1000, 0);
        Local.from_utc_datetime(&naive_datetime)
    });

    let since_cell = if let Some(updated) = updated_at {
        let since_str = since(updated);
        Cell::new(&since_str)
    } else {
        Cell::new("-")
    };

    let updated_at_cell = if let Some(updated) = updated_at {
        let updated_at_str = format!("{}", updated);
        Cell::new(&updated_at_str)
    } else {
        Cell::new("-")
    };

    let row = Row::new(vec![
        service_cell,
        Cell::new(resource_name.as_ref()),
        healthy_cell,
        since_cell,
        updated_at_cell,
    ]);

    row
}

fn since(updated_at: DateTime<Local>) -> String {
    let dt = updated_at.signed_duration_since(Local::now());
    let ht = HumanTime::from(dt);

    format!("{}", ht)
}
