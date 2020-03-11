// This mod's errors need an individual namespace because the sub_module macro imports the
// module::errors into this scope which leads to name / type conflicts.
mod errors {
    error_chain! {
        errors {
            FailedToQueryPivotalApi {
                description("Failed to query Pivotal Tracker API")
            }
            FailedToParseCmd(arg: String) {
                description("Failed to parse command line arguments")
                display("Failed to parse command line argument '{}'", arg)
            }
            StoryHasTasksAlready {
                description("This story already has tasks added")
            }
            StoryAlreadyStarted {
                description("This story is already started")
            }
            StoryIsNotEstimated {
                description("This story is not yet estimated")
            }
            OutputFailed {
                description("Output failed")
            }
        }
    }
}

sub_module!("stories", "Manage stories", export, prepare, start);

