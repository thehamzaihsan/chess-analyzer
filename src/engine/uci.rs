use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UciEval {
    pub depth: u32,
    pub score_cp: Option<i32>, // Centipawns from White's perspective (+ = White advantage)
    pub mate_in: Option<i32>,  // Mate in N moves (+ = White mates, - = Black mates)
    pub best_move: Option<String>,
    pub pv: Vec<String>,
}

pub struct UciEngine {
    _process: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
}

impl UciEngine {
    pub async fn spawn<P: AsRef<Path>>(
        binary_path: P,
        options: &[(&str, &str)],
    ) -> Result<Self> {
        let path = binary_path.as_ref();
        if !path.exists() {
            bail!("Engine executable not found at: {}", path.display());
        }

        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn engine process at {}", path.display()))?;

        let stdin = child.stdin.take().context("Failed to open engine stdin")?;
        let stdout = child.stdout.take().context("Failed to open engine stdout")?;
        let reader = BufReader::new(stdout);
        let lines = reader.lines();

        let mut engine = Self {
            _process: child,
            stdin,
            lines,
        };

        engine.init_uci(options).await?;
        Ok(engine)
    }

    async fn send_line(&mut self, line: &str) -> Result<()> {
        self.stdin
            .write_all(format!("{}\n", line).as_bytes())
            .await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn init_uci(&mut self, options: &[(&str, &str)]) -> Result<()> {
        self.send_line("uci").await?;

        // Wait for uciok
        while let Some(line) = self.lines.next_line().await? {
            if line.trim() == "uciok" {
                break;
            }
        }

        // Apply any custom options
        for (name, value) in options {
            self.send_line(&format!("setoption name {} value {}", name, value))
                .await?;
        }

        self.send_line("isready").await?;
        while let Some(line) = self.lines.next_line().await? {
            if line.trim() == "readyok" {
                break;
            }
        }

        self.send_line("ucinewgame").await?;
        Ok(())
    }

    /// Evaluates a given FEN up to `max_depth` or within `movetime_ms`.
    /// Returns the evaluation normalized from White's perspective (+cp = White is better).
    pub async fn evaluate_fen(
        &mut self,
        fen: &str,
        max_depth: u32,
        movetime_ms: Option<u64>,
    ) -> Result<UciEval> {
        let is_white_turn = !fen.contains(" b ");

        self.send_line(&format!("position fen {}", fen)).await?;

        let go_cmd = if let Some(ms) = movetime_ms {
            format!("go movetime {}", ms)
        } else {
            format!("go depth {}", max_depth)
        };
        self.send_line(&go_cmd).await?;

        let mut latest_eval = UciEval::default();

        // Read engine outputs until bestmove
        loop {
            let next_line = timeout(Duration::from_secs(15), self.lines.next_line()).await;
            let line = match next_line {
                Ok(Ok(Some(l))) => l,
                Ok(Ok(None)) => break,
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => {
                    // Timeout hit, stop search
                    let _ = self.send_line("stop").await;
                    break;
                }
            };

            let trimmed = line.trim();

            if trimmed.starts_with("info ") {
                Self::parse_info_line(trimmed, is_white_turn, &mut latest_eval);
            } else if trimmed.starts_with("bestmove ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    latest_eval.best_move = Some(parts[1].to_string());
                }
                break;
            }
        }

        Ok(latest_eval)
    }
    pub async fn configure_bot(&mut self, elo: u32) -> Result<()> {
        let skill = ((elo.saturating_sub(1350) as f32 / 850.0) * 20.0).clamp(0.0, 20.0) as u32;
        self.send_line("setoption name UCI_LimitStrength value true").await?;
        self.send_line(&format!("setoption name UCI_Elo value {}", elo)).await?;
        self.send_line(&format!("setoption name Skill Level value {}", skill)).await?;
        self.send_line("isready").await?;
        while let Some(line) = self.lines.next_line().await? {
            if line.trim() == "readyok" {
                break;
            }
        }
        Ok(())
    }

    pub async fn search_move(&mut self, fen: &str, movetime_ms: u64) -> Result<String> {
        let eval = self.evaluate_fen(fen, 10, Some(movetime_ms)).await?;
        eval.best_move.ok_or_else(|| anyhow::anyhow!("No move returned by engine"))
    }


    fn parse_info_line(line: &str, is_white_turn: bool, eval: &mut UciEval) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut idx = 0;

        while idx < parts.len() {
            match parts[idx] {
                "depth" if idx + 1 < parts.len() => {
                    if let Ok(d) = parts[idx + 1].parse::<u32>() {
                        eval.depth = d;
                    }
                    idx += 2;
                }
                "score" if idx + 2 < parts.len() => {
                    match parts[idx + 1] {
                        "cp" => {
                            if let Ok(cp) = parts[idx + 2].parse::<i32>() {
                                // UCI scores are from perspective of side to move.
                                // We convert to White perspective:
                                eval.score_cp = Some(if is_white_turn { cp } else { -cp });
                                eval.mate_in = None;
                            }
                        }
                        "mate" => {
                            if let Ok(m) = parts[idx + 2].parse::<i32>() {
                                eval.mate_in = Some(if is_white_turn { m } else { -m });
                                eval.score_cp = None;
                            }
                        }
                        _ => {}
                    }
                    idx += 3;
                }
                "pv" => {
                    eval.pv = parts[idx + 1..].iter().map(|s| s.to_string()).collect();
                    break;
                }
                _ => {
                    idx += 1;
                }
            }
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.send_line("stop").await
    }

    pub async fn quit(&mut self) -> Result<()> {
        let _ = self.send_line("quit").await;
        Ok(())
    }
}

/// Discovers the path to Stockfish or lc0 binary.
pub fn discover_engine_path(engine_type: &str) -> Option<PathBuf> {
    // 1. Check local assets folder
    let local_sf = PathBuf::from("assets/stockfish/linux/stockfish_bin");
    let local_sf_alt = PathBuf::from("assets/stockfish/linux/stockfish/stockfish-linux-x86-64-universal");
    let local_lc0 = PathBuf::from("assets/lc0/linux/lc0");

    match engine_type {
        "stockfish" => {
            if local_sf.exists() {
                return Some(local_sf);
            }
            if local_sf_alt.exists() {
                return Some(local_sf_alt);
            }
            // Check PATH
            if let Ok(path) = which_path("stockfish") {
                return Some(path);
            }
        }
        "lc0" => {
            if local_lc0.exists() {
                return Some(local_lc0);
            }
            if let Ok(path) = which_path("lc0") {
                return Some(path);
            }
        }
        _ => {}
    }

    None
}

fn which_path(binary_name: &str) -> Result<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    bail!("{} not found in PATH", binary_name)
}
