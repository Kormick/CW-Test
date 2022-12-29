use concurrency_exercise_01::{
    join_handle_iter::JoinHandleIter, join_handle_lock::JoinHandleLock, solution::Solution,
    statement::ServerName, std_futures::StdFutures, tokio_channels::TokioChannels,
};

#[tokio::main]
async fn main() {
    let repositories = vec![
        ServerName("One".to_string()),
        ServerName("Two".to_string()),
        ServerName("Three".to_string()),
        ServerName("Four".to_string()),
        ServerName("Five".to_string()),
        ServerName("Six".to_string()),
        ServerName("Seven".to_string()),
        ServerName("Eight".to_string()),
        ServerName("Nine".to_string()),
        ServerName("Ten".to_string()),
    ];

    match JoinHandleLock::solve(repositories.clone()).await {
        None => println!("[JoinHandleLock] All downloads failed!"),
        Some(binary) => println!("[JoinHandleLock] {binary} downloaded"),
    }

    match JoinHandleIter::solve(repositories.clone()).await {
        None => println!("[JoinHandleIter] All downloads failed!"),
        Some(binary) => println!("[JoinHandleIter] {binary} downloaded"),
    }

    match TokioChannels::solve(repositories.clone()).await {
        None => println!("[TokioChannels] All downloads failed!"),
        Some(binary) => println!("[TokioChannels] {binary} downloaded"),
    }

    match StdFutures::solve(repositories.clone()).await {
        None => println!("[StdFutures] All downloads failed!"),
        Some(binary) => println!("[StdFutures] {binary} downloaded"),
    }
}
