<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    
    let telemetryInterval: number | ReturnType<typeof setInterval>;
    let telemetryData = $state({
        cpu: 0,
        memory_total: 1,
        memory_used: 0,
        uptime: 0
    });

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

<div class="w-full max-w-5xl mx-auto flex flex-col gap-10">
    <div class="flex flex-col items-center justify-center p-6 bg-black/40 border border-neon-green/20 rounded-xl backdrop-blur-sm shadow-[0_0_30px_rgba(0,255,196,0.05)] w-full max-w-2xl mx-auto mt-6">
        <div class="flex items-center justify-center gap-8 w-full">
            <div class="flex flex-col items-center flex-1">
                <span class="text-[10px] text-neon-green/60 uppercase tracking-widest">CPU LOAD</span>
                <span class="text-2xl font-bold text-white my-2">{telemetryData.cpu.toFixed(1)}%</span>
                <div class="w-full h-1 bg-neon-green/10 rounded overflow-hidden">
                    <div class="h-full bg-neon-green transition-all duration-300" style="width: {Math.min(telemetryData.cpu, 100)}%"></div>
                </div>
            </div>
            
            <div class="w-px h-12 bg-neon-green/20"></div>

            <div class="flex flex-col items-center flex-1">
                <span class="text-[10px] text-neon-green/60 uppercase tracking-widest">MEMORY</span>
                <span class="text-2xl font-bold text-white my-2">{((telemetryData.memory_used / Math.max(telemetryData.memory_total, 1)) * 100).toFixed(1)}%</span>
                <span class="text-[9px] text-neon-green/40">{telemetryData.memory_used} MB / {telemetryData.memory_total} MB</span>
            </div>

            <div class="w-px h-12 bg-neon-green/20"></div>

            <div class="flex flex-col items-center flex-1">
                <span class="text-[10px] text-neon-green/60 uppercase tracking-widest">UPTIME</span>
                <span class="text-2xl font-bold text-neon-green my-2">{formatUptime(telemetryData.uptime)}</span>
            </div>
        </div>
    </div>
</div>
