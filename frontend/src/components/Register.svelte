<script>
    import { register, isError } from "../lib/api.js";
    import { isLoading } from "../lib/stores.js";

    export let onNavigate;

    // Form state
    let username = "";
    let email = "";
    let password = "";

    let registerError = "";
    let registerSuccess = "";

    async function handleRegister(e) {
        e.preventDefault();
        registerError = "";
        registerSuccess = "";

        if (!username || !email || !password) {
            registerError = "Please fill in all fields";
            return;
        }

        isLoading.set(true);
        try {
            const result = await register(username, email, password);
            if (isError(result)) {
                registerError = result.message;
            } else {
                registerSuccess = "Registration successful! You can now login.";
                // Clear form
                username = "";
                email = "";
                password = "";
                // Auto-navigate to login after successful registration
                setTimeout(() => onNavigate("login"), 2000);
            }
        } catch (e) {
            registerError = "Network error";
        } finally {
            isLoading.set(false);
        }
    }

    function navigateToLogin() {
        onNavigate("login");
    }
</script>

<section id="register-section" class="section">
    <h2>Register</h2>
    <p>Create a new account to start tracking expenses</p>

    <div class="auth-forms">
        <div class="card">
            <h3>Create Account</h3>
            <form on:submit={handleRegister} class="form">
                <div class="form-group">
                    <label for="register-username">Username</label>
                    <input
                        type="text"
                        id="register-username"
                        bind:value={username}
                        required
                    />
                </div>
                <div class="form-group">
                    <label for="register-email">Email</label>
                    <input
                        type="email"
                        id="register-email"
                        bind:value={email}
                        required
                    />
                </div>
                <div class="form-group">
                    <label for="register-password">Password</label>
                    <input
                        type="password"
                        id="register-password"
                        bind:value={password}
                        required
                    />
                </div>
                <button type="submit" class="btn btn-primary">Register</button>
            </form>
            {#if registerError}
                <p class="error-message">{registerError}</p>
            {/if}
            {#if registerSuccess}
                <p class="success-message">{registerSuccess}</p>
            {/if}
            <p class="text-center mt-2">
                Already have an account?
                <button on:click={navigateToLogin} class="btn btn-secondary"
                    >Back to Login</button
                >
            </p>
        </div>
    </div>
</section>
