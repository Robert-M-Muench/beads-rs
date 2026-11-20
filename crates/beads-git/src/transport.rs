//! Network bounds for libgit2 fetch and push.

/// Set libgit2's process-global connect and read/write timeouts.
///
/// Without a read timeout, a half-open TCP connection to the remote blocks a
/// fetch or push in `recv` forever (beads-rs-k4fh), and the caller's sync
/// never finishes. The options apply to every libgit2 transport in the
/// process, so set them once before any git thread starts.
///
/// # Errors
/// Returns the libgit2 error if an option cannot be set.
#[allow(unsafe_code)]
pub fn set_transport_timeouts(connect_ms: i32, read_ms: i32) -> Result<(), git2::Error> {
    // SAFETY: both options are process-global integers read by transports at
    // connect and read time; callers set them before starting git threads.
    unsafe {
        git2::opts::set_server_connect_timeout_in_milliseconds(connect_ms)?;
        git2::opts::set_server_timeout_in_milliseconds(read_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::set_transport_timeouts;
    use std::io::Read;
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    #[test]
    fn fetch_from_a_silent_server_times_out() {
        // A server that accepts the connection and never answers: the
        // half-open remote of beads-rs-k4fh.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 4096];
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let _ = stream.read(&mut buf);
            std::thread::sleep(Duration::from_secs(5));
        });

        set_transport_timeouts(1_000, 1_000).expect("set timeouts");
        let dir = std::env::temp_dir().join(format!("beads-k4fh-{}", std::process::id()));
        let repo = git2::Repository::init(&dir).expect("init repo");
        let mut remote = repo
            .remote_anonymous(&format!("http://127.0.0.1:{port}/store.git"))
            .expect("remote");
        let started = Instant::now();
        let result = remote.fetch(&["refs/heads/*:refs/remotes/silent/*"], None, None);
        let elapsed = started.elapsed();
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_err(), "fetch from a silent server must fail");
        assert!(
            elapsed < Duration::from_secs(4),
            "fetch must give up at the read timeout, took {elapsed:?}"
        );
        server.join().expect("server thread");
    }
}
