<script lang="ts">
    import { onMount } from 'svelte';

    let candidateConfig = $state(null as any);
    let wanInterface = $state("eth0");
    let enableNat = $state(false);
    
    // Port Forwarding state
    let portForwards = $state([] as any[]);
    let newForward = $state({
        proto: "tcp",
        ext_port: 80,
        int_ip: "192.168.1.10",
        int_port: 80,
        comment: ""
    });

    // QoS state
    let downloadLimit = $state(0);
    let uploadLimit = $state(0);

    async function loadConfig() {
        const resCand = await fetch('/api/v1/config/candidate');
        if (resCand.ok) {
            candidateConfig = (await resCand.json()).candidate;
            if (candidateConfig && candidateConfig.network) {
                wanInterface = candidateConfig.network.wan_interface || "eth0";
                enableNat = candidateConfig.network.enable_nat || false;
                portForwards = candidateConfig.network.port_forwards || [];
                downloadLimit = candidateConfig.network.qos?.download_mbps || 0;
                uploadLimit = candidateConfig.network.qos?.upload_mbps || 0;
            }
        }
    }

    function addForwardRule() {
        portForwards = [...portForwards, { ...newForward }];
        newForward = { proto: "tcp", ext_port: 80, int_ip: "192.168.1.10", int_port: 80, comment: "" };
    }

    function removeForwardRule(index: number) {
        portForwards = portForwards.filter((_, i) => i !== index);
    }

    async function saveRouting() {
        if (!candidateConfig) return;
        let payload = JSON.parse(JSON.stringify(candidateConfig));
        if (!payload.network) payload.network = {};
        
        payload.network.wan_interface = wanInterface;
        payload.network.enable_nat = enableNat;
        payload.network.port_forwards = portForwards;
        payload.network.qos = {
            download_mbps: downloadLimit,
            upload_mbps: uploadLimit
        };

        await fetch('/api/v1/config/candidate', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
    }
    
    async function commitNow() {
        await saveRouting();
        await fetch('/api/v1/config/commit', { method: 'POST' });
        loadConfig();
    }

    onMount(() => loadConfig());
</script>

<svelte:head>
    <title>ROUTING & NAT // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-6xl mx-auto mt-6 mb-10 flex flex-col gap-8">
    <div class="flex items-center justify-between mb-4">
        <h2 class="text-3xl text-white font-bold tracking-[0.2em]">ROUTING <span class="text-neon-green">&</span> NAT</h2>
        <div class="px-3 py-1 rounded text-[10px] font-bold tracking-widest uppercase border border-neon-green/40 text-neon-green bg-neon-green/10 shadow-[0_0_15px_rgba(0,255,196,0.05)]">
            NFTABLES DATA PLANE
        </div>
    </div>

    <!-- Overview & Basic NAT -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-8">
        
        <!-- WAN & NAT Toggle (Left Column) -->
        <div class="md:col-span-5 flex flex-col gap-8">
            <!-- WAN Selection -->
            <div class="p-6 rounded-xl border border-neon-green/20 bg-black/40 backdrop-blur-md relative overflow-hidden flex flex-col shadow-[0_0_20px_rgba(0,255,196,0.05)]">
                <h3 class="text-lg font-bold text-neon-green tracking-widest mb-4 border-b border-neon-green/10 pb-2">WAN CONFIG_</h3>
                <p class="text-[10px] text-gray-500 mb-6 uppercase tracking-wider">Define external network gateway interface.</p>
                <div class="flex flex-col gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-neon-green/60 uppercase tracking-[0.2em]">Interface Name</label>
                        <input type="text" bind:value={wanInterface} class="bg-black/80 border border-neon-green/30 rounded p-3 text-neon-green font-mono text-sm focus:outline-none focus:border-neon-green transition-all shadow-inner" />
                    </div>
                </div>
            </div>

            <!-- NAT Toggle -->
            <div class="p-6 rounded-xl border {enableNat ? 'border-neon-green/60 bg-neon-green/5 shadow-[0_0_30px_rgba(0,255,196,0.1)]' : 'border-gray-800 bg-black/40'} flex flex-col transition-all duration-500">
                <div class="flex justify-between items-center mb-4">
                    <div>
                        <h3 class="text-lg font-bold {enableNat ? 'text-neon-green' : 'text-gray-500'} tracking-widest uppercase">Ip Masquerade</h3>
                        <p class="text-[9px] {enableNat ? 'text-neon-green/60' : 'text-gray-600'} tracking-widest uppercase mt-1">Enable LAN Internet Sharing</p>
                    </div>
                    <button 
                        onclick={() => enableNat = !enableNat}
                        class="w-14 h-7 rounded-full p-1.5 transition-colors duration-300 focus:outline-none {enableNat ? 'bg-neon-green/80' : 'bg-gray-800 border border-gray-700'}"
                    >
                        <div class="w-4 h-4 rounded-full bg-black transform transition-transform duration-300 {enableNat ? 'translate-x-7' : 'translate-x-0'}"></div>
                    </button>
                </div>
            </div>

            <!-- QoS / Bandwidth Limit -->
            <div class="p-6 rounded-xl border border-blue-500/30 bg-black/40 backdrop-blur-md flex flex-col shadow-[0_0_20px_rgba(0,0,255,0.05)]">
                 <h3 class="text-lg font-bold text-blue-400 tracking-widest mb-4 border-b border-blue-500/20 pb-2 uppercase">QoS Management (CAKE)_</h3>
                 <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-blue-400/60 uppercase tracking-[0.1em]">Upload Limit (MBPS)</label>
                        <input type="number" bind:value={uploadLimit} class="bg-black/80 border border-blue-500/20 rounded p-3 text-blue-400 font-mono text-sm focus:outline-none focus:border-blue-400 shadow-inner" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-blue-400/60 uppercase tracking-[0.1em]">Download Limit (MBPS)</label>
                        <input type="number" bind:value={downloadLimit} class="bg-black/80 border border-blue-500/20 rounded p-3 text-blue-400 font-mono text-sm focus:outline-none focus:border-blue-400 shadow-inner" />
                    </div>
                 </div>
                 <p class="text-[9px] text-gray-500 mt-4 uppercase italic">Note: Only Upload Limit is applied to WAN via CAKE qdisc to prevent bufferbloat.</p>
            </div>
        </div>

        <!-- Port Forwarding (Right Column) -->
        <div class="md:col-span-7 p-6 rounded-xl border border-white/10 bg-black/40 backdrop-blur-md flex flex-col shadow-xl">
            <h3 class="text-lg font-bold text-white tracking-widest mb-6 border-b border-white/10 pb-4 uppercase">Port Forwarding Rules (DNAT)</h3>
            
            <!-- Rules Table -->
            <div class="flex-1 overflow-y-auto max-h-[300px] mb-6 pr-2">
                {#if portForwards.length === 0}
                    <div class="text-center py-10 text-gray-600 text-[11px] uppercase tracking-widest border border-dashed border-white/10 rounded">No active forwarding rules</div>
                {:else}
                    <table class="w-full text-left text-[11px] uppercase tracking-wider">
                        <thead class="text-gray-500 border-b border-white/10">
                            <tr>
                                <th class="pb-2">Proto</th>
                                <th class="pb-2">Ext Port</th>
                                <th class="pb-2">Internal Dest</th>
                                <th class="pb-2"></th>
                            </tr>
                        </thead>
                        <tbody class="text-gray-300">
                            {#each portForwards as rule, i}
                                <tr class="border-b border-white/5 group hover:bg-white/5 transition-all">
                                    <td class="py-3 font-mono">{rule.proto}</td>
                                    <td class="py-3 font-mono text-neon-green">{rule.ext_port}</td>
                                    <td class="py-3 font-mono">
                                        {rule.int_ip}<span class="text-gray-600">:</span>{rule.int_port}
                                    </td>
                                    <td class="py-3 text-right">
                                        <button onclick={() => removeForwardRule(i)} class="text-red-500 opacity-0 group-hover:opacity-100 transition-opacity hover:text-red-400">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                                        </button>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                {/if}
            </div>

            <!-- Add New Rule Form -->
            <div class="p-4 rounded-lg bg-white/5 border border-white/10">
                <h4 class="text-[10px] font-bold text-gray-400 mb-4 tracking-[0.2em] uppercase">Add New Rule_</h4>
                <div class="grid grid-cols-4 gap-3">
                    <select bind:value={newForward.proto} class="bg-black border border-white/10 rounded p-2 text-xs text-white focus:outline-none focus:border-neon-green">
                        <option value="tcp">TCP</option>
                        <option value="udp">UDP</option>
                    </select>
                    <input type="number" bind:value={newForward.ext_port} placeholder="Ext Port" class="bg-black border border-white/10 rounded p-2 text-xs text-white placeholder-gray-700" />
                    <input type="text" bind:value={newForward.int_ip} placeholder="Int IP" class="bg-black border border-white/10 rounded p-2 text-xs text-white placeholder-gray-700" />
                    <input type="number" bind:value={newForward.int_port} placeholder="Int Port" class="bg-black border border-white/10 rounded p-2 text-xs text-white placeholder-gray-700" />
                </div>
                <button onclick={addForwardRule} class="w-full mt-4 py-2 bg-white/10 hover:bg-neon-green hover:text-black rounded text-[10px] font-bold tracking-[0.2em] transition-all uppercase">
                    Register Forwarding Rule
                </button>
            </div>
        </div>
    </div>

    <!-- Deep Packet Inspection Status (L7) -->
    <div class="p-6 rounded-xl border border-red-500/20 bg-red-950/5 flex items-center justify-between">
        <div class="flex gap-4 items-center">
            <div class="p-3 rounded-full bg-red-500/20 text-red-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
            </div>
            <div>
                <h4 class="text-sm font-bold text-white tracking-widest uppercase">EBPF SNI FILTERING (DPI)</h4>
                <p class="text-[10px] text-gray-400 mt-1 uppercase tracking-wide">Status: Active & Parallel to DNS Sinkhole</p>
            </div>
        </div>
        <a href="/dns" class="text-[10px] font-bold text-neon-green hover:underline tracking-widest uppercase border border-neon-green/20 px-4 py-2 rounded">Manage Blocklist</a>
    </div>

    <!-- Action Bar -->
    <div class="flex gap-4 justify-end items-center mt-4">
        <span class="text-[9px] text-gray-600 italic tracking-widest uppercase">Changes require kernel commit to take effect</span>
        <button onclick={saveRouting} class="px-6 py-3 bg-black border border-neon-green/30 text-neon-green rounded font-bold text-[10px] tracking-widest hover:bg-neon-green/10 transition-all uppercase">
            Save Draft
        </button>
        <button onclick={commitNow} class="px-10 py-3 bg-neon-green text-black rounded font-bold text-[10px] tracking-widest hover:bg-white hover:shadow-[0_0_30px_rgba(0,255,196,0.5)] transition-all uppercase group">
            COMMIT CONFIG TO KERNEL_
        </button>
    </div>
</div>
