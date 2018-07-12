// This mod's errors need an individual namespace because the sub_module macro imports the
// module::errors into this scope which leads to name / type conflicts.
mod errors {
    error_chain! {
        errors {
            FailedQueryHeatlhCheck(reason: String) {
                description("health check query failed")
                display("health check query failed because {}", reason)
            }
            FailedToParseCmd(arg: String) {
                description("Failed to parse command line arguments")
                display("Failed to parse command line argument '{}'", arg)
            }
            FailedToParseOutputType {
                description("Failed to parse output type")
            }
            FailedOutput {
                description("Failed to output")
            }
        }
    }
}

sub_module!("health", "CenterDevice Health Status", check);

