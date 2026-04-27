<script lang="ts">
    import { onMount } from 'svelte';

    let config = $state({ network: { simple_queues: [] } } as any);
    let loading = $state(true);
    let saving = $state(false);
    let message = $state("");

    async function fetchData() {
        try {
            const res = await fetch('/api/v1/config/active');
            if (res.ok) {
                const data = await res.json();
                config = data.active;
                if (!config.network.simple_queues) {
                    config.network.simple_queues = [];
                }
            }
        } catch (e) {
            console.error("Failed to fetch queues data", e);
        } finally {
            loading = false;
        }
    }

    async function saveConfig() {
        saving = true;
        message = "";
        try {
            const res = await fetch('/api/v1/config/candidate', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(config)
            });
            
            if (res.ok) {
                await fetch('/api/v1/config/commit', { method: 'POST' });
                message = "Queues applied successfully!";
                fetchData();
            }
        } catch (e) {
            message = "Error applying changes";
        } finally {
            saving = false;
        }
    }

    function addQueue() {
        config.network.simple_queues = [...config.network.simple_queues, {
            name: 'New Queue',
            target: '192.168.1.0/24',
            download_mbps: 10,
            upload_mbps: 5,
            enabled: true
        }];
    }

    onMount(fetchData);
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Simple Queues_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Bandwidth Management</p>
        </div>
        
        <button 
            onclick={saveConfig}
            disabled={saving}
            class="rouman-btn-primary flex items-center gap-2"
        >
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Applying...</span>
            {:else}
                <span>💾 Apply Changes_</span>
            {/if}
        </button>
    </div>

    {#if message}
        <div class="p-3 bg-emerald-50 border border-emerald-100 text-emerald-600 text-[9px] font-black uppercase tracking-widest text-center rounded-xl shadow-sm">
            {message}
        </div>
    {/if}

    <div class="rouman-card flex flex-col gap-8 !bg-white/60">
        <div class="flex justify-between items-center border-b border-slate-50 pb-5">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">📊</div>
                <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Queue Rules_</h4>
            </div>
            <button onclick={addQueue} class="px-4 py-1.5 bg-sky-50 text-sky-600 rounded-full font-black text-[10px] uppercase hover:bg-sky-100 transition-all border border-sky-100 shadow-sm">
                + Add Queue
            </button>
        </div>

        <div class="flex flex-col gap-5 overflow-auto pr-2 custom-scrollbar max-h-[600px]">
            {#each config.network.simple_queues as queue, i}
                <div class="bg-slate-50/50 border border-slate-100 p-6 rounded-2xl flex flex-col gap-6 group relative hover:border-sky-200 hover:bg-white transition-all shadow-sm">
                    <div class="flex justify-between items-center">
                        <input bind:value={queue.name} class="bg-transparent border-none text-sky-600 font-black tracking-widest uppercase outline-none focus:text-sky-700 transition-colors text-[12px] w-1/2" />
                        <div class="flex items-center gap-6">
                            <button 
                                onclick={() => queue.enabled = !queue.enabled} 
                                class={`text-[9px] font-black px-4 py-1 rounded-full shadow-sm transition-all ${queue.enabled ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-100 text-slate-400 border border-slate-200'}`}
                            >
                                {queue.enabled ? 'ACTIVE' : 'PAUSED'}
                            </button>
                            <button onclick={() => config.network.simple_queues.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                            </button>
                        </div>
                    </div>
                    
                    <div class="grid grid-cols-3 gap-6">
                        <div class="flex flex-col gap-2">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Target IP / CIDR</label>
                            <input bind:value={queue.target} class="rouman-input !py-1.5 !text-slate-800 font-black font-mono" />
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="text-[9px] text-emerald-600 uppercase font-black tracking-widest px-1">Max Download (Mbps)</label>
                            <input type="number" bind:value={queue.download_mbps} class="rouman-input !py-1.5 !text-emerald-600 font-black font-mono" />
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="text-[9px] text-orange-600 uppercase font-black tracking-widest px-1">Max Upload (Mbps)</label>
                            <input type="number" bind:value={queue.upload_mbps} class="rouman-input !py-1.5 !text-orange-600 font-black font-mono" />
                        </div>
                    </div>
                </div>
            {/each}
            {#if config.network.simple_queues.length === 0}
                <div class="py-24 text-center text-slate-300 font-black uppercase tracking-[0.3em] bg-slate-50/30 rounded-3xl border-2 border-dashed border-slate-100">
                    No active limit rules configured
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
        background: rgba(0, 0, 0, 0.08);
        border-radius: 10px;
    }
</style>
