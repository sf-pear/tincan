use crate::branding;
use crate::cli::{self, Command, JournalArgs, RecordArgs, ReviewArgs, ReviewScope};
use crate::git;
use crate::model::{DecisionStatus, Kind, Record};
use crate::skill::{self, InstallOutcome};
use crate::store;
use crate::util::display_path;
use crate::workspace;
use chrono::{Datelike, Duration, Local, NaiveDate, SecondsFormat};
use std::collections::BTreeMap;
use std::io::Write;
use uuid::Uuid;

pub fn run(command: Result<Command, String>) -> Result<(), String> {
    let command = command?;
    let notify_about_skill_update =
        !matches!(command, Command::SkillInstall { .. } | Command::SkillStatus);
    let result = match command {
        Command::Help => {
            branding::print();
            print!("{}", cli::help());
            Ok(())
        }
        Command::Version => {
            println!("tincan {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Init { repo } => init(repo),
        Command::Review(args) => review(args),
        Command::ProjectsList => projects_list(),
        Command::ProjectsUnregister { target } => projects_unregister(target.as_deref()),
        Command::Record(args) => record(args),
        Command::Journal(args) => journal(args),
        Command::Plan { repo } => plan(repo),
        Command::Resume { repo } => resume(repo),
        Command::Search { repo, query } => search(repo, &query),
        Command::Show { repo, id } => show(repo, &id),
        Command::Lift { repo, id, from } => lift(repo, &id, &from),
        Command::Changes { repo } => changes(repo),
        Command::SkillInstall { path, force } => install_skill(path, force),
        Command::SkillStatus => skill_status(),
    };
    if result.is_ok() && notify_about_skill_update {
        skill::notify_if_update_available();
    }
    result
}

fn skill_status() -> Result<(), String> {
    for line in skill_status_lines(skill::installation_statuses()) {
        println!("{line}");
    }
    Ok(())
}

fn skill_status_lines(statuses: Vec<skill::InstallationStatus>) -> Vec<String> {
    if statuses.is_empty() {
        return vec!["No supported Agent Skills destinations detected.".into()];
    }
    let current = statuses
        .iter()
        .filter(|status| status.state == skill::InstallationState::Current)
        .collect::<Vec<_>>();
    let outdated = statuses
        .iter()
        .filter(|status| status.state == skill::InstallationState::UpdateAvailable)
        .collect::<Vec<_>>();
    if current.is_empty() && outdated.is_empty() {
        let mut lines = vec!["Agent Skill is not installed.".into()];
        append_skill_locations(&mut lines, "Available destinations:", statuses.iter());
        lines.push("Run `tincan skill install` to install it.".into());
        return lines;
    }
    if outdated.is_empty() {
        let mut lines = vec!["Agent Skill is up to date everywhere.".into()];
        append_skill_locations(&mut lines, "Installed in:", current.into_iter());
        return lines;
    }
    let mut lines = vec!["Agent Skill update available.".into()];
    append_skill_locations(&mut lines, "Needs update:", outdated.into_iter());
    if !current.is_empty() {
        append_skill_locations(&mut lines, "Already up to date:", current.into_iter());
    }
    lines.push("Run `tincan skill install` to update.".into());
    lines
}

fn append_skill_locations<'a>(
    lines: &mut Vec<String>,
    heading: &str,
    statuses: impl Iterator<Item = &'a skill::InstallationStatus>,
) {
    lines.push(heading.into());
    lines.extend(statuses.map(|status| {
        format!(
            "- {} ({})",
            status.root.name,
            skill::display_user_path(&status.root.path.join("tincan"))
        )
    }));
}

fn install_skill(path: Option<std::path::PathBuf>, force: bool) -> Result<(), String> {
    branding::print();
    let overwrite = overwrite_existing_skill(path.as_deref(), force);
    let roots = match path {
        Some(path) => vec![path],
        None => {
            let detected = skill::detect_roots();
            if detected.is_empty() {
                return Err(
                    "no supported Agent Skills destination was detected; pass `--path <skills-directory>`"
                        .to_string(),
                );
            }
            let pending = skill::pending_roots(&detected);
            if pending.is_empty() {
                println!("Tincan skill is already current everywhere.");
                return Ok(());
            }
            let Some(selected) = skill::choose_interactively(&pending)? else {
                println!("Skill installation cancelled.");
                return Ok(());
            };
            selected
        }
    };

    let outcomes = skill::install_many(&roots, overwrite)?;
    let mut installed = false;
    for outcome in outcomes {
        match outcome {
            InstallOutcome::Installed(path) => {
                installed = true;
                println!(
                    "Installed Tincan skill at {}",
                    skill::display_user_path(&path)
                );
            }
            InstallOutcome::AlreadyCurrent(path) => {
                println!(
                    "Tincan skill is already current at {}",
                    skill::display_user_path(&path)
                );
            }
        }
    }
    if installed {
        println!("Restart or reload the agent harness to discover it.");
    }
    Ok(())
}

fn overwrite_existing_skill(path: Option<&std::path::Path>, force: bool) -> bool {
    path.is_none() || force
}

fn init(path: std::path::PathBuf) -> Result<(), String> {
    let root = workspace::target(&path)?;
    let excluded = git::protect_workspace(&root)?;
    let tincan = store::initialize(&root)?;
    store::register_workspace(&root)?;
    branding::print();
    println!("Initialized Tincan at {}", display_path(&tincan));
    match excluded {
        Some(true) => println!("Kept .tincan private through Git's local exclude file."),
        Some(false) => println!(".tincan is already excluded from Git locally."),
        None => {
            println!("This workspace is outside Git; nested repositories cannot track .tincan.")
        }
    }
    Ok(())
}

fn review(args: ReviewArgs) -> Result<(), String> {
    if args.all_projects {
        if let Some(root) = workspace::find_optional(&args.repo)? {
            store::reconcile_workspace(&root)?;
        }
        return review_all_projects(&args);
    }
    let root = find_workspace(&args.repo)?;
    let documents = store::scan(&root)?;
    let dated = documents
        .iter()
        .map(|document| review_date(document).map(|date| (date, document)))
        .collect::<Result<Vec<_>, _>>()?;
    let content = match args.scope {
        ReviewScope::Overview => render_review_overview(&dated),
        scope => {
            let (range, label) = review_range(&scope)?;
            render_review_context(&dated, range, &label)
        }
    };
    write_review_output(&content, args.output.as_deref(), args.force)
}

fn projects_list() -> Result<(), String> {
    let (mut workspaces, problems) = store::registered_workspaces_with_problems()?;
    if workspaces.is_empty() && problems.is_empty() {
        println!("No registered Tincan projects.");
        return Ok(());
    }
    sort_registered_workspaces(&mut workspaces);
    for registered in workspaces {
        let status = if store::registered_workspace_is_available(&registered) {
            "available"
        } else {
            "unavailable"
        };
        println!("{}  {status}", registered.id);
        println!("  {}", display_path(&registered.path));
    }
    for problem in problems {
        println!("invalid  unavailable");
        println!("  {problem}");
    }
    Ok(())
}

fn projects_unregister(target: Option<&str>) -> Result<(), String> {
    let target = target
        .ok_or_else(|| "projects unregister requires a project path or workspace ID".to_string())?;
    let id = if Uuid::parse_str(target).is_ok() {
        target.to_string()
    } else {
        let root = workspace::find(std::path::Path::new(target))?;
        store::registered_workspaces_with_problems()?
            .0
            .into_iter()
            .find(|workspace| workspace.path == root)
            .map(|workspace| workspace.id)
            .ok_or_else(|| format!("no registered Tincan project at {}", root.display()))?
    };
    unregister_project(&id)
}

fn unregister_project(id: &str) -> Result<(), String> {
    let Some(path) = store::remove_registered_workspace(id)? else {
        return Err(format!("no registered Tincan project with id {id}"));
    };
    println!("Removed project registration: {}", display_path(&path));
    println!("Project files and journals were not changed.");
    Ok(())
}

fn sort_registered_workspaces(workspaces: &mut [store::RegisteredWorkspace]) {
    workspaces.sort_by(|left, right| {
        store::registered_workspace_is_available(left)
            .cmp(&store::registered_workspace_is_available(right))
            .then(left.path.cmp(&right.path))
    });
}

fn find_workspace(path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let root = workspace::find(path)?;
    store::reconcile_workspace(&root)?;
    Ok(root)
}

fn review_all_projects(args: &ReviewArgs) -> Result<(), String> {
    let (workspaces, registry_problems) = store::registered_workspaces_with_problems()?;
    if workspaces.is_empty() && registry_problems.is_empty() {
        return Err(
            "no registered Tincan projects; run `tincan init` in a project first".to_string(),
        );
    }
    let mut sections = Vec::new();
    let mut unavailable = registry_problems
        .into_iter()
        .map(|problem| format!("- Invalid registry entry: {problem}"))
        .collect::<Vec<_>>();
    let label = if args.scope == ReviewScope::Overview {
        "history overview".to_string()
    } else {
        review_range(&args.scope)?.1
    };
    for registered in workspaces {
        let name = registered
            .path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Unnamed project");
        if !store::registered_workspace_is_available(&registered) {
            unavailable.push(format!("- {name}: {}", display_path(&registered.path)));
            continue;
        }
        let documents = match store::scan(&registered.path) {
            Ok(documents) => documents,
            Err(error) => {
                unavailable.push(format!(
                    "- {name}: {} ({error})",
                    display_path(&registered.path)
                ));
                continue;
            }
        };
        let dated = match documents
            .iter()
            .map(|document| review_date(document).map(|date| (date, document)))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(dated) => dated,
            Err(error) => {
                unavailable.push(format!(
                    "- {name}: {} ({error})",
                    display_path(&registered.path)
                ));
                continue;
            }
        };
        let project_content = match args.scope {
            ReviewScope::Overview => render_review_overview(&dated),
            _ => {
                let (range, _) = review_range(&args.scope)?;
                render_review_context(&dated, range, &label)
            }
        };
        sections.push(render_project_review_section(
            name,
            &registered.path,
            &project_content,
        ));
    }
    let mut content = format!("# Tincan review: {label} — all projects\n");
    for section in sections {
        content.push_str(&section);
    }
    if !unavailable.is_empty() {
        content.push_str("\n## Unavailable projects\n\n");
        content.push_str(&unavailable.join("\n"));
        content.push_str(
            "\n\nMoved projects reconnect when Tincan runs from the new location. Use `tincan projects` to inspect registry problems.\n",
        );
    }
    write_review_output(&content, args.output.as_deref(), args.force)
}

fn render_project_review_section(name: &str, path: &std::path::Path, content: &str) -> String {
    let mut output = format!("\n## {name}\n\nPath: {}\n", display_path(path));
    for (index, line) in content.lines().enumerate() {
        if index == 0 && line.starts_with("# Tincan review:") {
            continue;
        }
        output.push('\n');
        if line.starts_with('#') {
            output.push('#');
        }
        output.push_str(line);
    }
    output.push('\n');
    output
}

#[derive(Default)]
struct ReviewCounts {
    journals: usize,
    decisions: usize,
    learnings: usize,
}

fn render_review_overview(dated: &[(NaiveDate, &store::Document)]) -> String {
    let Some(first) = dated.iter().map(|(date, _)| date).min() else {
        return "No review history recorded.\n".to_string();
    };
    let last = dated.iter().map(|(date, _)| date).max().unwrap_or(first);
    let mut years = BTreeMap::<i32, ReviewCounts>::new();
    for (date, document) in dated {
        let counts = years.entry(date.year()).or_default();
        match document.kind.as_str() {
            "journal" => counts.journals += 1,
            "decision" => counts.decisions += 1,
            "learning" => counts.learnings += 1,
            _ => {}
        }
    }
    let mut output = format!("Review history: {first} through {last}\n\n");
    output.push_str("Year   Journal days   Decisions   Learnings\n");
    for (year, counts) in years {
        output.push_str(&format!(
            "{year:<6} {:>12} {:>11} {:>11}\n",
            counts.journals, counts.decisions, counts.learnings
        ));
    }
    output.push_str(
        "\nUse:\n  tincan review --month 2025-06\n  tincan review --year 2025\n  tincan review --quarter 2025-Q3\n  tincan review --all-time\n",
    );
    output
}

fn review_range(scope: &ReviewScope) -> Result<(Option<(NaiveDate, NaiveDate)>, String), String> {
    match scope {
        ReviewScope::AllTime => Ok((None, "all history".to_string())),
        ReviewScope::Year(year) => Ok((
            Some((date(*year, 1, 1)?, date(*year, 12, 31)?)),
            year.to_string(),
        )),
        ReviewScope::Month(year, month) => Ok((
            Some((date(*year, *month, 1)?, last_day_of_month(*year, *month)?)),
            format!("{year}-{month:02}"),
        )),
        ReviewScope::Quarter(year, quarter) => {
            let start_month = (quarter - 1) * 3 + 1;
            let end_month = start_month + 2;
            Ok((
                Some((
                    date(*year, start_month, 1)?,
                    last_day_of_month(*year, end_month)?,
                )),
                format!("{year} Q{quarter}"),
            ))
        }
        ReviewScope::Half(year, half) => {
            let start_month = if *half == 1 { 1 } else { 7 };
            let end_month = if *half == 1 { 6 } else { 12 };
            Ok((
                Some((
                    date(*year, start_month, 1)?,
                    last_day_of_month(*year, end_month)?,
                )),
                format!("{year} H{half}"),
            ))
        }
        ReviewScope::Overview => unreachable!(),
    }
}

fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| format!("invalid calendar date: {year:04}-{month:02}-{day:02}"))
}

fn last_day_of_month(year: i32, month: u32) -> Result<NaiveDate, String> {
    if month == 12 {
        return date(year, 12, 31);
    }
    Ok(date(year, month + 1, 1)? - Duration::days(1))
}

fn review_date(document: &store::Document) -> Result<NaiveDate, String> {
    let value = if document.kind == "journal" {
        document.heading.as_str()
    } else {
        document.created_at.split('T').next().unwrap_or_default()
    };
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        format!(
            "invalid review date in {}: {value}",
            document.path.display(),
        )
    })
}

fn render_review_context(
    dated: &[(NaiveDate, &store::Document)],
    range: Option<(NaiveDate, NaiveDate)>,
    label: &str,
) -> String {
    let mut selected = dated
        .iter()
        .filter(|(date, _)| range.is_none_or(|(start, end)| *date >= start && *date <= end))
        .copied()
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(review_kind_order(left.1).cmp(&review_kind_order(right.1)))
            .then(left.1.path.cmp(&right.1.path))
    });
    let mut output = format!("# Tincan review: {label}\n");
    if selected.is_empty() {
        output.push_str("\nNo Tincan records were created in this period.\n");
        return output;
    }
    let mut current_date = None;
    for (record_date, document) in selected {
        if current_date != Some(record_date) {
            output.push_str(&format!("\n## {record_date}\n"));
            current_date = Some(record_date);
        }
        let kind = match document.kind.as_str() {
            "journal" => "Journal".to_string(),
            "decision" => document
                .status
                .as_deref()
                .map(|status| format!("Decision ({status}): {}", document.heading))
                .unwrap_or_else(|| format!("Decision: {}", document.heading)),
            "learning" => format!("Learning: {}", document.heading),
            other => other.to_string(),
        };
        output.push_str(&format!("\n### {kind}\n"));
        let body = review_body(&document.body);
        if !body.is_empty() {
            output.push('\n');
            output.push_str(&body);
            if !body.ends_with('\n') {
                output.push('\n');
            }
        }
    }
    output
}

fn review_kind_order(document: &store::Document) -> u8 {
    match document.kind.as_str() {
        "journal" => 0,
        "decision" => 1,
        "learning" => 2,
        _ => 3,
    }
}

fn review_body(body: &str) -> String {
    let mut blocks = Vec::<Vec<&str>>::new();
    let mut current = Vec::new();
    for line in body
        .lines()
        .skip_while(|line| !line.starts_with("# "))
        .skip(1)
    {
        if line.starts_with("## ") && !current.is_empty() {
            blocks.push(current);
            current = Vec::new();
        }
        current.push(line);
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    let mut output = String::new();
    for block in blocks {
        if !block.iter().any(|line| {
            !line.trim().is_empty() && line.trim() != "<!-- none -->" && !line.starts_with("## ")
        }) {
            continue;
        }
        for line in block {
            if line.trim() == "<!-- none -->" {
                continue;
            }
            if line.starts_with('#') {
                output.push_str("##");
            }
            output.push_str(line);
            output.push('\n');
        }
    }
    output.trim().to_string()
}

fn write_review_output(
    content: &str,
    output: Option<&std::path::Path>,
    force: bool,
) -> Result<(), String> {
    let Some(path) = output else {
        print!("{content}");
        return Ok(());
    };
    let mut options = std::fs::OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            format!(
                "review output already exists: {}; use --force to replace it",
                path.display()
            )
        } else {
            format!("cannot write review output {}: {error}", path.display())
        }
    })?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("cannot write review output {}: {error}", path.display()))?;
    println!("Wrote review: {}", display_path(path));
    Ok(())
}

fn record(args: RecordArgs) -> Result<(), String> {
    if args.kind != "decision" && !args.supersedes.is_empty() {
        return Err("--supersedes can only be used with a decision".to_string());
    }
    let root = find_workspace(&args.repo)?;
    let kind = Kind::parse(&args.kind)?;
    let id = Uuid::now_v7().to_string();
    let created_at = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let status = match kind {
        Kind::Decision => Some(DecisionStatus::Active),
        Kind::Learning => None,
        Kind::Journal => unreachable!("journal entries use the journal command"),
    };
    let superseded = store::active_decisions(&root, &args.supersedes)?;
    let record = Record {
        id,
        kind,
        created_at,
        statement: args.statement,
        status,
        files: args.files,
        topics: args.topics,
        evidence: args.evidence,
        related: args.related,
        supersedes: args.supersedes,
        branch: git::branch(&args.repo)?,
    };
    let path = store::write(&root, kind, &record.id, &record.render())?;
    if let Err(error) = store::mark_superseded(&superseded, &record.id) {
        return match std::fs::remove_file(&path) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(format!(
                "{error}; also could not remove incomplete replacement {}: {cleanup_error}",
                path.display()
            )),
        };
    }
    println!("Created {}: {}", kind.as_str(), display_path(&path));
    println!("Record ID: {}", record.id);
    println!("Add detailed context directly to the Markdown body when useful.");
    if !superseded.is_empty() {
        println!("Superseded {} earlier decision(s).", superseded.len());
    }
    Ok(())
}

fn journal(args: JournalArgs) -> Result<(), String> {
    let root = find_workspace(&args.repo)?;
    store::require(&root)?;
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let created_at = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    let sections = store::JournalSections {
        done: &args.done,
        planned: &args.planned,
        questions: &args.questions,
        next: &args.next,
    };
    let update = store::update_journal(&root, &date, &created_at, sections)?;
    println!("Updated journal: {}", display_path(&update.path));
    if update.added == 0 {
        println!("No new bullets; exact duplicates were already present.");
    } else {
        println!("Added {} bullet(s).", update.added);
    }
    Ok(())
}

fn resume(path: std::path::PathBuf) -> Result<(), String> {
    let root = find_workspace(&path)?;
    print_plan(&root)?;
    let Some((journal_path, content)) = store::latest_journal(&root)? else {
        println!();
        println!("No journal entries yet.");
        println!("Use `tincan journal --done <text>` as meaningful work develops.");
        return Ok(());
    };
    println!("\nLatest journal: {}\n", display_path(&journal_path));
    print!("{content}");
    Ok(())
}

fn plan(path: std::path::PathBuf) -> Result<(), String> {
    let root = find_workspace(&path)?;
    print_plan(&root)
}

fn print_plan(root: &std::path::Path) -> Result<(), String> {
    let (plan_path, content) = store::read_plan(root)?;
    println!("Plan: {}\n", display_path(&plan_path));
    print!("{content}");
    Ok(())
}

fn search(path: std::path::PathBuf, query: &str) -> Result<(), String> {
    let query = query.to_lowercase();
    let mut matches: Vec<_> = lookup_documents(&path)?
        .into_iter()
        .filter_map(|document| search_rank(&document, &query).map(|rank| (rank, document)))
        .collect();
    matches.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.path.cmp(&right.1.path)));
    if matches.is_empty() {
        println!("No Tincan records matched.");
        return Ok(());
    }
    for (_, document) in matches {
        print_document_summary(&document, Some(&query));
    }
    Ok(())
}

fn show(path: std::path::PathBuf, id: &str) -> Result<(), String> {
    let document = lookup_documents(&path)?
        .into_iter()
        .find(|document| document.id == id)
        .ok_or_else(|| format!("no Tincan record found with id {id}"))?;
    print!("{}", store::read_document(&document)?);
    Ok(())
}

fn lift(path: std::path::PathBuf, id: &str, from: &std::path::Path) -> Result<(), String> {
    let root = find_workspace(&path)?;
    let source = store::scan(&root)?
        .into_iter()
        .find(|document| document.id == id)
        .ok_or_else(|| format!("no project record found with id {id}"))?;
    if source.kind != "learning" {
        return Err(format!("{id} is not a project learning"));
    }
    if let Some(existing) = store::scan_global()?
        .into_iter()
        .find(|document| document.source_record.as_deref() == Some(id))
    {
        println!(
            "Learning is already global: {}",
            display_path(&existing.path)
        );
        println!("Record ID: {}", existing.id);
        return Ok(());
    }
    let global_id = Uuid::now_v7().to_string();
    let created_at = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let body = read_lift_body(from)?;
    store::validate_global_learning_body(&body)?;
    let source_workspace = root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("workspace");
    let global_path =
        store::write_global_learning(&source, &global_id, &created_at, source_workspace, &body)?;
    println!("Created global learning: {}", display_path(&global_path));
    println!("Record ID: {global_id}");
    println!("Source learning: {id}");
    Ok(())
}

fn read_lift_body(path: &std::path::Path) -> Result<String, String> {
    if path == std::path::Path::new("-") {
        let mut content = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut content)
            .map_err(|error| format!("cannot read global learning Markdown from stdin: {error}"))?;
        return Ok(content);
    }
    std::fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read global learning Markdown {}: {error}",
            path.display()
        )
    })
}

fn lookup_documents(path: &std::path::Path) -> Result<Vec<store::Document>, String> {
    let mut documents = match workspace::find_optional(path)? {
        Some(root) => {
            store::reconcile_workspace(&root)?;
            store::scan(&root)?
        }
        None => Vec::new(),
    };
    documents.extend(store::scan_global()?);
    Ok(documents)
}

fn search_rank(document: &store::Document, query: &str) -> Option<u8> {
    if document.id.to_lowercase() == query {
        return Some(0);
    }
    if document.heading.to_lowercase().contains(query) {
        return Some(1);
    }
    if metadata_text(document).to_lowercase().contains(query) {
        return Some(2);
    }
    document.body.to_lowercase().contains(query).then_some(3)
}

fn metadata_text(document: &store::Document) -> String {
    [
        vec![
            document.id.clone(),
            document.kind.clone(),
            document.status.clone().unwrap_or_default(),
        ],
        document.files.clone(),
        document.topics.clone(),
        document.related.clone(),
        document.supersedes.clone(),
        document.superseded_by.clone(),
        document.source_record.clone().into_iter().collect(),
    ]
    .concat()
    .join("\n")
}

fn print_document_summary(document: &store::Document, query: Option<&str>) {
    let kind = document
        .status
        .as_deref()
        .map(|status| format!("{} / {status}", document.kind))
        .unwrap_or_else(|| document.kind.clone());
    let label = format!("{} {kind}", document.scope.as_str());
    println!("{} [{label}]", document.heading);
    println!("  id: {}", document.id);
    if let Some(excerpt) = query.and_then(|query| matching_excerpt(document, query)) {
        println!("  matched: {excerpt}");
    }
    if !document.files.is_empty() {
        println!("  files: {}", document.files.join(", "));
    }
    if !document.topics.is_empty() {
        println!("  topics: {}", document.topics.join(", "));
    }
    if !document.supersedes.is_empty() {
        println!("  supersedes: {}", document.supersedes.join(", "));
    }
    if !document.superseded_by.is_empty() {
        println!("  superseded by: {}", document.superseded_by.join(", "));
    }
    println!("  {}", display_path(&document.path));
}

fn matching_excerpt(document: &store::Document, query: &str) -> Option<String> {
    let query = query.to_lowercase();
    document.body.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty()
            || line.strip_prefix("# ") == Some(document.heading.as_str())
            || !line.to_lowercase().contains(&query)
        {
            return None;
        }
        let cleaned = line
            .trim_start_matches('#')
            .trim_start_matches(['-', '*'])
            .trim();
        let mut excerpt: String = cleaned.chars().take(120).collect();
        if cleaned.chars().count() > 120 {
            excerpt.push('…');
        }
        Some(excerpt)
    })
}

fn changes(path: std::path::PathBuf) -> Result<(), String> {
    let root = find_workspace(&path)?;
    let Some(changed) = git::workspace_changed_files(&root)? else {
        println!("No Git repositories found in this Tincan workspace.");
        return Ok(());
    };
    if changed.is_empty() {
        println!("No changed files.");
        return Ok(());
    }

    let documents = store::scan(&root)?;
    for file in &changed {
        let related: Vec<_> = documents
            .iter()
            .filter(|document| {
                document
                    .files
                    .iter()
                    .any(|affected| paths_overlap(file, affected))
            })
            .collect();
        if related.is_empty() {
            println!(
                "{}  {}",
                branding::section(file),
                branding::path("no records")
            );
            continue;
        }
        for (index, document) in related.into_iter().enumerate() {
            let relative = document.path.strip_prefix(&root).unwrap_or(&document.path);
            let file_label = if index == 0 { file.as_str() } else { "" };
            println!(
                "{file_label}  {}: {}  {}",
                document.kind,
                branding::heading(&document.heading),
                branding::path(&display_path(relative))
            );
        }
    }
    Ok(())
}

fn paths_overlap(changed: &str, affected: &str) -> bool {
    let changed = changed.trim_matches('/').replace('\\', "/");
    let affected = affected.trim_matches('/').replace('\\', "/");
    changed == affected
        || changed.starts_with(&(affected.clone() + "/"))
        || affected.starts_with(&(changed + "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::process::Command as ProcessCommand;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn matches_files_and_directories() {
        assert!(paths_overlap("src/feature/a.rs", "src/feature"));
        assert!(paths_overlap("src/feature", "src/feature/a.rs"));
        assert!(!paths_overlap("src/a.rs", "src/b.rs"));
    }

    fn review_document(kind: &str, created_at: &str, heading: &str, body: &str) -> store::Document {
        store::Document {
            path: std::path::PathBuf::from(format!("{kind}/{heading}.md")),
            id: format!("{kind}-{heading}"),
            created_at: created_at.to_string(),
            heading: heading.to_string(),
            body: body.to_string(),
            kind: kind.to_string(),
            status: (kind == "decision").then(|| "active".to_string()),
            files: Vec::new(),
            topics: Vec::new(),
            related: Vec::new(),
            supersedes: Vec::new(),
            superseded_by: Vec::new(),
            source_record: None,
            scope: store::Scope::Project,
        }
    }

    #[test]
    fn review_overview_groups_record_counts_by_year() {
        let documents = [
            review_document(
                "journal",
                "2025-02-03T09:00:00Z",
                "2025-02-03",
                "# 2025-02-03\n",
            ),
            review_document(
                "decision",
                "2025-07-01T09:00:00Z",
                "Choose Markdown",
                "# Choose Markdown\n",
            ),
            review_document(
                "learning",
                "2026-01-10T09:00:00Z",
                "Keep evidence",
                "# Keep evidence\n",
            ),
        ];
        let dated = documents
            .iter()
            .map(|document| (review_date(document).unwrap(), document))
            .collect::<Vec<_>>();

        let output = render_review_overview(&dated);

        assert!(output.contains("Review history: 2025-02-03 through 2026-01-10"));
        assert!(output.contains("2025              1           1           0"));
        assert!(output.contains("2026              0           0           1"));
        assert!(output.contains("tincan review --month 2025-06"));
        assert!(output.contains("tincan review --all-time"));
    }

    #[test]
    fn review_context_filters_calendar_periods_and_omits_empty_sections() {
        let journal = review_document(
            "journal",
            "2025-08-05T09:00:00+02:00",
            "2025-08-05",
            "# 2025-08-05\n\n## Done\n\n- Shipped the handoff\n\n## Next\n\n<!-- none -->\n",
        );
        let later = review_document(
            "learning",
            "2025-10-01T09:00:00+02:00",
            "Outside Q3",
            "# Outside Q3\n",
        );
        let documents = [journal, later];
        let dated = documents
            .iter()
            .map(|document| (review_date(document).unwrap(), document))
            .collect::<Vec<_>>();
        let (range, label) = review_range(&ReviewScope::Quarter(2025, 3)).unwrap();

        let output = render_review_context(&dated, range, &label);

        assert!(output.contains("# Tincan review: 2025 Q3"));
        assert!(output.contains("#### Done"));
        assert!(output.contains("Shipped the handoff"));
        assert!(!output.contains("## Next"));
        assert!(!output.contains("Outside Q3"));
    }

    #[test]
    fn confirmed_interactive_skill_install_updates_existing_destinations() {
        assert!(overwrite_existing_skill(None, false));
        assert!(!overwrite_existing_skill(
            Some(std::path::Path::new("custom-skills")),
            false
        ));
        assert!(overwrite_existing_skill(
            Some(std::path::Path::new("custom-skills")),
            true
        ));
    }

    #[test]
    fn skill_status_groups_outdated_and_current_installations() {
        let statuses = vec![
            skill::InstallationStatus {
                root: skill::SkillRoot {
                    name: "Current".into(),
                    path: std::path::PathBuf::from("current"),
                },
                state: skill::InstallationState::Current,
            },
            skill::InstallationStatus {
                root: skill::SkillRoot {
                    name: "Outdated".into(),
                    path: std::path::PathBuf::from("outdated"),
                },
                state: skill::InstallationState::UpdateAvailable,
            },
            skill::InstallationStatus {
                root: skill::SkillRoot {
                    name: "Missing".into(),
                    path: std::path::PathBuf::from("missing"),
                },
                state: skill::InstallationState::NotInstalled,
            },
        ];

        let lines = skill_status_lines(statuses);
        assert_eq!(lines[0], "Agent Skill update available.");
        assert_eq!(lines[1], "Needs update:");
        assert!(lines[2].contains("Outdated"));
        assert_eq!(lines[3], "Already up to date:");
        assert!(lines[4].contains("Current"));
        assert!(lines.iter().all(|line| !line.contains("Missing")));
    }

    #[test]
    fn skill_status_leads_with_health_then_lists_installed_locations() {
        let statuses = vec![skill::InstallationStatus {
            root: skill::SkillRoot {
                name: "Current".into(),
                path: std::path::PathBuf::from("current"),
            },
            state: skill::InstallationState::Current,
        }];
        let lines = skill_status_lines(statuses);
        assert_eq!(lines[0], "Agent Skill is up to date everywhere.");
        assert_eq!(lines[1], "Installed in:");
        assert!(lines[2].contains("Current"));
    }

    #[test]
    fn creates_uuid_record_that_remains_searchable_after_body_edits() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-uuid-record-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        assert!(
            ProcessCommand::new("git")
                .args(["init", "--quiet"])
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        store::initialize(&repo).unwrap();

        record(RecordArgs {
            kind: "learning".to_string(),
            repo: repo.clone(),
            statement: "Paging did not reduce rendering work".to_string(),
            files: vec!["src/gallery.rs".to_string()],
            topics: vec!["performance".to_string()],
            evidence: vec!["Release trace".to_string()],
            related: Vec::new(),
            supersedes: Vec::new(),
        })
        .unwrap();

        let document = store::scan(&repo).unwrap().remove(0);
        assert!(Uuid::parse_str(&document.id).is_ok());
        assert_eq!(
            document.path.file_stem().and_then(|value| value.to_str()),
            Some(document.id.as_str())
        );
        writeln!(
            OpenOptions::new()
                .append(true)
                .open(&document.path)
                .unwrap(),
            "The renderer remained the measured bottleneck."
        )
        .unwrap();

        let edited = store::scan(&repo).unwrap().remove(0);
        assert_eq!(search_rank(&edited, "renderer"), Some(3));
        assert_eq!(
            matching_excerpt(&edited, "renderer").as_deref(),
            Some("The renderer remained the measured bottleneck.")
        );
        fs::remove_dir_all(repo).unwrap();
    }
}
