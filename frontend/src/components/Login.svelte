<script>
    import { login, isError } from '../lib/api.js';
    import { isLoading } from '../lib/stores.js';

    export let onLogin;
    export let onNavigate;

    // Form state
    let username = '';
    let password = '';

    let loginError = '';
    let loginSuccess = '';

    async function handleLogin(e) {
        e.preventDefault();
        loginError = '';
        loginSuccess = '';

        if (!username || !password) {
            loginError = 'Please enter both username and password';
            return;
        }

        isLoading.set(true);
        try {
            const result = await login(username, password);
            if (isError(result)) {
                loginError = result.message;
            } else {
                username = '';
                password = '';
                await onLogin(result);
                loginSuccess = 'Logged in successfully!';
                setTimeout(() => loginSuccess = '', 3000);
            }
        } catch (e) {
            loginError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }

    function navigateToRegister() {
        onNavigate('register');
    }
</script>

<section id="login-section" class="section">
    <h2>Welcome to Kvittis</h2>
    <p>Track shared expenses with friends and groups</p>

    <div class="auth-forms">
        <div class="card">
            <h3>Login</h3>
            <form on:submit={handleLogin} class="form">
                <div class="form-group">
                    <label for="login-username">Username</label>
                    <input
                        type="text"
                        id="login-username"
                        bind:value={username}
                        required
                    >
                </div>
                <div class="form-group">
                    <label for="login-password">Password</label>
                    <input
                        type="password"
                        id="login-password"
                        bind:value={password}
                        required
                    >
                </div>
                <button type="submit" class="btn btn-primary">Login</button>
            </form>
            {#if loginError}
                <p class="error-message">{loginError}</p>
            {/if}
            {#if loginSuccess}
                <p class="success-message">{loginSuccess}</p>
            {/if}
            <p class="text-center mt-2">
                <button on:click={navigateToRegister} class="btn btn-secondary">Register</button>
            </p>
        </div>
    </div>
</section>
