<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let neighbors = $state([] as any[]);
    let loading = $state(true);
    let errorMsg = $state("");
    let interval: any;

    async function fetchNeighbors() {
        try {
            const res = await fetch('/api/v1/network/neighbors');
            if (res.ok) {
                const data = await res.json();
                neighbors = data.neighbors;
            } else {
                neighbors = [];
                errorMsg = "Failed to fetch neighbors from server.";
            }
        } catch (e) {
            neighbors = [];
            errorMsg = "Network scan unreachable.";
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        fetchNeighbors();
        interval = setInterval(fetchNeighbors, 15000); // refresh every 15s
    });

    onDestroy(() => {
        if (interval) clearInterval(interval);
    });
</script>

<ServiceStarter featurePath="rdp.enabled" title="Neighbor Discovery" description="Rouman Discovery Protocol (RDP) for identifying edge nodes and routers in the network.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-emerald-600">Network Discovery_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">L2 Neighbor & Infrastructure Intelligence</p>
        </div>
        
        <button onclick={fetchNeighbors} class="rouman-btn-primary flex items-center gap-2">
            <span>📡</span> Force Scan_
        </button>
    </div>

    <div class="rouman-card flex flex-col gap-8 !bg-white/60">
        <div class="flex justify-between items-center border-b border-slate-50 pb-5">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">🌐</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Discovered Nodes</h4>
            </div>
            <div class="flex gap-2">
                <span class="text-[9px] bg-slate-50 text-slate-400 px-4 py-1.5 rounded-full font-black uppercase tracking-widest border border-slate-100 shadow-sm">COMPUTE_INTEGRATED</span>
                <span class="text-[9px] bg-emerald-50 text-emerald-600 px-4 py-1.5 rounded-full font-black uppercase tracking-widest border border-emerald-100 shadow-sm">RDP_ACTIVE</span>
            </div>
        </div>

        {#if loading}
            <div class="py-32 flex flex-col items-center gap-6">
                <div class="w-12 h-12 border-4 border-emerald-600 border-t-transparent rounded-full animate-spin shadow-lg"></div>
                <div class="text-[10px] text-emerald-600 font-black uppercase tracking-[0.4em] animate-pulse">Probing Layer 2 Fabric Matrix_</div>
            </div>
        {:else if neighbors.length === 0}
            <div class="py-32 flex flex-col items-center gap-6 border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                <span class="text-4xl grayscale opacity-20">🔍</span>
                <span class="text-[11px] text-slate-300 font-black uppercase tracking-[0.3em]">No neighbor infrastructure detected</span>
            </div>
        {:else}
            <div class="overflow-x-auto custom-scrollbar">
                <table class="w-full text-left border-collapse">
                    <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                        <tr>
                            <th class="p-4 px-2 font-black">Network Identity</th>
                            <th class="p-4 px-2 font-black">Type</th>
                            <th class="p-4 px-2 font-black">Layer 3 Addr</th>
                            <th class="p-4 px-2 font-black">Hardware ID</th>
                            <th class="p-4 px-2 font-black">OS Platform</th>
                            <th class="p-4 text-right pr-2 font-black">Status</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-600 font-mono text-[10px]">
                        {#each neighbors as nb}
                            <tr class="border-b border-slate-50 hover:bg-slate-50/50 transition-all group">
                                <td class="p-4 px-2">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="font-black text-slate-800 text-[11px] group-hover:text-emerald-600 transition-colors">{nb.hostname || 'UNDEFINED_ID'}</span>
                                        <span class="text-[8px] text-slate-400 font-black uppercase tracking-tighter">ROUMAN_COMPATIBLE_NODE</span>
                                    </div>
                                </td>
                                <td class="p-4 px-2">
                                    <span class="text-[9px] font-black uppercase px-2 py-0.5 rounded-md border 
                                        {nb.neighbor_type === 'MicroVM' ? 'bg-sky-50 text-sky-600 border-sky-100' : 
                                         nb.neighbor_type === 'Container' ? 'bg-indigo-50 text-indigo-600 border-indigo-100' : 
                                         'bg-slate-50 text-slate-600 border-slate-100'}">
                                        {nb.neighbor_type || 'Physical'}
                                    </span>
                                </td>
                                <td class="p-4 px-2 text-slate-800 font-black">{nb.ip || '-'}</td>
                                <td class="p-4 px-2 text-slate-400 group-hover:text-slate-600 transition-colors uppercase font-black">{nb.mac}</td>
                                <td class="p-4 px-2 text-slate-400 italic font-medium">{nb.version || '-'}</td>
                                <td class="p-4 text-right pr-2">
                                    <span class="text-[9px] font-black uppercase text-emerald-600 bg-emerald-50 px-2 py-1 rounded-md">Online</span>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </div>
</div>

</ServiceStarter>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
        height: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: rgba(0, 0, 0, 0.02);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(0, 0, 0, 0.08);
        border-radius: 10px;
    }
</style>
