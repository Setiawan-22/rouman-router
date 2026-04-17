<script lang="ts">
    import { authState } from '$lib/state/auth.svelte';
    import { goto } from '$app/navigation';
    
    let username = $state('');
    let password = $state('');
    let showPassword = $state(false);
    let errorMessage = $state('');
    let isSubmitting = $state(false);

    async function handleLogin(e: Event) {
        e.preventDefault();
        isSubmitting = true;
        errorMessage = '';

        try {
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password })
            });

            if (res.ok) {
                authState.isLoggedIn = true;
                authState.username = username;
                
                // Fetch me to set cookie state properly in some browsers
                await fetch('/api/auth/me');

                goto('/');
            } else {
                errorMessage = 'Access Denied: Invalid Credentials.';
            }
        } catch {
            errorMessage = 'System Error: Cannot reach Control Plane.';
        } finally {
            isSubmitting = false;
        }
    }
</script>

<svelte:head>
    <title>AUTH // ROUMAN OS</title>
</svelte:head>

<div class="relative w-full min-h-screen flex items-center justify-center overflow-hidden">
    <!-- Decorative Hacker Grid -->
    <div class="absolute inset-0 pointer-events-none opacity-20" style="background-size: 40px 40px; background-image: linear-gradient(to right, rgba(0, 255, 196, 0.1) 1px, transparent 1px), linear-gradient(to bottom, rgba(0, 255, 196, 0.1) 1px, transparent 1px);"></div>

    <!-- Login Glass Panel -->
    <div class="relative z-10 w-full max-w-md p-8 rounded-xl border border-neon-green/30 bg-black/60 backdrop-blur-md shadow-[0_0_40px_rgba(0,255,196,0.15)] flex flex-col gap-6">
        
        <div class="text-center">
            <h1 class="text-3xl font-bold tracking-widest text-neon-green [text-shadow:0_0_10px_rgba(0,255,196,0.6)]">ROUMAN</h1>
            <p class="text-sm tracking-widest mt-2 uppercase text-neon-green/60">Secure Gateway Access</p>
        </div>

        {#if errorMessage}
            <div class="bg-red-950/50 border border-red-500/50 text-red-400 p-3 rounded text-sm text-center">
                {errorMessage}
            </div>
        {/if}

        <form class="flex flex-col gap-5" onsubmit={handleLogin}>
            <div class="flex flex-col gap-1">
                <label for="username" class="text-xs tracking-wider uppercase text-neon-green/70">Sysadmin ID</label>
                <input 
                    type="text" 
                    id="username" 
                    bind:value={username} 
                    required
                    class="w-full bg-black/50 border border-neon-green/40 rounded p-3 text-neon-green focus:outline-none focus:border-neon-green focus:ring-1 focus:ring-neon-green transition-all"
                    placeholder="Enter ID"
                />
            </div>

            <div class="flex flex-col gap-1">
                <label for="password" class="text-xs tracking-wider uppercase text-neon-green/70">Password</label>
                <div class="relative w-full">
                    <input 
                        type={showPassword ? "text" : "password"}
                        id="password" 
                        bind:value={password}
                        required 
                        class="w-full bg-black/50 border border-neon-green/40 rounded p-3 pr-10 text-neon-green focus:outline-none focus:border-neon-green focus:ring-1 focus:ring-neon-green transition-all"
                        placeholder="••••••••"
                    />
                    <button 
                        type="button" 
                        class="absolute inset-y-0 right-3 flex items-center text-neon-green/50 hover:text-neon-green focus:outline-none"
                        onclick={() => showPassword = !showPassword}
                    >
                        {#if showPassword}
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5"><path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" /></svg>
                        {:else}
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5"><path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
                        {/if}
                    </button>
                </div>
            </div>

            <button 
                type="submit" 
                disabled={isSubmitting}
                class="mt-4 relative group w-full bg-neon-green/10 border border-neon-green text-neon-green/90 p-3 rounded font-bold tracking-widest uppercase transition-all overflow-hidden hover:bg-neon-green/20 focus:outline-none focus:ring-2 focus:ring-neon-green focus:ring-offset-2 focus:ring-offset-black disabled:opacity-50 disabled:cursor-not-allowed"
            >
                <span class="relative z-10">{isSubmitting ? 'VERIFYING...' : 'LOGIN'}</span>
                <!-- Hover Sweep Effect -->
                <div class="absolute inset-0 h-full w-0 bg-neon-green/20 transition-all duration-300 ease-out group-hover:w-full z-0"></div>
            </button>
        </form>

    </div>
</div>
