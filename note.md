Here's a short rundown for these solutions. All of them utilize `tokio` crate functionality.
1. `join_handle_lock.rs`
This solution creates queue of tokio tasks for each repository, then goes through this queue and awaits on each of them until the task is completed.
If task is successful, abort the remaining tasks and return the result. Otherwise, create a new task and add it to the queue, if retry count for this task is not depleted.

The main issue with this implementation is that we lock on the one task, when in the meantime one of the remaining tasks could already be completed.
Also we will not yield thread where we go through the queue, until we get successful result or all of the tasks fail.

2. `join_handle_iter.rs`
This solution also creates queue of tokio tasks for each repository and iterates through it, but it doesn't await on each task to finish.
Instead, it checks current status of the task, if the task is not finished - just go to the next task in queue.
Everything else is the same - if one of the tasks succeeded, abort remaining tasks and return, otherwise add new task to the queue.

This implementation resolves issue with locking on single task, but still doesn't yield the iterating thread.

3. `tokio_channels.rs`
This solution creates a tokio task for each repository and provides it with `mpsc` channel sender.
Then awaits on the channel receiver for the results. If result is successful, abort remaining tasks and return, otherwise create a new task.

This implementation resolves both issues - we don't lock on one of the tasks, and we also yield current thread if none of the tasks are finished.
