// This mod's errors need an individual namespace because the sub_module macro imports the
// module::errors into this scope which leads to name / type conflicts.
mod errors {
    error_chain! {
        errors {
            FailedToParseCmd(arg: String) {
                description("Failed to parse command line arguments")
                display("Failed to parse command line argument '{}'", arg)
            }
            NoCenterDeviceInProfile {
                description("No CenterDevice configuration found in selected profile")
            }
            FailedToAccessCenterDeviceApi {
                description("Failed to access CenterDevice API")
            }
            FailedToSaveToken {
                description("Failed to save token to configuration file")
            }
            FailedToSaveConfig {
                description("Failed to save configuration file")
            }
        }
    }
}

sub_module!("centerdevice", "Access CenterDevice from the CLI", auth);

