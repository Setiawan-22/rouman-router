<script lang="ts">
    import { onMount } from 'svelte';

    interface ServiceDef {
        id: string;
        name: string;
        description: string;
        configPath: string;
        icon: string;
        category: string;
    }

    const SERVICES: ServiceDef[] = [
        { id: 'dhcp', name: 'DHCP Server', description: 'Dynamic Host Configuration Protocol for automatic IP addressing', configPath: 'network.dhcp.enabled', icon: '🌐', category: 'Network' },
        { id: 'dns', name: 'DNS Sinkhole', description: 'Adblock & Privacy DNS with DoH/DoT support', configPath: 'dns.enabled', icon: '🛡️', category: 'Network' },
        { id: 'firewall', name: 'eBPF Firewall', description: 'XDP/eBPF packet filtering & IP blacklist engine', configPath: 'firewall.enabled', icon: '🔥', category: 'Security' },
        { id: 'proxy', name: 'Reverse Proxy', description: 'L7 reverse proxy with auto ACME SSL certificates', configPath: 'proxy.enabled', icon: '🔀', category: 'Network' },
        { id: 'cloudflare', name: 'Cloudflare Tunnel', description: 'Zero Trust tunnels for secure public access', configPath: 'cloudflare.enabled', icon: '☁️', category: 'Network' },
        { id: 'rdp', name: 'Neighbor Discovery', description: 'Rouman Discovery Protocol for L2 device scanning', configPath: 'rdp.enabled', icon: '📡', category: 'Network' },
        { id: 'radius', name: 'RADIUS Server', description: 'AAA authentication server for hotspot & enterprise access', configPath: 'radius.enabled', icon: '🔑', category: 'Security' },
        { id: 'pppoe', name: 'PPPoE Client', description: 'Point-to-Point Protocol over Ethernet dialer', configPath: 'network.pppoe.enabled', icon: '📞', category: 'Network' },
        { id: 'microvm', name: 'MicroVMs', description: 'Firecracker MicroVM engine for lightweight isolation', configPath: 'compute.microvm_enabled', icon: '🖥️', category: 'Compute' },
        { id: 'container', name: 'Containers', description: 'Youki/Containerd OCI container runtime', configPath: 'compute.container_enabled', icon: '📦', category: 'Compute' },
    ];

    let serviceStates = $state<Record<string, boolean>>({});
    let loading = $state(true);
    let toggling = $state<string | null>(null);

    function getConfigValue(obj: any, path: string): boolean {
        let val = obj;
        for (const part of path.split('.')) {
            if (val == null) return false;
            val = val[part];
        }
        return !!val;
    }

    function setConfigValue(obj: any, path: string, value: boolean): void {
        const parts = path.split('.');
        let target = obj;
        for (let i = 0; i < parts.length - 1; i++) {
            if (target[parts[i]] == null) target[parts[i]] = {};
            target = target[parts[i]];
        }
        target[parts[parts.length - 1]] = value;
    }

    async function loadStatus() {
        loading = true;
        try {
            const res = await fetch('/api/v1/config/active', { credentials: 'same-origin' });
            if (res.ok) {
                const cfg = (await res.json()).active;
                const states: Record<string, boolean> = {};
                for (const svc of SERVICES) {
                    states[svc.id] = getConfigValue(cfg, svc.configPath);
                }
                serviceStates = states;
            }
        } catch (e) {
            console.error(e);
        }
        loading = false;
    }

    async function toggleService(svc: ServiceDef) {
        toggling = svc.id;
        try {
            const res = await fetch('/api/v1/config/candidate', { credentials: 'same-origin' });
            if (!res.ok) { toggling = null; return; }
            const candidate = (await res.json()).candidate;

            const newValue = !serviceStates[svc.id];
            setConfigValue(candidate, svc.configPath, newValue);

            const putRes = await fetch('/api/v1/config/candidate', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(candidate),
                credentials: 'same-origin'
            });
            if (!putRes.ok) { toggling = null; return; }

            const commitRes = await fetch('/api/v1/config/commit', { method: 'POST', credentials: 'same-origin' });
            if (commitRes.ok) {
                serviceStates[svc.id] = newValue;
            }
        } catch (e) {
            console.error(e);
        }
        toggling = null;
    }

    let showUpdateModal = $state(false);
    let updateLogs = $state("");
    let updateStatus = $state<"idle" | "checking" | "upgrading" | "done" | "error">("idle");

    async function checkUpdates() {
        showUpdateModal = true;
        updateStatus = "checking";
        updateLogs = "> apt-get update && apt-get --just-print upgrade\n";
        try {
            const res = await fetch('/api/v1/system/update/check', { method: 'POST', credentials: 'same-origin' });
            const data = await res.json();
            if (data.status === "success") {
                updateLogs += data.stdout + "\n" + data.stderr;
                updateStatus = "idle";
            } else {
                updateLogs += "\nError: " + data.message;
                updateStatus = "error";
            }
        } catch (e) {
            updateLogs += "\nConnection error.";
            updateStatus = "error";
        }
    }

    async function runUpgrade() {
        updateStatus = "upgrading";
        updateLogs += "\n\n> DEBIAN_FRONTEND=noninteractive apt-get upgrade -y\n";
        try {
            const res = await fetch('/api/v1/system/update/upgrade', { method: 'POST', credentials: 'same-origin' });
            const data = await res.json();
            if (data.status === "success") {
                updateLogs += data.stdout + "\n" + data.stderr;
                updateStatus = "done";
            } else {
                updateLogs += "\nError: " + data.message;
                updateStatus = "error";
            }
        } catch (e) {
            updateLogs += "\nConnection error.";
            updateStatus = "error";
        }
    }

    let categories = $derived([...new Set(SERVICES.map(s => s.category))]);
    let enabledCount = $derived(Object.values(serviceStates).filter(Boolean).length);

    onMount(() => loadStatus());
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto relative">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Service Manager_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Zero-Trust Lifecycle Control</p>
        </div>
        <div class="flex items-center gap-4">
            <div class="flex items-center gap-2 px-4 py-2 bg-sky-50 rounded-full border border-sky-100 shadow-sm">
                <div class="w-2 h-2 rounded-full bg-sky-600 animate-pulse"></div>
                <span class="text-[9px] font-black text-sky-600 tracking-widest uppercase">{enabledCount} / {SERVICES.length} ACTIVE</span>
            </div>
            <button onclick={checkUpdates} class="px-5 py-2.5 bg-sky-600 text-white rounded-xl font-black text-[10px] tracking-widest uppercase hover:bg-sky-700 transition-all flex items-center gap-2 shadow-lg">
                <span>🔄</span> Update System
            </button>
            <button onclick={loadStatus} class="px-5 py-2.5 bg-white border border-slate-200 text-slate-500 rounded-xl font-black text-[10px] tracking-widest uppercase hover:bg-slate-50 transition-all shadow-sm">
                Refresh
            </button>
        </div>
    </div>

    {#if showUpdateModal}
        <div class="fixed inset-0 bg-slate-900/40 backdrop-blur-md z-50 flex items-center justify-center p-6">
            <div class="bg-slate-900 w-full max-w-3xl rounded-3xl shadow-2xl border border-slate-800 overflow-hidden flex flex-col max-h-[85vh]">
                <div class="p-5 border-b border-slate-800 flex items-center justify-between bg-slate-800/50 backdrop-blur-md">
                    <div class="flex items-center gap-3">
                        <div class="flex gap-2">
                            <div class="w-3 h-3 rounded-full bg-rose-500/80 shadow-lg shadow-rose-500/20"></div>
                            <div class="w-3 h-3 rounded-full bg-amber-500/80 shadow-lg shadow-amber-500/20"></div>
                            <div class="w-3 h-3 rounded-full bg-emerald-500/80 shadow-lg shadow-emerald-500/20"></div>
                        </div>
                        <span class="text-[10px] font-mono text-slate-400 ml-3 uppercase tracking-[0.3em] font-black">Core Upgrade Terminal_</span>
                    </div>
                    <button onclick={() => showUpdateModal = false} class="text-slate-500 hover:text-white transition-colors text-2xl font-black">&times;</button>
                </div>
                
                <div class="flex-1 overflow-auto p-6 font-mono text-[12px] text-sky-400 bg-slate-950/80 custom-scrollbar">
                    <pre class="whitespace-pre-wrap leading-relaxed">{updateLogs}</pre>
                    {#if updateStatus === "checking" || updateStatus === "upgrading"}
                        <div class="flex items-center gap-3 mt-4 animate-pulse">
                            <span class="w-2 h-4 bg-sky-500"></span>
                            <span class="text-white font-black uppercase tracking-widest text-[10px]">Processing Cluster Transaction...</span>
                        </div>
                    {/if}
                </div>

                <div class="p-6 border-t border-slate-800 bg-slate-900/50 flex items-center justify-end gap-4">
                    {#if updateStatus === "idle"}
                        <button onclick={runUpgrade} class="px-8 py-3 bg-sky-600 text-white rounded-xl font-black text-[10px] uppercase tracking-widest hover:bg-sky-500 shadow-lg shadow-sky-600/20 transition-all">
                            Initialize Installation_
                        </button>
                    {/if}
                    <button onclick={() => showUpdateModal = false} class="px-8 py-3 bg-slate-800 text-slate-300 rounded-xl font-black text-[10px] uppercase tracking-widest hover:bg-slate-700 transition-all">
                        {updateStatus === "done" ? "Exit Console" : "Abort Transaction"}
                    </button>
                </div>
            </div>
        </div>
    {/if}

    {#if loading}
        <div class="flex items-center justify-center py-32">
            <div class="flex flex-col items-center gap-4">
                <div class="w-12 h-12 border-4 border-sky-600 border-t-transparent rounded-full animate-spin shadow-lg"></div>
                <span class="text-[10px] text-sky-600 font-black tracking-[0.4em] uppercase animate-pulse">Synchronizing Service Matrix_</span>
            </div>
        </div>
    {:else}
        <div class="flex flex-col gap-12">
            {#each categories as category}
                <div class="flex flex-col gap-6">
                    <div class="flex items-center gap-4">
                        <span class="text-[10px] font-black text-slate-400 tracking-[0.4em] uppercase whitespace-nowrap">{category}</span>
                        <div class="h-[1px] flex-1 bg-gradient-to-r from-slate-100 to-transparent"></div>
                    </div>
                    
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        {#each SERVICES.filter(s => s.category === category) as svc}
                            <div class="rouman-card !p-6 flex flex-col gap-4 border-slate-100 transition-all duration-300 hover:border-sky-300 hover:shadow-xl group relative overflow-hidden !bg-white/60">
                                {#if serviceStates[svc.id]}
                                    <div class="absolute top-0 right-0 w-24 h-24 bg-sky-50/50 rounded-bl-full -mr-12 -mt-12 group-hover:scale-150 transition-transform duration-500"></div>
                                {/if}
                                
                                <div class="flex items-start justify-between gap-4 relative z-10">
                                    <div class="flex items-start gap-4 flex-1">
                                        <div class="w-12 h-12 rounded-2xl bg-slate-50 flex items-center justify-center text-2xl group-hover:scale-110 transition-transform duration-300 shadow-sm border border-slate-100">
                                            {svc.icon}
                                        </div>
                                        <div class="flex flex-col gap-1.5 flex-1 pt-1">
                                            <div class="flex items-center gap-3">
                                                <span class="font-black text-[13px] text-slate-800 tracking-tight">{svc.name}</span>
                                                {#if serviceStates[svc.id]}
                                                    <span class="px-2 py-0.5 bg-emerald-50 text-emerald-600 rounded-md text-[8px] font-black uppercase tracking-widest border border-emerald-100 shadow-sm">ACTIVE</span>
                                                {:else}
                                                    <span class="px-2 py-0.5 bg-slate-100 text-slate-400 rounded-md text-[8px] font-black uppercase tracking-widest border border-slate-200">INACTIVE</span>
                                                {/if}
                                            </div>
                                            <p class="text-[10px] text-slate-500 font-bold leading-relaxed pr-8">{svc.description}</p>
                                        </div>
                                    </div>
                                    
                                    <button 
                                        onclick={() => toggleService(svc)}
                                        disabled={toggling === svc.id}
                                        class={`relative w-12 h-6.5 rounded-full transition-all duration-500 shadow-inner flex items-center px-1 ${serviceStates[svc.id] ? 'bg-sky-600' : 'bg-slate-200'}`}
                                    >
                                        {#if toggling === svc.id}
                                            <div class="absolute inset-0 flex items-center justify-center">
                                                <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                            </div>
                                        {:else}
                                            <div class={`w-4.5 h-4.5 bg-white rounded-full shadow-lg transform transition-transform duration-500 ${serviceStates[svc.id] ? 'translate-x-5.5' : 'translate-x-0'}`}></div>
                                        {/if}
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    <div class="mt-8 p-6 rounded-3xl bg-amber-50 border border-amber-100 flex items-start gap-4 shadow-sm">
        <div class="w-10 h-10 rounded-2xl bg-white flex items-center justify-center text-xl shadow-sm border border-amber-100">⚠️</div>
        <div class="flex flex-col gap-1.5 pt-1">
            <span class="text-[10px] font-black text-amber-700 tracking-[0.2em] uppercase">Zero-Trust Policy Enforcement</span>
            <span class="text-[10px] text-amber-600/80 font-bold leading-relaxed">All system services are restricted by default. Activation authorizes kernel-level execution and resource mounting. Security audits recommend periodic service validation.</span>
        </div>
    </div>
</div>
