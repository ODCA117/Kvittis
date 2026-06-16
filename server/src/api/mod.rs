// API module for Kvittis server
// Split into domain-specific submodules

pub mod auth;
pub mod balance;
pub mod expenses;
pub mod groups;
pub mod users;

// Re-export types and handlers from submodules

// From auth
pub use auth::Claims;

// From users
pub use users::{authorized_user_handler, unauthorized_user_handler};

// From groups
pub use groups::group_handler;

// From expenses
pub use expenses::expense_handler;

// From balance
pub use balance::balance_handler;
