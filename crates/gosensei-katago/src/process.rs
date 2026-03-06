use std::path::PathBuf;

use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("failed to spawn KataGo: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("KataGo binary not found at {0}")]
    BinaryNotFound(PathBuf),

    #[error("KataGo process exited unexpectedly")]
    UnexpectedExit,

    #[error("failed to communicate with KataGo: {0}")]
    Communication(String),
}

pub struct KataGoProcess {
    child: Child,
    stdin_tx: mpsc::Sender<String>,
    stdout_rx: mpsc::Receiver<String>,
}

impl KataGoProcess {
    /// Spawn KataGo with the Analysis Engine protocol.
    pub async fn spawn(
        binary_path: PathBuf,
        model_path: PathBuf,
        config_path: Option<PathBuf>,
    ) -> Result<Self, ProcessError> {
        if !binary_path.exists() {
            return Err(ProcessError::BinaryNotFound(binary_path));
        }

        let mut cmd = Command::new(&binary_path);
        cmd.arg("analysis")
            .arg("-model")
            .arg(&model_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(config) = config_path {
            cmd.arg("-config").arg(config);
        }

        let mut child = cmd.spawn()?;

        let stdin = child.stdin.take().expect("stdin should be piped");
        let stdout = child.stdout.take().expect("stdout should be piped");
        let stderr = child.stderr.take().expect("stderr should be piped");

        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(32);
        let (stdout_tx, stdout_rx) = mpsc::channel::<String>(32);

        // Stdin writer task
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(line) = stdin_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                let _ = stdin.flush().await;
            }
        });

        // Stdout reader task
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stdout_tx.send(line).await.is_err() {
                    break;
                }
            }
        });

        // Stderr drain task — log to file for debugging and prevent pipe buffer deadlock
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt as _;
            let log_path = std::env::temp_dir().join("katago-stderr.log");
            let mut log_file = tokio::fs::File::create(&log_path).await.ok();
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                debug!(target: "katago::stderr", "{}", line);
                if let Some(ref mut f) = log_file {
                    let _ = f.write_all(format!("{line}\n").as_bytes()).await;
                }
            }
        });

        Ok(Self {
            child,
            stdin_tx,
            stdout_rx,
        })
    }

    /// Clone the stdin sender for use by the client.
    pub fn sender(&self) -> mpsc::Sender<String> {
        self.stdin_tx.clone()
    }

    /// Send a query line to KataGo.
    pub async fn send(&self, query: &str) -> Result<(), ProcessError> {
        self.stdin_tx
            .send(query.to_string())
            .await
            .map_err(|e| ProcessError::Communication(e.to_string()))
    }

    /// Receive the next response line from KataGo.
    pub async fn recv(&mut self) -> Result<String, ProcessError> {
        self.stdout_rx
            .recv()
            .await
            .ok_or(ProcessError::UnexpectedExit)
    }

    /// Check if the process is still running.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Kill the process.
    pub async fn kill(&mut self) -> Result<(), ProcessError> {
        self.child.kill().await.map_err(ProcessError::SpawnFailed)
    }
}
