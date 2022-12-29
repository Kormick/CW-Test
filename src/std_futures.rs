use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;

use crate::{
    solution::{Solution, RETRIES},
    statement::*,
};

struct DownloadTask {
    server_name: ServerName,
    task: Pin<Box<dyn Future<Output = Result<Binary, ServerError>> + Send>>,
    retries_left: usize,
}

impl DownloadTask {
    /// Creates `DownloadTask` object with `Future` for given server name.
    fn start(server_name: ServerName) -> Self {
        Self {
            server_name: server_name.clone(),
            task: Box::pin(download(server_name)),
            retries_left: RETRIES,
        }
    }

    /// Attempts to create a new download task if retries are available.
    /// Returns `None` if retry count is depleted.
    fn start_retry(self) -> Option<Self> {
        if self.retries_left > 0 {
            let task = Box::pin(download(self.server_name.clone()));
            Some(Self {
                task,
                retries_left: self.retries_left - 1,
                ..self
            })
        } else {
            None
        }
    }
}

/// Custom `Future` implementation that handles all of the download tasks.
/// Creates and stores a vector of tasks with separate `Future` for `download` function in each of them.
/// On each `poll()` call, polls every `Future` stored in the task one by one:
/// if it returns `Poll::Ready(Ok(_))` - collect the result and immediately return with `Poll::Ready(Binary)`,
/// if it returns `Poll::Ready(Err(_))` - restart the task if retry count is not depleted, otherwise remove task from polling list.
/// If all tasks are removed from polling list, returns `Poll::Ready(None)` and finishes execution,
/// in every other case - returns `Poll::Pending`.
struct JoinTasks {
    tasks: Vec<DownloadTask>,
}

impl JoinTasks {
    fn new(repositories: Vec<ServerName>) -> Self {
        Self {
            tasks: repositories.into_iter().map(DownloadTask::start).collect(),
        }
    }
}

impl Future for JoinTasks {
    type Output = Option<Binary>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut failed = vec![];
        for (i, task) in self.tasks.iter_mut().enumerate() {
            if let Poll::Ready(result) = Future::poll(task.task.as_mut(), cx) {
                if let Ok(result) = result {
                    return Poll::Ready(Some(result));
                } else {
                    failed.push(i);
                }
            }
        }

        failed.into_iter().rev().for_each(|i| {
            let task = self.tasks.remove(i);
            if let Some(task) = task.start_retry() {
                self.tasks.push(task);
            }
        });

        if self.tasks.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

pub struct StdFutures;

#[async_trait]
impl Solution for StdFutures {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        JoinTasks::new(repositories).await
    }
}
