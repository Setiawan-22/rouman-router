<script lang="ts">
    import { onMount } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let containers = $state([] as any[]);
    let loading = $state(false);

    // Form states Container
    let containerId = $state("");
    let containerImage = $state("");
    let isPulling = $state(false);

    async function fetchContainers() {
        loading = true;
        try {
            const res = await fetch('/api/v1/compute/containers');
            if (res.ok) containers = (await res.json()).containers || [];
        } catch {}
        loading = false;
    }

    async function pullImage() {
        if(!containerImage) return;
        isPulling = true;
        try {
            const res = await fetch('/api/v1/compute/containers', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ id: containerId, image: containerImage })
            });
            if (res.ok) {
                containerId = ""; containerImage = "";
                await fetchContainers();
            }
        } catch (e) {
            console.error("Failed to deploy container", e);
        } finally {
            isPulling = false;
        }
    }

    onMount(() => {
        fetchContainers();
    });
</script>

<ServiceStarter featurePath="compute.container_enabled" title="Containers" description="Youki & Containerd OCI Container Engine.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-orange-600">Container Registry_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">OCI Orchestration Engine</p>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Container List -->
        <div class="lg:col-span-2 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <span class="text-orange-600 text-lg shadow-sm">📦</span>
                <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Active Workloads</h4>
            </div>
            
            {#if containers.length === 0}
                <div class="py-24 text-center text-slate-300 font-black uppercase tracking-[0.3em] bg-slate-50/30 rounded-3xl border-2 border-dashed border-slate-100">
                    No active OCI containers
                </div>
            {:else}
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                            <tr>
                                <th class="p-4 px-2">Workload ID</th>
                                <th class="p-4 px-2">OCI Image Source</th>
                                <th class="p-4 text-right pr-2">Network IP</th>
                            </tr>
                        </thead>
                        <tbody class="text-slate-600 font-mono">
                            {#each containers as ct}
                                <tr class="border-b border-slate-50 hover:bg-slate-50/50 transition-all">
                                    <td class="p-4 px-2 font-black text-orange-600 text-[11px]">{ct.id}</td>
                                    <td class="p-4 px-2 text-[10px] lowercase text-slate-400 font-black truncate max-w-xs">{ct.image}</td>
                                    <td class="p-4 text-right pr-2 text-slate-400 text-[10px] font-black">{ct.ip}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        </div>

        <!-- Deployment Panel -->
        <div class="rouman-card flex flex-col gap-8 !bg-slate-50/50 border-slate-100 shadow-sm">
            <div class="flex items-center gap-3 border-b border-slate-200/50 pb-4">
                <span class="text-orange-600 text-lg shadow-sm">🚚</span>
                <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Deploy Workload</h4>
            </div>
            
            <div class="flex flex-col gap-5">
                <div class="flex flex-col gap-2">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Container Name</label>
                    <input type="text" bind:value={containerId} placeholder="web-node-alpha" class="rouman-input !py-1.5 !text-slate-800 font-black" />
                </div>
                
                <div class="flex flex-col gap-2">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">OCI Image Reference</label>
                    <input type="text" bind:value={containerImage} placeholder="docker.io/library/nginx:latest" class="rouman-input !py-1.5 !text-orange-600 font-black" />
                    <p class="text-[9px] text-slate-300 font-black tracking-tight mt-1 uppercase">Supports DockerHub & GCR</p>
                </div>
                
                <button disabled={isPulling} onclick={pullImage} class="rouman-btn-primary !bg-orange-600 hover:!bg-orange-700 mt-6 w-full !py-3 text-[10px] font-black tracking-widest shadow-lg">
                    {#if isPulling}
                        <div class="flex items-center justify-center gap-2">
                            <div class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                            <span>Pulling Layer...</span>
                        </div>
                    {:else}
                        <span>Deploy Workload_</span>
                    {/if}
                </button>
            </div>
        </div>
    </div>
</div>
</ServiceStarter>
