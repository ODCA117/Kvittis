<script>
    import { register, login, isError } from '../lib/api.js';
    import { isLoading } from '../lib/stores.js';

    export let onLogin;

    // Form state
    let loginUsername = '';
    let loginPassword = '';
    let registerUsername = '';
    let registerEmail = '';
    let registerPassword = '';

    let loginError = '';
    let loginSuccess = '';
    let registerError = '';
    let registerSuccess = '';

    async function handleLogin(e) {
        e.preventDefault();
        loginError = '';
        loginSuccess = '';

        if (!loginUsername || !loginPassword) {
            loginError = 'Please enter both username and password';
            return;
        }

        isLoading.set(true);
        try {
            const result = await login(loginUsername, loginPassword);
            if (isError(result)) {
                loginError = result.message;
            } else {
                loginUsername = '';
                loginPassword = '';
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

    async function handleRegister(e) {
        e.preventDefault();
        registerError = '';
        registerSuccess = '';

        if (!registerUsername || !registerEmail || !registerPassword) {
            registerError = 'Please fill in all fields';
            return;
        }

        isLoading.set(true);
        try {
            const result = await register(registerUsername, registerEmail, registerPassword);
            if (isError(result)) {
                registerError = result.message;
            } else {
                registerUsername = '';
                registerEmail = '';
                registerPassword = '';
                registerSuccess = 'Registration successful! You can now login.';
            }
        } catch (e) {
            registerError = 'Network error';
        } finally {
            isLoading.set(false);
        }
    }
</script>

<section id="auth-section" class="section">
    <h2>Welcome to Kvittis</h2>
    <p>Track shared expenses with friends and groups</p>

    <div class="auth-forms">
        <!-- Login Form -->
        <div class="card">
            <h3>Login</h3>
            <form on:submit={handleLogin} class="form">
                <div class="form-group">
                    <label for="login-username">Username</label>
                    <input
                        type="text"
                        id="login-username"
                        bind:value={loginUsername}
                        required
                    >
                </div>
                <div class="form-group">
                    <label for="login-password">Password</label>
                    <input
                        type="password"
                        id="login-password"
                        bind:value={loginPassword}
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
        </div>

        <!-- Register Form -->
        <div class="card">
            <h3>Register</h3>
            <form on:submit={handleRegister} class="form">
                <div class="form-group">
                    <label for="register-username">Username</label>
                    <input
                        type="text"
                        id="register-username"
                        bind:value={registerUsername}
                        required
                    >
                </div>
                <div class="form-group">
                    <label for="register-email">Email</label>
                    <input
                        type="email"
                        id="register-email"
                        bind:value={registerEmail}
                        required
                    >
                </div>
                <div class="form-group">
                    <label for="register-password">Password</label>
                    <input
                        type="password"
                        id="register-password"
                        bind:value={registerPassword}
                        required
                    >
                </div>
                <button type="submit" class="btn btn-secondary">Register</button>
            </form>
            {#if registerError}
                <p class="error-message">{registerError}</p>
            {/if}
            {#if registerSuccess}
                <p class="success-message">{registerSuccess}</p>
            {/if}
        </div>
    </div>
</section>
