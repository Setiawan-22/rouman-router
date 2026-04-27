<script lang="ts">
    import '../app.css';
    import { authState, checkAuth, logout } from '$lib/state/auth.svelte';
    import { onMount, onDestroy } from 'svelte';
    import { fade, fly } from 'svelte/transition';
    import { page } from '$app/state';
    import { goto } from '$app/navigation';
    import { APP_REGISTRY } from '$lib/apps/registry';
    
    // Import Apps
    import Dhcp from '$lib/apps/Dhcp.svelte';
    import Cloudflare from '$lib/apps/Cloudflare.svelte';
    import Config from '$lib/apps/Config.svelte';
    import Interfaces from '$lib/apps/Interfaces.svelte';
    import Firewall from '$lib/apps/Firewall.svelte';
    import Routing from '$lib/apps/Routing.svelte';
    import Wireguard from '$lib/apps/Wireguard.svelte';
    import Hotspot from '$lib/apps/Hotspot.svelte';
    import Queues from '$lib/apps/Queues.svelte';
    import Dns from '$lib/apps/Dns.svelte';
    import Automation from '$lib/apps/Automation.svelte';
    import Discovery from '$lib/apps/Discovery.svelte';
    import Dashboard from '$lib/apps/Dashboard.svelte';
    import Microvms from '$lib/apps/Microvms.svelte';
    import Containers from '$lib/apps/Containers.svelte';
    import Proxy from '$lib/apps/Proxy.svelte';
    import Services from '$lib/apps/Services.svelte';

    const COMPONENT_MAP: Record<string, any> = {
        'Dashboard.svelte': Dashboard,
        'Dhcp.svelte': Dhcp,
        'Cloudflare.svelte': Cloudflare,
        'Config.svelte': Config,
        'Interfaces.svelte': Interfaces,
        'Firewall.svelte': Firewall,
        'Routing.svelte': Routing,
        'Wireguard.svelte': Wireguard,
        'Hotspot.svelte': Hotspot,
        'Queues.svelte': Queues,
        'Dns.svelte': Dns,
        'Automation.svelte': Automation,
        'Discovery.svelte': Discovery,
        'Microvms.svelte': Microvms,
        'Containers.svelte': Containers,
        'Proxy.svelte': Proxy,
        'Services.svelte': Services
    };

    let { children } = $props();

    let sidebarOpen = $state(true);
    let internetStatus = $state('CHECKING...');
    let activeConfig = $state(null as any);
    let activeAppId = $state('dashboard');
    let checkInterval: number | ReturnType<typeof setInterval>;

    // Default ke dashboard
    let activeAppMeta = $derived(APP_REGISTRY.find(a => a.id === activeAppId) || APP_REGISTRY[0]);

    async function fetchStatus() {
        if (!authState.isLoggedIn) return;
        try {
            const [resInt, resCfg] = await Promise.all([
                fetch('/api/v1/system/internet-status'),
                fetch('/api/v1/config/active'),
                fetchNotifications()
            ]);
            
            if (resInt.ok) internetStatus = (await resInt.json()).status;
            if (resCfg.ok) activeConfig = (await resCfg.json()).active;
        } catch {
            internetStatus = 'DISCONNECTED';
        }
    }

    onMount(async () => {
        await checkAuth();
        if (authState.isLoggedIn) {
            fetchStatus();
            checkInterval = setInterval(fetchStatus, 15000);
        }
    });

    onDestroy(() => {
        if (checkInterval) clearInterval(checkInterval);
    });

    $effect(() => {
        if (!authState.isInitialized) return;
        if (!authState.isLoggedIn && page.url.pathname !== '/login') {
            goto('/login');
        } else if (authState.isLoggedIn && page.url.pathname === '/login') {
            goto('/');
        }
    });

    let filteredApps = $derived(APP_REGISTRY);

    const categories = ['Overview', 'Interfaces', 'IP', 'Compute', 'System'];

    let toasts = $state([] as {id: string, message: string, type: 'error' | 'success' | 'info'}[]);

    function addToast(message: string, type: 'error' | 'success' | 'info' = 'info') {
        const id = Math.random().toString(36).substring(2);
        toasts = [...toasts, { id, message, type }];
        setTimeout(() => {
            toasts = toasts.filter(t => t.id !== id);
        }, 5000);
    }

    async function fetchNotifications() {
        if (!authState.isLoggedIn) return;
        try {
            const res = await fetch('/api/v1/notifications');
            if (res.ok) {
                const data = await res.json();
                notifications = data.notifications;
            } else {
                const err = await res.json();
                if (res.status !== 401) addToast(err.error || 'Failed to fetch notifications', 'error');
            }
        } catch (e) {
            console.error('Failed to fetch notifications', e);
        }
    }

    async function markAsRead(id: number) {
        try {
            const res = await fetch(`/api/v1/notifications/read/${id}`, { method: 'POST' });
            if (res.ok) {
                await fetchNotifications();
            } else {
                const err = await res.json();
                addToast(err.error || 'Failed to mark as read', 'error');
            }
        } catch (e) {
            addToast('Network error while marking as read', 'error');
        }
    }

    async function clearAllNotifications() {
        try {
            const res = await fetch('/api/v1/notifications/clear', { method: 'POST' });
            if (res.ok) {
                notifications = [];
                notificationsOpen = false;
                addToast('All notifications cleared', 'success');
            } else {
                const err = await res.json();
                addToast(err.error || 'Failed to clear notifications', 'error');
            }
        } catch (e) {
            addToast('Network error while clearing notifications', 'error');
        }
    }

    let notificationsOpen = $state(false);
    let unreadCount = $derived(notifications.filter(n => !n.is_read).length);

    const docSections = [
        { id: 'intro', title: 'Introduction', icon: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z' },
        { id: 'firewall', title: 'Firewall & Security', icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04M12 21a8.966 8.966 0 01-5.917-2.24L6 18c.633-1.037 1.258-2.001 2.5-2.001h7c1.242 0 1.867.964 2.5 2.001l.083.136A8.966 8.966 0 0112 21z' },
        { id: 'network', title: 'WAN & Routing', icon: 'M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9' },
        { id: 'services', title: 'DHCP & DNS', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10' },
        { id: 'hotspot', title: 'Hotspot & RADIUS', icon: 'M12 11c0 3.517-1.009 6.799-2.753 9.571m-3.44-2.04l.054-.09A10.003 10.003 0 0012 21a10.003 10.003 0 008.139-4.187l.054.09A10.003 10.003 0 0112 21c-4.47 0-8.268-2.943-9.542-7' },
        { id: 'compute', title: 'Compute & Cloud', icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2' },
        { id: 'automation', title: 'Automation (Rhai)', icon: 'M13 10V3L4 14h7v7l9-11h-7z' },
        { id: 'api', title: 'Developer API', icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4' }
    ];
</script>

{#if !authState.isInitialized}
    <div class="min-h-screen bg-[#F0F2F5] flex items-center justify-center">
        <div class="text-[#1E88E5] animate-pulse text-sm font-semibold tracking-widest uppercase">Initializing Enterprise Core...</div>
    </div>
{:else}
    {#if page.url.pathname === '/login'}
        {@render children()}
    {:else}
        <div class="flex h-screen w-full bg-[#F8FAFC] text-[#1E293B] font-sans overflow-hidden">
            
            <!-- SIDEBAR: Enterprise Solid Navigation -->
            <aside 
                class="flex flex-col bg-white border-r border-slate-200 text-slate-600 transition-all duration-300 z-[100] shrink-0 relative shadow-xl"
                style="width: {sidebarOpen ? '240px' : '64px'}"
            >
                <!-- TOP LOGO -->
                <div class="h-16 flex items-center justify-center bg-white border-b border-slate-100 transition-all">
                    {#if sidebarOpen}
                        <div class="flex items-center gap-2">
                             <div class="w-8 h-8 rounded-lg bg-[#1E88E5] flex items-center justify-center font-black text-white text-lg shadow-[0_4px_12px_rgba(30,136,229,0.3)]">R</div>
                             <span class="font-bold tracking-widest text-sm text-slate-800 uppercase">ROUMAN CORE</span>
                        </div>
                    {:else}
                          <div class="w-8 h-8 rounded-lg bg-[#1E88E5] flex items-center justify-center font-black text-white text-lg">R</div>
                    {/if}
                </div>

                <!-- NAVIGATION ITEMS -->
                <nav class="flex-1 flex flex-col gap-1 px-3 py-6 overflow-y-auto no-scrollbar">
                    {#if page.url.pathname === '/docs'}
                        <div class="mt-4 mb-2 px-3">
                            <span class="text-[10px] font-black text-[#1E88E5] tracking-[0.2em] uppercase">{sidebarOpen ? 'Documentation' : 'DOCS'}</span>
                        </div>
                        {#each docSections as section}
                            <a 
                                href="/docs?s={section.id}"
                                class="flex items-center gap-4 px-3 py-2.5 rounded-xl transition-all group/item {page.url.searchParams.get('s') === section.id || (!page.url.searchParams.get('s') && section.id === 'intro') ? 'bg-[#1E88E5] text-white shadow-[0_8px_20px_rgba(30,136,229,0.25)]' : 'text-slate-500 hover:text-slate-900 hover:bg-slate-50'}"
                                title={section.title}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={section.icon} />
                                </svg>
                                {#if sidebarOpen}
                                    <span class="text-xs font-bold tracking-wide transition-opacity duration-300 opacity-100">{section.title}</span>
                                {/if}
                            </a>
                        {/each}
                        <div class="mt-auto pt-6 border-t border-slate-50">
                            <button 
                                onclick={() => goto('/')}
                                class="flex items-center gap-4 px-3 py-2.5 rounded-xl transition-all text-slate-400 hover:text-[#1E88E5] hover:bg-slate-50 w-full"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" /></svg>
                                {#if sidebarOpen}
                                    <span class="text-xs font-bold">Dashboard</span>
                                {/if}
                            </button>
                        </div>
                    {:else}
                        {#each categories as cat}
                            {#if sidebarOpen}
                                <div class="mt-4 mb-2 px-3">
                                    <span class="text-[10px] font-black text-slate-400 tracking-[0.2em] uppercase">{cat}</span>
                                </div>
                            {/if}
                            {#each filteredApps.filter(a => a.category === cat) as app}
                                <button 
                                    onclick={() => { activeAppId = app.id; if (page.url.pathname !== '/') goto('/'); }}
                                    class="flex items-center gap-4 px-3 py-2.5 rounded-xl transition-all group/item {activeAppId === app.id && page.url.pathname === '/' ? 'bg-[#1E88E5] text-white shadow-[0_8px_20px_rgba(30,136,229,0.25)]' : 'text-slate-500 hover:text-slate-900 hover:bg-slate-50'}"
                                    title={app.title}
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={app.icon} />
                                    </svg>
                                    {#if sidebarOpen}
                                        <span class="text-xs font-bold tracking-wide transition-opacity duration-300 opacity-100">{app.title}</span>
                                    {/if}
                                </button>
                            {/each}
                        {/each}
                    {/if}
                </nav>

                <!-- BOTTOM PROFILE / STATUS -->
                <div class="p-4 border-t border-slate-100 bg-slate-50/50 flex flex-col gap-3">
                    <div class="flex items-center gap-3">
                        <div class="w-2.5 h-2.5 shrink-0 rounded-full {internetStatus === 'ONLINE' ? 'bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.4)]' : 'bg-red-500 shadow-[0_0_12px_rgba(239,68,68,0.4)]'}"></div>
                        {#if sidebarOpen}
                            <div class="flex flex-col text-left">
                                <span class="text-[9px] text-slate-400 font-black uppercase tracking-widest">Master Node</span>
                                <span class="text-[10px] font-bold {internetStatus === 'ONLINE' ? 'text-emerald-600' : 'text-red-600'}">{internetStatus}</span>
                            </div>
                        {/if}
                    </div>

                    <button 
                        class="mt-2 w-full flex items-center justify-center p-2.5 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-xl transition-all" 
                        onclick={logout}
                        title="Logout"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 min-w-[20px]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" /></svg>
                        {#if sidebarOpen}
                            <span class="text-[10px] font-black ml-2 tracking-widest uppercase">LOGOUT_</span>
                        {/if}
                    </button>
                </div>
            </aside>

            <!-- MAIN CONTENT AREA -->
            <main class="flex-1 flex flex-col h-screen overflow-hidden overflow-y-auto w-full relative bg-[#F8FAFC]">
                <!-- TOPBAR (Title + Sidebar Toggle) -->
                <header class="h-16 bg-white/80 backdrop-blur-xl border-b border-slate-200 flex items-center justify-between px-8 shrink-0 relative z-20">
                    <div class="flex items-center gap-6">
                        <button onclick={() => sidebarOpen = !sidebarOpen} class="text-slate-400 hover:text-[#1E88E5] transition-colors p-2 hover:bg-slate-50 rounded-lg">
                             <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 transition-transform duration-300 {sidebarOpen ? '' : 'rotate-180'}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                             </svg>
                        </button>
                        <h1 class="text-sm font-black text-slate-800 tracking-[0.2em] uppercase">
                            {page.url.pathname === '/docs' ? 'Documentation' : activeAppMeta.title}
                        </h1>
                    </div>
                    
                    <!-- Topbar Widgets -->
                    <div class="flex items-center gap-6">
                        <!-- Documentation Icon -->
                        <a 
                            href="/docs" 
                            class="text-slate-400 hover:text-[#1E88E5] transition-all p-2 hover:bg-slate-50 rounded-xl relative group flex items-center justify-center"
                            title="Documentation"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                            </svg>
                        </a>

                        <!-- Notification Icon -->
                        <div class="relative">
                            <button 
                                onclick={() => notificationsOpen = !notificationsOpen}
                                class="text-slate-400 hover:text-[#1E88E5] transition-all p-2 hover:bg-slate-50 rounded-xl relative group flex items-center justify-center"
                                title="Notifications"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                </svg>
                                {#if unreadCount > 0}
                                    <span class="absolute top-2 right-2 w-1.5 h-1.5 bg-red-500 rounded-full border border-white"></span>
                                {/if}
                            </button>

                            {#if notificationsOpen}
                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                <div class="fixed inset-0 z-[190]" onclick={() => notificationsOpen = false}></div>
                                <div 
                                    transition:fly={{ y: 10, duration: 200 }}
                                    class="absolute right-0 mt-4 w-80 bg-white/90 backdrop-blur-2xl border border-slate-200 rounded-2xl shadow-[0_20px_50px_rgba(0,0,0,0.15)] overflow-hidden z-[200]"
                                >
                                    <div class="p-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/30">
                                        <span class="text-[10px] font-black text-slate-800 tracking-[0.2em] uppercase">Notifications ({unreadCount})</span>
                                        <button 
                                            class="text-[9px] font-bold text-[#1E88E5] hover:underline uppercase tracking-widest disabled:opacity-50"
                                            onclick={clearAllNotifications}
                                            disabled={notifications.length === 0}
                                        >
                                            Clear All
                                        </button>
                                    </div>
                                    <div class="max-h-[360px] overflow-y-auto no-scrollbar">
                                        {#if notifications.length === 0}
                                            <div class="p-12 text-center flex flex-col items-center gap-3">
                                                <div class="w-10 h-10 rounded-full bg-slate-50 flex items-center justify-center text-slate-300">
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" /></svg>
                                                </div>
                                                <span class="text-[10px] text-slate-400 font-black uppercase tracking-widest">Everything is quiet</span>
                                            </div>
                                        {:else}
                                            {#each notifications as note}
                                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                                <div 
                                                    class="p-4 border-b border-slate-50 hover:bg-slate-50/80 transition-colors cursor-pointer group/note relative overflow-hidden {note.is_read ? 'opacity-60' : ''}"
                                                    onclick={() => markAsRead(note.id)}
                                                >
                                                    <div class="flex gap-3 relative z-10">
                                                        <div class="w-2 h-2 rounded-full mt-1.5 shrink-0 {note.type === 'warning' ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.4)]' : note.type === 'success' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]' : 'bg-[#1E88E5] shadow-[0_0_8px_rgba(30,136,229,0.4)]'}"></div>
                                                        <div class="flex-1 flex flex-col gap-1">
                                                            <div class="flex items-center justify-between">
                                                                <span class="text-[11px] font-black text-slate-800 tracking-wide uppercase leading-none">{note.title}</span>
                                                                <span class="text-[9px] text-slate-400 font-bold uppercase tracking-tighter">
                                                                    {new Date(note.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                                                </span>
                                                            </div>
                                                            <p class="text-[11px] text-slate-500 leading-relaxed font-medium">{note.message}</p>
                                                        </div>
                                                    </div>
                                                    {#if !note.is_read}
                                                        <div class="absolute inset-y-0 left-0 w-0.5 bg-[#1E88E5] scale-y-100 transition-transform origin-top duration-300"></div>
                                                    {/if}
                                                </div>
                                            {/each}
                                        {/if}
                                    </div>
                                    <button class="w-full p-4 text-[10px] font-black text-slate-500 hover:text-[#1E88E5] hover:bg-[#1E88E5]/5 transition-all border-t border-slate-100 uppercase tracking-widest">
                                        View System Log
                                    </button>
                                </div>
                            {/if}
                        </div>

                        <div class="flex flex-col text-right">
                             <span class="text-[9px] text-slate-400 font-black uppercase tracking-widest leading-tight">Infrastructure Core</span>
                             <span class="text-[11px] text-[#1E88E5] font-black uppercase tracking-widest">Rouman Gateway Pro</span>
                        </div>
                    </div>
                </header>

                <!-- DYNAMIC APPLICATION CONTAINER -->
                <div class="p-8 w-full max-w-7xl mx-auto flex-1">
                    {#if page.url.pathname === '/docs'}
                        {@render children()}
                    {:else}
                        <svelte:component this={COMPONENT_MAP[activeAppMeta.componentPath]} />
                    {/if}
                </div>
            </main>
        </div>
    {/if}
{/if}

<!-- TOAST CONTAINER -->
<div class="fixed bottom-8 right-8 z-[999] flex flex-col gap-3 pointer-events-none">
    {#each toasts as toast (toast.id)}
        <div 
            transition:fly={{ x: 50, duration: 300 }}
            class="pointer-events-auto min-w-[300px] p-4 rounded-2xl shadow-[0_10px_30px_rgba(0,0,0,0.1)] border flex items-center gap-3 backdrop-blur-xl
            {toast.type === 'error' ? 'bg-red-50/90 border-red-100 text-red-600' : toast.type === 'success' ? 'bg-emerald-50/90 border-emerald-100 text-emerald-600' : 'bg-white/90 border-slate-100 text-slate-600'}"
        >
            <div class="shrink-0 w-8 h-8 rounded-full flex items-center justify-center {toast.type === 'error' ? 'bg-red-100' : toast.type === 'success' ? 'bg-emerald-100' : 'bg-slate-100'}">
                {#if toast.type === 'error'}
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                {:else if toast.type === 'success'}
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                {:else}
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                {/if}
            </div>
            <span class="text-xs font-bold tracking-wide">{toast.message}</span>
        </div>
    {/each}
</div>

<style>
    .no-scrollbar::-webkit-scrollbar {
        display: none;
    }
    .no-scrollbar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>
