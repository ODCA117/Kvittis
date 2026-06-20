<script>
    import { onMount } from 'svelte';
    import { token, currentUser, allUsers, incomingFriendRequests, outgoingFriendRequests, isLoading } from '../lib/stores.js';
    import { sendFriendRequest, getPendingFriendRequests, handleFriendRequest, listUsers, isError } from '../lib/api.js';

    export let navigateTo;

    // Local state
    let searchQuery = '';
    let searchResults = [];
    let sendRequestError = '';
    let sendRequestSuccess = '';
    let handleRequestError = '';
    let handleRequestSuccess = '';

    // Load friend requests on mount or when token changes
    async function loadFriendRequests() {
        if (!$token) return;
        
        isLoading.set(true);
        try {
            const result = await getPendingFriendRequests($token);
            if (!isError(result)) {
                incomingFriendRequests.set(result.incoming || []);
                outgoingFriendRequests.set(result.outgoing || []);
            }
        } catch (e) {
            console.error("Failed to load friend requests:", e);
        } finally {
            isLoading.set(false);
        }
    }

    // Search for users
    async function handleSearch() {
        if (!searchQuery.trim() || !$token) return;
        
        isLoading.set(true);
        try {
            const result = await searchUsers(searchQuery, $token);
            if (!isError(result)) {
                // Handle both SearchUserResponse (user array) and direct array
                const userList = result.user || result;
                searchResults = userList.filter(u => 
                    (u.username.toLowerCase().includes(searchQuery.toLowerCase()) ||
                     u.id.toString().toLowerCase().includes(searchQuery.toLowerCase())) &&
                    u.id !== $currentUser?.id
                );
            }
        } catch (e) {
            console.error("Failed to search users:", e);
        } finally {
            isLoading.set(false);
        }
    }

    // Send friend request
    async function sendRequest(userId) {
        if (!$token) return;
        
        sendRequestError = '';
        sendRequestSuccess = '';
        
        isLoading.set(true);
        try {
            const result = await sendFriendRequest(userId, $token);
            if (isError(result)) {
                sendRequestError = result.message;
            } else {
                sendRequestSuccess = 'Friend request sent!';
                // Reload friend requests
                await loadFriendRequests();
                // Clear search
                searchQuery = '';
                searchResults = [];
            }
        } catch (e) {
            sendRequestError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }

    // Handle friend request (accept, reject, cancel)
    async function handleRequest(requestId, action) {
        if (!$token) return;
        
        handleRequestError = '';
        handleRequestSuccess = '';
        
        isLoading.set(true);
        try {
            const result = await handleFriendRequest(requestId, action, $token);
            if (isError(result)) {
                handleRequestError = result.message;
            } else {
                handleRequestSuccess = `Friend request ${action}ed!`;
                // Reload friend requests
                await loadFriendRequests();
            }
        } catch (e) {
            handleRequestError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }

    // Helper to get username
    function getUsername(userId) {
        if ($currentUser?.id === userId) return 'You';
        return $allUsers?.get(userId) || userId;
    }

    // Load friend requests when component mounts
    onMount(() => {
        loadFriendRequests();
    });

    // Helper to get action button text based on request status
    function getStatusText(status) {
        switch (status) {
            case 'Pending': return 'Pending';
            case 'Accepted': return 'Accepted';
            case 'Rejected': return 'Rejected';
            default: return status;
        }
    }
</script>

<div class="friends">
    <h2>Friends</h2>

    <!-- Send Friend Request Section -->
    <div class="card">
        <h3>Send Friend Request</h3>
        <div class="form-group">
            <label for="friend-search">Search for users</label>
            <input
                type="text"
                id="friend-search"
                bind:value={searchQuery}
                placeholder="Enter username..."
                on:input={(e) => { if (e.target.value.trim()) handleSearch(); else searchResults = []; }}
            >
        </div>
        
        {#if searchResults.length > 0}
            <div class="search-results">
                <h4>Search Results:</h4>
                <ul>
                    {#each searchResults as user}
                        <li>
                            {user.username}
                            <button 
                                on:click={() => sendRequest(user.id)}
                                class="btn btn-primary"
                                disabled={$outgoingFriendRequests.some(r => r.to === user.id && r.status === 'Pending')}
                            >
                                Send Request
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}

        {#if sendRequestError}
            <p class="error-message">{sendRequestError}</p>
        {/if}
        {#if sendRequestSuccess}
            <p class="success-message">{sendRequestSuccess}</p>
        {/if}
    </div>

    <!-- Incoming Friend Requests -->
    <div class="card">
        <h3>Incoming Friend Requests</h3>
        
        {#if $incomingFriendRequests.length === 0}
            <p>No incoming friend requests.</p>
        {:else}
            <ul class="friend-request-list">
                {#each $incomingFriendRequests as request}
                    <li>
                        <div class="request-info">
                            <strong>{getUsername(request.from)}</strong> wants to be your friend
                            <span class="status-badge">{getStatusText(request.status)}</span>
                        </div>
                        {#if request.status === 'Pending'}
                            <div class="request-actions">
                                <button 
                                    on:click={() => handleRequest(request.id, 'Accept')}
                                    class="btn btn-success"
                                >
                                    Accept
                                </button>
                                <button 
                                    on:click={() => handleRequest(request.id, 'Reject')}
                                    class="btn btn-danger"
                                >
                                    Reject
                                </button>
                            </div>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </div>

    <!-- Outgoing Friend Requests -->
    <div class="card">
        <h3>Outgoing Friend Requests</h3>
        
        {#if $outgoingFriendRequests.length === 0}
            <p>No outgoing friend requests.</p>
        {:else}
            <ul class="friend-request-list">
                {#each $outgoingFriendRequests as request}
                    <li>
                        <div class="request-info">
                            You sent a request to <strong>{getUsername(request.to)}</strong>
                            <span class="status-badge">{getStatusText(request.status)}</span>
                        </div>
                        {#if request.status === 'Pending'}
                            <div class="request-actions">
                                <button 
                                    on:click={() => handleRequest(request.id, 'Cancel')}
                                    class="btn btn-warning"
                                >
                                    Cancel
                                </button>
                            </div>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </div>

    {#if handleRequestError}
        <p class="error-message">{handleRequestError}</p>
    {/if}
    {#if handleRequestSuccess}
        <p class="success-message">{handleRequestSuccess}</p>
    {/if}

    <button on:click={() => navigateTo('dashboard')} class="btn btn-secondary">
        Back to Dashboard
    </button>
</div>

<style>
    .friends {
        max-width: 800px;
        margin: 0 auto;
        padding: 1rem;
    }
    
    .card {
        background: var(--card-bg, #fff);
        border-radius: 8px;
        padding: 1.5rem;
        margin-bottom: 1.5rem;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }
    
    .card h3 {
        margin-top: 0;
        color: var(--text-primary, #333);
    }
    
    .form-group {
        margin-bottom: 1rem;
    }
    
    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
    }
    
    .form-group input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid var(--border-color, #ddd);
        border-radius: 4px;
        font-size: 1rem;
    }
    
    .search-results {
        margin-top: 1rem;
    }
    
    .search-results ul {
        list-style: none;
        padding: 0;
    }
    
    .search-results li {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem;
        border-bottom: 1px solid var(--border-color, #eee);
    }
    
    .search-results li:last-child {
        border-bottom: none;
    }
    
    .friend-request-list {
        list-style: none;
        padding: 0;
    }
    
    .friend-request-list li {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem;
        border-bottom: 1px solid var(--border-color, #eee);
    }
    
    .friend-request-list li:last-child {
        border-bottom: none;
    }
    
    .request-info {
        flex: 1;
    }
    
    .request-actions {
        display: flex;
        gap: 0.5rem;
    }
    
    .status-badge {
        margin-left: 0.5rem;
        padding: 0.25rem 0.5rem;
        border-radius: 12px;
        font-size: 0.8rem;
        background: var(--badge-bg, #e0e0e0);
    }
    
    .error-message {
        color: var(--error-color, #dc3545);
        margin: 0.5rem 0;
    }
    
    .success-message {
        color: var(--success-color, #28a745);
        margin: 0.5rem 0;
    }
    
    .btn {
        padding: 0.5rem 1rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.9rem;
    }
    
    .btn-primary {
        background: var(--primary-color, #007bff);
        color: white;
    }
    
    .btn-success {
        background: var(--success-color, #28a745);
        color: white;
    }
    
    .btn-danger {
        background: var(--danger-color, #dc3545);
        color: white;
    }
    
    .btn-warning {
        background: var(--warning-color, #ffc107);
        color: #333;
    }
    
    .btn-secondary {
        background: var(--secondary-color, #6c757d);
        color: white;
        margin-top: 1rem;
    }
    
    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
