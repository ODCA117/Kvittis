pub mod kvittis_client;

pub use kvittis_client::{AuthenticatedKvittisClient, KvittisClient};

// Convenience type aliases
pub type UnauthClient = KvittisClient;
pub type AuthClient = AuthenticatedKvittisClient;
