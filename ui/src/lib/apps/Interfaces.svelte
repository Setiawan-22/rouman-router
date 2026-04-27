<script lang="ts">
    import { onMount } from 'svelte';
    import { networkState } from '$lib/state/network.svelte';

    let physicalInterfaces = $state([] as any[]);
    let config = $state({ 
        network: { 
            wan_interface: 'eth0',
            interfaces: { vlans: [], bridges: [], assignments: [] } 
        } 
    } as any);
    let loading = $state(true);
    let saving = $state(false);
    let message = $state("");

    async function fetchData() {
        try {
            const [cfgRes] = await Promise.all([
                fetch('/api/v1/config/active')
            ]);
            
            if (cfgRes.ok) config = (await cfgRes.json()).active;
            await networkState.refresh();
            physicalInterfaces = networkState.interfaces;
        } catch (e) {
            console.error("Failed to fetch interfaces data", e);
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
                message = "Changes applied successfully!";
                fetchData();
            }
        } catch (e) {
            message = "Error applying changes";
        } finally {
            saving = false;
        }
    }

    function addAssignment() {
        config.network.interfaces.assignments = [...config.network.interfaces.assignments, {
            interface: networkState.interfaces[0]?.name || '',
            address: '192.168.100.1/24',
            enabled: true
        }];
    }

    onMount(fetchData);
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <!-- Premium Header -->
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Interfaces & Addresses_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">L1-L3 Network Topology</p>
        </div>
        
        <button onclick={saveConfig} disabled={saving} class="rouman-btn-primary flex items-center gap-2">
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Syncing Core...</span>
            {:else}
                <span>💾 Commit Changes_</span>
            {/if}
        </button>
    </div>

    {#if message}
        <div class="p-3 bg-emerald-50 border border-emerald-100 text-emerald-600 text-[10px] uppercase tracking-widest text-center rounded-2xl font-black shadow-sm">
            {message}
        </div>
    {/if}

    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        <!-- Physical Ports -->
        <div class="xl:col-span-5 rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-100 pb-4">
                <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">🔌</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Hardware Ports</h4>
            </div>
            
            <div class="flex flex-col gap-3">
                {#each physicalInterfaces as iface}
                    <div class="flex items-center justify-between p-4 bg-slate-50/50 border border-slate-100 rounded-2xl hover:border-sky-200 hover:bg-white transition-all group shadow-sm">
                        <div class="flex items-center gap-4">
                            <div class={`w-2.5 h-2.5 rounded-full ${iface.up ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]' : 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.4)]'}`}></div>
                            <div class="flex flex-col gap-0.5">
                                <span class="font-mono text-sky-600 font-black text-[13px]">{iface.name}</span>
                                <span class="text-[9px] text-slate-400 font-bold font-mono tracking-tighter uppercase">{iface.mac || 'no-mac-addr'}</span>
                            </div>
                        </div>
                        <span class="text-[9px] font-black text-slate-300 uppercase tracking-widest group-hover:text-sky-500 transition-colors">Physical</span>
                    </div>
                {/each}
            </div>
        </div>

        <!-- IP Assignments -->
        <div class="xl:col-span-7 rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-100 pb-4">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">📍</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">IP Address Pool</h4>
                </div>
                <button onclick={addAssignment} class="px-4 py-1.5 bg-emerald-50 text-emerald-600 rounded-full font-black text-[10px] uppercase hover:bg-emerald-100 transition-all border border-emerald-100 shadow-sm">
                    + Assign IP
                </button>
            </div>

            <div class="flex flex-col gap-4 overflow-y-auto max-h-[400px] pr-2 custom-scrollbar">
                {#each config.network.interfaces.assignments as assign, i}
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex items-center gap-6 group hover:border-emerald-200 hover:bg-white transition-all shadow-sm">
                        <div class="flex-1 flex flex-col gap-2">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Interface</label>
                            <select bind:value={assign.interface} class="rouman-input !py-1.5 !text-sky-600 appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMWU4OGU1Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                                {#each networkState.interfaces as iface}
                                    <option value={iface.name}>{iface.name}</option>
                                {/each}
                            </select>
                        </div>
                        <div class="flex-[1.5] flex flex-col gap-2">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Address / Netmask</label>
                            <input bind:value={assign.address} class="rouman-input !py-1.5 !text-emerald-600 font-black" />
                        </div>
                        <button onclick={() => config.network.interfaces.assignments = config.network.interfaces.assignments.filter((_, idx) => idx !== i)} class="mt-6 text-slate-300 hover:text-red-500 transition-colors p-2 hover:bg-red-50 rounded-lg">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                        </button>
                    </div>
                {/each}
                {#if config.network.interfaces.assignments.length === 0}
                    <div class="py-16 flex flex-col items-center gap-4 border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                        <span class="text-4xl grayscale opacity-20">🌐</span>
                        <span class="text-[10px] text-slate-300 font-black uppercase tracking-[0.3em]">No IP assignments configured</span>
                    </div>
                {/if}
            </div>
        </div>
    </div>

    <!-- Virtuals (VLANs & Bridges) -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <!-- VLANs -->
        <div class="rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-100 pb-4">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">🏷️</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">802.1Q VLANs</h4>
                </div>
                <button onclick={() => config.network.interfaces.vlans = [...config.network.interfaces.vlans, { name: '', parent: 'eth0', vlan_id: 10, enabled: true }]} class="px-4 py-1.5 bg-sky-50 text-sky-600 rounded-full font-black text-[10px] uppercase hover:bg-sky-100 transition-all border border-sky-100 shadow-sm">
                    + Add VLAN
                </button>
            </div>
            <div class="flex flex-col gap-4 overflow-y-auto max-h-[300px] pr-2 custom-scrollbar">
                {#each config.network.interfaces.vlans as vlan, i}
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-5 group hover:border-sky-200 hover:bg-white transition-all shadow-sm">
                        <div class="flex items-center justify-between">
                            <input bind:value={vlan.name} class="bg-transparent border-none text-sky-600 font-black text-[13px] w-1/2 outline-none" placeholder="VLAN ID Name" />
                            <button onclick={() => config.network.interfaces.vlans.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-2 hover:bg-red-50 rounded-lg">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                            </button>
                        </div>
                        <div class="grid grid-cols-2 gap-4">
                            <div class="flex flex-col gap-2">
                                <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Parent Port</label>
                                <select bind:value={vlan.parent} class="rouman-input !py-1.5 appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjMWU4OGU1Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                                    {#each physicalInterfaces as iface}
                                        <option value={iface.name}>{iface.name}</option>
                                    {/each}
                                </select>
                            </div>
                            <div class="flex flex-col gap-2">
                                <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Tag ID</label>
                                <input type="number" bind:value={vlan.vlan_id} class="rouman-input !py-1.5 !text-sky-600 font-black" />
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        </div>

        <!-- Bridges -->
        <div class="rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-100 pb-4">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-600 shadow-sm">🌉</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Bridge Domains</h4>
                </div>
                <button onclick={() => config.network.interfaces.bridges = [...config.network.interfaces.bridges, { name: '', members: [], enabled: true }]} class="px-4 py-1.5 bg-purple-50 text-purple-600 rounded-full font-black text-[10px] uppercase hover:bg-purple-100 transition-all border border-purple-100 shadow-sm">
                    + Add Bridge
                </button>
            </div>
            <div class="flex flex-col gap-4 overflow-y-auto max-h-[300px] pr-2 custom-scrollbar">
                {#each config.network.interfaces.bridges as bridge, i}
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-5 group hover:border-purple-200 hover:bg-white transition-all shadow-sm">
                        <div class="flex items-center justify-between">
                            <input bind:value={bridge.name} class="bg-transparent border-none text-purple-600 font-black text-[13px] w-1/2 outline-none" placeholder="Bridge ID" />
                            <button onclick={() => config.network.interfaces.bridges.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-2 hover:bg-red-50 rounded-lg">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                            </button>
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Slave Ports</label>
                            <input 
                                value={bridge.members.join(', ')} 
                                oninput={(e) => bridge.members = e.currentTarget.value.split(',').map(s => s.trim()).filter(Boolean)} 
                                class="rouman-input !py-1.5 !text-purple-600 font-black" 
                                placeholder="eth1, eth2..."
                            />
                            <p class="text-[9px] text-slate-400 font-bold italic mt-1 tracking-tight">Comma-separated list of interfaces</p>
                        </div>
                    </div>
                {/each}
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
