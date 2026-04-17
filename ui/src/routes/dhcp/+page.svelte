<script lang="ts">
    import { onMount } from 'svelte';

    let candidateConfig = $state(null as any);
    let interfaces = $state([] as any[]);
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
        const [resCfg, resIface] = await Promise.all([
            fetch('/api/v1/config/candidate'),
            fetch('/api/v1/network/interfaces')
        ]);

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

        if (resIface.ok) {
            interfaces = (await resIface.json()).interfaces || [];
        }
        
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
        
        alert("DHCP Draft Saved. Don't forget to COMMIT.");
    }

    async function commitNow() {
        await saveDhcp();
        await fetch('/api/v1/config/commit', { method: 'POST' });
        loadData();
    }

    onMount(() => loadData());
</script>

<svelte:head>
    <title>DHCP SERVER // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-6xl mx-auto mt-6 mb-10 flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div class="flex flex-col gap-1">
            <h2 class="text-3xl text-white font-bold tracking-[0.2em] uppercase">DHCP Server_</h2>
            <p class="text-[10px] text-neon-green/60 tracking-widest uppercase">MikroTik Style Flexible IP Orchestration</p>
        </div>
        <div class="flex gap-4">
            <!-- Global Enable Toggle -->
            <button 
                onclick={() => enabled = !enabled}
                class="flex items-center gap-4 px-6 py-2 rounded border {enabled ? 'border-neon-green bg-neon-green/10 text-neon-green shadow-[0_0_15px_rgba(0,255,196,0.2)]' : 'border-gray-800 text-gray-600'} transition-all font-bold text-[10px] tracking-widest uppercase"
            >
                {enabled ? 'Service: ACTIVE' : 'Service: DISABLED'}
                <div class="w-2 h-2 rounded-full {enabled ? 'bg-neon-green animate-pulse' : 'bg-gray-800'}"></div>
            </button>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
        
        <!-- MAIN CONFIG (Left) -->
        <div class="lg:col-span-4 flex flex-col gap-6">
            <div class="p-6 rounded-xl border border-white/5 bg-white/[0.02] backdrop-blur-md flex flex-col gap-6">
                <h3 class="text-xs font-bold text-white tracking-widest uppercase border-b border-white/10 pb-2">Network Pool_</h3>
                
                <div class="flex flex-col gap-4">
                    <div class="flex flex-col gap-1">
                        <label class="text-[9px] text-gray-500 uppercase tracking-widest">Interface</label>
                        <select bind:value={selectedInterface} class="bg-black/80 border border-white/10 rounded p-3 text-xs text-neon-green font-mono focus:outline-none focus:border-neon-green">
                            {#each interfaces as iface}
                                <option value={iface.name}>{iface.name} ({iface.is_up ? 'UP' : 'DOWN'})</option>
                            {/each}
                        </select>
                    </div>

                    <div class="flex flex-col gap-1">
                        <label class="text-[9px] text-gray-500 uppercase tracking-widest">Address Range Start</label>
                        <input type="text" bind:value={rangeStart} class="bg-black/80 border border-white/10 rounded p-3 text-xs text-white font-mono focus:outline-none focus:border-neon-green" />
                    </div>

                    <div class="flex flex-col gap-1">
                        <label class="text-[9px] text-gray-500 uppercase tracking-widest">Address Range End</label>
                        <input type="text" bind:value={rangeEnd} class="bg-black/80 border border-white/10 rounded p-3 text-xs text-white font-mono focus:outline-none focus:border-neon-green" />
                    </div>
                </div>
            </div>

            <div class="p-6 rounded-xl border border-white/5 bg-white/[0.02] backdrop-blur-md flex flex-col gap-6">
                <h3 class="text-xs font-bold text-white tracking-widest uppercase border-b border-white/10 pb-2">Client Options_</h3>
                
                <div class="flex flex-col gap-4">
                    <div class="flex flex-col gap-1">
                        <label class="text-[9px] text-gray-500 uppercase tracking-widest">Default Gateway</label>
                        <input type="text" bind:value={gateway} class="bg-black/80 border border-white/10 rounded p-3 text-xs text-white font-mono focus:outline-none focus:border-neon-green" />
                    </div>

                    <div class="flex flex-col gap-1">
                        <label class="text-[9px] text-gray-500 uppercase tracking-widest">DNS Servers (comma sep)</label>
                        <input type="text" bind:value={dnsServers} class="bg-black/80 border border-white/10 rounded p-3 text-xs text-white font-mono focus:outline-none focus:border-neon-green" />
                    </div>

                    <div class="flex flex-col gap-1">
                        <label class="text-[9px] text-gray-500 uppercase tracking-widest">Lease Time (seconds)</label>
                        <input type="number" bind:value={leaseTime} class="bg-black/80 border border-white/10 rounded p-3 text-xs text-white font-mono focus:outline-none focus:border-neon-green" />
                    </div>
                </div>
            </div>
        </div>

        <!-- STATIC LEASES (Right) -->
        <div class="lg:col-span-8 flex flex-col gap-6">
            <div class="flex-1 p-6 rounded-xl border border-white/5 bg-white/[0.02] backdrop-blur-md flex flex-col">
                <h3 class="text-xs font-bold text-white tracking-widest uppercase border-b border-white/10 pb-4 mb-4 flex justify-between">
                    Static Leases (Reservations)_
                    <span class="text-gray-600">IP -> DHCP -> Static</span>
                </h3>

                <div class="flex-1 overflow-y-auto max-h-[400px] mb-6 pr-2">
                    {#if staticLeases.length === 0}
                        <div class="text-center py-20 text-gray-700 text-[10px] uppercase tracking-widest italic border border-dashed border-white/5 rounded">
                            No static reservations found. Add one below.
                        </div>
                    {:else}
                        <table class="w-full text-left text-[11px] uppercase tracking-wider">
                            <thead class="text-gray-500 border-b border-white/10">
                                <tr>
                                    <th class="pb-2">MAC Address</th>
                                    <th class="pb-2">Fixed IP</th>
                                    <th class="pb-2">ID / Hostname</th>
                                    <th class="pb-2"></th>
                                </tr>
                            </thead>
                            <tbody class="text-gray-300">
                                {#each staticLeases as lease, i}
                                    <tr class="border-b border-white/5 hover:bg-white/5 transition-all">
                                        <td class="py-4 font-mono text-neon-green">{lease.mac}</td>
                                        <td class="py-4 font-mono text-white">{lease.ip}</td>
                                        <td class="py-4 text-gray-500 italic">{lease.hostname || 'No Label'}</td>
                                        <td class="py-4 text-right">
                                            <button onclick={() => removeStaticLease(i)} class="text-red-500 hover:text-red-400">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                                            </button>
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    {/if}
                </div>

                <!-- Add New Static -->
                <div class="p-4 rounded-lg bg-black/60 border border-white/10 grid grid-cols-1 md:grid-cols-4 gap-4">
                    <input type="text" bind:value={newStatic.mac} placeholder="MAC ADDR" class="bg-black border border-white/10 rounded p-2 text-xs text-white" />
                    <input type="text" bind:value={newStatic.ip} placeholder="FIXED IP" class="bg-black border border-white/10 rounded p-2 text-xs text-white" />
                    <input type="text" bind:value={newStatic.hostname} placeholder="COMMENT / HOST" class="bg-black border border-white/10 rounded p-2 text-xs text-white" />
                    <button onclick={addStaticLease} class="bg-neon-green text-black font-bold py-2 rounded text-[10px] tracking-widest uppercase hover:bg-white transition-all">Add Static</button>
                </div>
            </div>
        </div>
    </div>

    <!-- Actions -->
    <div class="flex justify-end gap-4 mt-4 border-t border-white/10 pt-8">
        <button onclick={saveDhcp} class="px-8 py-3 bg-black border border-white/20 text-white rounded font-bold text-[10px] tracking-widest hover:border-white transition-all uppercase">
            Save Draft
        </button>
        <button onclick={commitNow} class="px-10 py-3 bg-white text-black rounded font-bold text-[10px] tracking-widest hover:bg-neon-green transition-all uppercase shadow-[0_0_20px_rgba(255,255,255,0.2)]">
            COMMIT DHCP CONFIG_
        </button>
    </div>
</div>
