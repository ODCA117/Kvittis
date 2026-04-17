<script>
    export let user;
    export let onLogout;
</script>

<header class="header">
    <h1>Kvittis</h1>
    <div class="auth-status">
        {#if user}
            <span>Logged in as: {user.username}</span>
            <button on:click={onLogout} class="btn btn-danger">Logout</button>
        {/if}
    </div>
</header>
