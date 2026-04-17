<script>
    import { token, currentUser, isLoading, clearToken, saveToken, allUsers, balances, expenses, groups } from './lib/stores.js';
    import { login, listUsers, listExpensesForUser, getUserBalances, listGroups, isError } from './lib/api.js';
    import Auth from './components/Auth.svelte';
    import Dashboard from './components/Dashboard.svelte';
    import GroupsPage from './components/Groups.svelte';
    import LoadingOverlay from './components/LoadingOverlay.svelte';
    import Header from './components/Header.svelte';
    import './app.css';

    // Navigation state
    let currentPage = 'auth';

    // Get store values
    let tokenValue;
    token.subscribe(t => tokenValue = t);

    let userValue;
    currentUser.subscribe(u => {
        userValue = u;
        // When user changes, update page
        if (u) {
            currentPage = 'dashboard';
            loadDashboardData();
        } else if (!u && tokenValue) {
            // Token but no user - clear token
            clearToken();
            currentPage = 'auth';
        }
    });

    // Handle login success
    async function onLoginSuccess(result) {
        if (result.token && result.user) {
            saveToken(result.token);
            currentUser.set(result.user);
            currentPage = 'dashboard';
            await loadDashboardData();
        }
    }

    // Handle logout
    function handleLogout() {
        clearToken();
        currentUser.set(null);
        allUsers.set(new Map());
        balances.set([]);
        expenses.set([]);
        groups.set([]);
        currentPage = 'auth';
    }

    // Load dashboard data
    async function loadDashboardData() {
        const t = tokenValue;
        const u = userValue;
        if (!t || !u) return;

        isLoading.set(true);
        try {
            // Load users
            const usersResult = await listUsers(t);
            if (!isError(usersResult)) {
                const usersMap = new Map();
                usersResult.forEach(item => {
                    usersMap.set(item.user.id, item.user.username);
                });
                allUsers.set(usersMap);
            }

            // Load balances
            const balancesResult = await getUserBalances(u.id, t);
            if (!isError(balancesResult)) {
                balances.set(balancesResult);
            }

            // Load expenses
            const expensesResult = await listExpensesForUser(u.id, t);
            if (!isError(expensesResult)) {
                expenses.set(expensesResult.sort((a, b) => b.timestamp_ms - a.timestamp_ms));
            }

            // Load groups
            const groupsResult = await listGroups(t);
            if (!isError(groupsResult)) {
                groups.set(groupsResult);
            }
        } catch (e) {
            console.error('Failed to load dashboard data:', e);
        } finally {
            isLoading.set(false);
        }
    }

    // Navigation
    function navigateTo(page) {
        currentPage = page;
    }
</script>

<svelte:head>
    <title>Kvittis - Expense Tracker</title>
</svelte:head>

<div class="app">
    <Header user={userValue} onLogout={handleLogout} />

    <main class="main-content">
        {#if currentPage === 'auth'}
            <Auth onLogin={onLoginSuccess} />
        {:else if currentPage === 'dashboard'}
            <Dashboard onLogout={handleLogout} />
        {:else if currentPage === 'groups'}
            <GroupsPage navigateTo={navigateTo} />
        {/if}
    </main>

    {#if $isLoading}
        <LoadingOverlay />
    {/if}
</div>
