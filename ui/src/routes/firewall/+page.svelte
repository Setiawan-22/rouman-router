<script lang="ts">
    import { onMount } from 'svelte';

    let stats = $state({ status: "loading", dropped_packets: 0 });
    let blacklistIP = $state("");
    let loading = $state(false);

    async function loadStats() {
        try {
            const res = await fetch('/api/v1/firewall/stats');
            if (res.ok) {
                stats = await res.json();
            }
        } catch (e) {
            console.error("Failed to load firewall stats", e);
        }
    }

    async function addToBlacklist() {
        if (!blacklistIP) return;
        loading = true;
        try {
            const res = await fetch('/api/v1/firewall/blacklist', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip: blacklistIP })
            });
            if (res.ok) {
                blacklistIP = "";
                alert("IP Blocked successfully at XDP level!");
            }
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadStats();
        const interval = setInterval(loadStats, 5000);
        return () => clearInterval(interval);
    });
</script>

<svelte:head>
    <title>FIREWALL (eBPF) // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-5xl mx-auto mt-6 mb-10 flex flex-col gap-6">
    <div class="flex items-center justify-between mb-8">
        <h2 class="text-3xl text-white font-bold tracking-[0.2em]">FIREWALL <span class="text-neon-green">XDP</span></h2>
        <div class="px-3 py-1 rounded text-[10px] font-bold tracking-widest uppercase border border-neon-green/40 text-neon-green bg-neon-green/10 shadow-[0_0_15px_rgba(0,255,196,0.1)]">
            KERNEL-SPACE FILTERING
        </div>
    </div>

    <!-- Status Cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div class="p-6 rounded-xl border border-neon-green/20 bg-black/40 backdrop-blur-md">
            <span class="text-[10px] text-gray-500 uppercase tracking-widest">Engine Status</span>
            <div class="text-2xl font-bold text-neon-green mt-2 uppercase tracking-tight">{stats.status}</div>
        </div>
        <div class="p-6 rounded-xl border border-neon-green/20 bg-black/40 backdrop-blur-md">
            <span class="text-[10px] text-gray-500 uppercase tracking-widest">Packets Dropped</span>
            <div class="text-2xl font-bold text-white mt-2 font-mono">{stats.dropped_packets}</div>
        </div>
        <div class="p-6 rounded-xl border border-neon-green/20 bg-black/40 backdrop-blur-md">
            <span class="text-[10px] text-gray-500 uppercase tracking-widest">Active Hooks</span>
            <div class="text-2xl font-bold text-white mt-2 uppercase">XDP_INGRESS</div>
        </div>
    </div>

    <!-- Actions -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mt-4">
        <div class="p-8 rounded-xl border border-red-500/30 bg-red-950/5 shadow-[0_0_40px_rgba(255,0,0,0.05)]">
            <h3 class="text-xl font-bold text-red-500 tracking-widest mb-6">BLOCK SOURCE IP</h3>
            <p class="text-xs text-gray-400 mb-8 leading-relaxed">
                Add an IPv4 address to the hardware-accelerated blacklist. Packets from this source will be dropped instantly at the NIC driver level.
            </p>
            
            <div class="flex flex-col gap-4">
                <input 
                    type="text" 
                    bind:value={blacklistIP}
                    placeholder="e.g. 192.168.1.100"
                    class="bg-black/80 border border-red-500/20 rounded p-4 text-red-400 font-mono text-sm focus:outline-none focus:border-red-500 transition-all shadow-inner"
                />
                <button 
                    onclick={addToBlacklist}
                    disabled={loading}
                    class="w-full py-4 bg-red-600 hover:bg-red-500 text-white rounded font-bold tracking-widest transition-all disabled:opacity-50 uppercase text-xs"
                >
                    {loading ? "COMMUNICATING WITH KERNEL..." : "DROP ALL TRAFFIC FROM IP"}
                </button>
            </div>
        </div>

        <div class="p-8 rounded-xl border border-neon-green/20 bg-black/40 relative overflow-hidden">
             <div class="absolute inset-0 opacity-5 pointer-events-none bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-neon-green via-transparent to-transparent"></div>
             <h3 class="text-xl font-bold text-white tracking-widest mb-6 uppercase">Telemetry Log</h3>
             <div class="bg-black/60 rounded border border-white/5 p-4 h-48 font-mono text-[10px] text-neon-green/60 overflow-y-auto leading-relaxed">
                <div>[INFO] eBPF Program Loaded Successfully</div>
                <div>[INFO] Attaching XDP hook to eth0...</div>
                <div>[INFO] XDP hook attached. Priority: Standard</div>
                <div class="text-neon-green">[LOG] Blacklist MAP initialized (capacity: 1024)</div>
             </div>
        </div>
    </div>
</div>
