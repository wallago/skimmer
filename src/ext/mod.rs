//! Everything which talks to over the network.

/// Claude API.
mod claude;

/// Rss server.
mod rss;

/// The service clients and the types they hand back.
pub(crate) mod prelude {
    pub(crate) use super::claude::{Claude, output::Analysis};
    pub(crate) use super::rss::Rss;
}
