use chrono::prelude::*;
use std::fmt;

// This mod's errors need an individual namespace because the sub_module macro imports the
// module::errors into this scope which leads to name / type conflicts.
mod errors {
    error_chain! {
        errors {
            FailedToParseOutputType {
                description("Failed to parse output type")
                display("Failed to parse output type")
            }
            FailedToQueryStatusPage {
                description("Failed to query status page")
                display("Failed to query status page")
            }
            FailedOutput {
                description("Failed to output")
                display("Failed to output")
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageStatusResult {
    pub name: String,
    pub page_status: PageStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageStatus {
    pub page: Page,
    pub status: Status,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Page {
    pub id: String,
    pub name: String,
    pub url: String,
    pub time_zone: String,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub indicator: Indicator,
    pub description: String,
}

/// Impact
///
/// cf. http://doers.statuspage.io/api/v1/incidents/#create-realtime
#[derive(Debug, Deserialize, Serialize)]
pub enum Indicator {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "minor")]
    Minor,
    #[serde(rename = "major")]
    Major,
    #[serde(rename = "critical")]
    Critical,
}

impl fmt::Display for Indicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Indicator::None => "None",
            Indicator::Minor => "Minor",
            Indicator::Major => "Major",
            Indicator::Critical => "Critical",
        };
        write!(f, "{}", s)
    }
}

sub_module!("statuspages", "Status information from statuspage.io", show);

