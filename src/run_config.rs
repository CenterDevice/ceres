//! This module contains the runtime configuration for ceres
//!

/// `RunConfig` contains run time configuration parameters.
///
/// In contrast to `config::Config` it is meant to store parameters for individual executions of Ceres.
pub struct RunConfig {
    pub active_profile: String,
}
