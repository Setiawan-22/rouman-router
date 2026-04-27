<script lang="ts">
    import { onMount } from 'svelte';
    import { networkState } from '$lib/state/network.svelte';

    let config = $state({ network: { wans: [], lb_mode: 'None', enable_nat: false, port_forwards: [], qos: {}, pppoe: { username: '', password: '', mtu: 1492 } } } as any);
    let newForward = $state({ proto: "tcp", ext_port: 80, int_ip: "192.168.1.10", int_port: 80, comment: "" });
    let saving = $state(false);

    async function loadConfig() {
        const resCand = await fetch('/api/v1/config/candidate');
        if (resCand.ok) {
            const data = await resCand.json();
            if (data.candidate && data.candidate.network) {
                config.network = data.candidate.network;
                if (!config.network.wans) config.network.wans = [];
                if (!config.network.pppoe) config.network.pppoe = { username: '', password: '', mtu: 1492 };
            }
        }
        networkState.refresh();
    }

    function addForwardRule() {
        config.network.port_forwards = [...(config.network.port_forwards || []), { ...newForward }];
        newForward = { proto: "tcp", ext_port: 80, int_ip: "192.168.1.10", int_port: 80, comment: "" };
    }

    function addWan() {
        config.network.wans = [...config.network.wans, {
            name: `WAN${config.network.wans.length + 1}`,
            interface: networkState.interfaces[0]?.name || 'eth0',
            gateway: '192.168.0.1',
            weight: 1,
            distance: 1,
            enabled: true,
            pppoe_enabled: false
        }];
    }

    async function saveRouting() {
        saving = true;
        const res = await fetch('/api/v1/config/candidate');
        let payload = (await res.json()).candidate;
        payload.network = config.network;

        await fetch('/api/v1/config/candidate', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        saving = false;
    }
    
    async function commitNow() {
        await saveRouting();
        await fetch('/api/v1/config/commit', { method: 'POST' });
        loadConfig();
    }

    onMount(() => loadConfig());
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <!-- Premium Header -->
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">WAN & Routing_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Multi-Gateway & NAT Orchestration</p>
        </div>
        
        <button onclick={commitNow} disabled={saving} class="rouman-btn-primary flex items-center gap-2">
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Syncing...</span>
            {:else}
                <span>💾 Apply Config_</span>
            {/if}
        </button>
    </div>

    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        <div class="xl:col-span-8 flex flex-col gap-8">
            <!-- WAN Interfaces -->
            <div class="rouman-card flex flex-col gap-6 !bg-white/60">
                <div class="flex justify-between items-center border-b border-slate-100 pb-4">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">🌐</div>
                        <h4 class="font-black text-sm text-slate-800 tracking-wide">Internet Uplinks</h4>
                    </div>
                    <button onclick={addWan} class="px-4 py-1.5 bg-sky-50 text-sky-600 rounded-full font-black text-[10px] uppercase hover:bg-sky-100 transition-all border border-sky-100 shadow-sm">
                        + Add WAN
                    </button>
                </div>

                <div class="flex flex-col gap-5">
                    {#each config.network.wans as wan, i}
                        <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-5 group hover:border-sky-200 hover:bg-white transition-all shadow-sm">
                            <div class="flex justify-between items-center border-b border-slate-100 pb-4">
                                <div class="flex items-center gap-4">
                                    <input bind:value={wan.name} class="bg-transparent border-none text-sky-600 font-black text-[14px] w-28 outline-none" />
                                    <button 
                                        onclick={() => wan.enabled = !wan.enabled} 
                                        class={`px-4 py-1 rounded-full text-[9px] font-black tracking-widest shadow-sm transition-all ${wan.enabled ? 'bg-emerald-50 text-emerald-600' : 'bg-red-50 text-red-400'}`}
                                    >
                                        {wan.enabled ? 'ONLINE' : 'OFFLINE'}
                                    </button>
                                </div>
                                <div class="flex gap-5 items-center">
                                    <label class="flex items-center gap-2.5 cursor-pointer text-[10px] text-orange-600 font-black uppercase tracking-tight">
                                        <input type="checkbox" bind:checked={wan.pppoe_enabled} class="w-4 h-4 rounded border-slate-300 text-orange-600 focus:ring-orange-500"> PPPoE
                                    </label>
                                    <button onclick={() => config.network.wans.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                        </svg>
                                    </button>
                                </div>
                            </div>
                            
                            <div class="grid grid-cols-2 lg:grid-cols-4 gap-6">
                                <div class="flex flex-col gap-2">
                                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Uplink Port</label>
                                    <select bind:value={wan.interface} class="rouman-input !py-1.5 !text-sky-600 font-black appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMWU4OGU1Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                                        {#each networkState.interfaces as iface}
                                            <option value={iface.name}>{iface.name}</option>
                                        {/each}
                                    </select>
                                </div>
                                <div class="flex flex-col gap-2">
                                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Gateway IP</label>
                                    <input bind:value={wan.gateway} class="rouman-input !py-1.5 !text-slate-800 font-black" placeholder="0.0.0.0" />
                                </div>
                                <div class="flex flex-col gap-2">
                                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Metric Distance</label>
                                    <input type="number" bind:value={wan.distance} class="rouman-input !py-1.5 !text-emerald-600 font-black" />
                                </div>
                                <div class="flex flex-col gap-2">
                                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">PCC Weight</label>
                                    <input type="number" bind:value={wan.weight} class="rouman-input !py-1.5 !text-purple-600 font-black" />
                                </div>
                            </div>
                        </div>
                    {/each}
                    {#if config.network.wans?.length === 0}
                        <div class="py-16 flex flex-col items-center gap-4 border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                            <span class="text-4xl grayscale opacity-20">🛣️</span>
                            <span class="text-[10px] text-slate-300 font-black uppercase tracking-[0.3em]">No WAN interfaces configured</span>
                        </div>
                    {/if}
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                 <div class="rouman-card flex flex-col gap-6 !bg-emerald-50/30 border-emerald-100/50 shadow-sm">
                    <div class="flex flex-col gap-1 border-b border-emerald-100/50 pb-4">
                        <span class="font-black text-xs text-emerald-800 tracking-widest uppercase">IP Masquerade</span>
                        <span class="text-[9px] text-emerald-500 font-bold uppercase tracking-tight">Source NAT Core</span>
                    </div>
                    <button 
                        onclick={() => config.network.enable_nat = !config.network.enable_nat}
                        class={`py-2.5 rounded-2xl text-[10px] font-black tracking-widest transition-all shadow-sm ${config.network.enable_nat ? 'bg-emerald-500 text-white border border-emerald-600' : 'bg-white text-slate-400 border border-slate-200'}`}
                    >
                        {config.network.enable_nat ? 'NAT ACTIVE' : 'NAT DISABLED'}
                    </button>

                    <div class="flex flex-col gap-2.5 mt-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Load Balancing</label>
                        <select bind:value={config.network.lb_mode} class="rouman-input !py-1.5 !text-slate-800 font-black appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMWU4OGU1Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                            <option value="None">None (Static)</option>
                            <option value="Failover">Auto Failover</option>
                            <option value="Pcc">PCC Bonding</option>
                        </select>
                    </div>
                </div>

                <div class="md:col-span-2 rouman-card flex flex-col gap-6 !bg-orange-50/30 border-orange-100/50 shadow-sm">
                    <div class="flex flex-col gap-1 border-b border-orange-100/50 pb-4">
                        <span class="font-black text-xs text-orange-700 tracking-widest uppercase">PPPoE Dial Engine</span>
                        <span class="text-[9px] text-orange-400 font-bold uppercase tracking-tight">Carrier Authentication Service</span>
                    </div>
                    <div class="grid grid-cols-2 gap-6">
                        <div class="flex flex-col gap-2.5">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">ISP Username</label>
                            <input bind:value={config.network.pppoe.username} class="rouman-input !py-1.5 !text-orange-700 font-black" placeholder="user@isp.domain" />
                        </div>
                        <div class="flex flex-col gap-2.5">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">ISP Password</label>
                            <input type="password" bind:value={config.network.pppoe.password} class="rouman-input !py-1.5 font-black" />
                        </div>
                    </div>
                    <div class="flex gap-5 items-center mt-2">
                        <div class="flex flex-col gap-1.5">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">MTU Logic</label>
                            <input type="number" bind:value={config.network.pppoe.mtu} class="rouman-input !w-28 !py-1.5 text-center !text-slate-800 font-black" />
                        </div>
                        <span class="text-[9px] text-slate-400 font-black italic mt-5">REC: 1492 FOR DSL/FIBER</span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Port Forwarding -->
        <div class="xl:col-span-4 flex flex-col gap-8">
            <div class="rouman-card flex flex-col gap-8 flex-1 !bg-white/60">
                <div class="flex items-center gap-3 border-b border-slate-100 pb-5">
                    <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-600 shadow-sm">🛡️</div>
                    <h4 class="font-black text-sm text-slate-800 tracking-wide uppercase">DNAT Mapping</h4>
                </div>
                
                <div class="flex-1 overflow-y-auto max-h-[350px] custom-scrollbar">
                    <table class="w-full text-left border-collapse">
                        <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                            <tr>
                                <th class="pb-3 px-1">Proto</th>
                                <th class="pb-3 px-1">Port</th>
                                <th class="pb-3 px-1">Mapping</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody class="text-slate-600 font-mono text-[10px]">
                            {#each config.network.port_forwards || [] as rule, i}
                                <tr class="border-b border-slate-50 group hover:bg-slate-50/50 transition-all">
                                    <td class="py-4 px-1 uppercase text-[9px] font-black">{rule.proto}</td>
                                    <td class="py-4 px-1 text-sky-600 font-black">{rule.ext_port}</td>
                                    <td class="py-4 px-1 text-emerald-600 font-black tracking-tighter">{rule.int_ip}:{rule.int_port}</td>
                                    <td class="py-4 text-right pr-1">
                                        <button onclick={() => config.network.port_forwards.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
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

                <div class="bg-slate-50 p-5 rounded-2xl border border-slate-100 flex flex-col gap-4 shadow-inner">
                    <div class="grid grid-cols-3 gap-4">
                        <select bind:value={newForward.proto} class="rouman-input !py-1.5 !text-[10px] font-black uppercase appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMWU4OGU1Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:10px] bg-[right_8px_center] bg-no-repeat pr-8">
                            <option>tcp</option>
                            <option>udp</option>
                        </select>
                        <input bind:value={newForward.ext_port} placeholder="Ext Port" class="col-span-2 rouman-input !py-1.5 !text-sky-600 font-black" />
                    </div>
                    <div class="grid grid-cols-3 gap-4">
                        <input bind:value={newForward.int_ip} placeholder="Internal IP" class="col-span-2 rouman-input !py-1.5 !text-slate-800 font-black" />
                        <input bind:value={newForward.int_port} placeholder="Port" class="rouman-input !py-1.5 !text-emerald-600 font-black" />
                    </div>
                    <button onclick={addForwardRule} class="rouman-btn-primary !py-2.5 text-[10px] font-black tracking-widest">Commit NAT Mapping_</button>
                </div>
            </div>
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

