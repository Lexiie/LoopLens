use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use looplens_core::{
    read_failure_bundle, LearnInput, LoopLensEngine, RecallInput, VerificationEvidence,
};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

#[derive(Debug, Parser)]
#[command(name = "looplens")]
#[command(about = "Repository-scoped repair experience memory for AI coding agents")]
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
    /// Retrieve similar verified repairs for a failure bundle or problem text.
    Recall {
        #[arg(long, conflicts_with = "failure_bundle")]
        problem: Option<String>,

        #[arg(long = "failure-bundle", conflicts_with = "problem")]
        failure_bundle: Option<PathBuf>,

        #[arg(long, default_value_t = 3)]
        top_k: usize,
    },
    /// Store a verified PASS repair experience.
    Learn {
        #[arg(long = "verified-pass", action = ArgAction::SetTrue, required = true)]
        verified_pass: bool,

        #[arg(long)]
        problem: String,

        #[arg(long = "testsprite-hypothesis")]
        testsprite_hypothesis: Option<String>,

        #[arg(long = "failed-attempt")]
        failed_attempts: Vec<String>,

        #[arg(long = "successful-decision")]
        successful_decision: String,

        #[arg(long = "patch")]
        patches: Vec<String>,

        #[arg(long)]
        lesson: String,

        #[arg(long = "testsprite-run-id")]
        testsprite_run_id: Option<String>,

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

        #[arg(long, default_value_t = 0.85)]
        confidence: f32,
    },
    /// Regenerate .looplens/LOOP.md from verified experiences.
    ExportLoop,
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
            problem,
            failure_bundle,
            top_k,
        } => {
            let query = match (problem, failure_bundle) {
                (Some(problem), None) => problem,
                (None, Some(path)) => read_failure_bundle(path)?,
                _ => anyhow::bail!("provide --problem or --failure-bundle"),
            };
            let result = engine.recall(RecallInput { query, top_k })?;
            print_recall(result);
        }
        Command::Learn {
            verified_pass,
            problem,
            testsprite_hypothesis,
            failed_attempts,
            successful_decision,
            patches,
            lesson,
            testsprite_run_id,
            test_id,
            target_url,
            dashboard_url,
            commit_sha,
            branch,
            agent,
            files_changed,
            confidence,
        } => {
            if !verified_pass {
                anyhow::bail!("learn requires --verified-pass after a successful TestSprite run");
            }
            let inferred_commit = commit_sha.or_else(|| git_value(&root, &["rev-parse", "HEAD"]));
            let inferred_branch =
                branch.or_else(|| git_value(&root, &["branch", "--show-current"]));
            let changed_files = if files_changed.is_empty() {
                patches.clone()
            } else {
                files_changed
            };
            let experience = engine.learn(LearnInput {
                problem,
                testsprite_hypothesis,
                failed_attempts,
                successful_decision,
                patches,
                lesson,
                evidence: VerificationEvidence {
                    testsprite_run_id,
                    test_id,
                    target_url,
                    dashboard_url,
                    commit_sha: inferred_commit,
                    branch: inferred_branch,
                    agent,
                    files_changed: changed_files,
                },
                confidence,
            })?;
            println!("Stored verified repair experience {}", experience.id);
        }
        Command::ExportLoop => {
            let markdown = engine.export_loop()?;
            println!("{}", markdown);
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
    println!("# LoopLens Repair Context");
    println!();
    println!("Query: {}", result.query.trim());
    println!();

    if result.matches.is_empty() {
        println!("No similar verified repairs found yet.");
        return;
    }

    println!("## Similar Repairs");
    for item in &result.matches {
        let experience = &item.experience;
        println!(
            "- {} score {:.3}: {}",
            experience.id, item.score, experience.problem
        );
        if !item.matched_terms.is_empty() {
            println!("  matched terms: {}", item.matched_terms.join(", "));
        }
        if !item.matched_hypothesis_terms.is_empty() {
            println!(
                "  hypothesis overlap: {}",
                item.matched_hypothesis_terms.join(", ")
            );
        }
        if !item.matched_patch_terms.is_empty() {
            println!(
                "  patch/file overlap: {}",
                item.matched_patch_terms.join(", ")
            );
        }
        println!(
            "  score breakdown: lexical {:.2}, patch {:.2}, hypothesis {:.2}, confidence {:.2}, recency {:.2}",
            item.score_breakdown.lexical,
            item.score_breakdown.patch,
            item.score_breakdown.hypothesis,
            item.score_breakdown.confidence,
            item.score_breakdown.recency
        );
        println!(
            "  previous decision: {}",
            experience.trajectory_summary.successful_decision
        );
        println!("  lesson learned: {}", experience.lesson);
    }

    println!();
    println!("## Candidate Strategies");
    for strategy in result.candidate_strategies {
        println!("- {}", strategy);
    }
}
