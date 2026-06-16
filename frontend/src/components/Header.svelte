<script>
    import { theme, toggleTheme } from "../lib/stores.js";

    export let user;
    export let onLogout;
    export let onNavigate;
</script>

<header class="header">
    <h1>Kvittis</h1>
    <div class="header-controls">
        {#if user}
            <nav class="nav-links">
                <button on:click={() => onNavigate('dashboard')} class="nav-link">Dashboard</button>
                <button on:click={() => onNavigate('friends')} class="nav-link">Friends</button>
                <button on:click={() => onNavigate('groups')} class="nav-link">Groups</button>
            </nav>
        {/if}
        <button
            on:click={toggleTheme}
            class="theme-toggle"
            aria-label="Toggle theme"
        >
            {#if $theme === "dark"}
                <span class="theme-icon sun">☀️</span>
            {:else}
                <span class="theme-icon moon">🌙</span>
            {/if}
        </button>
        <div class="auth-status">
            {#if user}
                <span>Logged in as: {user.username}</span>
                <button on:click={onLogout} class="btn btn-danger"
                    >Logout</button
                >
            {/if}
        </div>
    </div>
</header>

<style>
    .header-controls {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .nav-links {
        display: flex;
        gap: 0.5rem;
    }

    .nav-link {
        background: transparent;
        border: none;
        padding: 0.5rem 1rem;
        cursor: pointer;
        color: var(--text-primary, #333);
        border-radius: 4px;
        transition: background-color 0.2s;
    }

    .nav-link:hover {
        background-color: var(--hover-bg, #f0f0f0);
    }

    .theme-toggle {
        background: transparent;
        border: none;
        cursor: pointer;
        font-size: 1.5rem;
        padding: 0.25rem;
        color: inherit;
        transition: transform 0.2s;
    }

    .theme-toggle:hover {
        transform: scale(1.1);
    }

    .theme-icon {
        display: inline-block;
    }

    .auth-status {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
</style>
