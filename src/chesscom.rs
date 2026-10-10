use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

const APP_USER_AGENT: &str = "ChessAnalyzer/1.0 (contact: local-desktop-chess-app; Rust-native)";

#[derive(Debug, Deserialize)]
struct ArchivesResponse {
    archives: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GameRecord {
    pub url: Option<String>,
    pub pgn: Option<String>,
    pub rules: Option<String>,
    pub time_class: Option<String>,
    pub time_control: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MonthlyGamesResponse {
    games: Vec<GameRecord>,
}

#[derive(Clone)]
pub struct ChessComClient {
    client: reqwest::Client,
}

impl ChessComClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(APP_USER_AGENT));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(20))
            .build()
            .expect("Failed to build reqwest client");

        Self { client }
    }

    async fn get_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        let mut retries = 0;
        let max_retries = 3;
        let mut wait = Duration::from_millis(500);

        loop {
            let res = self.client.get(url).send().await;
            match res {
                Ok(response) => {
                    let status = response.status();
                    if status.as_u16() == 429 {
                        if retries < max_retries {
                            retries += 1;
                            sleep(wait).await;
                            wait *= 2;
                            continue;
                        } else {
                            bail!("Chess.com API rate limit (429) exceeded for {}", url);
                        }
                    }
                    if !status.is_success() {
                        bail!("Chess.com API request failed with status {}: {}", status, url);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    if retries < max_retries {
                        retries += 1;
                        sleep(wait).await;
                        wait *= 2;
                        continue;
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
    }

    /// Fetch list of monthly archive URLs for a username (e.g. `https://api.chess.com/pub/player/{username}/games/archives`)
    pub async fn fetch_user_archives(&self, username: &str) -> Result<Vec<String>> {
        let clean_user = username.trim().to_lowercase();
        let url = format!("https://api.chess.com/pub/player/{}/games/archives", clean_user);
        let res = self.get_with_retry(&url).await?;
        let data: ArchivesResponse = res.json().await.context("Failed to parse archives JSON")?;
        Ok(data.archives)
    }

    /// Fetch games for a single monthly archive URL, filtering to standard chess and returning PGN strings.
    pub async fn fetch_archive_pgns(&self, archive_url: &str) -> Result<Vec<String>> {
        let res = self.get_with_retry(archive_url).await?;
        let data: MonthlyGamesResponse = res.json().await.context("Failed to parse monthly games JSON")?;

        let mut pgns = Vec::new();
        for game in data.games {
            // Filter out non-standard chess variants (e.g. chess960, bughouse, crazyhouse)
            if let Some(rules) = &game.rules {
                if rules != "chess" {
                    continue;
                }
            }
            if let Some(pgn) = game.pgn {
                let trimmed = pgn.trim();
                if !trimmed.is_empty() {
                    pgns.push(trimmed.to_string());
                }
            }
        }

        Ok(pgns)
    }

    pub async fn fetch_all_games(&self, username: &str) -> Result<String> {
        let archives = self.fetch_user_archives(username).await?;
        let target_archives = if archives.len() > 3 {
            &archives[archives.len() - 3..]
        } else {
            &archives[..]
        };

        let mut all_pgns = Vec::new();
        for arc in target_archives {
            if let Ok(pgns) = self.fetch_archive_pgns(arc).await {
                all_pgns.extend(pgns);
            }
        }

        Ok(all_pgns.join("\n\n"))
    }

}
