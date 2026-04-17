<script>
    import { token, currentUser, allUsers, balances, expenses, isLoading } from '../lib/stores.js';
    import { createExpense, listUsers, getUserBalances, listExpensesForUser, isError, centsToDollars, formatDate } from '../lib/api.js';
    import UserInfo from './UserInfo.svelte';
    import NetBalance from './NetBalance.svelte';
    import BalancesTable from './BalancesTable.svelte';
    import ExpensesTable from './ExpensesTable.svelte';

    export let onLogout;

    // Expense form state
    let expenseAmount = '';
    let expenseDescription = '';
    let expenseParticipants = '';
    let expensePayer = '';
    let expenseGroup = '';

    let expenseError = '';
    let expenseSuccess = '';

    // Store values
    let tokenValue;
    token.subscribe(t => tokenValue = t);

    let currentUserValue;
    currentUser.subscribe(u => currentUserValue = u);

    let allUsersValue;
    allUsers.subscribe(u => allUsersValue = u);

    // Populate payer select options
    $: payerOptions = Array.from(allUsersValue?.entries() || [])
        .sort((a, b) => a[1].localeCompare(b[1]))
        .map(([id, username]) => ({
            value: id,
            label: `${username}${id === currentUserValue?.id ? ' (You)' : ''}`
        }));

    // Helper to get username
    function getUsername(id) {
        if (currentUserValue?.id === id) return 'You';
        return allUsersValue?.get(id) || id;
    }

    async function handleExpenseSubmit(e) {
        e.preventDefault();
        expenseError = '';
        expenseSuccess = '';

        const amountCents = Math.round(Number(expenseAmount || 0) * 100);
        const payerId = expensePayer || '';
        const description = expenseDescription || null;

        const participantNames = (expenseParticipants || '')
            .split(',')
            .map(s => s.trim())
            .filter(s => s.length > 0);

        const participants = [];

        for (const name of participantNames) {
            let userId = null;
            if (currentUserValue?.username === name || currentUserValue?.username.toLowerCase() === name.toLowerCase()) {
                userId = currentUserValue.id;
            } else {
                for (const [id, uname] of allUsersValue || []) {
                    if (uname === name || uname.toLowerCase() === name.toLowerCase()) {
                        userId = id;
                        break;
                    }
                }
            }
            if (userId) {
                participants.push(userId);
            } else {
                expenseError = `User "${name}" not found`;
                return;
            }
        }

        if (!payerId || participants.length === 0) {
            expenseError = 'Please select a payer and add participants';
            return;
        }

        if (!participants.includes(payerId)) {
            participants.push(payerId);
        }

        isLoading.set(true);
        try {
            const result = await createExpense({
                payer: payerId,
                participants,
                amount: amountCents,
                description,
                group_id: null
            }, tokenValue);

            if (isError(result)) {
                expenseError = result.message;
            } else {
                expenseAmount = '';
                expenseDescription = '';
                expenseParticipants = '';
                expenseSuccess = 'Expense added successfully!';

                // Reload data
                const balancesResult = await getUserBalances(currentUserValue.id, tokenValue);
                if (!isError(balancesResult)) {
                    balances.set(balancesResult);
                }

                const expensesResult = await listExpensesForUser(currentUserValue.id, tokenValue);
                if (!isError(expensesResult)) {
                    expenses.set(expensesResult.sort((a, b) => b.timestamp_ms - a.timestamp_ms));
                }

                setTimeout(() => expenseSuccess = '', 3000);
            }
        } catch (e) {
            expenseError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }
</script>

<div class="dashboard">
    <div class="dashboard-layout">
        <UserInfo user={currentUserValue} onLogout={onLogout} />
        <NetBalance balance={$balances} />
    </div>

    <BalancesTable balances={$balances} getUsername={getUsername} />

    <div class="card">
        <h3>Add Expense</h3>
        <form on:submit={handleExpenseSubmit} class="form">
            <div class="form-row">
                <div class="form-group">
                    <label for="expense-amount">Amount ($)</label>
                    <input
                        type="number"
                        id="expense-amount"
                        bind:value={expenseAmount}
                        step="0.01"
                        required
                    >
                </div>
                <div class="form-group">
                    <label for="expense-payer">Payer</label>
                    <select id="expense-payer" bind:value={expensePayer} required>
                        <option value="">Select payer...</option>
                        {#each payerOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                </div>
            </div>
            <div class="form-group">
                <label for="expense-description">Description</label>
                <input
                    type="text"
                    id="expense-description"
                    bind:value={expenseDescription}
                    placeholder="Pizza night"
                >
            </div>
            <div class="form-group">
                <label for="expense-participants">Participants (comma separated usernames)</label>
                <input
                    type="text"
                    id="expense-participants"
                    bind:value={expenseParticipants}
                    placeholder="alice,bob,charlie"
                    required
                >
            </div>
            <button type="submit" class="btn btn-primary">Add Expense</button>
        </form>
        {#if expenseError}
            <p class="error-message">{expenseError}</p>
        {/if}
        {#if expenseSuccess}
            <p class="success-message">{expenseSuccess}</p>
        {/if}
    </div>

    <ExpensesTable expenses={$expenses} getUsername={getUsername} />
</div>
