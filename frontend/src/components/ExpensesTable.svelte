<script>
    import { centsToDollars, formatDate } from '../lib/api.js';

    export let expenses;
    export let getUsername;

    $: isEmpty = expenses.length === 0;
    $: recentExpenses = expenses.slice(0, 10);
</script>

<div class="card">
    <h3>Recent Expenses</h3>
    <table class="data-table">
        <thead>
            <tr>
                <th>Description</th>
                <th>Amount</th>
                <th>Payer</th>
                <th>Participants</th>
                <th>Date</th>
            </tr>
        </thead>
        <tbody>
            {#if isEmpty}
                <tr>
                    <td colspan="5" class="empty-state">No expenses yet.</td>
                </tr>
            {:else}
                {#each recentExpenses as expense}
                    <tr>
                        <td>{expense.description || 'No description'}</td>
                        <td>${centsToDollars(expense.amount)}</td>
                        <td>{getUsername(expense.payer)}</td>
                        <td>{expense.participants.map(id => getUsername(id)).join(', ')}</td>
                        <td>{formatDate(expense.created_at || expense.timestamp_ms)}</td>
                    </tr>
                {/each}
            {/if}
        </tbody>
    </table>
</div>
