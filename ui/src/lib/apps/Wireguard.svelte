<script lang="ts">
    import { onMount } from 'svelte';

    let config = $state({ wireguard: { interfaces: [] } } as any);
    let saving = $state(false);

    async function loadConfig() {
        const res = await fetch('/api/v1/config/candidate');
        if (res.ok) {
            const data = await res.json();
            if (data.candidate && data.candidate.wireguard) {
                config.wireguard = data.candidate.wireguard;
                if (!config.wireguard.interfaces) config.wireguard.interfaces = [];
            }
        }
    }

    function addInterface() {
        config.wireguard.interfaces = [...config.wireguard.interfaces, {
            name: `wg${config.wireguard.interfaces.length}`,
            private_key: '',
            listen_port: 51820 + config.wireguard.interfaces.length,
            address: '10.0.0.1/24',
            enabled: true,
            peers: []
        }];
    }

    function addPeer(ifaceIndex: number) {
        config.wireguard.interfaces[ifaceIndex].peers = [
            ...(config.wireguard.interfaces[ifaceIndex].peers || []),
            { name: `peer${Date.now().toString().slice(-4)}`, public_key: '', endpoint: '', allowed_ips: ['10.0.0.2/32'] }
        ];
    }

    async function saveWG() {
        saving = true;
        const res = await fetch('/api/v1/config/candidate');
        let payload = (await res.json()).candidate;
        payload.wireguard = config.wireguard;

        await fetch('/api/v1/config/candidate', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        await fetch('/api/v1/config/commit', { method: 'POST' });
        saving = false;
        loadConfig();
    }

    onMount(() => loadConfig());
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">WireGuard VPN_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Secure Tunnels & Peers</p>
        </div>
        <button onclick={saveWG} disabled={saving} class="rouman-btn-primary flex items-center gap-2">
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Applying...</span>
            {:else}
                <span>💾 Apply Changes_</span>
            {/if}
        </button>
    </div>

    <!-- Interfaces -->
    <div class="flex flex-col gap-8">
        <div class="flex justify-between items-center border-b border-slate-100 pb-4">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">🔒</div>
                <h4 class="font-black text-[10px] text-slate-400 tracking-[0.2em] uppercase">WG Interfaces_</h4>
            </div>
            <button onclick={addInterface} class="px-4 py-1.5 bg-sky-50 text-sky-600 rounded-full font-black text-[10px] uppercase hover:bg-sky-100 transition-all border border-sky-100 shadow-sm">+ Add Interface</button>
        </div>

        {#each config.wireguard.interfaces as iface, i}
            <div class="rouman-card flex flex-col gap-8 !bg-white/60">
                <div class="flex items-start justify-between border-b border-slate-50 pb-4">
                    <div class="flex gap-6 items-center">
                        <input bind:value={iface.name} class="bg-transparent border-none text-sky-600 font-black text-[14px] outline-none w-24 focus:text-sky-700 transition-colors" />
                        <button 
                            onclick={() => iface.enabled = !iface.enabled} 
                            class={`text-[9px] font-black px-4 py-1 rounded-full shadow-sm transition-all ${iface.enabled ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-100 text-slate-400 border border-slate-200'}`}
                        >
                            {iface.enabled ? 'ENABLED' : 'DISABLED'}
                        </button>
                    </div>
                    <button onclick={() => config.wireguard.interfaces.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 pb-6 border-b border-slate-50">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Listen Port</label>
                        <input type="number" bind:value={iface.listen_port} class="rouman-input !py-1.5 !text-slate-800 font-black font-mono" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Tunnel Address</label>
                        <input bind:value={iface.address} class="rouman-input !py-1.5 !text-sky-600 font-black font-mono" placeholder="10.0.0.1/24" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1 flex justify-between">
                            Private Key <button class="text-purple-600 hover:underline lowercase font-black" onclick={(e) => { e.preventDefault(); iface.private_key = 'GENERATE_NEW_KEY'; }}>[regenerate]</button>
                        </label>
                        <input bind:value={iface.private_key} type="password" class="rouman-input !py-1.5 !text-purple-600 font-black font-mono" placeholder="(auto-generated)" />
                    </div>
                </div>

                <!-- Peers -->
                <div class="flex flex-col gap-6">
                    <div class="flex justify-between items-center px-1">
                        <div class="flex items-center gap-2">
                            <span class="w-2 h-2 bg-purple-400 rounded-full"></span>
                            <span class="font-black tracking-widest text-slate-400 text-[9px] uppercase">Peers & Remote Clients</span>
                        </div>
                        <button onclick={() => addPeer(i)} class="text-sky-600 hover:text-sky-700 transition-colors text-[9px] font-black uppercase tracking-widest">+ Add Peer</button>
                    </div>
                    
                    <div class="flex flex-col gap-4">
                        {#each iface.peers || [] as peer, j}
                            <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-5 group relative hover:border-purple-200 hover:bg-white transition-all shadow-sm">
                                <div class="flex justify-between items-center">
                                    <input bind:value={peer.name} class="bg-transparent border-none text-slate-800 font-black outline-none w-1/2 text-[12px] focus:text-purple-600 transition-colors" placeholder="peer_identifier" />
                                    <button onclick={() => iface.peers.splice(j, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                        </svg>
                                    </button>
                                </div>
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                    <div class="flex flex-col gap-1.5">
                                        <label class="text-[8px] text-slate-400 uppercase font-black tracking-widest">Public Key</label>
                                        <input bind:value={peer.public_key} class="rouman-input !py-1 !text-purple-600 font-mono text-[9px] font-black" />
                                    </div>
                                    <div class="flex flex-col gap-1.5">
                                        <label class="text-[8px] text-slate-400 uppercase font-black tracking-widest">Endpoint (Optional)</label>
                                        <input bind:value={peer.endpoint} class="rouman-input !py-1 !text-slate-800 font-mono text-[9px] font-black" placeholder="IP:Port" />
                                    </div>
                                    <div class="flex flex-col gap-1.5">
                                        <label class="text-[8px] text-slate-400 uppercase font-black tracking-widest">Allowed IPs</label>
                                        <input 
                                            value={(peer.allowed_ips || []).join(', ')} 
                                            oninput={(e) => peer.allowed_ips = e.currentTarget.value.split(',').map(s => s.trim())} 
                                            class="rouman-input !py-1 !text-sky-600 font-mono text-[9px] font-black" 
                                            placeholder="10.0.0.2/32" 
                                        />
                                    </div>
                                </div>
                            </div>
                        {/each}
                        {#if (iface.peers || []).length === 0}
                            <div class="py-12 text-center text-slate-300 font-black uppercase tracking-[0.3em] bg-slate-50/20 rounded-2xl border-2 border-dashed border-slate-100">No Peers connected to {iface.name}</div>
                        {/if}
                    </div>
                </div>
            </div>
        {/each}
        {#if config.wireguard.interfaces?.length === 0}
            <div class="py-24 text-center text-slate-300 font-black uppercase tracking-[0.3em] bg-slate-50/30 rounded-3xl border-2 border-dashed border-slate-100">No WireGuard Interfaces Configured</div>
        {/if}
    </div>
</div>
