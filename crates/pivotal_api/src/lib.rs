#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

// header! { (XTrackerToken, "X-TrackerToken") => [String] }

use futures::{future::result, Future, Stream};
use reqwest::{
    async::Client as ReqwestClient,
    header::{CONNECTION, CONTENT_TYPE},
};
use serde::de::DeserializeOwned;

error_chain! {
    errors {
        FailedToQueryPivotalApi {
            description("Failed to query Pivotal Tracker API")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Story {
    pub id:            u64,
    pub project_id:    Option<u64>,
    pub name:          Option<String>,
    pub description:   Option<String>,
    pub url:           Option<String>,
    pub story_type:    Option<StoryType>,
    pub current_state: Option<StoryState>,
    pub estimate:      Option<f32>,
    pub created_at:    Option<String>,
    pub updated_at:    Option<String>,
    pub accepted_at:   Option<String>,
    pub requested_by:  Option<Person>,
    #[serde(default)]
    pub owners:        Vec<Person>,
    #[serde(default)]
    pub labels:        Vec<Label>,
    #[serde(default)]
    pub tasks:         Vec<Task>,
    #[serde(default)]
    pub pull_requests: Vec<PullRequest>,
    #[serde(default)]
    pub comments:      Vec<Comment>,
    #[serde(default)]
    pub transitions:   Vec<Transition>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum StoryType {
    #[serde(rename = "feature")]
    Feature,
    #[serde(rename = "bug")]
    Bug,
    #[serde(rename = "chore")]
    Chore,
    #[serde(rename = "release")]
    Release,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum StoryState {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "delivered")]
    Delivered,
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "planned")]
    Planned,
    #[serde(rename = "unstarted")]
    Unstarted,
    #[serde(rename = "unscheduled")]
    Unscheduled,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub id:       u64,
    pub name:     String,
    pub email:    String,
    pub initials: String,
    pub username: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub id:   u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id:          u64,
    pub description: String,
    pub complete:    bool,
    pub position:    u64,
    pub created_at:  String,
    pub updated_at:  String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PullRequest {
    pub id:         u64,
    pub owner:      String,
    pub repo:       String,
    pub number:     u64,
    pub host_url:   String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id:                u64,
    pub text:              Option<String>,
    pub person_id:         u64,
    pub commit_identifier: Option<String>,
    pub commit_type:       Option<String>,
    pub created_at:        String,
    pub updated_at:        String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub state:           StoryState,
    pub occurred_at:     String,
    pub performed_by_id: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectMember {
    pub person: Person,
}

pub fn get_story(
    client: &ReqwestClient,
    project_id: u64,
    story_id: u64,
    token: &str,
) -> impl Future<Item = Story, Error = Error> {
    let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}?fields=project_id,name,description,requested_by,url,story_type,estimate,current_state,created_at,updated_at,accepted_at,owners,labels,tasks,pull_requests,comments,transitions",
      project_id=project_id,
      story_id=story_id);
    get(&url, client, token)
}

pub fn get_project_members(
    client: &ReqwestClient,
    project_id: u64,
    token: &str,
) -> impl Future<Item = Vec<ProjectMember>, Error = Error> {
    let url = format!(
        "https://www.pivotaltracker.com/services/v5/projects/{project_id}/memberships?fields=person",
        project_id = project_id
    );
    get(&url, client, token)
}

fn get<T>(url: &str, client: &ReqwestClient, token: &str) -> impl Future<Item = T, Error = Error>
where
    T: DeserializeOwned,
{
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONNECTION, "close".parse().unwrap());
    headers.insert("X-TrackerToken", token.parse().unwrap());

    client
        .get(url)
        .headers(headers)
        .send()
        .and_then(|res| {
            trace!("Received response with status = {}.", res.status());
            let body = res.into_body();
            body.concat2()
        })
        .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryPivotalApi))
        .and_then(|body| {
            trace!("Parsing body.");
            let res =
                serde_json::from_slice::<T>(&body).chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(res)
        })
}

pub fn start_story(
    client: &ReqwestClient,
    project_id: u64,
    story_id: u64,
    token: &str,
) -> impl Future<Item = Story, Error = Error> {
    let url = format!(
        "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}",
        project_id = project_id,
        story_id = story_id
    );

    #[derive(Debug, Serialize)]
    struct StoryRequest {
        current_state: StoryState,
    }
    let data = serde_json::to_string(&StoryRequest {
        current_state: StoryState::Started,
    })
    .unwrap(); // This is safe

    trace!("StoryRequest: {:?}", data);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONNECTION, "close".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert("X-TrackerToken", token.parse().unwrap());

    client
        .put(&url)
        .headers(headers)
        .body(data)
        .send()
        .and_then(|res| {
            trace!("Received response with status = {}.", res.status());
            let body = res.into_body();
            body.concat2()
        })
        .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryPivotalApi))
        .and_then(|body| {
            let body = String::from_utf8_lossy(&body).to_string();
            trace!("Parsing body {:?}", &body);
            let task = serde_json::from_slice::<Story>(&body.as_bytes())
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(task)
        })
}

pub fn create_task(
    client: &ReqwestClient,
    project_id: u64,
    story_id: u64,
    token: &str,
    position: usize,
    description: &str,
) -> impl Future<Item = Story, Error = Error> {
    let url = format!(
        "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}/tasks",
        project_id = project_id,
        story_id = story_id
    );

    let data = json!({
       "description": format!("{}. {}", position, description),
       "position": position
    })
    .to_string();

    trace!("Task: {:?}", data);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONNECTION, "close".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert("X-TrackerToken", token.parse().unwrap());

    client
        .post(&url)
        .headers(headers)
        .body(data)
        .send()
        .and_then(|res| {
            trace!("Received response with status = {}.", res.status());
            let body = res.into_body();
            body.concat2()
        })
        .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryPivotalApi))
        .and_then(|body| {
            let body = String::from_utf8_lossy(&body).to_string();
            trace!("Parsing body {:?}", &body);
            let task = serde_json::from_slice::<Story>(&body.as_bytes())
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(task)
        })
}

pub fn set_description(
    client: &ReqwestClient,
    project_id: u64,
    story_id: u64,
    token: &str,
    description: &str,
) -> impl Future<Item = Story, Error = Error> {
    let url = format!(
        "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}",
        project_id = project_id,
        story_id = story_id
    );

    let data = json!({
       "description": description.to_string(),
    })
    .to_string();

    trace!("Description: {:?}", data);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONNECTION, "close".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert("X-TrackerToken", token.parse().unwrap());

    client
        .put(&url)
        .body(data)
        .send()
        .and_then(|res| {
            trace!("Received response with status = {}.", res.status());
            let body = res.into_body();
            body.concat2()
        })
        .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryPivotalApi))
        .and_then(|body| {
            let body = String::from_utf8_lossy(&body).to_string();
            trace!("Parsing body {:?}", &body);
            let task = serde_json::from_slice::<Story>(&body.as_bytes())
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(task)
        })
}
