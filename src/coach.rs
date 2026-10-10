use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Auto-discovers the local coach runner binary (`llama-cli`) packaged with the application.
pub fn discover_coach_binary() -> Result<PathBuf> {
    // 1. Current executable relative directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidates = [
                parent.join("assets/coach/linux/llama-cli"),
                parent.join("../../assets/coach/linux/llama-cli"),
                parent.join("../../../assets/coach/linux/llama-cli"),
            ];
            for c in &candidates {
                if c.exists() {
                    return Ok(c.canonicalize().unwrap_or_else(|_| c.clone()));
                }
            }
        }
    }

    // 2. Relative to working directory
    let cwd_candidates = [
        "assets/coach/linux/llama-cli",
        "../assets/coach/linux/llama-cli",
        "../../assets/coach/linux/llama-cli",
    ];
    for p in &cwd_candidates {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }

    bail!("Coach runner binary llama-cli not found in assets/coach/linux/")
}

/// Auto-discovers the locally downloaded Gemma model (.gguf) packaged with the application.
pub fn discover_gemma_model() -> Result<PathBuf> {
    let search_dirs = [
        "assets/models/gemma",
        "assets/gemma",
        "../assets/models/gemma",
        "../../assets/models/gemma",
    ];

    for dir_str in &search_dirs {
        let dir = PathBuf::from(dir_str);
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "gguf" {
                            return Ok(path.canonicalize().unwrap_or(path));
                        }
                    }
                }
            }
        }
    }

    bail!("Gemma model weights (.gguf) not found in assets/models/gemma/")
}

pub struct CoachClient {
    pub binary_path: Option<PathBuf>,
    pub model_path: Option<PathBuf>,
}

impl CoachClient {
    pub fn new(binary_path: Option<PathBuf>, model_path: Option<PathBuf>) -> Self {
        let binary = binary_path.or_else(|| discover_coach_binary().ok());
        let model = model_path.or_else(|| discover_gemma_model().ok());

        Self {
            binary_path: binary,
            model_path: model,
        }
    }

    /// Check if the packaged local model and runner binary are present on disk.
    pub async fn check_status(&self) -> Result<bool> {
        Ok(self.binary_path.is_some() && self.model_path.is_some())
    }

    /// List the packaged model(s) available locally.
    pub async fn list_models(&self) -> Result<Vec<String>> {
        if let Some(ref m) = self.model_path {
            let name = m
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "gemma-2-2b-it".to_string());
            Ok(vec![name])
        } else {
            Ok(vec![])
        }
    }

    /// Ask the packaged local Gemma model to explain why a move was a blunder/mistake and why the engine's best line works.
    pub async fn explain_move(
        &self,
        fen: &str,
        played_san: &str,
        classification: &str,
        win_prob_loss: f64,
        best_line: &[String],
    ) -> Result<String> {
        let binary = match &self.binary_path {
            Some(b) if b.exists() => b.clone(),
            _ => {
                return Ok(format!(
                    "Packaged Gemma runner not found. Tactical note: `{played_san}` was classified as {classification} (-{win_prob_loss:.1}% win probability). Engine recommends: {}",
                    if best_line.is_empty() { "N/A".to_string() } else { best_line.join(" ") }
                ));
            }
        };

        let model = match &self.model_path {
            Some(m) if m.exists() => m.clone(),
            _ => {
                return Ok(format!(
                    "Packaged Gemma model weights not found in assets/models/gemma/. Tactical note: `{played_san}` was classified as {classification} (-{win_prob_loss:.1}% win probability). Engine recommends: {}",
                    if best_line.is_empty() { "N/A".to_string() } else { best_line.join(" ") }
                ));
            }
        };

        let best_line_str = if best_line.is_empty() {
            "not available".to_string()
        } else {
            best_line.join(" ")
        };

        let prompt = format!(
            "<start_of_turn>user\n\
You are an expert, encouraging chess coach analyzing a student's game.\n\
The position is: FEN `{fen}`.\n\
The student played: `{played_san}`.\n\
Move classification: {classification} (evaluated win% drop of {loss:.1}%).\n\
The computer's suggested best continuation is: `{best_line_str}`.\n\n\
In 2 concise, clear paragraphs:\n\
1. Explain what tactic, hanging piece, or positional weakness made `{played_san}` a {classification}.\n\
2. Explain the tactical idea or positional advantage behind the suggested best line `{best_line_str}`.\n\
3. Give the student one concrete rule of thumb to remember for similar future positions.<end_of_turn>\n\
<start_of_turn>model\n",
            fen = fen,
            played_san = played_san,
            classification = classification,
            loss = win_prob_loss,
            best_line_str = best_line_str,
        );

        let binary_dir = binary.parent().unwrap_or_else(|| Path::new("."));

        let mut cmd = Command::new(&binary);
        cmd.current_dir(binary_dir)
            .arg("-m")
            .arg(&model)
            .arg("-p")
            .arg(&prompt)
            .arg("-n")
            .arg("160")
            .arg("-t")
            .arg("4")
            .arg("--temp")
            .arg("0.3")
            .arg("--no-warmup")
            .arg("--no-display-prompt")
            .arg("--log-disable")
            .arg("--simple-io")
            .arg("--single-turn")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Gemma runner exited with code {:?}: {}",
                output.status.code(),
                stderr.trim()
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cleaned = extract_gemma_response(&stdout);

        if cleaned.is_empty() {
            Ok(format!(
                "The move `{played_san}` is a {classification} causing a {win_prob_loss:.1}% win rate drop. Consider the suggested line `{best_line_str}` instead."
            ))
        } else {
            Ok(cleaned)
        }
    }
}

fn extract_gemma_response(stdout: &str) -> String {
    let text = if let Some(idx) = stdout.rfind("... (truncated)") {
        &stdout[idx + "... (truncated)".len()..]
    } else if let Some(idx) = stdout.rfind("<start_of_turn>model") {
        &stdout[idx + "<start_of_turn>model".len()..]
    } else if let Some(idx) = stdout.rfind("\n> ") {
        let after_prompt = &stdout[idx + 3..];
        if let Some(empty_line_idx) = after_prompt.find("\n\n") {
            &after_prompt[empty_line_idx + 2..]
        } else {
            after_prompt
        }
    } else {
        stdout
    };

    let mut text = text;
    for stop in ["[ Prompt:", "Exiting...", "<end_of_turn>"] {
        if let Some(idx) = text.find(stop) {
            text = &text[..idx];
        }
    }

    text.trim().to_string()
}
