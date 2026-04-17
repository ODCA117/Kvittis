<script>
    import { token, groups } from '../lib/stores.js';
    import { listGroups, isError } from '../lib/api.js';

    export let navigateTo;

    async function loadGroups() {
        if (!$token) return;

        const result = await listGroups($token);
        if (!isError(result)) {
            groups.set(result);
        }
    }
</script>

<div class="groups">
    <div class="card">
        <h3>Your Groups</h3>
        <button on:click={() => navigateTo('dashboard')} class="btn btn-secondary">Back to Dashboard</button>
        <button id="create-group-btn" class="btn btn-secondary">Create Group</button>
        <div class="groups-list">
            {#if $groups.length === 0}
                <p class="empty-state">No groups yet.</p>
            {:else}
                {#each $groups as group}
                    <div class="group-card">
                        <h4>{group.name}</h4>
                        <p>{group.members?.length || 0} members</p>
                    </div>
                {/each}
            {/if}
        </div>
    </div>
</div>
