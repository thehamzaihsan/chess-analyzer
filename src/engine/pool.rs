use crate::db::{CachedEval, Database};
use crate::engine::uci::{discover_engine_path, UciEngine, UciEval};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AnalysisJob {
    pub game_id: i64,
    pub move_idx: usize,
    pub fen: String,
    pub depth: u32,
    pub movetime_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub game_id: i64,
    pub move_idx: usize,
    pub fen: String,
    pub eval: UciEval,
    pub was_cached: bool,
}

#[derive(Debug, Clone)]
pub enum PoolProgress {
    Started { total: usize },
    Progress { completed: usize, total: usize, result: AnalysisResult },
    Finished { total: usize },
    Error(String),
}

pub struct EnginePool {
    engine_path: std::path::PathBuf,
    num_workers: usize,
    db: Database,
}

impl EnginePool {
    pub fn new(db: Database) -> Result<Self> {
        let path = discover_engine_path("stockfish")
            .ok_or_else(|| anyhow::anyhow!("Stockfish binary not found. Please install Stockfish or place it in assets/stockfish/."))?;

        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let num_workers = (cores.saturating_sub(1)).max(1);

        Ok(Self {
            engine_path: path,
            num_workers,
            db,
        })
    }

    pub fn with_engine_path(
        db: Database,
        engine_path: std::path::PathBuf,
        num_workers: usize,
    ) -> Self {
        Self {
            engine_path,
            num_workers: num_workers.max(1),
            db,
        }
    }

    /// Spawns background worker tasks to process the batch of analysis jobs.
    /// Emits progress messages via the returned Receiver channel.
    pub fn analyze_batch(
        &self,
        jobs: Vec<AnalysisJob>,
    ) -> Receiver<PoolProgress> {
        let (tx, rx) = mpsc::channel(100);
        let total = jobs.len();
        let engine_path = self.engine_path.clone();
        let max_workers = self.num_workers;
        let db = self.db.clone();

        tokio::spawn(async move {
            let _ = tx.send(PoolProgress::Started { total }).await;

            if total == 0 {
                let _ = tx.send(PoolProgress::Finished { total }).await;
                return;
            }

            let completed_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut uncached_jobs = Vec::new();

            for job in jobs {
                if let Some(c) = db.get_cached_eval(&job.fen).unwrap_or(None) {
                    let completed =
                        completed_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    let res = AnalysisResult {
                        game_id: job.game_id,
                        move_idx: job.move_idx,
                        fen: job.fen,
                        eval: UciEval {
                            depth: c.depth,
                            score_cp: c.score_cp,
                            mate_in: c.mate_in,
                            best_move: c.best_move,
                            pv: c.pv,
                        },
                        was_cached: true,
                    };
                    let _ = tx
                        .send(PoolProgress::Progress {
                            completed,
                            total,
                            result: res,
                        })
                        .await;
                } else {
                    uncached_jobs.push(job);
                }
            }

            if !uncached_jobs.is_empty() {
                let num_workers = max_workers.min(uncached_jobs.len()).max(1);
                uncached_jobs.reverse();
                let jobs_queue = Arc::new(Mutex::new(uncached_jobs));
                let mut worker_handles = Vec::new();

                for _ in 0..num_workers {
                    let queue = jobs_queue.clone();
                    let counter = completed_counter.clone();
                    let tx_worker = tx.clone();
                    let path = engine_path.clone();
                    let db_clone = db.clone();

                    let handle = tokio::spawn(async move {
                        let options = [("Threads", "1"), ("Hash", "32")];
                        let mut engine: Option<UciEngine> = None;

                        loop {
                            let job = {
                                let mut q = queue.lock().await;
                                q.pop()
                            };

                            let job = match job {
                                Some(j) => j,
                                None => break,
                            };

                            // Re-check cache in case a sibling worker evaluated the same FEN.
                            let cached = db_clone.get_cached_eval(&job.fen).unwrap_or(None);
                            let (eval, was_cached) = if let Some(c) = cached {
                                (
                                    UciEval {
                                        depth: c.depth,
                                        score_cp: c.score_cp,
                                        mate_in: c.mate_in,
                                        best_move: c.best_move,
                                        pv: c.pv,
                                    },
                                    true,
                                )
                            } else {
                                if engine.is_none() {
                                    match UciEngine::spawn(&path, &options).await {
                                        Ok(e) => engine = Some(e),
                                        Err(e) => {
                                            let _ = tx_worker
                                                .send(PoolProgress::Error(format!(
                                                    "Failed to start worker: {}",
                                                    e
                                                )))
                                                .await;
                                            break;
                                        }
                                    }
                                }

                                match engine
                                    .as_mut()
                                    .expect("engine initialized")
                                    .evaluate_fen(&job.fen, job.depth, job.movetime_ms)
                                    .await
                                {
                                    Ok(ev) => {
                                        let _ = db_clone.set_cached_eval(&CachedEval {
                                            fen: job.fen.clone(),
                                            depth: ev.depth,
                                            score_cp: ev.score_cp,
                                            mate_in: ev.mate_in,
                                            best_move: ev.best_move.clone(),
                                            pv: ev.pv.clone(),
                                        });
                                        (ev, false)
                                    }
                                    Err(e) => {
                                        let _ = tx_worker
                                            .send(PoolProgress::Error(format!(
                                                "Eval failed: {}",
                                                e
                                            )))
                                            .await;
                                        continue;
                                    }
                                }
                            };

                            let completed =
                                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                            let res = AnalysisResult {
                                game_id: job.game_id,
                                move_idx: job.move_idx,
                                fen: job.fen,
                                eval,
                                was_cached,
                            };

                            let _ = tx_worker
                                .send(PoolProgress::Progress {
                                    completed,
                                    total,
                                    result: res,
                                })
                                .await;
                        }

                        if let Some(mut eng) = engine {
                            let _ = eng.quit().await;
                        }
                    });

                    worker_handles.push(handle);
                }

                for h in worker_handles {
                    let _ = h.await;
                }
            }

            let _ = tx.send(PoolProgress::Finished { total }).await;
        });

        rx
    }
}
