<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    
    let telemetryInterval: number | ReturnType<typeof setInterval>;
    let telemetryData = $state({
        cpu: 0,
        memory_total: 1,
        memory_used: 0,
        uptime: 0
    });
    let internetStatus = $state("CHECKING...");

    function formatUptime(seconds: number) {
        if (!seconds) return '0h 0m';
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        return `${h}h ${m}m`;
    }

    async function checkTelemetry() {
        try {
            const res = await fetch('/api/v1/system/telemetry');
            if (res.ok) {
                telemetryData = await res.json();
            }

            const statusRes = await fetch('/api/v1/system/internet-status');
            if (statusRes.ok) {
                const data = await statusRes.json();
                internetStatus = data.status;
            }
        } catch {}
    }

    onMount(() => {
        checkTelemetry();
        telemetryInterval = setInterval(checkTelemetry, 1500);
    });

    onDestroy(() => {
        if (telemetryInterval) clearInterval(telemetryInterval);
    });
</script>

<div class="w-full max-w-6xl mx-auto flex flex-col gap-8 p-4">
    <!-- HERO SECTION -->
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-black text-slate-800 tracking-tight">System Dashboard_</h1>
            <p class="text-slate-400 text-sm mt-1">Real-time health and telemetry metrics.</p>
        </div>
        <div class="flex items-center gap-4">
             <div class="flex items-center gap-2 px-4 py-2 bg-white/60 border border-white shadow-sm rounded-2xl backdrop-blur-xl">
                <div class="w-2 h-2 rounded-full {internetStatus === 'CONNECTED' ? 'bg-emerald-500 animate-pulse' : 'bg-rose-500'}"></div>
                <span class="text-[10px] font-black tracking-widest {internetStatus === 'CONNECTED' ? 'text-emerald-600' : 'text-rose-600'}">INTERNET: {internetStatus}</span>
             </div>
        </div>
    </div>

    <!-- MAIN METRICS GRID -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- CPU CARD -->
        <div class="rouman-card flex flex-col gap-4">
            <div class="flex items-center justify-between">
                <span class="text-[10px] text-slate-400 uppercase font-black tracking-widest">Processing Power</span>
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-sky-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" /></svg>
            </div>
            <div class="flex flex-col">
                <span class="text-4xl font-black text-slate-800 tracking-tight">{telemetryData.cpu.toFixed(1)}%</span>
                <span class="text-[11px] text-slate-400 mt-1 uppercase font-bold tracking-wider">Average CPU Load</span>
            </div>
            <div class="w-full h-2 bg-slate-100 rounded-full overflow-hidden mt-2">
                <div class="h-full bg-sky-500 transition-all duration-1000" style="width: {Math.min(telemetryData.cpu, 100)}%"></div>
            </div>
        </div>

        <!-- MEMORY CARD -->
        <div class="rouman-card flex flex-col gap-4">
             <div class="flex items-center justify-between">
                <span class="text-[10px] text-slate-400 uppercase font-black tracking-widest">Memory Utilization</span>
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>
            </div>
            <div class="flex flex-col">
                <span class="text-4xl font-black text-slate-800 tracking-tight">{((telemetryData.memory_used / Math.max(telemetryData.memory_total, 1)) * 100).toFixed(1)}%</span>
                <span class="text-[11px] text-slate-400 mt-1 uppercase font-bold tracking-wider">{telemetryData.memory_used} MB / {telemetryData.memory_total} MB Used</span>
            </div>
            <div class="w-full h-2 bg-slate-100 rounded-full overflow-hidden mt-2">
                <div class="h-full bg-emerald-500 transition-all duration-1000" style="width: {(telemetryData.memory_used / Math.max(telemetryData.memory_total, 1)) * 100}%"></div>
            </div>
        </div>

        <!-- UPTIME CARD -->
        <div class="rouman-card flex flex-col gap-4">
             <div class="flex items-center justify-between">
                <span class="text-[10px] text-slate-400 uppercase font-black tracking-widest">System Longevity</span>
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            </div>
            <div class="flex flex-col">
                <span class="text-4xl font-black text-slate-800 tracking-tight">{formatUptime(telemetryData.uptime)}</span>
                <span class="text-[11px] text-slate-400 mt-1 uppercase font-bold tracking-wider">Total Time Online</span>
            </div>
            <div class="flex gap-1 mt-2">
                {#each Array(10) as _, i}
                    <div class="h-1.5 flex-1 rounded-full {i < 8 ? 'bg-amber-400/30' : 'bg-slate-100'}"></div>
                {/each}
            </div>
        </div>
    </div>
</div>

<style>
    :global(body) {
        background-color: #F8FAFC;
    }
</style>
