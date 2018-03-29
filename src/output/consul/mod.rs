use std::io::Write;

use output::*;
use service_world::consul::Catalog;

pub mod json_output;
pub mod table_output;

pub use self::json_output::JsonOutputCatalogResult;
pub use self::table_output::TableOutputCatalogResult;

pub trait OutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, results: &Catalog) -> Result<()>;
}

