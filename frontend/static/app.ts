// ============================================
// Kvittis Frontend - TypeScript
// ============================================

// Types matching Rust backend
type UserId = string;
type GroupId = string;
type ExpenseId = string;

interface User {
  id: UserId;
  username: string;
  friends: UserId[];
  email: string;
  created_at: string;
  updated_at: string;
}

interface NewUser {
  username: string;
  email: string;
  password: string;
}

interface Group {
  id: GroupId;
  name: string;
  owner_id: UserId;
  members: UserId[];
}

interface Expense {
  id: ExpenseId;
  payer: UserId;
  participants: UserId[];
  amount: number; // in cents
  description: string | null;
  group_id: GroupId | null;
  timestamp_ms: number;
}

interface BalanceEntry {
  other: UserId;
  amount: number; // in cents, positive = they owe me
}

interface GroupBalance {
  from: UserId;
  to: UserId;
  amount: number; // in cents
}

interface AppState {
  token: string | null;
  currentUser: User | null;
  allUsers: Map<UserId, string>;
  groups: Group[];
  isLoading: boolean;
  balances?: BalanceEntry[];
}

const state: AppState = {
  token: null,
  currentUser: null,
  allUsers: new Map(),
  groups: [],
  isLoading: false,
};

// ============================================
// Utilities
// ============================================

function centsToDollars(cents: number): string {
  return (cents / 100).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

function formatDate(timestampMs: number): string {
  return new Date(timestampMs).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function getAuthHeaders(): HeadersInit {
  return {
    "Content-Type": "application/json",
    Authorization: state.token ? `Bearer ${state.token}` : "",
  };
}

function saveToken(token: string): void {
  state.token = token;
  localStorage.setItem("kvittis_token", token);
}

function clearToken(): void {
  state.token = null;
  localStorage.removeItem("kvittis_token");
}

function loadToken(): string | null {
  const token = localStorage.getItem("kvittis_token");
  if (token) state.token = token;
  return token;
}

// Check if API response is an error (Rust returns {message} for errors)
function hasMessage(resp: any): resp is { message: string } {
  return resp && typeof resp === "object" && "message" in resp;
}

// ============================================
// DOM Helpers
// ============================================

function hide(el: string): void {
  document.getElementById(el)?.classList.add("hidden");
}

function show(el: string): void {
  document.getElementById(el)?.classList.remove("hidden");
}

function text(el: string, t: string): void {
  const e = document.getElementById(el);
  if (e) e.textContent = t;
}

function html(el: string, h: string): void {
  const e = document.getElementById(el);
  if (e) e.innerHTML = h;
}

function setError(el: string, msg: string): void {
  const e = document.getElementById(el);
  if (e) {
    e.textContent = msg;
    e.style.display = msg ? "block" : "none";
  }
}

function setSuccess(el: string, msg: string): void {
  const e = document.getElementById(el);
  if (e) {
    e.textContent = msg;
    e.style.display = msg ? "block" : "none";
  }
}

function setLoading(show: boolean): void {
  const el = document.getElementById("loading-overlay");
  if (show) {
    el?.classList.remove("hidden");
  } else {
    el?.classList.add("hidden");
  }
  state.isLoading = show;
}

// ============================================
// API Functions
// ============================================

async function apiPost<T>(
  endpoint: string,
  data: any,
): Promise<T | { message: string }> {
  setLoading(true);
  try {
    const resp = await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    return await resp.json();
  } catch (e) {
    return { message: "Network error" };
  } finally {
    setLoading(false);
  }
}

async function apiAuthPost<T>(
  endpoint: string,
  data: any,
): Promise<T | { message: string }> {
  setLoading(true);
  try {
    const resp = await fetch(endpoint, {
      method: "POST",
      headers: getAuthHeaders(),
      body: JSON.stringify(data),
    });
    return await resp.json();
  } catch (e) {
    return { message: "Network error" };
  } finally {
    setLoading(false);
  }
}

// ============================================
// Auth API
// ============================================

async function register(
  username: string,
  email: string,
  password: string,
): Promise<{ user: User } | { message: string }> {
  return apiPost<{ user: User }>("/api/unauth_user", {
    action: "register",
    user: { username, email, password },
  });
}

async function login(
  username: string,
  password: string,
): Promise<{ user: User; token: string } | { message: string }> {
  return apiPost<{ user: User; token: string }>("/api/unauth_user", {
    action: "login",
    username,
    password,
  });
}

// ============================================
// User API
// ============================================

async function listUsers(): Promise<{ user: User }[] | { message: string }> {
  return apiAuthPost<{ user: User }[]>("/api/auth_user", { action: "list" });
}

// ============================================
// Expense API
// ============================================

async function createExpense(params: {
  payer: UserId;
  participants: UserId[];
  amount: number;
  description: string | null;
  group_id: GroupId | null;
}): Promise<Expense | { message: string }> {
  return apiAuthPost<Expense>("/api/expense", { action: "create", ...params });
}

async function listExpensesForUser(
  userId: UserId,
): Promise<Expense[] | { message: string }> {
  return apiAuthPost<Expense[]>("/api/expense", {
    action: "list_for_user",
    user_id: userId,
  });
}

// ============================================
// Balance API
// ============================================

async function getUserBalances(
  userId: UserId,
): Promise<BalanceEntry[] | { message: string }> {
  return apiAuthPost<BalanceEntry[]>("/api/balance", {
    action: "user",
    user_id: userId,
  });
}

// ============================================
// Group API
// ============================================

async function listGroups(): Promise<Group[] | { message: string }> {
  return apiAuthPost<Group[]>("/api/group", { action: "search", query: "" });
}

// ============================================
// Rendering
// ============================================

function getUsername(id: UserId): string {
  if (state.currentUser?.id === id) return "You";
  return state.allUsers.get(id) || id;
}

async function updateNetBalance(): Promise<void> {
  if (!state.currentUser) return;

  const result = await getUserBalances(state.currentUser.id);
  if (hasMessage(result)) {
    console.error(
      "Failed to get balances:",
      (result as { message: string }).message,
    );
    return;
  }

  state.balances = result as BalanceEntry[];
  const netAmount = state.balances.reduce((sum, e) => sum + e.amount, 0);

  const amountEl = document.querySelector("#net-balance .balance-amount");
  const labelEl = document.querySelector("#net-balance .balance-label");

  if (amountEl && labelEl) {
    const dollars = centsToDollars(netAmount);
    amountEl.textContent = `$${dollars}`;
    amountEl.className =
      "balance-amount " +
      (netAmount > 0 ? "positive" : netAmount < 0 ? "negative" : "neutral");

    if (netAmount > 0) {
      labelEl.textContent =
        "You are owed a total of " + centsToDollars(netAmount);
    } else if (netAmount < 0) {
      labelEl.textContent = "You owe a total of " + centsToDollars(-netAmount);
    } else {
      labelEl.textContent = "All settled up!";
    }
  }
}

async function renderBalances(): Promise<void> {
  if (!state.currentUser) return;

  const result = await getUserBalances(state.currentUser.id);
  if (hasMessage(result)) {
    console.error(
      "Failed to get balances:",
      (result as { message: string }).message,
    );
    return;
  }

  state.balances = result as BalanceEntry[];
  await updateNetBalance();

  const tbody = document.getElementById("balances-list");
  if (!tbody) return;

  if (state.balances.length === 0) {
    tbody.innerHTML =
      '<tr><td colspan="3" class="empty-state">No balances yet. Add an expense to get started!</td></tr>';
    return;
  }

  tbody.innerHTML = state.balances
    .map((entry) => {
      const userId = entry.other;
      const username = getUsername(userId);
      const dollars = centsToDollars(entry.amount);
      const amountClass =
        entry.amount > 0 ? "amount-positive" : "amount-negative";
      const sign = entry.amount > 0 ? "+" : "";
      const status = entry.amount > 0 ? "Owes you" : "You owe";

      return `<tr>
            <td>${username}</td>
            <td class="${amountClass}">${sign}$${dollars}</td>
            <td>${status}</td>
        </tr>`;
    })
    .join("");
}

async function renderExpenses(): Promise<void> {
  if (!state.currentUser) return;

  const result = await listExpensesForUser(state.currentUser.id);
  if (hasMessage(result)) {
    console.error(
      "Failed to get expenses:",
      (result as { message: string }).message,
    );
    return;
  }

  const expenses = (result as Expense[])
    .sort((a, b) => b.timestamp_ms - a.timestamp_ms)
    .slice(0, 10);
  const tbody = document.getElementById("expenses-list");
  if (!tbody) return;

  if (expenses.length === 0) {
    tbody.innerHTML =
      '<tr><td colspan="5" class="empty-state">No expenses yet.</td></tr>';
    return;
  }

  tbody.innerHTML = expenses
    .map((e) => {
      const payer = getUsername(e.payer);
      const participants = e.participants
        .map((id) => getUsername(id))
        .join(", ");
      const desc = e.description || "No description";
      return `<tr>
            <td>${desc}</td>
            <td>$${centsToDollars(e.amount)}</td>
            <td>${payer}</td>
            <td>${participants}</td>
            <td>${formatDate(e.timestamp_ms)}</td>
        </tr>`;
    })
    .join("");
}

function populatePayerSelect(): void {
  const select = document.getElementById(
    "expense-payer",
  ) as HTMLSelectElement | null;
  if (!select) return;

  const options = Array.from(state.allUsers.entries())
    .sort((a, b) => a[1].localeCompare(b[1]))
    .map(
      ([id, username]) =>
        `<option value="${id}">${username}${state.currentUser?.id === id ? " (You)" : ""}</option>`,
    )
    .join("");

  select.innerHTML = '<option value="">Select payer...</option>' + options;
  if (state.currentUser) select.value = state.currentUser.id;
}

async function loadUsers(): Promise<void> {
  const result = await listUsers();
  if (hasMessage(result)) {
    console.error(
      "Failed to load users:",
      (result as { message: string }).message,
    );
    return;
  }

  state.allUsers.clear();
  for (const item of result as { user: User }[]) {
    state.allUsers.set(item.user.id, item.user.username);
  }
  populatePayerSelect();
}

async function renderDashboard(): Promise<void> {
  if (!state.currentUser) return;

  text("user-username", state.currentUser.username);
  text("user-email", state.currentUser.email);

  await Promise.all([renderBalances(), renderExpenses(), loadUsers()]);
}

// ============================================
// Event Handlers
// ============================================

async function handleLogin(e: Event): Promise<void> {
  e.preventDefault();
  setError("login-error", "");

  const userEl = document.getElementById(
    "login-username",
  ) as HTMLInputElement | null;
  const passEl = document.getElementById(
    "login-password",
  ) as HTMLInputElement | null;
  const username = userEl?.value || "";
  const password = passEl?.value || "";

  if (!username || !password) {
    setError("login-error", "Please enter both username and password");
    return;
  }

  const result = await login(username, password);
  if (hasMessage(result)) {
    setError("login-error", (result as { message: string }).message);
    return;
  }

  const data = result as { user: User; token: string };
  saveToken(data.token);
  state.currentUser = data.user;

  await loadUsers();

  hide("auth-section");
  show("dashboard-section");
  await renderDashboard();

  html(
    "auth-status",
    `<span>Logged in as: ${state.currentUser.username}</span>`,
  );

  setSuccess("login-error", "Logged in successfully!");
  setTimeout(() => setSuccess("login-error", ""), 3000);
}

async function handleRegister(e: Event): Promise<void> {
  e.preventDefault();
  setError("register-error", "");
  setSuccess("register-error", "");

  const userEl = document.getElementById(
    "register-username",
  ) as HTMLInputElement | null;
  const emailEl = document.getElementById(
    "register-email",
  ) as HTMLInputElement | null;
  const passEl = document.getElementById(
    "register-password",
  ) as HTMLInputElement | null;
  const username = userEl?.value || "";
  const email = emailEl?.value || "";
  const password = passEl?.value || "";

  if (!username || !email || !password) {
    setError("register-error", "Please fill in all fields");
    return;
  }

  const result = await register(username, email, password);
  if (hasMessage(result)) {
    setError("register-error", (result as { message: string }).message);
    return;
  }

  setSuccess("register-error", "Registration successful! You can now login.");
  if (userEl) userEl.value = "";
  if (emailEl) emailEl.value = "";
  if (passEl) passEl.value = "";
}

async function handleLogout(): Promise<void> {
  clearToken();
  state.currentUser = null;
  state.allUsers.clear();

  hide("dashboard-section");
  show("auth-section");

  html("auth-status", "");
  (document.getElementById("expense-form") as HTMLFormElement)?.reset();
  setError("login-error", "");
}

async function handleExpenseSubmit(e: Event): Promise<void> {
  e.preventDefault();
  setError("expense-error", "");
  setSuccess("expense-success", "");

  const amountEl = document.getElementById(
    "expense-amount",
  ) as HTMLInputElement | null;
  const payerEl = document.getElementById(
    "expense-payer",
  ) as HTMLSelectElement | null;
  const descEl = document.getElementById(
    "expense-description",
  ) as HTMLInputElement | null;
  const participantsEl = document.getElementById(
    "expense-participants",
  ) as HTMLInputElement | null;

  const amountCents = Math.round(Number(amountEl?.value || 0) * 100);
  const payerId = payerEl?.value || "";
  const description = descEl?.value || null;

  const participantNames = (participantsEl?.value || "")
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const participants: UserId[] = [];
  for (const name of participantNames) {
    let userId: UserId | null = null;
    if (
      state.currentUser?.username === name ||
      state.currentUser?.username.toLowerCase() === name.toLowerCase()
    ) {
      userId = state.currentUser.id;
    } else {
      for (const [id, uname] of state.allUsers) {
        if (uname === name || uname.toLowerCase() === name.toLowerCase()) {
          userId = id;
          break;
        }
      }
    }
    if (userId) {
      participants.push(userId);
    } else {
      setError("expense-error", `User "${name}" not found`);
      return;
    }
  }

  if (!payerId || participants.length === 0) {
    setError("expense-error", "Please select a payer and add participants");
    return;
  }

  if (!participants.includes(payerId)) {
    participants.push(payerId);
  }

  const result = await createExpense({
    payer: payerId,
    participants,
    amount: amountCents,
    description,
    group_id: null,
  });

  if (hasMessage(result)) {
    setError("expense-error", (result as { message: string }).message);
    return;
  }

  setSuccess("expense-success", "Expense added successfully!");
  (e.target as HTMLFormElement).reset();

  await renderBalances();
  await renderExpenses();

  setTimeout(() => setSuccess("expense-success", ""), 3000);
}

// ============================================
// Initialization
// ============================================

function init(): void {
  console.log("Initializing Kvittis...");

  document
    .getElementById("login-form")
    ?.addEventListener("submit", handleLogin);
  document
    .getElementById("register-form")
    ?.addEventListener("submit", handleRegister);
  document
    .getElementById("expense-form")
    ?.addEventListener("submit", handleExpenseSubmit);
  document
    .getElementById("logout-btn")
    ?.addEventListener("click", handleLogout);

  // Try to restore session
  loadToken();

  const authStatus = document.getElementById("auth-status");
  if (authStatus && state.currentUser) {
    authStatus.innerHTML = `<span>Logged in as: ${state.currentUser.username}</span>`;
  }

  console.log("Kvittis initialized");
}

// Start
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
