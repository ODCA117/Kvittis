<script>
    import { centsToDollars } from '../lib/api.js';

    export let balance; // array of balance entries

    $: netAmount = balance.reduce((sum, entry) => sum + entry.amount, 0);
    $: dollars = centsToDollars(netAmount);
    $: amountClass = netAmount > 0 ? 'positive' : netAmount < 0 ? 'negative' : 'neutral';
    $: label = netAmount > 0
        ? `You are owed a total of ${centsToDollars(netAmount)}`
        : netAmount < 0
            ? `You owe a total of ${centsToDollars(-netAmount)}`
            : 'All settled up!';
</script>

<div class="card balance-card">
    <h3>Your Net Balance</h3>
    <div class="net-balance">
        <span class="balance-amount {amountClass}">${dollars}</span>
        <span class="balance-label">{label}</span>
    </div>
</div>
