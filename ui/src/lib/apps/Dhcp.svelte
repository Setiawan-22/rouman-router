<script lang="ts">
    import { onMount } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';
    import { networkState } from '$lib/state/network.svelte';

    let candidateConfig = $state(null as any);
    let loading = $state(true);

    // Form states
    let enabled = $state(false);
    let selectedInterface = $state("eth1");
    let rangeStart = $state("192.168.1.100");
    let rangeEnd = $state("192.168.1.200");
    let gateway = $state("192.168.1.1");
    let dnsServers = $state("1.1.1.1, 8.8.8.8");
    let leaseTime = $state(86400);

    // Static Leases
    let staticLeases = $state([] as any[]);
    let newStatic = $state({ mac: "", ip: "", hostname: "" });

    async function loadData() {
        const resCfg = await fetch('/api/v1/config/candidate');

        if (resCfg.ok) {
            candidateConfig = (await resCfg.json()).candidate;
            const d = candidateConfig.network.dhcp;
            if (d) {
                enabled = d.enabled;
                selectedInterface = d.interface;
                rangeStart = d.range_start;
                rangeEnd = d.range_end;
                gateway = d.gateway;
                dnsServers = d.dns_servers.join(", ");
                leaseTime = d.lease_time_secs;
                staticLeases = d.static_leases || [];
            }
        }

        await networkState.refresh();
        loading = false;
    }

    function addStaticLease() {
        if (!newStatic.mac || !newStatic.ip) return;
        staticLeases = [...staticLeases, { ...newStatic }];
        newStatic = { mac: "", ip: "", hostname: "" };
    }

    function removeStaticLease(index: number) {
        staticLeases = staticLeases.filter((_, i) => i !== index);
    }

    async function saveDhcp() {
        if (!candidateConfig) return;
        let payload = JSON.parse(JSON.stringify(candidateConfig));
        payload.network.dhcp = {
            enabled,
            interface: selectedInterface,
            range_start: rangeStart,
            range_end: rangeEnd,
            gateway,
            dns_servers: dnsServers.split(",").map(s => s.trim()),
            lease_time_secs: leaseTime,
            static_leases: staticLeases
        };

        await fetch('/api/v1/config/candidate', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        // Refresh and notify
    }

    onMount(() => loadData());
</script>

<ServiceStarter featurePath="network.dhcp.enabled" title="DHCP Server" description="Dynamic Host Configuration Protocol service for automatic IP addressing on your LAN.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-emerald-600">DHCP Orchestrator_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Automated IP Asset Management</p>
        </div>
        
        <button 
            onclick={() => enabled = !enabled}
            class={`flex items-center gap-4 px-5 py-2.5 rounded-2xl border transition-all font-black text-[10px] tracking-widest uppercase shadow-sm ${enabled ? 'border-emerald-200 bg-emerald-50 text-emerald-600' : 'border-slate-100 bg-slate-50 text-slate-400'}`}
        >
            {enabled ? 'SYSTEM ACTIVE' : 'SYSTEM STANDBY'}
            <div class={`w-2.5 h-2.5 rounded-full ${enabled ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)] animate-pulse' : 'bg-slate-300'}`}></div>
        </button>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
        <div class="lg:col-span-7 flex flex-col gap-8">
            <!-- Pool Config -->
            <div class="rouman-card flex flex-col gap-8 !bg-white/60">
                <div class="flex items-center gap-3 border-b border-slate-100 pb-5">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">⚡</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Network Pool Parameters</h4>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Target Interface</label>
                        <select bind:value={selectedInterface} class="rouman-input !py-1.5 !text-emerald-600 appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMTA4OTgxIj48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                            {#each networkState.interfaces as iface}
                                <option value={iface.name}>{iface.name}</option>
                            {/each}
                        </select>
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Lease Duration (sec)</label>
                        <input type="number" bind:value={leaseTime} class="rouman-input !py-1.5 !text-sky-600 font-black" />
                    </div>
                </div>

                <div class="flex flex-col gap-2">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Allocation Range</label>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="relative">
                            <span class="absolute left-4 top-1/2 -translate-y-1/2 text-[8px] text-slate-400 font-black uppercase tracking-tighter">Start</span>
                            <input type="text" bind:value={rangeStart} class="rouman-input !py-1.5 !pl-14 !text-slate-800 font-black" />
                        </div>
                        <div class="relative">
                            <span class="absolute left-4 top-1/2 -translate-y-1/2 text-[8px] text-slate-400 font-black uppercase tracking-tighter">End</span>
                            <input type="text" bind:value={rangeEnd} class="rouman-input !py-1.5 !pl-12 !text-slate-800 font-black" />
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-8">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Default Gateway</label>
                        <input type="text" bind:value={gateway} class="rouman-input !py-1.5 !text-emerald-600 font-black" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">DNS Resolvers</label>
                        <input type="text" bind:value={dnsServers} class="rouman-input !py-1.5 !text-sky-600 font-black" placeholder="8.8.8.8, 1.1.1.1" />
                    </div>
                </div>
            </div>
        </div>

        <div class="lg:col-span-5 flex flex-col gap-8">
            <!-- Static Bindings -->
            <div class="rouman-card flex flex-col gap-8 flex-1 min-h-[400px] !bg-white/60">
                <div class="flex items-center justify-between border-b border-slate-100 pb-5">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-600 shadow-sm">📌</div>
                        <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Fixed IP Bindings</h4>
                    </div>
                    <span class="text-[9px] text-slate-300 font-black uppercase tracking-widest">MAC RESERVE</span>
                </div>

                <div class="flex-1 overflow-y-auto max-h-[300px] custom-scrollbar">
                    <table class="w-full text-left border-collapse">
                        <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                            <tr>
                                <th class="pb-3 px-2">Hardware Address</th>
                                <th class="pb-3 px-2">Static IP</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody class="text-slate-600 font-mono text-[10px]">
                            {#each staticLeases as lease, i}
                                <tr class="border-b border-slate-50 group hover:bg-slate-50/50 transition-all">
                                    <td class="py-4 px-2 font-black text-sky-600">{lease.mac}</td>
                                    <td class="py-4 px-2 text-emerald-600 font-black">{lease.ip}</td>
                                    <td class="py-4 text-right pr-2">
                                        <button onclick={() => removeStaticLease(i)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                            </svg>
                                        </button>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
                
                <div class="bg-slate-50 p-5 rounded-2xl border border-slate-100 flex flex-col gap-4 mt-auto shadow-inner">
                    <div class="grid grid-cols-2 gap-4">
                        <input type="text" bind:value={newStatic.mac} placeholder="MAC ADDR" class="rouman-input !py-2 !text-sky-600 font-black" />
                        <input type="text" bind:value={newStatic.ip} placeholder="TARGET IP" class="rouman-input !py-2 !text-emerald-600 font-black" />
                    </div>
                    <button onclick={addStaticLease} class="rouman-btn-primary !py-2.5 text-[10px] font-black tracking-widest">Reserve IP Address_</button>
                </div>
            </div>
        </div>
    </div>

    <div class="flex justify-end pt-6 border-t border-slate-200">
        <button onclick={saveDhcp} class="rouman-btn-primary px-16 !py-3 shadow-xl">
            Save Pool Draft_
        </button>
    </div>
</div>
</ServiceStarter>

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
