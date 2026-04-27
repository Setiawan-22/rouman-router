export const authState = $state({
    isLoggedIn: false,
    username: null as string | null,
    isInitialized: false,
});

export async function checkAuth() {
    // Pada aplikasi nyata, endpoint ini mengecek validitas token JWT dari HttpOnly cookie
    try {
        const res = await fetch('/api/auth/me', { credentials: 'same-origin' });
        if (res.ok) {
            const data = await res.json();
            authState.isLoggedIn = true;
            authState.username = data.username;
        } else {
            authState.isLoggedIn = false;
            authState.username = null;
        }
    } catch {
        authState.isLoggedIn = false;
        authState.username = null;
    } finally {
        authState.isInitialized = true;
    }
}

export async function logout() {
    try {
        await fetch('/api/auth/logout', { method: 'POST', credentials: 'same-origin' });
    } catch (e) {
        console.error("Logout failed at backend", e);
    } finally {
        authState.isLoggedIn = false;
        authState.username = null;
    }
}
