<script>
    export let user;
    export let onLogout;
</script>

<div class="card user-info">
    <h3>Your Account</h3>
    <p>{user?.username}</p>
    <p>{user?.email}</p>
    <button on:click={onLogout} class="btn btn-danger">Logout</button>
</div>
