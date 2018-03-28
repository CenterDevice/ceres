use std::io::Write;

use output::*;
use service_world::consul::Catalog;

pub mod json_output;

pub use self::json_output::JsonOutputCatalogResult;

pub trait OutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, results: &Catalog) -> Result<()>;
}

