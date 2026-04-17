<script>
    import { centsToDollars } from '../lib/api.js';

    export let balances;
    export let getUsername;

    $: isEmpty = balances.length === 0;
</script>

<div class="card">
    <h3>Your Balances</h3>
    <p class="subtitle">Who owes you and who you owe</p>
    <table class="data-table">
        <thead>
            <tr>
                <th>Friend</th>
                <th>Amount</th>
                <th>Status</th>
            </tr>
        </thead>
        <tbody>
            {#if isEmpty}
                <tr>
                    <td colspan="3" class="empty-state">No balances yet. Add an expense to get started!</td>
                </tr>
            {:else}
                {#each balances as entry}
                    <tr>
                        <td>{getUsername(entry.other)}</td>
                        <td class="{entry.amount > 0 ? 'amount-positive' : 'amount-negative'}">
                            {entry.amount > 0 ? '+' : ''}${centsToDollars(entry.amount)}
                        </td>
                        <td>{entry.amount > 0 ? 'Owes you' : 'You owe'}</td>
                    </tr>
                {/each}
            {/if}
        </tbody>
    </table>
</div>
