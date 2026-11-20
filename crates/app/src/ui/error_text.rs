//! The one place that turns errors into words for the user.
//! Technical detail goes to the log, not the screen.

use dk_core::Error;

/// Short, user-facing message for an error.
#[allow(dead_code)] // first caller: the first real tool page's job status.
pub fn user_message(error: &Error) -> String {
    match error {
        Error::NotImplemented(id) => format!("{id} is coming soon."),
        Error::Cancelled => "Cancelled.".into(),
        Error::Invalid(why) => why.clone(),
        Error::Network(_) => "Couldn't reach the internet. Showing the last saved copy.".into(),
        Error::Io(e) => format!("Couldn't read or write a file: {e}"),
        Error::Db(_) => "Couldn't save app data. Your files are not affected.".into(),
        Error::Internal(_) => "Something went wrong. Details are in the log file.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_errors_hide_detail() {
        let msg = user_message(&Error::Internal("secret path /home/x".into()));
        assert!(!msg.contains("/home/x"));
    }
}
