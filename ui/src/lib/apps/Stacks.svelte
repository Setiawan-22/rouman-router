<script lang="ts">
    import { onMount } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let stacks = $state([] as any[]);
    let newStackName = $state("");
    let yamlConfig = $state(`version: "1.0"
services:
  web-app:
    image: nginx:alpine
    type: container
    resources:
      cpu: 1
      ram: 512
  edge-worker:
    image: alpine-custom
    type: microvm
    resources:
      cpu: 2
      ram: 1024`);
    let isDeploying = $state(false);

    async function fetchStacks() {
        try {
            const res = await fetch('/api/v1/compute/stacks');
            if (res.ok) stacks = (await res.json()).stacks || [];
        } catch {}
    }

    async function deployStack() {
        if(!newStackName || !yamlConfig) return;
        isDeploying = true;
        try {
            const res = await fetch('/api/v1/compute/stacks', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: newStackName, config: yamlConfig })
            });
            if (res.ok) {
                newStackName = "";
                await fetchStacks();
            }
        } catch (e) {
            console.error(e);
        } finally {
            isDeploying = false;
        }
    }

    onMount(fetchStacks);
</script>

<ServiceStarter featurePath="compute.container_enabled" title="Compute Stacks" description="Orchestrate multiple containers and MicroVMs using YAML-based declarative configurations (Compose style).">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Compute Stacks_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Multi-Service Orchestration</p>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <!-- Stack Editor -->
        <div class="rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <span class="text-sky-600 text-lg shadow-sm">🛠️</span>
                <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Deploy New Stack</h4>
            </div>
            
            <div class="flex flex-col gap-6">
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Stack Identifier</label>
                    <input bind:value={newStackName} placeholder="production-cluster-01" class="rouman-input !py-1.5 !text-slate-800 font-black" />
                </div>
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">YAML Configuration (Compose Spec)</label>
                    <textarea 
                        bind:value={yamlConfig} 
                        rows="12" 
                        class="w-full bg-slate-900 text-sky-400 border border-slate-800 rounded-2xl p-4 text-[11px] font-mono outline-none focus:border-sky-500 shadow-2xl custom-scrollbar"
                    ></textarea>
                </div>
                <button 
                    disabled={isDeploying} 
                    onclick={deployStack} 
                    class="mt-4 w-full py-3 bg-sky-600 text-white rounded-xl font-black text-[10px] tracking-[0.2em] uppercase hover:bg-sky-700 transition-all shadow-lg disabled:opacity-50"
                >
                    {isDeploying ? 'Deploying Engine...' : 'Deploy Stack Now_'}
                </button>
            </div>
        </div>

        <!-- Active Stacks -->
        <div class="flex flex-col gap-6">
            <div class="flex items-center gap-3 pb-2">
                <h4 class="font-black text-[10px] text-slate-400 tracking-[0.2em] uppercase">Active Orchestrations</h4>
            </div>
            
            {#if stacks.length === 0}
                <div class="py-24 text-center text-slate-300 font-black uppercase tracking-[0.3em] bg-slate-50/30 rounded-3xl border-2 border-dashed border-slate-100">
                    No active stacks deployed
                </div>
            {:else}
                <div class="flex flex-col gap-5">
                    {#each stacks as stack}
                        <div class="rouman-card !bg-white/40 border-slate-100 p-6 flex flex-col gap-4 group hover:border-sky-300 transition-all">
                            <div class="flex justify-between items-center">
                                <span class="font-black text-sky-600 uppercase tracking-widest text-[12px]">{stack.name}</span>
                                <span class="px-3 py-1 bg-emerald-50 text-emerald-600 rounded-full text-[9px] font-black uppercase shadow-sm">Operational</span>
                            </div>
                            <div class="grid grid-cols-2 gap-4 text-[10px] text-slate-400 font-black uppercase tracking-tight">
                                <div class="flex items-center gap-2">
                                    <span class="w-1.5 h-1.5 bg-sky-400 rounded-full"></span>
                                    Services: {stack.service_count}
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="w-1.5 h-1.5 bg-emerald-400 rounded-full"></span>
                                    Uptime: {stack.uptime}
                                </div>
                            </div>
                            <div class="flex gap-4 mt-2">
                                <button class="flex-1 py-2 bg-slate-50 text-slate-600 rounded-xl font-black hover:bg-white transition-all uppercase text-[9px] border border-slate-100 shadow-sm">Details</button>
                                <button class="flex-1 py-2 bg-red-50 text-red-500 rounded-xl font-black hover:bg-white transition-all uppercase text-[9px] border border-red-100 shadow-sm">Destroy</button>
                            </div>
                        </div>
                    {/each}
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
        background: rgba(0, 0, 0, 0.2);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(14, 165, 233, 0.3);
        border-radius: 10px;
    }
</style>
</ServiceStarter>
