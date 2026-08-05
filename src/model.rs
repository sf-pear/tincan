use crate::git::Snapshot;
use crate::util::{display_path, yaml_string};

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Attempt,
    Decision,
    Learning,
    Handoff,
    Session,
    FieldNote,
}

impl Kind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "attempt" => Ok(Self::Attempt),
            "decision" => Ok(Self::Decision),
            "learning" => Ok(Self::Learning),
            _ => Err(format!("unsupported record kind: {value}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attempt => "attempt",
            Self::Decision => "decision",
            Self::Learning => "learning",
            Self::Handoff => "handoff",
            Self::Session => "session",
            Self::FieldNote => "field-note",
        }
    }

    pub fn directory(self) -> &'static str {
        match self {
            Self::Attempt => "attempts",
            Self::Decision => "decisions",
            Self::Learning => "learnings",
            Self::Handoff => "handoffs",
            Self::Session => "sessions",
            Self::FieldNote => "field-notes",
        }
    }
}

pub struct Record {
    pub id: String,
    pub kind: Kind,
    pub timestamp: u64,
    pub title: String,
    pub status: String,
    pub summary: String,
    pub result: Option<String>,
    pub affects: Vec<String>,
    pub topics: Vec<String>,
    pub evidence: Vec<String>,
    pub related: Vec<String>,
    pub branch: String,
}

impl Record {
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("---\n");
        field(&mut output, "id", &self.id);
        field(&mut output, "type", self.kind.as_str());
        field(&mut output, "status", &self.status);
        output.push_str(&format!("created_at_unix: {}\n", self.timestamp));
        field(&mut output, "title", &self.title);
        field(&mut output, "branch", &self.branch);
        list(&mut output, "affects", &self.affects);
        list(&mut output, "topics", &self.topics);
        list(&mut output, "related", &self.related);
        output.push_str("---\n\n");
        section(&mut output, "Summary", &self.summary);
        if let Some(result) = &self.result {
            section(&mut output, "Result", result);
        }
        if !self.evidence.is_empty() {
            output.push_str("## Evidence\n\n");
            bullets(&mut output, &self.evidence);
        }
        match self.kind {
            Kind::Attempt => output
                .push_str("## Conclusion\n\n<!-- What should a future agent do or avoid? -->\n"),
            Kind::Decision => {
                output.push_str("## Consequences\n\n<!-- Tradeoffs and follow-up work -->\n")
            }
            Kind::Learning => output.push_str(
                "## Applies To\n\n<!-- Where else should this learning influence work? -->\n",
            ),
            _ => {}
        }
        output
    }
}

pub fn render_field_note(
    id: &str,
    timestamp: u64,
    title: &str,
    source_id: &str,
    source_title: &str,
) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    field(&mut output, "id", id);
    field(&mut output, "type", "field-note");
    field(&mut output, "status", "draft");
    output.push_str(&format!("created_at_unix: {timestamp}\n"));
    field(&mut output, "title", title);
    field(&mut output, "source", source_id);
    output.push_str("affects:\ntopics:\n---\n\n");
    output.push_str("## What I expected\n\n<!-- State the original assumption. -->\n\n");
    output
        .push_str("## What happened\n\n<!-- Describe the observation, not the transcript. -->\n\n");
    output.push_str(
        "## Evidence\n\n<!-- Measurements, commits, tests, or reproduced behavior. -->\n\n",
    );
    output.push_str("## What I changed\n\n<!-- Explain the decision and its tradeoffs. -->\n\n");
    output.push_str("## What I would reuse\n\n<!-- General lesson for another project. -->\n\n");
    output.push_str("## Source record\n\n");
    output.push_str(&format!("- `{source_id}` — {source_title}\n"));
    output
}

pub fn render_session(id: &str, timestamp: u64, title: &str, snapshot: &Snapshot) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    field(&mut output, "id", id);
    field(&mut output, "type", "session");
    field(&mut output, "status", "draft");
    output.push_str(&format!("created_at_unix: {timestamp}\n"));
    field(&mut output, "title", title);
    field(&mut output, "repository", &display_path(&snapshot.root));
    field(&mut output, "branch", &snapshot.branch);
    list(&mut output, "affects", &snapshot.changed_files);
    output.push_str("---\n\n");
    output.push_str("## Outcomes\n\n<!-- What changed for the user or project? -->\n\n");
    output.push_str("## Working Tree\n\n```text\n");
    output.push_str(if snapshot.status.is_empty() {
        "clean"
    } else {
        &snapshot.status
    });
    output.push_str("\n```\n\n");
    output.push_str("## Diff Summary\n\n```text\n");
    output.push_str(if snapshot.diff_stat.is_empty() {
        "No unstaged or staged diff."
    } else {
        &snapshot.diff_stat
    });
    output.push_str("\n```\n\n");
    output.push_str("## Recent Commits\n\n```text\n");
    output.push_str(&snapshot.recent_commits);
    output.push_str("\n```\n\n");
    output.push_str("## Decisions and Attempts\n\n");
    output.push_str("<!-- Add accepted decisions and failed or superseded attempts. -->\n\n");
    output.push_str("## Verification\n\n<!-- Record only checks that actually ran. -->\n\n");
    output.push_str("## Learning Candidates\n\n");
    output.push_str("<!-- Capture only reusable findings supported by evidence. -->\n");
    output
}

pub fn render_handoff(
    id: &str,
    timestamp: u64,
    title: &str,
    snapshot: &Snapshot,
    next: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    field(&mut output, "id", id);
    field(&mut output, "type", "handoff");
    field(&mut output, "status", "active");
    output.push_str(&format!("created_at_unix: {timestamp}\n"));
    field(&mut output, "title", title);
    field(&mut output, "repository", &display_path(&snapshot.root));
    field(&mut output, "branch", &snapshot.branch);
    list(&mut output, "affects", &snapshot.changed_files);
    output.push_str("---\n\n");
    output.push_str("## Outcome\n\n<!-- What is now true? -->\n\n");
    output.push_str("## Completed\n\n<!-- Work already finished -->\n\n");
    output.push_str("## Next Actions\n\n");
    if next.is_empty() {
        output.push_str("- <!-- Exact next useful action -->\n");
    } else {
        bullets(&mut output, next);
    }
    output
        .push_str("\n## Decisions\n\n<!-- Accepted decisions; link records where possible -->\n\n");
    output.push_str("## Risks and Verification Gaps\n\n");
    output.push_str("<!-- What remains uncertain or untested? -->\n\n");
    output.push_str("## Working Tree\n\n```text\n");
    output.push_str(if snapshot.status.is_empty() {
        "clean"
    } else {
        &snapshot.status
    });
    output.push_str("\n```\n");
    output
}

fn field(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push_str(": ");
    output.push_str(&yaml_string(value));
    output.push('\n');
}

fn list(output: &mut String, key: &str, values: &[String]) {
    output.push_str(key);
    output.push_str(":\n");
    for value in values {
        output.push_str("  - ");
        output.push_str(&yaml_string(value));
        output.push('\n');
    }
}

fn section(output: &mut String, title: &str, body: &str) {
    output.push_str("## ");
    output.push_str(title);
    output.push_str("\n\n");
    output.push_str(body.trim());
    output.push_str("\n\n");
}

fn bullets(output: &mut String, values: &[String]) {
    for value in values {
        output.push_str("- ");
        output.push_str(value);
        output.push('\n');
    }
}
