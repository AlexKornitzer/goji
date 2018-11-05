//! Interfaces for accessing and managing issues

// Third party
use serde_json;
use url::form_urlencoded;

// Ours
use crate::{Board, Issue, Jira, Result, SearchOptions};
use std::collections::BTreeMap;

/// issue options
#[derive(Debug)]
pub struct Issues {
    jira: Jira,
}

#[derive(Serialize, Debug, Clone)]
pub struct Assignee {
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct IssueType {
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Priority {
    pub id: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Project {
    pub key: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Component {
    pub name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub issuetype: IssueType,
    pub project: Project,
    // XXX: Is this admin only, this is autoset by the 'logged on' user
    // pub reporter: Assignee,
    pub summary: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Assignee>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,

    #[serde(flatten)]
    pub custom: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct Comment {
    pub body: String,
    pub visibility: Option<Visibility>,
}

#[derive(Serialize, Debug)]
pub struct CreateIssue {
    pub fields: Fields,
}

#[derive(Debug, Deserialize)]
pub struct CreateResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct IssueResults {
    pub expand: String,
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    #[serde(rename = "startAt")]
    pub start_at: u64,
    pub total: u64,
    pub issues: Vec<Issue>,
}

impl Issues {
    pub fn new(jira: &Jira) -> Issues {
        Issues { jira: jira.clone() }
    }

    pub fn get<I>(&self, id: I) -> Result<Issue>
    where
        I: Into<String>,
    {
        self.jira.get("api", &format!("/issue/{}", id.into()))
    }
    pub fn create(&self, data: CreateIssue) -> Result<CreateResponse> {
        self.jira.post("api", "/issue", data)
    }

    /// returns a single page of issues results
    /// https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getIssuesForBoard
    pub fn list(&self, board: &Board, options: &SearchOptions) -> Result<IssueResults> {
        let mut path = vec![format!("/board/{}/issue", board.id)];
        let query_options = options.serialize().unwrap_or_default();
        let query = form_urlencoded::Serializer::new(query_options).finish();

        path.push(query);

        self.jira
            .get::<IssueResults>("agile", path.join("?").as_ref())
    }

    /// runs a type why may be used to iterate over consecutive pages of results
    /// https://docs.atlassian.com/jira-software/REST/latest/#agile/1.0/board-getIssuesForBoard
    pub fn iter<'a>(&self, board: &'a Board, options: &'a SearchOptions) -> Result<IssuesIter<'a>> {
        IssuesIter::new(board, options, &self.jira)
    }
}

/// provides an iterator over multiple pages of search results
#[derive(Debug)]
pub struct IssuesIter<'a> {
    jira: Jira,
    board: &'a Board,
    results: IssueResults,
    search_options: &'a SearchOptions,
}

impl<'a> IssuesIter<'a> {
    fn new(board: &'a Board, options: &'a SearchOptions, jira: &Jira) -> Result<Self> {
        let results = jira.issues().list(board, options)?;
        Ok(IssuesIter {
            board,
            jira: jira.clone(),
            results,
            search_options: options,
        })
    }

    fn more(&self) -> bool {
        (self.results.start_at + self.results.max_results) <= self.results.total
    }
}

impl<'a> Iterator for IssuesIter<'a> {
    type Item = Issue;
    fn next(&mut self) -> Option<Issue> {
        self.results.issues.pop().or_else(|| {
            if self.more() {
                match self.jira.issues().list(
                    self.board,
                    &self
                        .search_options
                        .as_builder()
                        .max_results(self.results.max_results)
                        .start_at(self.results.start_at + self.results.max_results)
                        .build(),
                ) {
                    Ok(new_results) => {
                        self.results = new_results;
                        self.results.issues.pop()
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
    }
    pub fn comment<I>(&self, id: I, comment: Comment) -> Result<CommentResponse>
    where
        I: Into<String>,
    {
        self.jira
            .post(&format!("/issue/{}/comment", id.into()), comment)
    }
    pub fn update<I>(&self, id: I, fields: Fields) -> Result<()>
    where
        I: Into<String>,
    {
        self.jira.put(
            &format!("/issue/{}", id.into()),
            json!({ "fields": fields }),
        )
    }
}
