use crate::util::yaml_string;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Decision,
    Learning,
}

#[derive(Debug, Clone, Copy)]
pub enum DecisionStatus {
    Active,
    Superseded,
}

impl DecisionStatus {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "active" => Ok(Self::Active),
            "superseded" => Ok(Self::Superseded),
            _ => Err(format!(
                "invalid decision status: {value}; expected active or superseded"
            )),
        }
    }
}

impl DecisionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Superseded => "superseded",
        }
    }
}

impl Kind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "decision" => Ok(Self::Decision),
            "learning" => Ok(Self::Learning),
            _ => Err(format!("unsupported record kind: {value}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Learning => "learning",
        }
    }

    pub fn directory(self) -> &'static str {
        match self {
            Self::Decision => "decisions",
            Self::Learning => "learnings",
        }
    }
}

pub struct Record {
    pub id: String,
    pub kind: Kind,
    pub timestamp: u64,
    pub title: String,
    pub status: Option<DecisionStatus>,
    pub note: String,
    pub files: Vec<String>,
    pub topics: Vec<String>,
    pub evidence: Vec<String>,
    pub related: Vec<String>,
    pub supersedes: Vec<String>,
    pub branch: String,
}

impl Record {
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("---\n");
        field(&mut output, "id", &self.id);
        field(&mut output, "type", self.kind.as_str());
        if let Some(status) = self.status {
            field(&mut output, "status", status.as_str());
        }
        output.push_str(&format!("created_at_unix: {}\n", self.timestamp));
        field(&mut output, "title", &self.title);
        field(&mut output, "branch", &self.branch);
        list(&mut output, "files", &self.files);
        list(&mut output, "topics", &self.topics);
        list(&mut output, "related", &self.related);
        list(&mut output, "supersedes", &self.supersedes);
        output.push_str("superseded_by:\n");
        output.push_str("---\n\n");
        output.push_str("# ");
        output.push_str(&self.title);
        output.push_str("\n\n");
        output.push_str(self.note.trim());
        output.push_str("\n\n");
        if !self.evidence.is_empty() {
            output.push_str("## Evidence\n\n");
            bullets(&mut output, &self.evidence);
        }
        output
    }
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

fn bullets(output: &mut String, values: &[String]) {
    for value in values {
        output.push_str("- ");
        output.push_str(value);
        output.push('\n');
    }
}
