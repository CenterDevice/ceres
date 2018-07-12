use chrono::prelude::*;
use chrono_humanize::HumanTime;
use prettytable::cell::Cell;
use prettytable::row::Row;
use prettytable::{color, format, Attr, Table};
use serde_json;
use std::io::Write;

use modules::health::check::{HealthCheck, HealthCheckResult};
use output::*;

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
                            resource.time_stamp,
                            resource.stampling_time,
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
                    for resource_name in checks.keys() {
                        let resource = &checks[resource_name]; // Safe, because iter over keys

                        let service_cell = match previous_hc_name {
                            Some(name) if name == &hc.name => Cell::new(""),
                            Some(_) | None => {
                                previous_hc_name = Some(hc.name.as_ref());
                                Cell::new(hc.name.as_ref())
                            }
                        };

                        let healthy_cell = if resource.healthy {
                            Cell::new("up").with_style(Attr::ForegroundColor(color::GREEN))
                        } else {
                            Cell::new("down").with_style(Attr::ForegroundColor(color::RED))
                        };

                        let naive_datetime =
                            NaiveDateTime::from_timestamp(resource.time_stamp / 1000, 0);
                        let updated_at: DateTime<Local> = Local.from_utc_datetime(&naive_datetime);
                        let since_cell = {
                            let since_str = since(updated_at);
                            Cell::new(&since_str)
                        };

                        let updated_at_cell = {
                            let updated_at = format!("{}", updated_at);
                            Cell::new(&updated_at)
                        };

                        let row = Row::new(vec![
                            service_cell,
                            Cell::new(resource_name.as_ref()),
                            healthy_cell,
                            since_cell,
                            updated_at_cell,
                        ]);
                        table.add_row(row);
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

fn since(updated_at: DateTime<Local>) -> String {
    let dt = updated_at.signed_duration_since(Local::now());
    let ht = HumanTime::from(dt);

    format!("{}", ht)
}
