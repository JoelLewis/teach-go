use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, oneshot};

use crate::process::{KataGoProcess, ProcessError};
use crate::protocol::{AnalysisQuery, AnalysisResponse};

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
    process: Arc<Mutex<KataGoProcess>>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<AnalysisResponse>>>>,
}

impl KataGoClient {
    pub fn new(process: KataGoProcess) -> Self {
        let process = Arc::new(Mutex::new(process));
        let pending: Arc<Mutex<HashMap<String, oneshot::Sender<AnalysisResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn response reader
        let process_clone = process.clone();
        let pending_clone = pending.clone();
        tokio::spawn(async move {
            loop {
                let line = {
                    let mut proc = process_clone.lock().await;
                    match proc.recv().await {
                        Ok(line) => line,
                        Err(_) => break,
                    }
                };

                if let Ok(response) = serde_json::from_str::<AnalysisResponse>(&line) {
                    let mut pending = pending_clone.lock().await;
                    if let Some(tx) = pending.remove(&response.id) {
                        let _ = tx.send(response);
                    }
                }
            }
        });

        Self { process, pending }
    }

    /// Send a query and wait for the response.
    pub async fn query(&self, query: AnalysisQuery) -> Result<AnalysisResponse, ClientError> {
        let rx = self.query_fire(query).await?;
        let id_for_err = "unknown".to_string();
        rx.await.map_err(|_| ClientError::Timeout(id_for_err))
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
            pending.insert(id, tx);
        }

        {
            let proc = self.process.lock().await;
            proc.send(&json).await?;
        }

        Ok(rx)
    }
}
