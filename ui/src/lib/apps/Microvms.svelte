<script lang="ts">
    import { onMount } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let vms = $state([] as any[]);
    let loading = $state(false);

    // Form states VM
    let vmId = $state("");
    let vcpu = $state(1);
    let ram = $state(512);
    let imageFile = $state("");

    async function fetchVms() {
        loading = true;
        try {
            const res = await fetch('/api/v1/compute/vms');
            if (res.ok) vms = (await res.json()).vms || [];
        } catch {}
        loading = false;
    }

    async function startVm() {
        if(!vmId) return;
        try {
            const res = await fetch('/api/v1/compute/vms', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ id: vmId, vcpu, ram, image: imageFile })
            });
            if (res.ok) {
                vmId = "";
                await fetchVms();
            }
        } catch (e) {
            console.error("Failed to start VM", e);
        }
    }

    onMount(() => {
        fetchVms();
    });
</script>

<ServiceStarter featurePath="compute.microvm_enabled" title="MicroVMs" description="Firecracker MicroVM Engine for lightweight virtual machine orchestration.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">MicroVM Center_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Firecracker Virtualization Engine</p>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Instance List -->
        <div class="lg:col-span-2 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <span class="text-sky-600 text-lg shadow-sm">🖥️</span>
                <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Active Instances</h4>
            </div>
            
            {#if vms.length === 0}
                <div class="py-24 text-center text-slate-300 font-black uppercase tracking-[0.3em] bg-slate-50/30 rounded-3xl border-2 border-dashed border-slate-100">
                    No active MicroVMs detected
                </div>
            {:else}
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                            <tr>
                                <th class="p-4 px-2">VM ID</th>
                                <th class="p-4 px-2">Hardware Specs</th>
                                <th class="p-4 px-2 text-center">Lifecycle</th>
                                <th class="p-4 text-right pr-2">Uptime</th>
                            </tr>
                        </thead>
                        <tbody class="text-slate-600 font-mono">
                            {#each vms as vm}
                                <tr class="border-b border-slate-50 hover:bg-slate-50/50 transition-all">
                                    <td class="p-4 px-2 font-black text-sky-600 text-[11px]">{vm.id}</td>
                                    <td class="p-4 px-2 text-[10px] uppercase font-black tracking-tight">{vm.vcpu} vCPU / {vm.ram}MB</td>
                                    <td class="p-4 text-center">
                                        <span class={`px-3 py-1 rounded-full text-[9px] font-black shadow-sm ${vm.status === 'Running' ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-amber-50 text-amber-600 border border-amber-100'}`}>
                                            {vm.status}
                                        </span>
                                    </td>
                                    <td class="p-4 text-right pr-2 text-slate-400 text-[10px] font-black">{vm.uptime}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        </div>

        <!-- Launch Panel -->
        <div class="rouman-card flex flex-col gap-8 !bg-slate-50/50 border-slate-100 shadow-sm">
            <div class="flex items-center gap-3 border-b border-slate-200/50 pb-4">
                <span class="text-sky-600 text-lg shadow-sm">🚀</span>
                <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Launch Engine</h4>
            </div>
            
            <div class="flex flex-col gap-5">
                <div class="flex flex-col gap-2">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Instance ID</label>
                    <input type="text" bind:value={vmId} placeholder="vm-node-alpha" class="rouman-input !py-1.5 !text-slate-800 font-black" />
                </div>
                
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">vCPU Core</label>
                        <input type="number" bind:value={vcpu} class="rouman-input !py-1.5 !text-emerald-600 font-black" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">RAM (MB)</label>
                        <input type="number" bind:value={ram} class="rouman-input !py-1.5 !text-sky-600 font-black" />
                    </div>
                </div>
                
                <div class="flex flex-col gap-2">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">OS Disk Image</label>
                    <select bind:value={imageFile} class="rouman-input !py-1.5 !text-slate-800 font-black appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMWU4OGU1Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                        <option value="">-- Select Distribution --</option>
                        <option value="alpine">Alpine Linux 3.20 (Stable)</option>
                        <option value="ubuntu">Ubuntu 24.04 LTS (Cloud)</option>
                        <option value="debian">Debian 12 (Minimal)</option>
                    </select>
                </div>
                
                <button onclick={startVm} class="rouman-btn-primary mt-6 w-full !py-3 text-[10px] font-black tracking-widest shadow-lg">
                    Initialize MicroVM_
                </button>
            </div>
        </div>
    </div>
</div>
</ServiceStarter>
