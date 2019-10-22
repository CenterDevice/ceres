use handlebars::Handlebars;
use std::collections::HashMap;

use output::stories::*;

pub struct MarkDownOutputStory;

impl OutputStory for MarkDownOutputStory {
    fn output<T: Write>(
        &self,
        writer: &mut T,
        story: &Story,
        members: &[ProjectMember],
    ) -> Result<()> {
        let lookup = members_to_lookup(&members);

        use self::formatting::FromWithPersonLookup;
        let story = formatting::Story::from_with(story, &lookup);
        render(writer, &story)
    }
}

fn members_to_lookup(members: &[ProjectMember]) -> HashMap<u64, &str> {
    members
        .iter()
        .map(|pj| (pj.person.id, pj.person.name.as_ref()))
        .collect()
}

fn render<T: Write>(writer: &mut T, story: &formatting::Story) -> Result<()> {
    let reg = Handlebars::new();

    let template = include_str!("../../../includes/stories.export.markdown.hbs");

    let md = reg
        .render_template(template, story)
        .chain_err(|| ErrorKind::OutputFailed)?;
    writer
        .write(md.as_bytes())
        .chain_err(|| ErrorKind::OutputFailed)?;

    Ok(())
}

mod formatting {
    use std::collections::HashMap;

    use modules::stories::pivotal_api::{self, Label, PullRequest, StoryState, StoryType, Task};

    pub trait FromWithPersonLookup<'a, T> {
        fn from_with(_: &'a T, persons: &HashMap<u64, &'a str>) -> Self;
    }

    #[derive(Debug, Serialize)]
    pub struct Story<'a> {
        pub id: u64,
        pub project_id: Option<&'a u64>,
        pub name: Option<&'a String>,
        pub description: Option<&'a String>,
        pub url: Option<&'a String>,
        pub story_type: Option<&'a StoryType>,
        pub current_state: Option<&'a StoryState>,
        pub estimate: Option<&'a f32>,
        pub created_at: Option<&'a String>,
        pub updated_at: Option<&'a String>,
        pub accepted_at: Option<&'a String>,
        pub requested_by: &'a str,
        pub owners: Vec<&'a String>,
        pub labels: &'a [Label],
        pub tasks: &'a [Task],
        pub pull_requests: &'a [PullRequest],
        pub comments: Vec<Comment<'a>>,
        pub transitions: Vec<Transition<'a>>,
    }

    impl<'b, 'a: 'b> FromWithPersonLookup<'a, pivotal_api::Story> for Story<'b> {
        fn from_with(s: &'a pivotal_api::Story, persons: &HashMap<u64, &'a str>) -> Self {
            Story {
                id: s.id,
                project_id: s.project_id.as_ref(),
                name: s.name.as_ref(),
                description: s.description.as_ref(),
                url: s.url.as_ref(),
                story_type: s.story_type.as_ref(),
                current_state: s.current_state.as_ref(),
                estimate: s.estimate.as_ref(),
                created_at: s.created_at.as_ref(),
                updated_at: s.updated_at.as_ref(),
                accepted_at: s.accepted_at.as_ref(),
                requested_by: &s.requested_by.name,
                owners: s.owners.iter().map(|p| &p.name).collect(),
                labels: s.labels.as_ref(),
                tasks: s.tasks.as_ref(),
                pull_requests: s.pull_requests.as_ref(),
                comments: s
                    .comments
                    .iter()
                    .map(|c| Comment::from_with(c, persons))
                    .collect(),
                transitions: s
                    .transitions
                    .iter()
                    .map(|c| Transition::from_with(c, persons))
                    .collect(),
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct Comment<'a> {
        pub text: String,
        pub person: &'a str,
        pub commit_identifier: Option<&'a String>,
        pub commit_type: Option<&'a String>,
        pub created_at: &'a str,
        pub updated_at: &'a str,
    }

    impl<'b, 'a: 'b> FromWithPersonLookup<'a, pivotal_api::Comment> for Comment<'b> {
        fn from_with(c: &'a pivotal_api::Comment, persons: &HashMap<u64, &'a str>) -> Self {
            Comment {
                // TODO: This fall back message must not be true; it's just a heuristic right now
                text: c
                    .text
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "_Some Attachemnt_".to_string()),
                person: persons.get(&c.person_id).unwrap_or(&"<unknown>"),
                commit_identifier: c.commit_identifier.as_ref(),
                commit_type: c.commit_type.as_ref(),
                created_at: &c.created_at,
                updated_at: &c.updated_at,
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct Transition<'a> {
        pub state: &'a StoryState,
        pub occurred_at: &'a str,
        pub performed_by: &'a str,
    }

    impl<'b, 'a: 'b> FromWithPersonLookup<'a, pivotal_api::Transition> for Transition<'b> {
        fn from_with(t: &'a pivotal_api::Transition, persons: &HashMap<u64, &'a str>) -> Self {
            Transition {
                state: &t.state,
                occurred_at: &t.occurred_at,
                performed_by: persons.get(&t.performed_by_id).unwrap_or(&"<unknown>"),
            }
        }
    }
}
