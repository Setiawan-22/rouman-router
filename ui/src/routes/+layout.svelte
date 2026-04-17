<script lang="ts">
    import '../app.css';
    import { authState, checkAuth } from '$lib/state/auth.svelte';
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/state';
    import { goto } from '$app/navigation';

    let { children } = $props();

    // Persist & Layout state
    let sidebarOpen = $state(true);
    let menuOpen = $state(false);
    let internetStatus = $state('CHECKING...');
    let checkInterval: number | ReturnType<typeof setInterval>;

    async function checkInternet() {
        if (!authState.isLoggedIn) return; // limit traffic if not auth
        try {
            const res = await fetch('/api/v1/system/internet-status');
            if (res.ok) {
                const data = await res.json();
                internetStatus = data.status;
            } else {
                internetStatus = 'DISCONNECTED';
            }
        } catch {
            internetStatus = 'DISCONNECTED';
        }
    }

    onMount(async () => {
        await checkAuth();
        checkInternet();
        checkInterval = setInterval(checkInternet, 10000);
    });

    onDestroy(() => {
        if (checkInterval) clearInterval(checkInterval);
    });

    $effect(() => {
        if (!authState.isInitialized) return; // Tunggu checkAuth selesai
        if (!authState.isLoggedIn && page.url.pathname !== '/login') {
            goto('/login');
        } else if (authState.isLoggedIn && page.url.pathname === '/login') {
            goto('/');
        }
    });

    async function handleLogout() {
        try {
            await fetch('/api/auth/logout', { method: 'POST' });
        } finally {
            authState.isLoggedIn = false;
            authState.username = null;
            menuOpen = false;
            goto('/login');
        }
    }
</script>

{#if !authState.isInitialized}
    <div class="min-h-screen bg-black flex items-center justify-center">
        <div class="text-neon-green/80 animate-pulse text-sm tracking-widest uppercase">Securing Connection...</div>
    </div>
{:else}
    {#if page.url.pathname === '/login'}
        {@render children()}
    {:else}
        <!-- APP SHELL -->
        <div class="flex flex-col h-screen w-full overflow-hidden bg-black text-white">
            
            <!-- TOP FULL-WIDTH HEADER -->
            <header class="h-16 w-full flex justify-between items-center px-6 border-b border-neon-green/10 bg-black/60 backdrop-blur-md z-50 shrink-0">
                <!-- LEfT: Logo -->
                <div class="flex items-center">
                    <h1 class="text-lg font-bold tracking-widest text-neon-green [text-shadow:0_0_10px_rgba(0,255,196,0.3)]">ROUMAN_</h1>
                </div>

                <!-- RIGHT: Status & User Info -->
                <div class="flex items-center gap-4 text-xs tracking-widest text-neon-green/60 uppercase relative">
                    <div class="hidden sm:block">Status: 
                        {#if internetStatus === 'CONNECTED'}
                            <span class="text-neon-green drop-shadow-[0_0_5px_rgba(0,255,196,0.5)]">CONNECTED</span>
                        {:else if internetStatus === 'CHECKING...'}
                            <span class="text-yellow-400 drop-shadow-[0_0_5px_rgba(255,255,0,0.5)] animate-pulse">CHECKING...</span>
                        {:else}
                            <span class="text-red-500 drop-shadow-[0_0_5px_rgba(255,0,0,0.5)]">DISCONNECTED</span>
                        {/if}
                    </div>
                    <div class="w-px h-4 bg-neon-green/20 hidden sm:block"></div>
                    
                    <button 
                        class="flex items-center gap-2 hover:text-neon-green transition-colors focus:outline-none relative z-50"
                        onclick={() => menuOpen = !menuOpen}
                    >
                        <span>User: <span class="text-white">{authState.username}</span></span>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 transition-transform duration-200 {menuOpen ? 'rotate-180' : ''}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </button>

                    {#if menuOpen}
                        <div class="absolute top-8 right-0 mt-2 w-48 bg-black/95 border border-neon-green/30 backdrop-blur-xl rounded overflow-hidden shadow-[0_0_20px_rgba(0,255,196,0.15)] flex flex-col z-50">
                            <button class="px-4 py-3 text-left hover:bg-neon-green/10 transition-colors border-b border-neon-green/10 text-neon-green/80 flex items-center gap-3" onclick={() => menuOpen = false}>
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" /></svg>
                                Profile
                            </button>
                            <button class="px-4 py-3 text-left hover:bg-red-900/40 text-red-400 transition-colors flex items-center gap-3" onclick={handleLogout}>
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" /></svg>
                                Logout
                            </button>
                        </div>
                    {/if}
                    
                    {#if menuOpen}
                        <button class="fixed inset-0 h-screen w-screen z-40 cursor-default" onclick={() => menuOpen = false} tabindex="-1"></button>
                    {/if}
                </div>
            </header>

            <!-- BELow Header -->
            <div class="flex flex-1 overflow-hidden">
                <!-- SIDEBAR -->
                <aside class="{sidebarOpen ? 'w-56' : 'w-16'} flex flex-col border-r border-neon-green/10 bg-black/40 backdrop-blur-xl transition-all duration-300 z-40 relative shrink-0">
                    
                    <!-- Edge Floating Toggle -->
                    <button 
                        onclick={() => sidebarOpen = !sidebarOpen} 
                        class="absolute -right-3.5 top-1/2 transform -translate-y-1/2 w-7 h-7 bg-black border border-neon-green/30 rounded-full flex items-center justify-center text-neon-green/80 hover:text-neon-green hover:border-neon-green hover:shadow-[0_0_10px_rgba(0,255,196,0.4)] transition-all z-50 focus:outline-none group"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 transition-transform duration-300 {sidebarOpen ? '' : 'rotate-180'} group-hover:scale-110" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                           <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>

                    <!-- Menu Links -->
                    <nav class="flex-1 pt-4 flex flex-col gap-1 overflow-y-auto overflow-x-hidden">
                        <a href="/" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname === '/' ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">DASHBOARD</span>{/if}
                        </a>
                        
                        <a href="/network" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/network') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">NETWORK</span>{/if}
                        </a>

                        <a href="/interfaces" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/interfaces') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1v-2zM4 21a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1v-2z" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">INTERFACES</span>{/if}
                        </a>

                        <a href="/routing" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/routing') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">ROUTING & NAT</span>{/if}
                        </a>

                        <a href="/dhcp" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/dhcp') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">DHCP SERVER</span>{/if}
                        </a>

                        <a href="/firewall" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/firewall') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-7.618 3.016L4 20l8 4 8-4-1.382-14.016z" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">FIREWALL (XDP)</span>{/if}
                        </a>

                        <a href="/config" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/config') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold">SYSTEM CONFIG</span>{/if}
                        </a>

                        <a href="/zero-trust" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/zero-trust') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold uppercase">Zero Trust</span>{/if}
                        </a>

                        <a href="/dns" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/dns') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold uppercase">DNS SINKHOLE</span>{/if}
                        </a>

                        <a href="/devices" class="flex items-center gap-3 px-4 py-2.5 {page.url.pathname.includes('/devices') ? 'bg-neon-green/10 text-neon-green border-r-2 border-neon-green shadow-[inset_0_0_20px_rgba(0,255,196,0.05)]' : 'text-neon-green/60 hover:bg-neon-green/5 transition-colors'}">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" /></svg>
                            {#if sidebarOpen}<span class="text-xs tracking-widest font-bold uppercase">Devices</span>{/if}
                        </a>
                    </nav>
                </aside>

                <!-- SCROLLABLE PAGE SLOT CONTENT -->
                <main class="flex-1 overflow-y-auto overflow-x-hidden relative p-8 bg-black/90">
                    {@render children()}
                </main>
            </div>
        </div>
    {/if}
{/if}
