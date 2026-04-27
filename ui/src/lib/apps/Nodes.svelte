<script lang="ts">
    import { onMount } from 'svelte';

    let nodes = $state([] as any[]);
    let loading = $state(true);
    let newNode = $state({ hostname: '', ip: '' });

    async function fetchNodes() {
        try {
            const res = await fetch('/api/v1/compute/nodes');
            if (res.ok) {
                const data = await res.json();
                nodes = data.nodes;
            }
        } catch (e) {
            console.error('Failed to fetch compute nodes', e);
        } finally {
            loading = false;
        }
    }

    async function addNode() {
        if (!newNode.hostname || !newNode.ip) return;
        try {
            const res = await fetch('/api/v1/compute/nodes', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    id: Math.random().toString(36).substring(2),
                    hostname: newNode.hostname,
                    ip: newNode.ip,
                    status: 'Probing',
                    is_local: false
                })
            });
            if (res.ok) {
                newNode = { hostname: '', ip: '' };
                await fetchNodes();
            }
        } catch (e) {
            console.error('Failed to add compute node', e);
        }
    }

    onMount(fetchNodes);
</script>

<div class="rouman-card flex flex-col gap-8 !bg-white/60">
    <div class="flex justify-between items-center border-b border-slate-50 pb-5">
        <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">🖥️</div>
            <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Infrastructure Nodes</h4>
        </div>
        <div class="flex gap-4">
            <input 
                type="text" 
                placeholder="Hostname" 
                bind:value={newNode.hostname}
                class="rouman-input !py-1.5 !w-32"
            />
            <input 
                type="text" 
                placeholder="IP Address" 
                bind:value={newNode.ip}
                class="rouman-input !py-1.5 !w-32"
            />
            <button 
                onclick={addNode}
                class="px-5 py-2 bg-sky-600 text-white rounded-xl text-[10px] font-black tracking-widest uppercase hover:bg-sky-700 transition-all shadow-lg"
            >
                + Add Agent_
            </button>
        </div>
    </div>

    {#if loading}
        <div class="py-12 flex justify-center">
            <div class="w-8 h-8 border-2 border-sky-600 border-t-transparent rounded-full animate-spin"></div>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {#each nodes as node}
                <div class="p-6 rounded-3xl border border-slate-100 bg-white shadow-sm flex flex-col gap-4 group hover:border-sky-300 transition-all">
                    <div class="flex justify-between items-start">
                        <div class="flex flex-col gap-1">
                            <div class="flex items-center gap-2">
                                <span class="font-black text-slate-800 text-[13px]">{node.hostname}</span>
                                {#if node.is_local}
                                    <span class="text-[8px] bg-sky-50 text-sky-600 px-2 py-0.5 rounded-md font-black uppercase tracking-tighter">Primary</span>
                                {/if}
                            </div>
                            <span class="text-[10px] text-slate-400 font-mono">{node.ip}</span>
                        </div>
                        <div class="flex flex-col items-end gap-1">
                            <span class={`px-3 py-1 rounded-full text-[9px] font-black shadow-sm ${node.status === 'Online' ? 'bg-emerald-50 text-emerald-600' : 'bg-red-50 text-red-600'}`}>
                                {node.status}
                            </span>
                        </div>
                    </div>
                    <div class="flex justify-between items-center text-[10px] border-t border-slate-50 pt-4 mt-2">
                        <div class="flex gap-4">
                            <button class="text-sky-600 font-black hover:underline uppercase text-[9px]">Manage</button>
                            <button class="text-slate-400 font-black hover:underline uppercase text-[9px]">Settings</button>
                        </div>
                        {#if !node.is_local}
                            <button class="text-red-400 font-black hover:underline uppercase text-[9px]">Remove</button>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
