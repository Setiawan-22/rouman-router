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
                body: JSON.stringify({ username, password }),
                credentials: 'same-origin'
            });

            if (res.ok) {
                authState.isLoggedIn = true;
                authState.username = username;
                await fetch('/api/auth/me', { credentials: 'same-origin' });
                goto('/');
            } else {
                const data = await res.json();
                errorMessage = data.error || 'Invalid username or password. Please try again.';
            }
        } catch (e) {
            errorMessage = 'System unreachable. Please check your network connection.';
        } finally {
            isSubmitting = false;
        }
    }
</script>

<svelte:head>
    <title>Login | ROUMAN OS</title>
</svelte:head>

<div class="relative w-full min-h-screen flex items-center justify-center bg-gray-50 overflow-hidden">
    <!-- Premium Background Image -->
    <div class="absolute inset-0 z-0">
        <img 
            src="/images/login-bg.png" 
            alt="Background" 
            class="w-full h-full object-cover opacity-60 scale-105 blur-[2px]"
        />
        <div class="absolute inset-0 bg-gradient-to-tr from-white/80 via-transparent to-white/40"></div>
    </div>

    <!-- Login Card -->
    <div class="relative z-10 w-full max-w-[400px] mx-4 animate-in fade-in zoom-in duration-700">
        <div class="bg-white/90 backdrop-blur-xl p-10 rounded-3xl shadow-[0_20px_50px_rgba(0,0,0,0.1)] border border-white/50 flex flex-col gap-8">
            
            <div class="flex flex-col items-center gap-3">
                <div class="w-16 h-16 bg-[#1E88E5] rounded-2xl flex items-center justify-center shadow-lg shadow-blue-500/30 transform -rotate-3">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                </div>
                <div class="text-center mt-2">
                    <h1 class="text-2xl font-black text-gray-900 tracking-tight">ROUMAN <span class="text-[#1E88E5]">OS</span></h1>
                    <p class="text-[10px] font-bold text-gray-400 uppercase tracking-[0.3em]">Management Console</p>
                </div>
            </div>

            {#if errorMessage}
                <div class="bg-red-50 border border-red-100 text-red-600 p-4 rounded-xl text-xs font-medium flex items-center gap-3 animate-shake">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                    {errorMessage}
                </div>
            {/if}

            <form class="flex flex-col gap-5" onsubmit={handleLogin}>
                <div class="flex flex-col gap-2">
                    <label for="username" class="text-[10px] font-bold text-gray-400 uppercase tracking-widest px-1">Account ID</label>
                    <div class="relative group">
                        <div class="absolute inset-y-0 left-4 flex items-center pointer-events-none text-gray-400 group-focus-within:text-[#1E88E5] transition-colors">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                            </svg>
                        </div>
                        <input 
                            type="text" 
                            id="username" 
                            bind:value={username} 
                            required
                            class="w-full bg-gray-50 border border-gray-100 rounded-xl py-3.5 pl-12 pr-4 text-sm text-gray-800 placeholder:text-gray-300 focus:outline-none focus:border-[#1E88E5] focus:bg-white focus:ring-4 focus:ring-blue-500/5 transition-all"
                            placeholder="Enter username"
                        />
                    </div>
                </div>

                <div class="flex flex-col gap-2">
                    <label for="password" class="text-[10px] font-bold text-gray-400 uppercase tracking-widest px-1">Access Key</label>
                    <div class="relative group">
                        <div class="absolute inset-y-0 left-4 flex items-center pointer-events-none text-gray-400 group-focus-within:text-[#1E88E5] transition-colors">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                            </svg>
                        </div>
                        <input 
                            type={showPassword ? "text" : "password"}
                            id="password" 
                            bind:value={password}
                            required 
                            class="w-full bg-gray-50 border border-gray-100 rounded-xl py-3.5 pl-12 pr-12 text-sm text-gray-800 placeholder:text-gray-300 focus:outline-none focus:border-[#1E88E5] focus:bg-white focus:ring-4 focus:ring-blue-500/5 transition-all"
                            placeholder="••••••••"
                        />
                        <button 
                            type="button" 
                            class="absolute inset-y-0 right-4 flex items-center text-gray-300 hover:text-gray-600 focus:outline-none"
                            onclick={() => showPassword = !showPassword}
                        >
                            {#if showPassword}
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" /></svg>
                            {:else}
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
                            {/if}
                        </button>
                    </div>
                </div>

                <button 
                    type="submit" 
                    disabled={isSubmitting}
                    class="mt-4 relative w-full bg-[#1E88E5] text-white py-4 rounded-xl font-bold text-xs tracking-[0.2em] uppercase transition-all hover:bg-blue-600 hover:-translate-y-0.5 active:translate-y-0 shadow-lg shadow-blue-500/30 disabled:opacity-50 disabled:translate-y-0 disabled:shadow-none overflow-hidden"
                >
                    <span class="relative z-10">{isSubmitting ? 'Verifying...' : 'Sign In'}</span>
                    {#if isSubmitting}
                        <div class="absolute inset-0 bg-blue-600/50 flex items-center justify-center">
                            <div class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                        </div>
                    {/if}
                </button>
            </form>

            <div class="flex flex-col items-center gap-4 border-t border-gray-100 pt-8 mt-2">
                <p class="text-[9px] text-gray-400 font-medium text-center leading-relaxed">
                    Licensed to Rouman Enterprise Network<br/>
                    © 2024 Mandala Rouman Systems. All Rights Reserved.
                </p>
                <div class="flex gap-4">
                    <span class="w-1.5 h-1.5 rounded-full bg-green-400 shadow-[0_0_8px_rgba(74,222,128,0.5)]"></span>
                    <span class="text-[8px] font-bold text-gray-400 uppercase tracking-widest">Gateway Active</span>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    :global(body) {
        margin: 0;
        font-family: 'Inter', system-ui, -apple-system, sans-serif;
    }

    @keyframes shake {
        0%, 100% { transform: translateX(0); }
        25% { transform: translateX(-4px); }
        75% { transform: translateX(4px); }
    }

    .animate-shake {
        animation: shake 0.4s ease-in-out 0s 1;
    }
</style>

