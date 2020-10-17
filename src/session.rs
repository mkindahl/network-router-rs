//! Module for handlign sessions.

trait Session {
    /// Run the session.
    ///
    /// Start executing the session. This will unpack the session and
    /// run it to completion.
    async fn run(self) -> Result<(), io::Error>;
}
