use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const LOOPLENS_DIR: &str = ".looplens";
const PROJECT_FILE: &str = "project.toml";
const LEGACY_CONFIG_FILE: &str = "config.toml";
const EXPERIENCES_DIR: &str = "experiences";
const TRAJECTORIES_DIR: &str = "trajectories";
const LOOP_FILE: &str = "LOOP.md";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub package_manager: Option<String>,
    #[serde(default)]
    pub test_frameworks: Vec<String>,
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            name: "LoopLens project".to_string(),
            languages: vec!["rust".to_string()],
            frameworks: Vec::new(),
            runtime: Some("native".to_string()),
            package_manager: Some("cargo".to_string()),
            test_frameworks: vec!["cargo test".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub project: String,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub experiences_dir: String,
    pub trajectories_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "0.2".to_string(),
            project: "LoopLens engineering memory".to_string(),
            storage: StorageConfig {
                experiences_dir: EXPERIENCES_DIR.to_string(),
                trajectories_dir: TRAJECTORIES_DIR.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineeringExperience {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
    pub task: TaskRecord,
    pub context: ProjectContext,
    pub trajectory: TrajectorySummary,
    pub lesson: String,
    pub verification: VerificationEvidence,
    pub outcome: ExperienceOutcome,
    pub evidence: CodeEvidence,
    pub scope: MemoryScope,
    pub confidence: f32,
}

pub type RepairExperience = EngineeringExperience;

impl<'de> Deserialize<'de> for EngineeringExperience {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Fields {
            id: String,
            created_at: DateTime<Utc>,
            #[serde(default)]
            verified_at: Option<DateTime<Utc>>,
            #[serde(default)]
            task: Option<TaskRecord>,
            #[serde(default)]
            problem: Option<String>,
            #[serde(default)]
            task_type: Option<TaskType>,
            #[serde(default)]
            context: Option<ProjectContext>,
            #[serde(default)]
            hypothesis: Option<String>,
            #[serde(default)]
            trajectory: Option<TrajectorySummary>,
            #[serde(default)]
            trajectory_summary: Option<TrajectorySummary>,
            #[serde(default)]
            patches: Vec<String>,
            lesson: String,
            #[serde(default)]
            verification: Option<VerificationEvidence>,
            #[serde(default)]
            verified: Option<LegacyVerificationStatus>,
            #[serde(default)]
            outcome: Option<ExperienceOutcome>,
            #[serde(default)]
            evidence: Option<LegacyEvidence>,
            #[serde(default)]
            scope: MemoryScope,
            confidence: f32,
        }

        let fields = Fields::deserialize(deserializer)?;
        let loaded_task = fields.task.unwrap_or_default();
        let summary = if loaded_task.summary.trim().is_empty() {
            fields.problem.unwrap_or_default()
        } else {
            loaded_task.summary
        };
        let mut task = TaskRecord {
            summary,
            task_type: fields.task_type.unwrap_or(loaded_task.task_type),
            hypothesis: loaded_task.hypothesis.or(fields.hypothesis),
        };
        let mut verification = fields.verification.unwrap_or_default();
        if verification.source == VerificationSource::Unspecified {
            verification.source = match fields.verified {
                Some(LegacyVerificationStatus::Pass) => VerificationSource::Custom,
                None => VerificationSource::Unspecified,
            };
        }
        if verification.result == VerificationResult::Unknown {
            verification.result = match fields.verified {
                Some(LegacyVerificationStatus::Pass) => VerificationResult::Passed,
                None => VerificationResult::Unknown,
            };
        }
        let legacy_evidence = fields.evidence.unwrap_or_default();
        let mut code_evidence = CodeEvidence {
            commit_sha: legacy_evidence.commit_sha,
            branch: legacy_evidence.branch,
            agent: legacy_evidence.agent,
            files_changed: legacy_evidence.files_changed.clone(),
        };
        if code_evidence.files_changed.is_empty() {
            code_evidence.files_changed = fields.patches.clone();
        }
        verification.run_id = verification.run_id.or(legacy_evidence.run_id);
        verification.test_id = verification.test_id.or(legacy_evidence.test_id);
        verification.target_url = verification.target_url.or(legacy_evidence.target_url);
        verification.dashboard_url = verification.dashboard_url.or(legacy_evidence.dashboard_url);
        if verification.files_changed.is_empty() {
            verification.files_changed = code_evidence.files_changed.clone();
        }
        if task.hypothesis.is_none() {
            task.hypothesis = verification.reference.clone();
        }

        Ok(Self {
            id: fields.id,
            created_at: fields.created_at,
            verified_at: fields.verified_at.unwrap_or(fields.created_at),
            task,
            context: fields.context.unwrap_or_default(),
            trajectory: fields
                .trajectory
                .or(fields.trajectory_summary)
                .unwrap_or_default(),
            lesson: fields.lesson,
            verification,
            outcome: fields.outcome.unwrap_or(ExperienceOutcome::VerifiedSuccess),
            evidence: code_evidence,
            scope: fields.scope,
            confidence: fields.confidence,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskRecord {
    pub summary: String,
    #[serde(default, rename = "type")]
    pub task_type: TaskType,
    #[serde(default)]
    pub hypothesis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Bugfix,
    Feature,
    Refactor,
    Migration,
    Performance,
    Build,
    Deployment,
    Testing,
    Configuration,
    Dependency,
    Other,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::Other
    }
}

impl FromStr for TaskType {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "bugfix" | "bug" | "fix" => Ok(Self::Bugfix),
            "feature" | "feat" => Ok(Self::Feature),
            "refactor" => Ok(Self::Refactor),
            "migration" | "migrate" => Ok(Self::Migration),
            "performance" | "perf" => Ok(Self::Performance),
            "build" => Ok(Self::Build),
            "deployment" | "deploy" => Ok(Self::Deployment),
            "testing" | "test" => Ok(Self::Testing),
            "configuration" | "config" => Ok(Self::Configuration),
            "dependency" | "deps" => Ok(Self::Dependency),
            "other" => Ok(Self::Other),
            other => Err(format!("unknown task type `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrajectorySummary {
    #[serde(default)]
    pub failed_attempts: Vec<String>,
    #[serde(default)]
    pub successful_decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationSource {
    Test,
    Build,
    Lint,
    Ci,
    Human,
    Custom,
    #[serde(other)]
    Unspecified,
}

impl Default for VerificationSource {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl FromStr for VerificationSource {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "test" | "tests" => Ok(Self::Test),
            "build" => Ok(Self::Build),
            "lint" => Ok(Self::Lint),
            "ci" => Ok(Self::Ci),
            "human" | "approval" => Ok(Self::Human),
            "custom" => Ok(Self::Custom),
            "unspecified" | "unknown" => Ok(Self::Unspecified),
            other => Err(format!("unknown verification source `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationResult {
    Passed,
    Failed,
    Unknown,
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationEvidence {
    #[serde(default)]
    pub source: VerificationSource,
    #[serde(default)]
    pub result: VerificationResult,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub test_id: Option<String>,
    #[serde(default)]
    pub target_url: Option<String>,
    #[serde(default)]
    pub dashboard_url: Option<String>,
    #[serde(default)]
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeEvidence {
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LegacyEvidence {
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    test_id: Option<String>,
    #[serde(default)]
    target_url: Option<String>,
    #[serde(default)]
    dashboard_url: Option<String>,
    #[serde(default)]
    commit_sha: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    files_changed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExperienceOutcome {
    VerifiedSuccess,
    VerifiedFailure,
    Unverified,
}

impl Default for ExperienceOutcome {
    fn default() -> Self {
        Self::Unverified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct MemoryScope {
    #[serde(default = "default_true")]
    pub project: bool,
    #[serde(default = "default_true")]
    pub stack: bool,
    #[serde(default)]
    pub global: bool,
}

impl Default for MemoryScope {
    fn default() -> Self {
        Self {
            project: true,
            stack: true,
            global: false,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum LegacyVerificationStatus {
    Pass,
}

#[derive(Debug, Clone)]
pub struct LearnInput {
    pub task: String,
    pub task_type: TaskType,
    pub hypothesis: Option<String>,
    pub failed_attempts: Vec<String>,
    pub successful_decision: String,
    pub files: Vec<String>,
    pub lesson: String,
    pub verification: VerificationEvidence,
    pub evidence: CodeEvidence,
    pub scope: MemoryScope,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct RecallInput {
    pub task: String,
    pub task_type: Option<TaskType>,
    pub files: Vec<String>,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub top_k: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallMatch {
    pub experience: EngineeringExperience,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub matched_context_terms: Vec<String>,
    pub matched_file_terms: Vec<String>,
    pub reason: Vec<String>,
    pub score_breakdown: RecallScoreBreakdown,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallScoreBreakdown {
    pub task_similarity: f32,
    pub stack_match: f32,
    pub file_match: f32,
    pub confidence: f32,
    pub recency: f32,
    pub scope: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallResult {
    pub query: String,
    pub matches: Vec<RecallMatch>,
    pub candidate_strategies: Vec<String>,
    pub avoid: Vec<String>,
    pub recommended_checks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InitResult {
    pub root: PathBuf,
    pub created: Vec<PathBuf>,
}

pub struct LoopLensEngine {
    root: PathBuf,
}

impl LoopLensEngine {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn init(&self) -> Result<InitResult> {
        let base = self.memory_dir();
        let paths = [
            base.clone(),
            base.join(EXPERIENCES_DIR),
            base.join(TRAJECTORIES_DIR),
        ];
        let mut created = Vec::new();

        for path in paths {
            if !path.exists() {
                fs::create_dir_all(&path)
                    .with_context(|| format!("failed to create {}", path.display()))?;
                created.push(path);
            }
        }

        let project_path = base.join(PROJECT_FILE);
        if !project_path.exists() {
            let context = toml::to_string_pretty(&ProjectContext::default())?;
            fs::write(&project_path, context)
                .with_context(|| format!("failed to write {}", project_path.display()))?;
            created.push(project_path);
        }

        let config_path = base.join(LEGACY_CONFIG_FILE);
        if !config_path.exists() {
            let config = toml::to_string_pretty(&Config::default())?;
            fs::write(&config_path, config)
                .with_context(|| format!("failed to write {}", config_path.display()))?;
            created.push(config_path);
        }

        let loop_path = base.join(LOOP_FILE);
        if !loop_path.exists() {
            fs::write(&loop_path, empty_loop_doc())
                .with_context(|| format!("failed to write {}", loop_path.display()))?;
            created.push(loop_path);
        }

        Ok(InitResult {
            root: base,
            created,
        })
    }

    pub fn learn(&self, input: LearnInput) -> Result<EngineeringExperience> {
        self.ensure_initialized()?;
        validate_learn_input(&input)?;

        let existing = self.load_experiences()?;
        let id = next_id(existing.len() + 1);
        let verified_at = Utc::now();
        let mut verification = input.verification;
        if verification.result == VerificationResult::Unknown {
            verification.result = VerificationResult::Passed;
        }
        if verification.files_changed.is_empty() {
            verification.files_changed = input.files.clone();
        }
        let mut evidence = input.evidence;
        if evidence.files_changed.is_empty() {
            evidence.files_changed = input.files.clone();
        }
        let experience = EngineeringExperience {
            id: id.clone(),
            created_at: verified_at,
            verified_at,
            task: TaskRecord {
                summary: input.task,
                task_type: input.task_type,
                hypothesis: input.hypothesis,
            },
            context: self.project_context().unwrap_or_default(),
            trajectory: TrajectorySummary {
                failed_attempts: input.failed_attempts,
                successful_decision: input.successful_decision,
            },
            lesson: input.lesson,
            verification,
            outcome: ExperienceOutcome::VerifiedSuccess,
            evidence,
            scope: input.scope,
            confidence: input.confidence,
        };

        let path = self
            .experiences_dir()
            .join(format!("{}.yaml", id.to_lowercase()));
        let yaml = serde_yaml::to_string(&experience)?;
        fs::write(&path, yaml).with_context(|| format!("failed to write {}", path.display()))?;
        self.write_trajectory(&experience)?;
        Ok(experience)
    }

    pub fn recall(&self, input: RecallInput) -> Result<RecallResult> {
        self.ensure_initialized()?;
        let top_k = input.top_k.max(1);
        let task_tokens = tokenize(&input.task);
        let file_tokens = tokenize(&input.files.join(" "));
        let stack_tokens = tokenize(&format!(
            "{} {}",
            input.languages.join(" "),
            input.frameworks.join(" ")
        ));
        let experiences = self.load_experiences()?;
        let document_frequency = document_frequency(&experiences);
        let total_docs = experiences.len().max(1) as f32;

        let mut matches: Vec<RecallMatch> = experiences
            .into_iter()
            .filter_map(|experience| {
                let doc = experience_text(&experience);
                let doc_tokens = tokenize(&doc);
                let matched_terms = overlap_terms(&task_tokens, &doc_tokens);
                let context_tokens = tokenize(&experience_context_text(&experience));
                let matched_context_terms = overlap_terms(&stack_tokens, &context_tokens);
                let experience_file_tokens = tokenize(&format!(
                    "{} {}",
                    experience.verification.files_changed.join(" "),
                    experience.evidence.files_changed.join(" ")
                ));
                let matched_file_terms = overlap_terms(&file_tokens, &experience_file_tokens);

                if matched_terms.is_empty()
                    && matched_context_terms.is_empty()
                    && matched_file_terms.is_empty()
                {
                    return None;
                }

                let task_weighted = matched_terms.iter().fold(0.0, |score, term| {
                    let df = *document_frequency.get(term).unwrap_or(&1) as f32;
                    let idf = ((total_docs + 1.0) / (df + 1.0)).ln() + 1.0;
                    score + idf
                });

                let task_similarity =
                    normalize_score(task_weighted, task_tokens.len().max(1) as f32);
                let stack_match = if stack_tokens.is_empty() {
                    0.0
                } else {
                    ratio(matched_context_terms.len(), stack_tokens.len())
                };
                let file_match = if file_tokens.is_empty() {
                    0.0
                } else {
                    ratio(matched_file_terms.len(), file_tokens.len())
                };
                let confidence = high_confidence_score(&experience);
                let recency = recency_score(experience.verified_at);
                let scope = scope_score(&experience.scope);
                let score_breakdown = RecallScoreBreakdown {
                    task_similarity,
                    stack_match,
                    file_match,
                    confidence,
                    recency,
                    scope,
                };
                let score = task_similarity * 0.35
                    + stack_match * 0.20
                    + file_match * 0.20
                    + confidence * 0.10
                    + recency * 0.10
                    + scope * 0.05;

                Some(RecallMatch {
                    reason: recall_reasons(
                        &matched_terms,
                        &matched_context_terms,
                        &matched_file_terms,
                        &experience,
                    ),
                    experience,
                    score,
                    matched_terms,
                    matched_context_terms,
                    matched_file_terms,
                    score_breakdown,
                })
            })
            .collect();

        matches.sort_by(|a, b| b.score.total_cmp(&a.score));
        matches.truncate(top_k);

        let candidate_strategies = matches
            .iter()
            .filter(|m| m.experience.outcome == ExperienceOutcome::VerifiedSuccess)
            .map(|m| m.experience.trajectory.successful_decision.clone())
            .collect();
        let avoid = matches
            .iter()
            .flat_map(|m| m.experience.trajectory.failed_attempts.clone())
            .collect();
        let recommended_checks = matches
            .iter()
            .map(|m| m.experience.lesson.clone())
            .collect();

        Ok(RecallResult {
            query: input.task,
            matches,
            candidate_strategies,
            avoid,
            recommended_checks,
        })
    }

    pub fn project_context(&self) -> Result<ProjectContext> {
        let project_path = self.memory_dir().join(PROJECT_FILE);
        if project_path.exists() {
            let raw = fs::read_to_string(&project_path)
                .with_context(|| format!("failed to read {}", project_path.display()))?;
            return toml::from_str(&raw)
                .with_context(|| format!("failed to parse {}", project_path.display()));
        }
        let config_path = self.memory_dir().join(LEGACY_CONFIG_FILE);
        if config_path.exists() {
            let raw = fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read {}", config_path.display()))?;
            let config: Config = toml::from_str(&raw)
                .with_context(|| format!("failed to parse {}", config_path.display()))?;
            return Ok(ProjectContext {
                name: config.project,
                ..ProjectContext::default()
            });
        }
        Ok(ProjectContext::default())
    }

    pub fn export_loop(&self) -> Result<String> {
        self.ensure_initialized()?;
        let mut experiences = self.load_experiences()?;
        experiences.sort_by(|a, b| a.id.cmp(&b.id));
        let markdown = render_loop_doc(&experiences);
        let path = self.memory_dir().join(LOOP_FILE);
        fs::write(&path, &markdown)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(markdown)
    }

    pub fn load_experiences(&self) -> Result<Vec<EngineeringExperience>> {
        let dir = self.experiences_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut experiences = Vec::new();
        for entry in
            fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }

            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let experience: EngineeringExperience = serde_yaml::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            experiences.push(experience);
        }

        experiences.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(experiences)
    }

    fn ensure_initialized(&self) -> Result<()> {
        let memory = self.memory_dir();
        if !memory.join(PROJECT_FILE).exists() && !memory.join(LEGACY_CONFIG_FILE).exists() {
            anyhow::bail!("LoopLens is not initialized. Run `looplens init` first.");
        }
        Ok(())
    }

    fn write_trajectory(&self, experience: &EngineeringExperience) -> Result<()> {
        let mut lines = vec![
            format!("# {} Trajectory", experience.id),
            String::new(),
            format!("Task: {}", experience.task.summary),
            format!("Type: {:?}", experience.task.task_type),
            format!(
                "Outcome: {:?} at {}",
                experience.outcome,
                experience.verified_at.to_rfc3339()
            ),
            String::new(),
        ];

        if let Some(command) = &experience.verification.command {
            lines.push(format!("Verification command: {}", command));
        }
        if let Some(reference) = &experience.verification.reference {
            lines.push(format!("Verification reference: {}", reference));
        }
        if let Some(run_id) = &experience.verification.run_id {
            lines.push(format!("Verification run: {}", run_id));
        }
        if let Some(target_url) = &experience.verification.target_url {
            lines.push(format!("Target URL: {}", target_url));
        }
        if let Some(commit_sha) = &experience.evidence.commit_sha {
            lines.push(format!("Commit: {}", commit_sha));
        }
        if let Some(branch) = &experience.evidence.branch {
            lines.push(format!("Branch: {}", branch));
        }
        if let Some(agent) = &experience.evidence.agent {
            lines.push(format!("Agent: {}", agent));
        }
        if !experience.evidence.files_changed.is_empty() {
            lines.push(format!(
                "Files changed: {}",
                experience.evidence.files_changed.join(", ")
            ));
        }
        lines.push(String::new());

        for attempt in &experience.trajectory.failed_attempts {
            lines.push(format!("- FAILED: {}", attempt));
        }
        if !experience.trajectory.successful_decision.is_empty() {
            lines.push(format!(
                "- SUCCESS: {}",
                experience.trajectory.successful_decision
            ));
        }

        let path = self
            .memory_dir()
            .join(TRAJECTORIES_DIR)
            .join(format!("{}.md", experience.id.to_lowercase()));
        fs::write(&path, lines.join("\n"))
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    fn memory_dir(&self) -> PathBuf {
        self.root.join(LOOPLENS_DIR)
    }

    fn experiences_dir(&self) -> PathBuf {
        self.memory_dir().join(EXPERIENCES_DIR)
    }
}

fn validate_learn_input(input: &LearnInput) -> Result<()> {
    if input.task.trim().is_empty() {
        anyhow::bail!("task is required");
    }
    if input.successful_decision.trim().is_empty() {
        anyhow::bail!("successful decision is required");
    }
    if input.lesson.trim().is_empty() {
        anyhow::bail!("lesson is required");
    }
    if input.verification.result == VerificationResult::Failed {
        anyhow::bail!("learn stores verified successes; record failed attempts in the trajectory");
    }
    if !(0.0..=1.0).contains(&input.confidence) {
        anyhow::bail!("confidence must be between 0.0 and 1.0");
    }
    Ok(())
}

fn next_id(next: usize) -> String {
    format!("EXP-{next:03}")
}

fn tokenize(text: &str) -> HashSet<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_lowercase();
            (token.len() > 2).then_some(token)
        })
        .collect()
}

fn overlap_terms(left: &HashSet<String>, right: &HashSet<String>) -> Vec<String> {
    let mut terms: Vec<String> = left.intersection(right).cloned().collect();
    terms.sort();
    terms
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f32 / denominator as f32
}

fn normalize_score(score: f32, denominator: f32) -> f32 {
    (score / denominator.max(1.0)).clamp(0.0, 1.0)
}

fn recency_score(verified_at: DateTime<Utc>) -> f32 {
    let age_days = Utc::now()
        .signed_duration_since(verified_at)
        .num_days()
        .max(0) as f32;
    (1.0 / (1.0 + age_days / 90.0)).clamp(0.0, 1.0)
}

fn high_confidence_score(experience: &EngineeringExperience) -> f32 {
    match experience.outcome {
        ExperienceOutcome::VerifiedSuccess => experience.confidence.clamp(0.0, 1.0),
        ExperienceOutcome::VerifiedFailure => (experience.confidence * 0.35).clamp(0.0, 0.35),
        ExperienceOutcome::Unverified => (experience.confidence * 0.15).clamp(0.0, 0.15),
    }
}

fn scope_score(scope: &MemoryScope) -> f32 {
    let mut score = 0.0;
    if scope.project {
        score += 0.45;
    }
    if scope.stack {
        score += 0.35;
    }
    if scope.global {
        score += 0.20;
    }
    score
}

fn document_frequency(experiences: &[EngineeringExperience]) -> HashMap<String, usize> {
    let mut frequency = HashMap::new();
    for experience in experiences {
        for token in tokenize(&experience_text(experience)) {
            *frequency.entry(token).or_insert(0) += 1;
        }
    }
    frequency
}

fn experience_text(experience: &EngineeringExperience) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        experience.task.summary,
        experience.task.hypothesis.as_deref().unwrap_or_default(),
        experience.trajectory.failed_attempts.join(" "),
        experience.trajectory.successful_decision,
        experience.verification.files_changed.join(" "),
        experience.evidence.files_changed.join(" "),
        experience.lesson,
    )
}

fn experience_context_text(experience: &EngineeringExperience) -> String {
    format!(
        "{} {} {} {}",
        experience.context.languages.join(" "),
        experience.context.frameworks.join(" "),
        experience.context.runtime.as_deref().unwrap_or_default(),
        experience
            .context
            .package_manager
            .as_deref()
            .unwrap_or_default(),
    )
}

fn recall_reasons(
    terms: &[String],
    context_terms: &[String],
    file_terms: &[String],
    experience: &EngineeringExperience,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if !terms.is_empty() {
        reasons.push(format!("task overlap: {}", terms.join(", ")));
    }
    if !context_terms.is_empty() {
        reasons.push(format!("stack overlap: {}", context_terms.join(", ")));
    }
    if !file_terms.is_empty() {
        reasons.push(format!("file/path overlap: {}", file_terms.join(", ")));
    }
    if experience.outcome == ExperienceOutcome::VerifiedSuccess {
        reasons.push("verified successful outcome".to_string());
    }
    reasons
}

fn empty_loop_doc() -> &'static str {
    "# LOOP.md\n\nNo engineering experiences recorded yet.\n"
}

fn render_loop_doc(experiences: &[EngineeringExperience]) -> String {
    let mut out =
        String::from("# LOOP.md\n\nEngineering experience memory generated by LoopLens.\n\n");
    if experiences.is_empty() {
        out.push_str("No engineering experiences recorded yet.\n");
        return out;
    }

    for experience in experiences {
        out.push_str(&format!(
            "## {} - {}\n\n",
            experience.id, experience.task.summary
        ));
        out.push_str(&format!(
            "Outcome: {:?} at {}\n\n",
            experience.outcome,
            experience.verified_at.to_rfc3339()
        ));
        if let Some(hypothesis) = &experience.task.hypothesis {
            out.push_str(&format!("Context/hypothesis: {}\n\n", hypothesis));
        }
        out.push_str(&format!(
            "Project context: {} | languages: {} | frameworks: {}\n\n",
            experience.context.name,
            csv_or_none(&experience.context.languages),
            csv_or_none(&experience.context.frameworks)
        ));
        let verification = &experience.verification;
        out.push_str("Verification:\n");
        out.push_str(&format!("- Source: {:?}\n", verification.source));
        out.push_str(&format!("- Result: {:?}\n", verification.result));
        if let Some(command) = &verification.command {
            out.push_str(&format!("- Command: {}\n", command));
        }
        if let Some(reference) = &verification.reference {
            out.push_str(&format!("- Reference: {}\n", reference));
        }
        if let Some(run_id) = &verification.run_id {
            out.push_str(&format!("- Run: {}\n", run_id));
        }
        if let Some(target_url) = &verification.target_url {
            out.push_str(&format!("- Target URL: {}\n", target_url));
        }
        if let Some(commit_sha) = &experience.evidence.commit_sha {
            out.push_str(&format!("- Commit: {}\n", commit_sha));
        }
        if let Some(branch) = &experience.evidence.branch {
            out.push_str(&format!("- Branch: {}\n", branch));
        }
        if let Some(agent) = &experience.evidence.agent {
            out.push_str(&format!("- Agent: {}\n", agent));
        }
        if !experience.evidence.files_changed.is_empty() {
            out.push_str("- Files changed:\n");
            for file in &experience.evidence.files_changed {
                out.push_str(&format!("  - {}\n", file));
            }
        }
        out.push('\n');
        out.push_str("Failed attempts to avoid:\n");
        if experience.trajectory.failed_attempts.is_empty() {
            out.push_str("- None recorded\n");
        } else {
            for attempt in &experience.trajectory.failed_attempts {
                out.push_str(&format!("- {}\n", attempt));
            }
        }
        out.push_str(&format!(
            "\nSuccessful decision: {}\n\nLesson: {}\n\nScope: project={} stack={} global={}\n\nConfidence: {:.2}\n\n",
            experience.trajectory.successful_decision,
            experience.lesson,
            experience.scope.project,
            experience.scope.stack,
            experience.scope.global,
            experience.confidence
        ));
    }
    out
}

fn csv_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

pub fn read_failure_bundle(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recalls_similar_verified_experience() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        engine
            .learn(LearnInput {
                task: "Login flow failed after auth state render".into(),
                task_type: TaskType::Bugfix,
                hypothesis: Some("Missing login button".into()),
                failed_attempts: vec!["Changed selector".into()],
                successful_decision: "Fix auth state rendering".into(),
                files: vec!["app/login/page.tsx".into()],
                lesson: "Check auth-state rendering before modifying selectors.".into(),
                verification: VerificationEvidence {
                    source: VerificationSource::Custom,
                    result: VerificationResult::Passed,
                    run_id: Some("run_123".into()),
                    target_url: Some("https://example.com".into()),
                    files_changed: vec!["app/login/page.tsx".into()],
                    ..VerificationEvidence::default()
                },
                evidence: CodeEvidence {
                    files_changed: vec!["app/login/page.tsx".into()],
                    ..CodeEvidence::default()
                },
                scope: MemoryScope::default(),
                confidence: 0.94,
            })
            .unwrap();

        let recall = engine
            .recall(RecallInput {
                task: "auth login button missing".into(),
                task_type: Some(TaskType::Bugfix),
                files: vec!["app/login/page.tsx".into()],
                languages: vec!["rust".into()],
                frameworks: vec![],
                top_k: 3,
            })
            .unwrap();

        assert_eq!(recall.matches.len(), 1);
        assert_eq!(recall.matches[0].experience.id, "EXP-001");
        assert!(recall.matches[0]
            .matched_terms
            .contains(&"login".to_string()));
        assert!(recall.matches[0]
            .matched_file_terms
            .contains(&"login".to_string()));
        assert!(recall.matches[0].score_breakdown.confidence > 0.9);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn loads_legacy_experience_shape() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        let legacy_yaml = r#"id: EXP-001
created_at: "2026-01-02T03:04:05Z"
problem: Legacy repair record
hypothesis: Missing login button
trajectory_summary:
  failed_attempts:
    - Tried the old route first
  successful_decision: Keep loading old experience files
patches:
  - packages/core/src/lib.rs
lesson: Default newly-added verification metadata when absent.
verified: PASS
confidence: 0.82
"#;
        fs::write(root.join(".looplens/experiences/exp-001.yaml"), legacy_yaml).unwrap();

        let loaded = engine.load_experiences().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].task.summary, "Legacy repair record");
        assert_eq!(loaded[0].verification.source, VerificationSource::Custom);
        assert_eq!(loaded[0].verification.result, VerificationResult::Passed);
        assert_eq!(loaded[0].outcome, ExperienceOutcome::VerifiedSuccess);
        assert_eq!(loaded[0].verified_at, loaded[0].created_at);

        let exported = engine.export_loop().unwrap();
        assert!(exported.contains("Engineering experience memory"));
        assert!(exported.contains("Legacy repair record"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn init_creates_v2_memory_layout() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();

        assert!(root.join(".looplens/project.toml").exists());
        assert!(root.join(".looplens/config.toml").exists());
        assert!(root.join(".looplens/experiences").is_dir());
        assert!(root.join(".looplens/trajectories").is_dir());
        assert!(root.join(".looplens/LOOP.md").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_confidence() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        let result = engine.learn(LearnInput {
            task: "Confidence must be bounded".into(),
            task_type: TaskType::Testing,
            hypothesis: None,
            failed_attempts: vec![],
            successful_decision: "Reject invalid confidence".into(),
            files: vec![],
            lesson: "Confidence remains reviewable when bounded.".into(),
            verification: VerificationEvidence::default(),
            evidence: CodeEvidence::default(),
            scope: MemoryScope::default(),
            confidence: 1.2,
        });

        assert!(result.is_err());
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("looplens-test-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
