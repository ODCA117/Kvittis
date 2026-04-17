import { writable, derived } from "svelte/store";

// Token store - persisted in localStorage
export const token = writable(null);

// Load token from localStorage - call this on app init
function loadInitialToken() {
  const stored = localStorage.getItem("kvittis_token");
  if (stored) {
    token.set(stored);
  }
  return stored;
}

// Save token to localStorage
export function saveToken(newToken) {
  token.set(newToken);
  if (newToken) {
    localStorage.setItem("kvittis_token", newToken);
  } else {
    localStorage.removeItem("kvittis_token");
  }
}

// Clear token
export function clearToken() {
  token.set(null);
  localStorage.removeItem("kvittis_token");
}

// Current user store
export const currentUser = writable(null);

// All users map: userId -> username
export const allUsers = writable(new Map());

// User's balances
export const balances = writable([]);

// User's expenses
export const expenses = writable([]);

// Groups
export const groups = writable([]);

// Loading state
export const isLoading = writable(false);

// Net balance (derived from balances)
export const netBalance = derived(balances, ($balances) => {
  return $balances.reduce((sum, entry) => sum + entry.amount, 0);
});

// Helper to get username from userId
export function getUsername(id, currentUserVal, allUsersMap) {
  if (currentUserVal?.id === id) return "You";
  return allUsersMap.get(id) || id;
}

// Initialize - call this once on app start
loadInitialToken();
