<script>
    import { onMount } from 'svelte';
    import { token, groups, currentGroup, groupBalances, isLoading, currentUser, allUsers } from '../lib/stores.js';
    import { listGroups, createGroup, getGroup, deleteGroup, getGroupBalances, isError, formatDateShort, centsToDollars } from '../lib/api.js';

    export let navigateTo;

    // Local state
    let newGroupName = '';
    let showCreateModal = false;
    let groupError = '';
    let groupSuccess = '';
    let selectedGroupId = null;
    let showGroupDetail = false;

    async function loadGroups() {
        if (!$token) return;

        isLoading.set(true);
        try {
            const result = await listGroups($token);
            if (!isError(result)) {
                groups.set(result);
            }
        } catch (e) {
            console.error("Failed to load groups:", e);
        } finally {
            isLoading.set(false);
        }
    }

    async function handleCreateGroup(e) {
        e.preventDefault();
        groupError = '';
        groupSuccess = '';

        if (!newGroupName.trim()) {
            groupError = 'Please enter a group name';
            return;
        }

        isLoading.set(true);
        try {
            const result = await createGroup(newGroupName, $token);
            if (isError(result)) {
                groupError = result.message;
            } else {
                groupSuccess = 'Group created successfully!';
                newGroupName = '';
                showCreateModal = false;
                await loadGroups();
                setTimeout(() => groupSuccess = '', 3000);
            }
        } catch (e) {
            groupError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }

    async function viewGroupDetails(groupId) {
        isLoading.set(true);
        try {
            const result = await getGroup(groupId, $token);
            if (!isError(result)) {
                currentGroup.set(result);
                selectedGroupId = groupId;
                showGroupDetail = true;
                
                // Load group balances
                const balancesResult = await getGroupBalances(groupId, $token);
                if (!isError(balancesResult)) {
                    groupBalances.set(balancesResult);
                }
            }
        } catch (e) {
            console.error("Failed to load group details:", e);
        } finally {
            isLoading.set(false);
        }
    }

    async function handleDeleteGroup(groupId) {
        if (!confirm('Are you sure you want to delete this group?')) return;
        
        isLoading.set(true);
        try {
            const result = await deleteGroup(groupId, $token);
            if (isError(result)) {
                groupError = result.message;
            } else {
                groupSuccess = 'Group deleted successfully!';
                showGroupDetail = false;
                await loadGroups();
                setTimeout(() => groupSuccess = '', 3000);
            }
        } catch (e) {
            groupError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }

    function closeGroupDetail() {
        showGroupDetail = false;
        currentGroup.set(null);
        groupBalances.set([]);
    }

    // Load groups on mount
    onMount(() => {
        loadGroups();
    });

    // Helper to format date
    function formatSettledDate(dateString) {
        if (!dateString) return 'Never';
        return formatDateShort(dateString);
    }
    
    // Helper to get username from ID
    function getUsername(userId) {
        if ($currentUser?.id === userId) return 'You';
        return $allUsers?.get(userId) || userId;
    }
</script>

<div class="groups">
    <div class="card">
        <h3>Your Groups</h3>
        <div class="groups-actions">
            <button on:click={() => navigateTo('dashboard')} class="btn btn-secondary">Back to Dashboard</button>
            <button on:click={() => showCreateModal = true} class="btn btn-primary">Create Group</button>
        </div>
        
        {#if groupError}
            <p class="error-message">{groupError}</p>
        {/if}
        {#if groupSuccess}
            <p class="success-message">{groupSuccess}</p>
        {/if}

        <div class="groups-list">
            {#if $groups.length === 0}
                <p class="empty-state">No groups yet.</p>
            {:else}
                {#each $groups as group}
                    <div class="group-card" on:click={() => viewGroupDetails(group.id)}>
                        <h4>{group.name}</h4>
                        <p>
                            <span>{group.members?.length || 0} members</span>
                            {#if group.last_settled}
                                <span class="last-settled">| Last settled: {formatSettledDate(group.last_settled)}</span>
                            {/if}
                        </p>
                    </div>
                {/each}
            {/if}
        </div>
    </div>

    <!-- Create Group Modal -->
    {#if showCreateModal}
        <div class="modal-overlay" on:click={() => showCreateModal = false}>
            <div class="modal" on:click|stopPropagation>
                <h3>Create New Group</h3>
                <form on:submit={handleCreateGroup} class="form">
                    <div class="form-group">
                        <label for="group-name">Group Name</label>
                        <input
                            type="text"
                            id="group-name"
                            bind:value={newGroupName}
                            placeholder="My Friends"
                            required
                        >
                    </div>
                    <div class="modal-actions">
                        <button type="button" on:click={() => showCreateModal = false} class="btn btn-secondary">
                            Cancel
                        </button>
                        <button type="submit" class="btn btn-primary">Create</button>
                    </div>
                </form>
            </div>
        </div>
    {/if}

    <!-- Group Detail Modal -->
    {#if showGroupDetail && $currentGroup}
        <div class="modal-overlay" on:click={closeGroupDetail}>
            <div class="modal group-detail" on:click|stopPropagation>
                <h3>{$currentGroup.name}</h3>
                <p><strong>Members:</strong> {$currentGroup.members?.length || 0}</p>
                <p><strong>Last settled:</strong> {formatSettledDate($currentGroup.last_settled)}</p>
                
                {#if $currentGroup.members && $currentGroup.members.length > 0}
                    <div class="members-list">
                        <h4>Group Members</h4>
                        <ul>
                            {#each $currentGroup.members as member}
                                <li>
                                    {getUsername(member[0])}
                                    {#if member[1] === 'Admin'}
                                        <span class="member-role"> (Admin)</span>
                                    {:else if member[1] === 'Member'}
                                        <span class="member-role"> (Member)</span>
                                    {/if}
                                </li>
                            {/each}
                        </ul>
                    </div>
                {/if}
                
                <h4>Group Balances</h4>
                {#if $groupBalances.length === 0}
                    <p class="empty-state">No group balances yet.</p>
                {:else}
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>From</th>
                                <th>To</th>
                                <th>Amount</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each $groupBalances as balance}
                                <tr>
                                    <td>{getUsername(balance.from)}</td>
                                    <td>{getUsername(balance.to)}</td>
                                    <td class="{balance.amount > 0 ? 'amount-positive' : 'amount-negative'}">
                                        {centsToDollars(Math.abs(balance.amount))}
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                {/if}
                
                <div class="modal-actions">
                    <button on:click={closeGroupDetail} class="btn btn-secondary">Close</button>
                    <button on:click={() => handleDeleteGroup($currentGroup.id)} class="btn btn-danger">
                        Delete Group
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>

<style>
    .groups {
        max-width: 800px;
        margin: 0 auto;
        padding: 1rem;
    }
    
    .groups-actions {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;
    }
    
    .card {
        background: var(--card-bg, #fff);
        border-radius: 8px;
        padding: 1.5rem;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }
    
    .card h3 {
        margin-top: 0;
        color: var(--text-primary, #333);
    }
    
    .groups-list {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 1rem;
        margin-top: 1rem;
    }
    
    .group-card {
        background: var(--card-bg, #fff);
        border: 1px solid var(--border-color, #ddd);
        border-radius: 8px;
        padding: 1rem;
        cursor: pointer;
        transition: transform 0.2s, box-shadow 0.2s;
    }
    
    .group-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
    }
    
    .group-card h4 {
        margin: 0 0 0.5rem 0;
        color: var(--text-primary, #333);
    }
    
    .group-card p {
        margin: 0;
        color: var(--text-secondary, #666);
        font-size: 0.9rem;
    }
    
    .last-settled {
        margin-left: 0.5rem;
        font-size: 0.85rem;
        color: var(--text-muted, #888);
    }
    
    .empty-state {
        text-align: center;
        padding: 1rem;
        color: var(--text-muted, #888);
    }
    
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }
    
    .modal {
        background: var(--card-bg, #fff);
        border-radius: 8px;
        padding: 1.5rem;
        min-width: 300px;
        max-width: 500px;
        width: 90%;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
    }
    
    .modal h3 {
        margin-top: 0;
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
    
    .modal-actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
        margin-top: 1.5rem;
    }
    
    .group-detail {
        max-width: 600px;
    }
    
    .group-detail h4 {
        margin-top: 1rem;
        margin-bottom: 0.5rem;
    }
    
    .error-message {
        color: var(--error-color, #dc3545);
        margin: 0.5rem 0;
    }
    
    .success-message {
        color: var(--success-color, #28a745);
        margin: 0.5rem 0;
    }
    
    .data-table {
        width: 100%;
        border-collapse: collapse;
        margin-top: 1rem;
    }
    
    .data-table th,
    .data-table td {
        padding: 0.75rem;
        text-align: left;
        border-bottom: 1px solid var(--border-color, #eee);
    }
    
    .data-table th {
        background: var(--table-header-bg, #f8f9fa);
        font-weight: 600;
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
    
    .btn-secondary {
        background: var(--secondary-color, #6c757d);
        color: white;
    }
    
    .btn-danger {
        background: var(--danger-color, #dc3545);
        color: white;
    }
    
    .amount-positive {
        color: var(--success-color, #28a745);
        font-weight: 500;
    }
    
    .amount-negative {
        color: var(--danger-color, #dc3545);
        font-weight: 500;
    }
    
    .members-list {
        margin: 1rem 0;
    }
    
    .members-list ul {
        list-style: none;
        padding: 0;
        margin: 0.5rem 0 0 0;
    }
    
    .members-list li {
        padding: 0.5rem;
        border-bottom: 1px solid var(--border-color, #eee);
    }
    
    .members-list li:last-child {
        border-bottom: none;
    }
    
    .member-role {
        color: var(--text-muted, #666);
        font-size: 0.85rem;
    }
</style>
