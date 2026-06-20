<script>
    import { formatDateShort } from '../lib/api.js';
    
    export let user;
    export let onLogout;
    
    $: friendsCount = user?.friends?.length || 0;
</script>

<div class="card user-info">
    <h3>Your Account</h3>
    <p><strong>Username:</strong> {user?.username}</p>
    <p><strong>Email:</strong> {user?.email}</p>
    <p><strong>Friends:</strong> {friendsCount}</p>
    <p><strong>Member since:</strong> {formatDateShort(user?.created_at)}</p>
    <button on:click={onLogout} class="btn btn-danger">Logout</button>
</div>
