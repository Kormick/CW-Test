use std::collections::VecDeque;

use async_trait::async_trait;
use tokio::task::JoinHandle;

use crate::{
    solution::{Solution, RETRIES},
    statement::*,
};

struct DownloadTask {
    server_name: ServerName,
    task: Option<JoinHandle<Result<Binary, ServerError>>>,
    retries_left: usize,
}

impl DownloadTask {
    /// Creates `DownloadTask` object and spawns `tokio` task for given server name.
    fn start(server_name: ServerName) -> Self {
        Self {
            server_name: server_name.clone(),
            task: Some(tokio::spawn(download(server_name))),
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
            let task = tokio::spawn(download(self.server_name.clone()));
            Some(Self {
                task: Some(task),
                retries_left: self.retries_left - 1,
                ..self
            })
        } else {
            None
        }
    }

    /// Waits for the task to finish and returns the result.
    /// Returns `Some(Binary)` if task is successful
    /// Returns `None` otherwise
    async fn complete(&mut self) -> Option<Binary> {
        let task = self.task.take()?;
        task.await.ok()?.ok()
    }
}

/// Spawns a `tokio` task for each repository.
/// Then iterate through each task in order and wait for it to finish.
/// If task if successful, collect the result and break the loop,
/// otherwise spawn a new task for failed repository if retry count is not depleted.
///
/// Abort all the remaining tasks after the first successful task or if all tasks failed
/// and return the result.
///
/// Returns `Some(Binary)` if one of the tasks succeeded
/// Returns `None` if all of the tasks failed
pub struct JoinHandleLock;

#[async_trait]
impl Solution for JoinHandleLock {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        let mut handlers = repositories
            .into_iter()
            .map(DownloadTask::start)
            .collect::<VecDeque<_>>();

        let mut result = None;
        while let Some(mut task) = handlers.pop_front() {
            if let Some(binary) = task.complete().await {
                result = Some(binary);
                break;
            } else if let Some(task) = task.start_retry() {
                handlers.push_back(task);
            }
        }

        handlers.iter_mut().for_each(|task| task.abort());

        result
    }
}
