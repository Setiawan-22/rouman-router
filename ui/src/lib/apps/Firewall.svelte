<script lang="ts">
    import { onMount, onDestroy } from 'svelte';

    let config = $state({ dns: {}, security: {}, firewall: { blacklist_ips: [] } } as any);
    let stats = $state({ total_packets: 0, dropped_packets: 0, engine: 'None' });
    let loading = $state(true);
    let newIp = $state("");
    let statusInterval: any;

    async function fetchData() {
        try {
            const configRes = await fetch('/api/v1/config/active');
            const configData = await configRes.json();
            config = configData.active;

            const statsRes = await fetch('/api/v1/firewall/stats');
            stats = await statsRes.json();
        } catch (e) {
            console.error("Failed to fetch security data", e);
        } finally {
            loading = false;
        }
    }

    async function addToBlacklist() {
        if (!newIp) return;
        try {
            const res = await fetch('/api/v1/firewall/blacklist', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip: newIp })
            });
            if (res.ok) {
                config.firewall.blacklist_ips = [...config.firewall.blacklist_ips, newIp];
                newIp = "";
            }
        } catch (e) {
            console.error(e);
        }
    }

    onMount(() => {
        fetchData();
        statusInterval = setInterval(fetchData, 5000);
    });

    onDestroy(() => {
        if (statusInterval) clearInterval(statusInterval);
    });
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-red-600">Security Center_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Intrusion Prevention & Global Firewall Logic</p>
        </div>
        
        <div class="px-5 py-2.5 bg-red-50 border border-red-100 text-red-600 rounded-2xl text-[10px] font-black tracking-widest animate-pulse shadow-sm">
            SYSTEM SHIELD: ACTIVE
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
        <!-- Encryption -->
        <div class="rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">🛡️</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Hardware Security</h4>
            </div>
            <div class="flex items-center gap-4 p-4 bg-slate-50/50 rounded-2xl border border-slate-100 shadow-inner">
                 <div class="w-2.5 h-2.5 rounded-full shadow-[0_0_12px_rgba(16,185,129,0.4)] {config.security?.encryption_enabled ? 'bg-emerald-500' : 'bg-red-500'}"></div>
                 <div class="flex flex-col">
                    <span class="font-black text-slate-800 uppercase tracking-widest text-[10px]">{config.security?.encryption_enabled ? 'AES-256-GCM ACTIVE' : 'CIPHER DISABLED'}</span>
                    <span class="text-[8px] text-slate-400 font-bold font-mono tracking-tighter">HW BOUND: {config.security?.hardware_bound ? 'ENABLED' : 'STRICT_OFF'}</span>
                 </div>
            </div>
        </div>

        <!-- Privacy -->
         <div class="rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">🕶️</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">DNS Privacy</h4>
            </div>
            <div class="space-y-4 pt-2">
                <div class="flex justify-between items-center px-3 py-2 bg-slate-50/30 rounded-xl">
                    <span class="text-slate-400 font-black uppercase text-[9px] tracking-widest">DNS-over-HTTPS</span>
                    <span class={`font-black text-[10px] ${config.dns?.doh_enabled ? 'text-emerald-600' : 'text-slate-300'}`}>{config.dns?.doh_enabled ? 'ACTIVE' : 'DORMANT'}</span>
                </div>
                <div class="flex justify-between items-center px-3 py-2 bg-slate-50/30 rounded-xl">
                    <span class="text-slate-400 font-black uppercase text-[9px] tracking-widest">Intercept Mode</span>
                    <span class={`font-black text-[10px] ${config.dns?.transparent_intercept ? 'text-emerald-600' : 'text-slate-300'}`}>{config.dns?.transparent_intercept ? 'STRICT' : 'PASSIVE'}</span>
                </div>
            </div>
        </div>

        <!-- Metrics -->
        <div class="rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <div class="w-8 h-8 rounded-lg bg-amber-50 flex items-center justify-center text-amber-600 shadow-sm">📊</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Packet Metrics</h4>
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5 p-4 bg-slate-50/50 rounded-2xl border border-slate-100 shadow-inner">
                    <span class="text-slate-400 uppercase font-black text-[8px] tracking-tighter">Core Engine</span>
                    <span class="text-[11px] text-amber-600 font-black font-mono tracking-tighter">{stats.engine}</span>
                </div>
                <div class="flex flex-col gap-1.5 p-4 bg-slate-50/50 rounded-2xl border border-slate-100 shadow-inner">
                    <span class="text-slate-400 uppercase font-black text-[8px] tracking-tighter">Threats Dropped</span>
                    <span class="text-[11px] text-red-600 font-black font-mono tracking-tighter">{stats.dropped_packets.toLocaleString()}</span>
                </div>
            </div>
        </div>
    </div>

    <!-- Blacklist Manager -->
    <div class="rouman-card !border-red-100 !bg-red-50/30 flex flex-col gap-8 shadow-sm">
        <div class="flex justify-between items-center border-b border-red-100 pb-5">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-red-100/50 flex items-center justify-center text-red-600 shadow-sm">🔥</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-red-700">Global IP Blacklist</h4>
            </div>
            <span class="text-[9px] text-red-400 font-black uppercase tracking-[0.2em] opacity-60">DROP ENGINE v2</span>
        </div>
        
        <div class="flex gap-4">
            <input 
                bind:value={newIp}
                placeholder="Target IPv4 or CIDR Block..." 
                class="rouman-input !border-red-100 focus:!border-red-300 !text-red-700 !bg-white/80 flex-1 !py-2.5 font-black"
            />
            <button onclick={addToBlacklist} class="rouman-btn-primary !bg-red-600 hover:!bg-red-700 !py-2.5 px-10 text-[10px] font-black tracking-widest">COMMIT DROP_</button>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-5 gap-4 max-h-[350px] overflow-y-auto pr-2 custom-scrollbar">
            {#each config.firewall?.blacklist_ips || [] as ip}
                <div class="flex items-center justify-between p-4 bg-white/80 border border-red-50 rounded-2xl group hover:border-red-300 transition-all shadow-sm">
                    <span class="font-mono text-red-700 font-black group-hover:text-red-600 transition-colors text-[10px] tracking-tighter">{ip}</span>
                    <button class="text-slate-300 hover:text-red-500 transition-colors font-black text-[12px] p-1.5 hover:bg-red-50 rounded-lg">×</button>
                </div>
            {/each}
            {#if (config.firewall?.blacklist_ips || []).length === 0}
                <div class="col-span-full py-16 text-center text-slate-300 font-black uppercase tracking-[0.3em] border-2 border-dashed border-red-100 rounded-3xl bg-red-50/20">
                    No active network blocks found
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: rgba(0, 0, 0, 0.02);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(239, 68, 68, 0.1);
        border-radius: 10px;
    }
</style>
