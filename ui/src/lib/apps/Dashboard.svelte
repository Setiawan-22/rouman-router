<script lang="ts">
    import { onMount, onDestroy } from 'svelte';

    let config = $state(null as any);
    let neighbors = $state([] as any[]);
    let internetStatus = $state('Checking...');
    let activeUsers = $state([] as any[]);
    let pollingInterval: number;

    async function fetchTopology() {
        try {
            // 1. Get Internet Status
            const resInt = await fetch('/api/v1/system/internet-status');
            if (resInt.ok) internetStatus = (await resInt.json()).status;

            // 2. Get Config (Router details)
            const resCfg = await fetch('/api/v1/config/active');
            if (resCfg.ok) config = (await resCfg.json()).active;

            // 3. Get Neighbors (Switches/APs)
            const resNeigh = await fetch('/api/v1/network/neighbors');
            if (resNeigh.ok) neighbors = (await resNeigh.json()).neighbors;

            // 4. Get Real DHCP Leases for Active Users
            const resLeases = await fetch('/api/v1/network/leases');
            if (resLeases.ok) {
                const leaseData = await resLeases.json();
                activeUsers = leaseData.leases.map((l:any) => ({
                    mac: l.mac,
                    ip: l.ip,
                    name: l.hostname || l.mac
                }));
            }

        } catch (e) {
            console.error("Topology fetch error", e);
        }
    }

    onMount(() => {
        fetchTopology();
        pollingInterval = setInterval(fetchTopology, 15000);
    });

    onDestroy(() => {
        if (pollingInterval) clearInterval(pollingInterval);
    });
</script>

<div class="flex flex-col gap-8">
    <!-- Header Area -->
    <div class="rouman-card flex items-center justify-between !py-5 shadow-sm">
        <div>
            <h2 class="text-sm font-black text-slate-800 tracking-[0.2em] uppercase">Network Topology_</h2>
            <p class="text-[10px] text-slate-400 uppercase tracking-widest mt-1 font-bold">Real-time infrastructure hierarchy</p>
        </div>
        <div class="flex items-center gap-4 bg-slate-50 px-5 py-2.5 rounded-2xl border border-slate-100 shadow-inner">
            <div class="w-2.5 h-2.5 rounded-full {internetStatus === 'ONLINE' ? 'bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.4)]' : 'bg-red-500 shadow-[0_0_12px_rgba(239,68,68,0.4)]'}"></div>
            <span class="text-[10px] font-black tracking-widest uppercase {internetStatus === 'ONLINE' ? 'text-emerald-600' : 'text-red-600'}">
                {internetStatus === 'ONLINE' ? 'Uplink Established' : 'Link Failure'}
            </span>
        </div>
    </div>

    <!-- Topology Canvas -->
    <div class="rouman-card !p-12 flex flex-col items-center justify-center min-h-[550px] overflow-auto relative !bg-slate-50/50">
        {#if !config}
            <div class="flex flex-col items-center gap-6">
                <div class="w-14 h-14 border-4 border-sky-500 border-t-transparent rounded-full animate-spin"></div>
                <div class="text-sky-600 font-black text-[11px] uppercase tracking-[0.3em] animate-pulse">Mapping Infrastructure...</div>
            </div>
        {:else}
            <div class="flex flex-col items-center gap-16 w-full max-w-5xl">
                
                <!-- LEVEL 1: WAN / INTERNET -->
                <div class="flex flex-col items-center group relative z-10">
                    <div class="w-24 h-24 bg-sky-500/5 border-2 border-sky-500/20 rounded-full flex items-center justify-center shadow-lg z-10 transition-all group-hover:scale-110 group-hover:border-sky-500/40 group-hover:shadow-xl">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-sky-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" /></svg>
                    </div>
                    <div class="mt-4 text-center">
                        <div class="text-[10px] font-black text-slate-800 uppercase tracking-[0.2em]">Global Backbone</div>
                        <div class="text-[9px] text-sky-600 font-bold font-mono mt-1">{config.network.wans.filter((w:any) => w.enabled).length} ACTIVE WAN LINKS</div>
                    </div>
                    <!-- Wire down -->
                    <div class="absolute top-24 w-[2px] h-16 bg-gradient-to-b from-sky-200 to-slate-200 z-0"></div>
                </div>

                <!-- LEVEL 2: ROUTER (ROUMAN OS) -->
                <div class="flex flex-col items-center group relative z-10">
                    <div class="w-40 h-24 bg-white border border-slate-200 border-b-4 border-sky-500 rounded-2xl flex items-center justify-center shadow-2xl z-10 transition-all group-hover:scale-105 group-hover:border-slate-300">
                        <div class="flex flex-col items-center gap-3">
                             <div class="w-10 h-10 rounded-lg bg-[#1E88E5] flex items-center justify-center font-black text-white text-xl shadow-lg">R</div>
                             <div class="flex gap-1.5">
                                 <div class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></div>
                                 <div class="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                                 <div class="w-1.5 h-1.5 rounded-full bg-slate-200"></div>
                             </div>
                        </div>
                    </div>
                    <div class="mt-4 text-center bg-white border border-slate-100 px-8 py-3 rounded-2xl shadow-xl backdrop-blur-md">
                        <div class="text-xs font-black text-slate-800 uppercase tracking-widest">{config.system.hostname}</div>
                        <div class="text-[9px] text-slate-400 font-bold font-mono mt-1 tracking-[0.2em]">L3 EDGE ORCHESTRATOR</div>
                    </div>
                    <!-- Wire down -->
                    <div class="absolute top-24 w-[2px] h-16 bg-slate-200 z-0"></div>
                </div>

                <!-- LEVEL 3: DIVIDER -->
                <div class="w-full flex justify-center relative">
                    <!-- Horizontal bus rail -->
                    <div class="absolute top-[-64px] w-[90%] max-w-[800px] h-[2px] bg-slate-200 z-0">
                         <!-- Left drop line -->
                         <div class="absolute left-0 top-0 w-[2px] h-16 bg-slate-200"></div>
                         <!-- Center drop line -->
                         <div class="absolute left-1/2 top-0 w-[2px] h-16 bg-slate-200 -translate-x-1/2"></div>
                         <!-- Right drop line -->
                         <div class="absolute right-0 top-0 w-[2px] h-16 bg-slate-200"></div>
                    </div>
                </div>

                <!-- LEVEL 4: NODES -->
                <div class="w-full grid grid-cols-3 gap-12 px-4 mt-[-16px]">
                    
                    <!-- L2 NEIGHBORS -->
                    <div class="flex flex-col items-center">
                        <div class="w-24 h-20 bg-white border border-slate-200 border-t-2 border-t-indigo-500 rounded-2xl flex flex-col items-center justify-center shadow-xl relative z-10 transition-all hover:bg-slate-50 hover:border-indigo-300">
                             <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>
                             <div class="absolute -top-3 -right-3 bg-indigo-600 text-white text-[9px] font-black px-3 py-1 rounded-full shadow-lg">{neighbors.length}</div>
                        </div>
                        <div class="mt-4 text-center">
                            <div class="text-[10px] font-black text-indigo-600 uppercase tracking-widest">L2 Fabric</div>
                        </div>
                        <div class="mt-4 flex flex-col gap-2.5 text-[9px] text-slate-500 text-center bg-white p-4 rounded-2xl border border-slate-100 max-h-40 overflow-auto custom-scrollbar w-full shadow-sm">
                            {#each neighbors as n}
                                <div class="flex flex-col border-b border-slate-50 pb-2">
                                    <span class="text-slate-800 font-bold uppercase tracking-tighter">{n.hostname}</span>
                                    <span class="font-mono text-[8px] text-slate-400">{n.ip}</span>
                                </div>
                            {/each}
                            {#if neighbors.length === 0}
                                <span class="italic text-slate-300 font-medium py-2 uppercase tracking-widest text-[8px]">No Neighbors Found</span>
                            {/if}
                        </div>
                    </div>

                    <!-- LOCAL BRIDGE & CLIENTS -->
                    <div class="flex flex-col items-center">
                        <div class="w-28 h-14 bg-white border border-slate-200 rounded-2xl flex items-center justify-center shadow-xl relative z-10 group hover:border-slate-300 transition-all">
                            <div class="flex gap-2">
                                <div class="w-2 h-2 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.3)]"></div>
                                <div class="w-2 h-2 bg-emerald-500 rounded-full"></div>
                                <div class="w-2 h-2 bg-emerald-500 rounded-full"></div>
                                <div class="w-2 h-2 bg-slate-200 rounded-full"></div>
                            </div>
                        </div>
                        <div class="mt-4 text-center">
                            <div class="text-[10px] font-black text-slate-800 uppercase tracking-widest">Local Mesh</div>
                        </div>
                        <!-- Drop line to clients -->
                        <div class="w-[2px] h-10 bg-slate-200 mt-4"></div>

                        <!-- CLIENTS NODE -->
                        <div class="bg-white border border-slate-100 p-5 rounded-3xl flex flex-col items-center shadow-2xl w-64 mt-0 relative z-10 backdrop-blur-md hover:border-emerald-200 transition-all">
                            <h4 class="text-[10px] font-black text-emerald-600 mb-4 tracking-[0.2em] uppercase">{activeUsers.length} Devices Wired</h4>
                            <div class="flex flex-col gap-3 w-full text-[10px] overflow-y-auto max-h-56 custom-scrollbar pr-1">
                                {#each activeUsers as user}
                                    <div class="flex justify-between items-center border-b border-slate-50 pb-3 group/u">
                                        <div class="flex flex-col">
                                            <span class="text-slate-800 font-bold truncate max-w-[90px]">{user.name}</span>
                                            <span class="text-[8px] text-slate-400 font-mono tracking-tighter uppercase">{user.mac}</span>
                                        </div>
                                        <span class="text-emerald-600 font-black font-mono text-[9px]">{user.ip}</span>
                                    </div>
                                {/each}
                                {#if activeUsers.length === 0}
                                    <span class="text-slate-300 italic text-center py-4 font-bold uppercase tracking-widest text-[9px]">Silent Segment</span>
                                {/if}
                            </div>
                        </div>
                    </div>

                    <!-- SERVICES MESH -->
                    <div class="flex flex-col items-center">
                        <div class="w-24 h-24 bg-orange-50 border border-orange-100 rounded-full flex flex-col items-center justify-center shadow-xl z-10 transition-all hover:scale-105 hover:bg-orange-100/50">
                             <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-orange-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" /></svg>
                        </div>
                        <div class="mt-4 text-center text-[10px] font-black text-orange-600 uppercase tracking-widest">
                            Managed Stack
                        </div>
                         <div class="mt-4 flex flex-col gap-3 text-[10px] text-slate-500 text-center bg-white p-5 rounded-2xl border border-slate-100 min-w-[150px] shadow-lg">
                            <div class="flex justify-between items-center pb-2 border-b border-slate-50">
                                <span class="font-bold tracking-tighter">CLOUDFLARE</span>
                                <span class={`font-black text-[9px] ${config.cloudflare.tunnels.some((t:any) => t.enabled) ? 'text-emerald-500' : 'text-red-500'}`}>{config.cloudflare.tunnels.some((t:any) => t.enabled) ? 'LIVE' : 'OFF'}</span>
                            </div>
                            <div class="flex justify-between items-center pb-2 border-b border-slate-50">
                                <span class="font-bold tracking-tighter">DHCP ENGINE</span>
                                <span class={`font-black text-[9px] ${config.network.dhcp.enabled ? 'text-emerald-500' : 'text-red-500'}`}>{config.network.dhcp.enabled ? 'ACTIVE' : 'OFF'}</span>
                            </div>
                            <div class="flex justify-between items-center">
                                <span class="font-bold tracking-tighter">SECURE DNS</span>
                                <span class={`font-black text-[9px] ${config.dns.sinkhole_enabled ? 'text-emerald-500' : 'text-red-500'}`}>{config.dns.sinkhole_enabled ? 'SECURED' : 'OPEN'}</span>
                            </div>
                        </div>
                    </div>

                </div>

            </div>
        {/if}
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
        background: rgba(0, 0, 0, 0.05);
        border-radius: 10px;
    }
</style>
