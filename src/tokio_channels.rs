use std::collections::HashMap;

use async_trait::async_trait;
use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};

use crate::{
    solution::{Solution, RETRIES},
    statement::*,
};

struct DownloadTask {
    server_name: ServerName,
    task: Option<JoinHandle<()>>,
    tx: Sender<Result<Binary, ServerError>>,
    retries_left: usize,
}

impl DownloadTask {
    /// Creates `DownloadTask` object and spawns `tokio` task for given server name with channel sender.
    fn start(server_name: ServerName, tx: Sender<Result<Binary, ServerError>>) -> Self {
        Self {
            server_name: server_name.clone(),
            tx: tx.clone(),
            task: Some(tokio::spawn(async move {
                let res = download(server_name).await;
                tx.send(res)
                    .await
                    .expect("Failed to send result to the channel");
            })),
            retries_left: RETRIES,
        }
    }

    /// Aborts task if its available.
    fn abort(&mut self) {
        if let Some(ref task) = self.task {
            task.abort();
        }
    }

    /// Attempts to create a new download task if retries are available.
    /// Returns `None` if retry count is depleted.
    fn start_retry(mut self) -> Option<Self> {
        self.abort();
        if self.retries_left > 0 {
            let tx = self.tx.clone();
            let name = self.server_name.clone();
            let task = tokio::spawn(async move {
                let res = download(name).await;
                tx.send(res)
                    .await
                    .expect("Failed to send result to the channel");
            });
            Some(Self {
                task: Some(task),
                retries_left: self.retries_left - 1,
                ..self
            })
        } else {
            None
        }
    }
}

/// Spawns a `tokio` task with channel sender for each repository.
/// Then await on the channel receiver for finished tasks results.
/// If task is successful, collect the result and break the loop,
/// otherwise spawn a new task for failed repository if retry count is not depleted.
///
/// Abort all the remaining tasks after the first successful task or if all tasks failed
/// and return the result.
///
/// Returns `Some(Binary)` if one of the tasks succeeded
/// Returns `None` if all of the tasks failed
pub struct TokioChannels;

#[async_trait]
impl Solution for TokioChannels {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        let (tx, mut rx) = mpsc::channel(32);

        let mut handlers = repositories
            .into_iter()
            .map(|name| (name.0.clone(), DownloadTask::start(name, tx.clone())))
            .collect::<HashMap<_, _>>();
        drop(tx);

        let mut result = None;
        while let Some(res) = rx.recv().await {
            match res {
                Ok(binary) => {
                    result = Some(binary);
                    break;
                }
                Err(ServerError::Disconnected(name)) => {
                    if let Some(handler) = handlers
                        .remove(&name.0)
                        .and_then(|handler| handler.start_retry())
                    {
                        handlers.insert(name.0, handler);
                    }
                }
            }
        }

        handlers.iter_mut().for_each(|(_name, h)| h.abort());

        result
    }
}
