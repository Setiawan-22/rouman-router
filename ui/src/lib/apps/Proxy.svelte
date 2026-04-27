<script lang="ts">
    import { onMount } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let hosts = $state([] as any[]);
    let nodes = $state([] as any[]);
    let newHost = $state({ domain: '', upstream_url: '', ssl_enabled: false, node_id: 'local' });
    let settings = $state({ acme_email: '', cf_api_token: '' });
    let loading = $state(false);

    async function loadProxyData() {
        loading = true;
        try {
            const [resHosts, resSettings, resNodes] = await Promise.all([
                fetch('/api/v1/proxy/hosts'),
                fetch('/api/v1/proxy/settings'),
                fetch('/api/v1/compute/nodes')
            ]);

            if (resHosts.ok) hosts = await resHosts.json();
            if (resSettings.ok) settings = await resSettings.json();
            if (resNodes.ok) {
                const nodeData = await resNodes.json();
                nodes = nodeData.nodes;
            }
        } catch (e) {
            console.error(e);
        }
        loading = false;
    }

    async function addHost() {
        if (!newHost.domain || !newHost.upstream_url) return;
        try {
            const res = await fetch('/api/v1/proxy/hosts', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(newHost)
            });
            if (res.ok) {
                newHost = { domain: '', upstream_url: '', ssl_enabled: false, node_id: 'local' };
                await loadProxyData();
            }
        } catch {}
    }

    async function removeHost(id: number) {
        try {
            const res = await fetch(`/api/v1/proxy/hosts/${id}`, { method: 'DELETE' });
            if (res.ok) await loadProxyData();
        } catch {}
    }

    async function saveSettings() {
        try {
            const res = await fetch('/api/v1/proxy/settings', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(settings)
            });
            if (res.ok) alert("Proxy Settings Saved!");
        } catch {}
    }

    onMount(() => loadProxyData());
</script>

<ServiceStarter featurePath="proxy.enabled" title="Reverse Proxy" description="Nginx-style reverse proxy with automatic ACME SSL for routing public traffic to internal services.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-blue-600">Traffic Proxy_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">L7 Reverse Proxy & SSL Termination</p>
        </div>
        <div class="px-4 py-2 bg-blue-50 border border-blue-100 text-blue-600 rounded-2xl text-[10px] font-black tracking-widest uppercase shadow-sm">
            Hyper-V Edge Relay
        </div>
    </div>

    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        <!-- Settings Panel -->
        <div class="xl:col-span-4 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <div class="w-8 h-8 rounded-lg bg-blue-50 flex items-center justify-center text-blue-600 shadow-sm">🛡️</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">TLS Automation</h4>
            </div>
            
            <div class="flex flex-col gap-5">
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">ACME Registry Email</label>
                    <input type="email" bind:value={settings.acme_email} placeholder="ops@infrastructure.io" class="rouman-input !py-1.5 !text-slate-800 font-black" />
                </div>
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Cloudflare API Token</label>
                    <input type="password" bind:value={settings.cf_api_token} placeholder="••••••••••••••••" class="rouman-input !py-1.5 !text-blue-600 font-black" />
                </div>
                <button onclick={saveSettings} class="mt-4 rouman-btn-primary !bg-slate-50 border !border-slate-200 hover:!bg-white !text-slate-800 !py-3 text-[10px] font-black tracking-widest shadow-sm">
                    Commit TLS Settings_
                </button>
            </div>
        </div>

        <!-- Hosts Panel -->
        <div class="xl:col-span-8 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-50 pb-4">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">🔀</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Routing Table</h4>
                </div>
                <span class="text-[9px] text-slate-300 font-black uppercase tracking-widest">ACTIVE RELAY</span>
            </div>
            
            <div class="flex-1 overflow-y-auto max-h-[350px] custom-scrollbar">
                {#if hosts.length === 0}
                    <div class="py-20 text-center text-slate-300 font-black uppercase tracking-[0.3em] border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                        No active relay paths detected
                    </div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="w-full text-left border-collapse">
                            <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                                <tr>
                                    <th class="p-4 px-2">Public Entrypoint</th>
                                    <th class="p-4 px-2">Internal Sink</th>
                                    <th class="p-4 px-2">Deployment Node</th>
                                    <th class="p-4 px-2 text-center">Security</th>
                                    <th class="p-4"></th>
                                </tr>
                            </thead>
                            <tbody class="text-slate-600">
                                {#each hosts as host}
                                    <tr class="border-b border-slate-50 hover:bg-slate-50/50 transition-all">
                                        <td class="p-4 px-2 font-black text-blue-600 lowercase text-[12px]">{host.domain}</td>
                                        <td class="p-4 px-2 font-black text-slate-400 lowercase font-mono text-[11px]">{host.upstream_url}</td>
                                        <td class="p-4 px-2">
                                            <span class={`text-[9px] font-black uppercase px-2 py-0.5 rounded-md border ${host.node_id === 'local' ? 'bg-sky-50 text-sky-600 border-sky-100' : 'bg-indigo-50 text-indigo-600 border-indigo-100'}`}>
                                                {nodes.find(n => n.id === host.node_id)?.hostname || host.node_id}
                                            </span>
                                        </td>
                                        <td class="p-4 text-center">
                                            <div class={`inline-block px-3 py-1 rounded-full text-[9px] font-black shadow-sm ${host.ssl_enabled ? 'bg-emerald-50 text-emerald-600' : 'bg-slate-100 text-slate-400'}`}>
                                                {host.ssl_enabled ? 'SSL ACTIVE' : 'PLAINTEXT'}
                                            </div>
                                        </td>
                                        <td class="p-4 text-right pr-2">
                                            <button onclick={() => removeHost(host.id)} class="text-slate-300 hover:text-red-500 transition-colors p-2 hover:bg-red-50 rounded-lg">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                </svg>
                                            </button>
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </div>

            <div class="mt-auto pt-8 border-t border-slate-100 flex flex-col gap-6">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Public Domain</label>
                        <input type="text" bind:value={newHost.domain} placeholder="service.rouman.net" class="rouman-input !py-1.5 !text-slate-800 font-black" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Internal Sink URL</label>
                        <input type="text" bind:value={newHost.upstream_url} placeholder="http://192.168.10.5:8080" class="rouman-input !py-1.5 !text-emerald-600 font-black" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Target Node</label>
                        <select bind:value={newHost.node_id} class="rouman-input !py-1.5 !text-indigo-600 font-black">
                            {#each nodes as node}
                                <option value={node.id}>{node.hostname}</option>
                            {/each}
                        </select>
                    </div>
                </div>
                <div class="flex items-center justify-between">
                    <label class="flex items-center gap-4 cursor-pointer group">
                        <div class="relative">
                            <input type="checkbox" bind:checked={newHost.ssl_enabled} class="sr-only" />
                            <div class={`w-10 h-5 rounded-full transition-all shadow-inner ${newHost.ssl_enabled ? 'bg-emerald-500' : 'bg-slate-200'}`}></div>
                            <div class={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-all shadow-sm ${newHost.ssl_enabled ? 'translate-x-5' : ''}`}></div>
                        </div>
                        <span class="text-[10px] font-black text-slate-400 uppercase tracking-widest group-hover:text-slate-600 transition-colors">Automatic ACME SSL Certification</span>
                    </label>
                    <button onclick={addHost} class="rouman-btn-primary !py-2.5 px-10 text-[10px] font-black tracking-widest shadow-lg">Establish Route_</button>
                </div>
            </div>
        </div>
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
