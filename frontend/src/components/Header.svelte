<script>
    import { theme, toggleTheme } from "../lib/stores.js";

    export let user;
    export let onLogout;
</script>

<header class="header">
    <h1>Kvittis</h1>
    <div class="header-controls">
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
</style>
