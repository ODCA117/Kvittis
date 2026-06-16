// API module for Kvittis
// Contains request/response types for all API endpoints

pub mod auth;
pub mod balance;
pub mod expenses;
pub mod friends;
pub mod groups;
pub mod types;
pub mod users;

// Re-export all types from submodules

// Auth types
pub use auth::{LoginResponse, RegisterResponse, TokenType, UnauthorizedUserRequest};

// User types
pub use users::{AuthorizedUserRequest, GetUserResponse, SearchUserResponse};

// Group types
pub use groups::{
    CreateGroupRequest, CreateGroupResponse, GetGroupResponse, GroupRequest, NewGroupMemberRequest,
};

// Expense types
pub use expenses::{
    CreateExpenseRequest, CreateExpenseResponse, DeleteExpenseRequest, ExpenseRequest,
    GetExpenseRequest, GetExpenseResponse,
};

// Balance types
pub use balance::{BalanceEntry, BalanceRequest, GroupBalance};

// Friend types
pub use friends::{
    FriendRequestResponse, HandleFriendRequestResponse, PendingFriendRequestResponse,
};

// Shared types
pub use types::ApiResponse;
