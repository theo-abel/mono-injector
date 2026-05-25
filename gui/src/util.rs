/// Converts an empty string slice to `None`, preserving non-empty strings as `Some`.
pub fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

/// Formats a process as `"name (pid)"` for display in process input fields.
pub fn format_process_label(name: &str, pid: u32) -> String {
    format!("{name} ({pid})")
}

/// Formats a Unix timestamp as a human-readable relative duration like `"3m ago"`.
pub fn relative_time(unix_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let elapsed = now.saturating_sub(unix_secs);
    if elapsed < 60 {
        format!("{elapsed}s ago")
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else {
        format!("{}h ago", elapsed / 3600)
    }
}

/// Runs a blocking closure on the tokio blocking pool, flattening both the join
/// error and the closure's own `Result` into a single `Result<T, String>`.
pub async fn run_blocking<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r)
}
