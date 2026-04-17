<script lang="ts">
    import { onMount, onDestroy } from 'svelte';

    let leases = $state([] as any[]);
    let loading = $state(true);
    let interval: number | ReturnType<typeof setInterval>;

    async function fetchLeases() {
        try {
            const res = await fetch('/api/v1/network/leases');
            if (res.ok) {
                const data = await res.json();
                leases = data.leases || [];
            }
        } catch (e) {
            console.error('Failed to fetch leases:', e);
        } finally {
            loading = false;
        }
    }

    function formatTimeRemaining(expiryTime: number) {
        const now = Math.floor(Date.now() / 1000);
        const remaining = expiryTime - now;
        if (remaining <= 0) return 'EXPIRED';
        
        const hours = Math.floor(remaining / 3600);
        const minutes = Math.floor((remaining % 3600) / 60);
        const seconds = remaining % 60;
        
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }

    onMount(() => {
        fetchLeases();
        interval = setInterval(fetchLeases, 5000);
    });

    onDestroy(() => {
        if (interval) clearInterval(interval);
    });
</script>

<svelte:head>
    <title>CONNECTED DEVICES // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-6xl mx-auto mt-6 mb-10 flex flex-col gap-6">
    <div class="flex items-center justify-between mb-8">
        <div class="flex flex-col gap-1">
            <h2 class="text-3xl text-white font-bold tracking-[0.2em] uppercase">Connected Devices</h2>
            <p class="text-[10px] text-neon-green/60 tracking-widest uppercase">Real-time DHCP Lease Monitoring</p>
        </div>
        <div class="px-4 py-2 rounded text-[10px] font-bold tracking-widest uppercase border border-neon-green/40 text-neon-green bg-neon-green/5 flex items-center gap-3">
            <div class="w-2 h-2 rounded-full bg-neon-green animate-pulse"></div>
            NATIVE STACK ACTIVE
        </div>
    </div>

    <!-- Stats Overview -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-4">
        <div class="p-6 rounded-xl border border-white/5 bg-white/[0.02] backdrop-blur-md flex flex-col gap-1">
            <span class="text-[10px] text-gray-500 uppercase tracking-widest">Active Leases</span>
            <span class="text-3xl font-bold text-white font-mono">{leases.length}</span>
        </div>
        <div class="p-6 rounded-xl border border-white/5 bg-white/[0.02] backdrop-blur-md flex flex-col gap-1">
            <span class="text-[10px] text-gray-500 uppercase tracking-widest">DHCP Status</span>
            <span class="text-xs font-bold text-neon-green tracking-widest uppercase mt-2">Operational</span>
        </div>
        <div class="p-6 rounded-xl border border-white/5 bg-white/[0.02] backdrop-blur-md flex flex-col gap-1">
            <span class="text-[10px] text-gray-500 uppercase tracking-widest">Server Protocol</span>
            <span class="text-xs font-bold text-white tracking-widest uppercase mt-2">Rust Native UDP/67</span>
        </div>
    </div>

    <!-- Leases Table -->
    <div class="rounded-2xl border border-white/10 bg-black/40 backdrop-blur-xl overflow-hidden shadow-2xl">
        <div class="overflow-x-auto">
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b border-white/10 bg-white/[0.02]">
                        <th class="px-6 py-4 text-[10px] font-bold text-gray-400 uppercase tracking-widest">Hostname</th>
                        <th class="px-6 py-4 text-[10px] font-bold text-gray-400 uppercase tracking-widest">IP Address</th>
                        <th class="px-6 py-4 text-[10px] font-bold text-gray-400 uppercase tracking-widest">MAC Address</th>
                        <th class="px-6 py-4 text-[10px] font-bold text-gray-400 uppercase tracking-widest text-right">Time Remaining</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-white/5">
                    {#if loading && leases.length === 0}
                        <tr>
                            <td colspan="4" class="px-6 py-20 text-center text-neon-green/40 animate-pulse tracking-[0.2em] text-xs uppercase">
                                Polling DHCP Controller...
                            </td>
                        </tr>
                    {:else if leases.length === 0}
                        <tr>
                            <td colspan="4" class="px-6 py-20 text-center text-gray-600 tracking-widest text-xs uppercase italic">
                                No active devices detected on network
                            </td>
                        </tr>
                    {:else}
                        {#each leases as lease}
                            <tr class="hover:bg-white/[0.03] transition-colors group">
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="flex items-center gap-3">
                                        <div class="w-8 h-8 rounded bg-neon-green/10 flex items-center justify-center text-neon-green">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" /></svg>
                                        </div>
                                        <span class="text-sm font-bold text-white tracking-wide group-hover:text-neon-green transition-colors">{lease.hostname || 'UNRESOLVED'}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class="px-2 py-1 bg-white/5 rounded text-xs font-mono text-neon-green border border-white/5">{lease.ip}</span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class="text-xs font-mono text-gray-400 group-hover:text-white transition-colors">{lease.mac}</span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-right">
                                    <span class="text-xs font-mono {formatTimeRemaining(lease.expires) === 'EXPIRED' ? 'text-red-500' : 'text-gray-300'}">
                                        {formatTimeRemaining(lease.expires)}
                                    </span>
                                </td>
                            </tr>
                        {/each}
                    {/if}
                </tbody>
            </table>
        </div>
    </div>

    <!-- Alert / Tips -->
    <div class="mt-4 p-4 rounded-xl border border-neon-green/20 bg-neon-green/5 flex gap-4 items-center">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-neon-green shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-[10px] text-neon-green/70 uppercase tracking-widest leading-relaxed">
            Hint: In Rouman, leases are tracked in real-time memory. Expired entries are automatically flushed from the dashboard every 5 seconds.
        </p>
    </div>
</div>
