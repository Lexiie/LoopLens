use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use looplens_core::{
    read_failure_bundle, CodeEvidence, LearnInput, LoopLensEngine, MemoryScope, RecallInput,
    TaskType, VerificationEvidence, VerificationResult, VerificationSource,
};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

#[derive(Debug, Parser)]
#[command(name = "looplens")]
#[command(about = "Persistent engineering memory for AI coding agents")]
struct Cli {
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create .looplens storage in the current repository.
    Init,
    /// Retrieve relevant engineering experience for a task context.
    Recall {
        #[arg(long, conflicts_with = "failure_bundle")]
        task: Option<String>,

        #[arg(long = "problem", conflicts_with = "failure_bundle", hide = true)]
        legacy_problem: Option<String>,

        #[arg(long = "failure-bundle", conflicts_with_all = ["task", "legacy_problem"])]
        failure_bundle: Option<PathBuf>,

        #[arg(long = "file")]
        files: Vec<String>,

        #[arg(long = "language")]
        languages: Vec<String>,

        #[arg(long = "framework")]
        frameworks: Vec<String>,

        #[arg(long, default_value_t = 3)]
        top_k: usize,
    },
    /// Store a verified engineering experience.
    Learn {
        #[arg(long = "verified", action = ArgAction::SetTrue)]
        verified: bool,

        #[arg(long = "verified-pass", action = ArgAction::SetTrue, hide = true)]
        legacy_verified_pass: bool,

        #[arg(long)]
        task: Option<String>,

        #[arg(long = "problem", hide = true)]
        legacy_problem: Option<String>,

        #[arg(long = "type", default_value = "other")]
        task_type: TaskType,

        #[arg(long)]
        hypothesis: Option<String>,

        #[arg(long = "failed-attempt")]
        failed_attempts: Vec<String>,

        #[arg(long = "successful-decision")]
        successful_decision: String,

        #[arg(long = "file")]
        files: Vec<String>,

        #[arg(long = "patch", hide = true)]
        legacy_patches: Vec<String>,

        #[arg(long)]
        lesson: String,

        #[arg(long = "verification-source", default_value = "custom")]
        verification_source: VerificationSource,

        #[arg(long = "verification-command")]
        verification_command: Option<String>,

        #[arg(long = "verification-reference")]
        verification_reference: Option<String>,

        #[arg(long = "run-id")]
        run_id: Option<String>,

        #[arg(long = "test-id")]
        test_id: Option<String>,

        #[arg(long = "target-url")]
        target_url: Option<String>,

        #[arg(long = "dashboard-url")]
        dashboard_url: Option<String>,

        #[arg(long = "commit-sha")]
        commit_sha: Option<String>,

        #[arg(long)]
        branch: Option<String>,

        #[arg(long)]
        agent: Option<String>,

        #[arg(long = "file-changed")]
        files_changed: Vec<String>,

        #[arg(long = "global-scope", action = ArgAction::SetTrue)]
        global_scope: bool,

        #[arg(long, default_value_t = 0.85)]
        confidence: f32,
    },
    /// Regenerate .looplens/LOOP.md from verified experiences.
    ExportLoop,
    /// Print project stack metadata exposed to agents.
    ProjectContext,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = cli.root;
    let engine = LoopLensEngine::new(root.clone());

    match cli.command {
        Command::Init => {
            let result = engine.init()?;
            println!("LoopLens initialized at {}", result.root.display());
            for path in result.created {
                println!("created {}", path.display());
            }
        }
        Command::Recall {
            task,
            legacy_problem,
            failure_bundle,
            files,
            languages,
            frameworks,
            top_k,
        } => {
            let query = match (task.or(legacy_problem), failure_bundle) {
                (Some(task), None) => task,
                (None, Some(path)) => read_failure_bundle(path)?,
                _ => anyhow::bail!("provide --task or --failure-bundle"),
            };
            let result = engine.recall(RecallInput {
                task: query,
                task_type: None,
                files,
                languages,
                frameworks,
                top_k,
            })?;
            print_recall(result);
        }
        Command::Learn {
            verified,
            legacy_verified_pass,
            task,
            legacy_problem,
            task_type,
            hypothesis,
            failed_attempts,
            successful_decision,
            files,
            legacy_patches,
            lesson,
            verification_source,
            verification_command,
            verification_reference,
            run_id,
            test_id,
            target_url,
            dashboard_url,
            commit_sha,
            branch,
            agent,
            files_changed,
            global_scope,
            confidence,
        } => {
            if !verified && !legacy_verified_pass {
                anyhow::bail!("learn requires --verified after successful verification");
            }
            let task = task
                .or(legacy_problem)
                .ok_or_else(|| anyhow::anyhow!("--task is required"))?;
            let inferred_commit = commit_sha.or_else(|| git_value(&root, &["rev-parse", "HEAD"]));
            let inferred_branch =
                branch.or_else(|| git_value(&root, &["branch", "--show-current"]));
            let files = if files.is_empty() {
                legacy_patches
            } else {
                files
            };
            let changed_files = if files_changed.is_empty() {
                files.clone()
            } else {
                files_changed
            };
            let experience = engine.learn(LearnInput {
                task,
                task_type,
                hypothesis,
                failed_attempts,
                successful_decision,
                files: files.clone(),
                lesson,
                verification: VerificationEvidence {
                    source: verification_source,
                    result: VerificationResult::Passed,
                    command: verification_command,
                    reference: verification_reference,
                    run_id,
                    test_id,
                    target_url,
                    dashboard_url,
                    files_changed: changed_files.clone(),
                },
                evidence: CodeEvidence {
                    commit_sha: inferred_commit,
                    branch: inferred_branch,
                    agent,
                    files_changed: changed_files,
                },
                scope: MemoryScope {
                    project: true,
                    stack: true,
                    global: global_scope,
                },
                confidence,
            })?;
            println!("Stored verified engineering experience {}", experience.id);
        }
        Command::ExportLoop => {
            let markdown = engine.export_loop()?;
            println!("{}", markdown);
        }
        Command::ProjectContext => {
            let context = engine.project_context()?;
            println!("Project: {}", context.name);
            println!("Languages: {}", display_list(&context.languages));
            println!("Frameworks: {}", display_list(&context.frameworks));
            if let Some(runtime) = context.runtime {
                println!("Runtime: {}", runtime);
            }
            if let Some(package_manager) = context.package_manager {
                println!("Package manager: {}", package_manager);
            }
            println!(
                "Test frameworks: {}",
                display_list(&context.test_frameworks)
            );
        }
    }

    Ok(())
}

fn git_value(root: &PathBuf, args: &[&str]) -> Option<String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn print_recall(result: looplens_core::RecallResult) {
    println!("# LoopLens Engineering Context");
    println!();
    println!("Query: {}", result.query.trim());
    println!();

    if result.matches.is_empty() {
        println!("No relevant engineering experience found yet.");
        return;
    }

    println!("## Relevant Experience");
    for item in &result.matches {
        let experience = &item.experience;
        println!(
            "- {} score {:.3}: {}",
            experience.id, item.score, experience.task.summary
        );
        if !item.matched_terms.is_empty() {
            println!("  matched terms: {}", item.matched_terms.join(", "));
        }
        if !item.matched_context_terms.is_empty() {
            println!("  stack overlap: {}", item.matched_context_terms.join(", "));
        }
        if !item.matched_file_terms.is_empty() {
            println!(
                "  file/path overlap: {}",
                item.matched_file_terms.join(", ")
            );
        }
        if !item.reason.is_empty() {
            println!("  why: {}", item.reason.join("; "));
        }
        println!(
            "  score breakdown: task {:.2}, stack {:.2}, file {:.2}, confidence {:.2}, recency {:.2}, scope {:.2}",
            item.score_breakdown.task_similarity,
            item.score_breakdown.stack_match,
            item.score_breakdown.file_match,
            item.score_breakdown.confidence,
            item.score_breakdown.recency,
            item.score_breakdown.scope
        );
        println!(
            "  previous decision: {}",
            experience.trajectory.successful_decision
        );
        println!("  lesson learned: {}", experience.lesson);
    }

    println!();
    println!("## Candidate Strategies");
    for strategy in result.candidate_strategies {
        println!("- {}", strategy);
    }

    if !result.avoid.is_empty() {
        println!();
        println!("## Avoid");
        for attempt in result.avoid {
            println!("- {}", attempt);
        }
    }

    if !result.recommended_checks.is_empty() {
        println!();
        println!("## Recommended Checks");
        for check in result.recommended_checks {
            println!("- {}", check);
        }
    }
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
