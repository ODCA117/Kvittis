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
export async function getUser(token) {
  return apiPost("/api/auth_user", { action: "get" }, token);
}

export async function deleteUser(token) {
  return apiPost("/api/auth_user", { action: "delete" }, token);
}

export async function searchUsers(query, token) {
  return apiPost("/api/auth_user", { action: "search", query }, token);
}

export async function listUsers(token) {
  return searchUsers("", token);
}

export async function logout(token) {
  return apiPost("/api/auth_user", { action: "logout" }, token);
}

// Expense API
export async function createExpense(params, token) {
  return apiPost("/api/expense", { action: "create", ...params }, token);
}

export async function getExpense(expenseId, token) {
  return apiPost("/api/expense", { action: "get", id: expenseId }, token);
}

export async function deleteExpense(expenseId, token) {
  return apiPost("/api/expense", { action: "delete", id: expenseId }, token);
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

export async function listExpensesForGroup(groupId, token) {
  return apiPost(
    "/api/expense",
    {
      action: "list_for_group",
      group_id: groupId,
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

export async function getGroupBalances(groupId, token) {
  return apiPost(
    "/api/balance",
    {
      action: "group",
      group_id: groupId,
    },
    token,
  );
}

// Group API
export async function createGroup(name, token) {
  return apiPost("/api/group", { action: "create", name }, token);
}

export async function getGroup(groupId, token) {
  return apiPost("/api/group", { action: "get", group_id: groupId }, token);
}

export async function deleteGroup(groupId, token) {
  return apiPost("/api/group", { action: "delete", group_id: groupId }, token);
}

export async function searchGroups(query, token) {
  return apiPost("/api/group", { action: "search", query }, token);
}

export async function listGroups(token) {
  return searchGroups("", token);
}

export async function addGroupMember(groupId, newMember, role, token) {
  return apiPost(
    "/api/group",
    { action: "add_member", group_id: groupId, new_member: newMember, role },
    token,
  );
}

export async function updateGroupMember(groupId, member, role, token) {
  return apiPost(
    "/api/group",
    { action: "update_member", group_id: groupId, member, role },
    token,
  );
}

export async function removeGroupMember(groupId, member, token) {
  return apiPost(
    "/api/group",
    { action: "remove_member", group_id: groupId, member },
    token,
  );
}

// Friend Request API
export async function sendFriendRequest(friendId, token) {
  return apiPost(
    "/api/auth_user",
    { action: "send_friend_request", friend_id: friendId },
    token,
  );
}

export async function getPendingFriendRequests(token) {
  return apiPost(
    "/api/auth_user",
    { action: "get_pending_friend_requests" },
    token,
  );
}

export async function handleFriendRequest(requestId, action, token) {
  return apiPost(
    "/api/auth_user",
    { action: "handle_friend_request", request_id: requestId, request_action: action },
    token,
  );
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

export function formatDate(dateInput) {
  // Handle both ISO strings and timestamps
  let date;
  if (typeof dateInput === 'string') {
    // ISO 8601 format from backend
    date = new Date(dateInput);
  } else if (typeof dateInput === 'number') {
    // Legacy timestamp in ms
    date = new Date(dateInput);
  } else {
    date = new Date();
  }
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

// Format date for display without time
export function formatDateShort(dateInput) {
  let date;
  if (typeof dateInput === 'string') {
    date = new Date(dateInput);
  } else if (typeof dateInput === 'number') {
    date = new Date(dateInput);
  } else {
    date = new Date();
  }
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}
