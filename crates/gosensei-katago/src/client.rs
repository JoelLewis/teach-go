use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, mpsc, oneshot};

use crate::process::{KataGoProcess, ProcessError};
use crate::protocol::{AnalysisQuery, AnalysisResponse};

const DEFAULT_QUERY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("process error: {0}")]
    Process(#[from] ProcessError),

    #[error("failed to serialize query: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("query {0} timed out")]
    Timeout(String),

    #[error("engine not started")]
    NotStarted,
}

pub struct KataGoClient {
    stdin_tx: mpsc::Sender<String>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<AnalysisResponse>>>>,
}

impl KataGoClient {
    pub fn new(mut process: KataGoProcess) -> Self {
        let stdin_tx = process.sender();
        let pending: Arc<Mutex<HashMap<String, oneshot::Sender<AnalysisResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn response reader — owns the process, no shared lock needed
        let pending_clone = pending.clone();
        tokio::spawn(async move {
            while let Ok(line) = process.recv().await {
                if let Ok(response) = serde_json::from_str::<AnalysisResponse>(&line) {
                    let mut pending = pending_clone.lock().await;
                    if let Some(tx) = pending.remove(&response.id) {
                        let _ = tx.send(response);
                    }
                }
            }
        });

        Self { stdin_tx, pending }
    }

    /// Send a query and wait for the response with a default 30s timeout.
    pub async fn query(&self, query: AnalysisQuery) -> Result<AnalysisResponse, ClientError> {
        self.query_with_timeout(query, DEFAULT_QUERY_TIMEOUT).await
    }

    /// Send a query and wait for the response with a custom timeout.
    pub async fn query_with_timeout(
        &self,
        query: AnalysisQuery,
        timeout: Duration,
    ) -> Result<AnalysisResponse, ClientError> {
        let id = query.id.clone();
        let rx = self.query_fire(query).await?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => Err(ClientError::Timeout(id)),
            Err(_) => {
                // Remove the pending entry so it doesn't leak
                self.pending.lock().await.remove(&id);
                Err(ClientError::Timeout(id))
            }
        }
    }

    /// Send a query and return the receiver for later collection.
    /// Use this for pipelining multiple queries (e.g., batch review analysis).
    pub async fn query_fire(
        &self,
        query: AnalysisQuery,
    ) -> Result<oneshot::Receiver<AnalysisResponse>, ClientError> {
        let id = query.id.clone();
        let json = serde_json::to_string(&query)?;

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id.clone(), tx);
        }

        if let Err(e) = self.stdin_tx.send(json).await {
            self.pending.lock().await.remove(&id);
            return Err(ClientError::Process(ProcessError::Communication(e.to_string())));
        }

        Ok(rx)
    }
}
