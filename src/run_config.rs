//! This module contains the runtime configuration for ceres
//!

use std::path::Path;

/// `RunConfig` contains run time configuration parameters.
///
/// In contrast to `config::Config` it is meant to store parameters for individual executions of
/// Ceres.
pub struct RunConfig<'a> {
    pub color: bool,
    pub active_profile: String,
    pub active_config: &'a Path,
}
