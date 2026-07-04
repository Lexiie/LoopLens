use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const LOOPLENS_DIR: &str = ".looplens";
const CONFIG_FILE: &str = "config.toml";
const EXPERIENCES_DIR: &str = "experiences";
const TRAJECTORIES_DIR: &str = "trajectories";
const LOOP_FILE: &str = "LOOP.md";

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
            version: "0.1".to_string(),
            project: "LoopLens repository memory".to_string(),
            storage: StorageConfig {
                experiences_dir: EXPERIENCES_DIR.to_string(),
                trajectories_dir: TRAJECTORIES_DIR.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairExperience {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
    pub problem: String,
    pub testsprite_hypothesis: Option<String>,
    pub trajectory_summary: TrajectorySummary,
    pub patches: Vec<String>,
    pub lesson: String,
    pub evidence: VerificationEvidence,
    pub verified: VerificationStatus,
    pub confidence: f32,
}

impl<'de> Deserialize<'de> for RepairExperience {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RepairExperienceFields {
            id: String,
            created_at: DateTime<Utc>,
            #[serde(default)]
            verified_at: Option<DateTime<Utc>>,
            problem: String,
            testsprite_hypothesis: Option<String>,
            trajectory_summary: TrajectorySummary,
            patches: Vec<String>,
            lesson: String,
            #[serde(default)]
            evidence: VerificationEvidence,
            verified: VerificationStatus,
            confidence: f32,
        }

        let fields = RepairExperienceFields::deserialize(deserializer)?;
        Ok(Self {
            id: fields.id,
            created_at: fields.created_at,
            verified_at: fields.verified_at.unwrap_or(fields.created_at),
            problem: fields.problem,
            testsprite_hypothesis: fields.testsprite_hypothesis,
            trajectory_summary: fields.trajectory_summary,
            patches: fields.patches,
            lesson: fields.lesson,
            evidence: fields.evidence,
            verified: fields.verified,
            confidence: fields.confidence,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub testsprite_run_id: Option<String>,
    pub test_id: Option<String>,
    pub target_url: Option<String>,
    pub dashboard_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectorySummary {
    pub failed_attempts: Vec<String>,
    pub successful_decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum VerificationStatus {
    Pass,
}

#[derive(Debug, Clone)]
pub struct LearnInput {
    pub problem: String,
    pub testsprite_hypothesis: Option<String>,
    pub failed_attempts: Vec<String>,
    pub successful_decision: String,
    pub patches: Vec<String>,
    pub lesson: String,
    pub evidence: VerificationEvidence,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct RecallInput {
    pub query: String,
    pub top_k: usize,
}

#[derive(Debug, Clone)]
pub struct RecallMatch {
    pub experience: RepairExperience,
    pub score: f32,
    pub matched_terms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RecallResult {
    pub query: String,
    pub matches: Vec<RecallMatch>,
    pub candidate_strategies: Vec<String>,
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

        let config_path = base.join(CONFIG_FILE);
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

    pub fn learn(&self, input: LearnInput) -> Result<RepairExperience> {
        self.ensure_initialized()?;
        validate_learn_input(&input)?;

        let existing = self.load_experiences()?;
        let id = next_id(existing.len() + 1);
        let verified_at = Utc::now();
        let experience = RepairExperience {
            id: id.clone(),
            created_at: verified_at,
            verified_at,
            problem: input.problem,
            testsprite_hypothesis: input.testsprite_hypothesis,
            trajectory_summary: TrajectorySummary {
                failed_attempts: input.failed_attempts,
                successful_decision: input.successful_decision,
            },
            patches: input.patches,
            lesson: input.lesson,
            evidence: input.evidence,
            verified: VerificationStatus::Pass,
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
        let query_tokens = tokenize(&input.query);
        let experiences = self.load_experiences()?;
        let document_frequency = document_frequency(&experiences);
        let total_docs = experiences.len().max(1) as f32;

        let mut matches: Vec<RecallMatch> = experiences
            .into_iter()
            .filter_map(|experience| {
                let doc = experience_text(&experience);
                let doc_tokens = tokenize(&doc);
                let matched_terms: Vec<String> =
                    query_tokens.intersection(&doc_tokens).cloned().collect();

                if matched_terms.is_empty() {
                    return None;
                }

                let lexical = matched_terms.iter().fold(0.0, |score, term| {
                    let df = *document_frequency.get(term).unwrap_or(&1) as f32;
                    let idf = ((total_docs + 1.0) / (df + 1.0)).ln() + 1.0;
                    score + idf
                });
                let coverage = matched_terms.len() as f32 / query_tokens.len().max(1) as f32;
                let score = lexical * 0.65 + coverage * 0.25 + experience.confidence * 0.10;

                Some(RecallMatch {
                    experience,
                    score,
                    matched_terms,
                })
            })
            .collect();

        matches.sort_by(|a, b| b.score.total_cmp(&a.score));
        matches.truncate(top_k);

        let candidate_strategies = matches
            .iter()
            .map(|m| m.experience.trajectory_summary.successful_decision.clone())
            .collect();

        Ok(RecallResult {
            query: input.query,
            matches,
            candidate_strategies,
        })
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

    pub fn load_experiences(&self) -> Result<Vec<RepairExperience>> {
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
            let experience: RepairExperience = serde_yaml::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            experiences.push(experience);
        }

        experiences.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(experiences)
    }

    fn ensure_initialized(&self) -> Result<()> {
        let config = self.memory_dir().join(CONFIG_FILE);
        if !config.exists() {
            anyhow::bail!("LoopLens is not initialized. Run `looplens init` first.");
        }
        Ok(())
    }

    fn write_trajectory(&self, experience: &RepairExperience) -> Result<()> {
        let mut lines = vec![
            format!("# {} Trajectory", experience.id),
            String::new(),
            format!("Problem: {}", experience.problem),
            format!("Verified: PASS at {}", experience.verified_at.to_rfc3339()),
            String::new(),
        ];

        if let Some(run_id) = &experience.evidence.testsprite_run_id {
            lines.push(format!("TestSprite run: {}", run_id));
        }
        if let Some(test_id) = &experience.evidence.test_id {
            lines.push(format!("TestSprite test: {}", test_id));
        }
        if let Some(target_url) = &experience.evidence.target_url {
            lines.push(format!("Target URL: {}", target_url));
        }
        if let Some(dashboard_url) = &experience.evidence.dashboard_url {
            lines.push(format!("Dashboard: {}", dashboard_url));
        }
        lines.push(String::new());

        for attempt in &experience.trajectory_summary.failed_attempts {
            lines.push(format!("- FAIL: {}", attempt));
        }
        lines.push(format!(
            "- PASS: {}",
            experience.trajectory_summary.successful_decision
        ));

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
    if input.problem.trim().is_empty() {
        anyhow::bail!("problem is required");
    }
    if input.successful_decision.trim().is_empty() {
        anyhow::bail!("successful decision is required");
    }
    if input.lesson.trim().is_empty() {
        anyhow::bail!("lesson is required");
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

fn document_frequency(experiences: &[RepairExperience]) -> HashMap<String, usize> {
    let mut frequency = HashMap::new();
    for experience in experiences {
        for token in tokenize(&experience_text(experience)) {
            *frequency.entry(token).or_insert(0) += 1;
        }
    }
    frequency
}

fn experience_text(experience: &RepairExperience) -> String {
    format!(
        "{} {} {} {} {} {}",
        experience.problem,
        experience
            .testsprite_hypothesis
            .as_deref()
            .unwrap_or_default(),
        experience.trajectory_summary.failed_attempts.join(" "),
        experience.trajectory_summary.successful_decision,
        experience.patches.join(" "),
        experience.lesson,
    )
}

fn empty_loop_doc() -> &'static str {
    "# LOOP.md\n\nNo verified repair experiences recorded yet.\n"
}

fn render_loop_doc(experiences: &[RepairExperience]) -> String {
    let mut out = String::from("# LOOP.md\n\nRepair experience memory generated by LoopLens.\n\n");
    if experiences.is_empty() {
        out.push_str("No verified repair experiences recorded yet.\n");
        return out;
    }

    for experience in experiences {
        out.push_str(&format!(
            "## {} - {}\n\n",
            experience.id, experience.problem
        ));
        out.push_str(&format!(
            "Verified: PASS at {}\n\n",
            experience.verified_at.to_rfc3339()
        ));
        if let Some(hypothesis) = &experience.testsprite_hypothesis {
            out.push_str(&format!("TestSprite hypothesis: {}\n\n", hypothesis));
        }
        let evidence = &experience.evidence;
        if evidence.testsprite_run_id.is_some()
            || evidence.test_id.is_some()
            || evidence.target_url.is_some()
            || evidence.dashboard_url.is_some()
        {
            out.push_str("Evidence:\n");
            if let Some(run_id) = &evidence.testsprite_run_id {
                out.push_str(&format!("- TestSprite run: {}\n", run_id));
            }
            if let Some(test_id) = &evidence.test_id {
                out.push_str(&format!("- TestSprite test: {}\n", test_id));
            }
            if let Some(target_url) = &evidence.target_url {
                out.push_str(&format!("- Target URL: {}\n", target_url));
            }
            if let Some(dashboard_url) = &evidence.dashboard_url {
                out.push_str(&format!("- Dashboard: {}\n", dashboard_url));
            }
            out.push('\n');
        }
        out.push_str("Failed attempts:\n");
        if experience.trajectory_summary.failed_attempts.is_empty() {
            out.push_str("- None recorded\n");
        } else {
            for attempt in &experience.trajectory_summary.failed_attempts {
                out.push_str(&format!("- {}\n", attempt));
            }
        }
        out.push_str(&format!(
            "\nSuccessful decision: {}\n\nLesson: {}\n\nConfidence: {:.2}\n\n",
            experience.trajectory_summary.successful_decision,
            experience.lesson,
            experience.confidence
        ));
    }
    out
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
                problem: "Login flow failed after auth state render".into(),
                testsprite_hypothesis: Some("Missing login button".into()),
                failed_attempts: vec!["Changed selector".into()],
                successful_decision: "Fix auth state rendering".into(),
                patches: vec!["app/login/page.tsx".into()],
                lesson: "Check auth-state rendering before modifying selectors.".into(),
                evidence: VerificationEvidence {
                    testsprite_run_id: Some("run_123".into()),
                    test_id: Some("test_123".into()),
                    target_url: Some("https://example.com".into()),
                    dashboard_url: Some("https://www.testsprite.com/dashboard/tests/demo".into()),
                },
                confidence: 0.94,
            })
            .unwrap();

        let recall = engine
            .recall(RecallInput {
                query: "auth login button missing".into(),
                top_k: 3,
            })
            .unwrap();

        assert_eq!(recall.matches.len(), 1);
        assert_eq!(recall.matches[0].experience.id, "EXP-001");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unverified_shape_by_only_modeling_pass() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        let result = engine.learn(LearnInput {
            problem: "".into(),
            testsprite_hypothesis: None,
            failed_attempts: vec![],
            successful_decision: "Fix route".into(),
            patches: vec![],
            lesson: "Keep route and test expectation aligned".into(),
            evidence: VerificationEvidence::default(),
            confidence: 0.8,
        });
        assert!(result.is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn init_creates_repo_memory_layout() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();

        assert!(root.join(".looplens/config.toml").exists());
        assert!(root.join(".looplens/experiences").is_dir());
        assert!(root.join(".looplens/trajectories").is_dir());
        assert!(root.join(".looplens/LOOP.md").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn persists_reloads_and_exports_evidence() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        engine
            .learn(LearnInput {
                problem: "Demo flow needs verified repair context".into(),
                testsprite_hypothesis: Some("Workflow indicator did not activate".into()),
                failed_attempts: vec!["Adjusted visual spacing".into()],
                successful_decision: "Wire Learn PASS to activate PASS and LOOP indicators".into(),
                patches: vec!["examples/demo-app/src/App.jsx".into()],
                lesson: "Keep the verification surface aligned with the actual CLI loop.".into(),
                evidence: VerificationEvidence {
                    testsprite_run_id: Some("7e9da0ed-e9a1-4cee-9a4d-92c272bd557e".into()),
                    test_id: Some("1d52848a-4f5a-46af-a83f-f7cb9e9c0b29".into()),
                    target_url: Some("https://demo-app-pink-omega.vercel.app".into()),
                    dashboard_url: Some("https://www.testsprite.com/dashboard/tests/demo".into()),
                },
                confidence: 0.97,
            })
            .unwrap();

        let reloaded = LoopLensEngine::new(&root).load_experiences().unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(
            reloaded[0].evidence.testsprite_run_id.as_deref(),
            Some("7e9da0ed-e9a1-4cee-9a4d-92c272bd557e")
        );

        let exported = engine.export_loop().unwrap();
        assert!(exported.contains("Evidence:"));
        assert!(exported.contains("TestSprite run: 7e9da0ed-e9a1-4cee-9a4d-92c272bd557e"));
        assert!(exported.contains("Target URL: https://demo-app-pink-omega.vercel.app"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn loads_legacy_experience_without_verification_evidence() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        let legacy_yaml = r#"id: EXP-001
created_at: "2026-01-02T03:04:05Z"
problem: Legacy repair record
testsprite_hypothesis: null
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
        assert_eq!(loaded[0].verified_at, loaded[0].created_at);
        assert!(loaded[0].evidence.testsprite_run_id.is_none());

        let exported = engine.export_loop().unwrap();
        assert!(exported.contains("Legacy repair record"));
        assert!(!exported.contains("Evidence:"));

        let learned = engine
            .learn(LearnInput {
                problem: "New record after upgrade".into(),
                testsprite_hypothesis: None,
                failed_attempts: vec![],
                successful_decision: "Continue after reading legacy records".into(),
                patches: vec![],
                lesson: "Learning must not be blocked by older YAML.".into(),
                evidence: VerificationEvidence::default(),
                confidence: 0.9,
            })
            .unwrap();
        assert_eq!(learned.id, "EXP-002");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_confidence() {
        let root = temp_root();
        let engine = LoopLensEngine::new(&root);
        engine.init().unwrap();
        let result = engine.learn(LearnInput {
            problem: "Confidence must be bounded".into(),
            testsprite_hypothesis: None,
            failed_attempts: vec![],
            successful_decision: "Reject invalid confidence".into(),
            patches: vec![],
            lesson: "Confidence remains reviewable when bounded.".into(),
            evidence: VerificationEvidence::default(),
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
