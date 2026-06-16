// API client for Kvittis
// Matches the Rust server API endpoints

const API_BASE = ""; // Proxy handles /api in dev, same origin in prod

// Helper to check if response has error message
function hasError(resp) {
  return resp && typeof resp === "object" && "message" in resp;
}

// Helper to get auth headers
function getAuthHeaders(token) {
  return {
    "Content-Type": "application/json",
    ...(token && { Authorization: `Bearer ${token}` }),
  };
}

// Generic POST request
async function apiPost(endpoint, data, token = null) {
  try {
    const resp = await fetch(endpoint, {
      method: "POST",
      headers: getAuthHeaders(token),
      body: JSON.stringify(data),
    });
    return await resp.json();
  } catch (e) {
    return { message: "Network error" };
  }
}

// Auth API - no auth header needed
export async function register(username, email, password) {
  return apiPost("/api/unauth_user", {
    action: "register",
    user: { username, email, password },
  });
}

export async function login(username, password) {
  return apiPost("/api/unauth_user", {
    action: "login",
    username,
    password,
  });
}

// User API - requires auth
export async function listUsers(token) {
  return apiPost("/api/auth_user", { action: "search", query: "" }, token);
}

// Expense API
export async function createExpense(params, token) {
  return apiPost("/api/expense", { action: "create", ...params }, token);
}

export async function listExpensesForUser(userId, token) {
  return apiPost(
    "/api/expense",
    {
      action: "list_for_user",
      user_id: userId,
    },
    token,
  );
}

// Balance API
export async function getUserBalances(userId, token) {
  return apiPost(
    "/api/balance",
    {
      action: "user",
      user_id: userId,
    },
    token,
  );
}

// Group API
export async function listGroups(token) {
  return apiPost("/api/group", { action: "search", query: "" }, token);
}

// Friend Request API
export async function sendFriendRequest(friendId, token) {
  return apiPost("/api/auth_user", { action: "send_friend_request", friend_id: friendId }, token);
}

export async function getPendingFriendRequests(token) {
  return apiPost("/api/auth_user", { action: "get_pending_friend_requests" }, token);
}

export async function handleFriendRequest(requestId, action, token) {
  return apiPost("/api/auth_user", { action: "handle_friend_request", request_id: requestId, request_action: action }, token);
}

// Utility functions
// Check if API response is an error
export function isError(resp) {
  return hasError(resp);
}

// Format helpers
export function centsToDollars(cents) {
  return (cents / 100).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export function formatDate(timestampMs) {
  return new Date(timestampMs).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
