use crate::util::yaml_string;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Decision,
    Learning,
    Journal,
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
            "journal" => Ok(Self::Journal),
            _ => Err(format!("unsupported record kind: {value}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Learning => "learning",
            Self::Journal => "journal",
        }
    }

    pub fn directory(self) -> &'static str {
        match self {
            Self::Decision => "decisions",
            Self::Learning => "learnings",
            Self::Journal => "journal",
        }
    }
}

pub struct Record {
    pub id: String,
    pub kind: Kind,
    pub created_at: String,
    pub statement: String,
    pub status: Option<DecisionStatus>,
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
        field(&mut output, "created_at", &self.created_at);
        field(&mut output, "branch", &self.branch);
        list(&mut output, "files", &self.files);
        list(&mut output, "topics", &self.topics);
        list(&mut output, "related", &self.related);
        list(&mut output, "supersedes", &self.supersedes);
        output.push_str("superseded_by:\n");
        output.push_str("---\n\n");
        output.push_str("# ");
        output.push_str(&self.statement);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_managed_frontmatter_and_an_editable_markdown_body() {
        let record = Record {
            id: "019c4ea8-7e42-7b31-a211-8df9357d747c".to_string(),
            kind: Kind::Learning,
            created_at: "2026-08-06T10:00:00Z".to_string(),
            statement: "Paging did not reduce rendering work".to_string(),
            status: None,
            files: vec!["src/gallery.rs".to_string()],
            topics: vec!["performance".to_string()],
            evidence: vec!["Release trace".to_string()],
            related: Vec::new(),
            supersedes: Vec::new(),
            branch: "main".to_string(),
        };

        let rendered = record.render();
        assert!(rendered.contains("id: \"019c4ea8-7e42-7b31-a211-8df9357d747c\""));
        assert!(rendered.contains("created_at: \"2026-08-06T10:00:00Z\""));
        assert!(rendered.contains("# Paging did not reduce rendering work"));
        assert!(rendered.contains("## Evidence\n\n- Release trace"));
        assert!(!rendered.contains("title:"));
    }
}
