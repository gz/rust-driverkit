use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub enum WaitError {
    Timeout,
}

/// Wait until a given condition is satisfied or a timeout is reached.
/// Returns true if condition is satisfied or false if the timeout was reached instead.
pub fn wait_until<F>(cond_fn: F, max_wait: Duration) -> Result<(), WaitError>
where
    F: Fn() -> bool,
{
    let now = SystemTime::now();

    while !cond_fn() {
        match now.elapsed() {
            Ok(waited) => {
                if waited > max_wait {
                    return Err(WaitError::Timeout);
                }
            }
            Err(_e) => return Err(WaitError::Timeout),
        }

        // Wait a little longer...
        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}
