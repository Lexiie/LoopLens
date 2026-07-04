use anyhow::Result;
use clap::{Parser, Subcommand};
use looplens_core::{read_failure_bundle, LearnInput, LoopLensEngine, RecallInput};
use std::path::PathBuf;

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

        #[arg(long, default_value_t = 0.85)]
        confidence: f32,
    },
    /// Regenerate .looplens/LOOP.md from verified experiences.
    ExportLoop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let engine = LoopLensEngine::new(cli.root);

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
            problem,
            testsprite_hypothesis,
            failed_attempts,
            successful_decision,
            patches,
            lesson,
            confidence,
        } => {
            let experience = engine.learn(LearnInput {
                problem,
                testsprite_hypothesis,
                failed_attempts,
                successful_decision,
                patches,
                lesson,
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
