<script lang="ts">
    import { onMount } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let config = $state({ dns: { sinkhole_enabled: true, dot_enabled: true, doh_enabled: true, udp_enabled: true, doh_url: 'https://cloudflare-dns.com/dns-query', adblock_enabled: true, transparent_intercept: true } } as any);
    let saving = $state(false);

    async function loadConfig() {
        const res = await fetch('/api/v1/config/candidate');
        if (res.ok) {
            const data = await res.json();
            if (data.candidate && data.candidate.dns) {
                config.dns = data.candidate.dns;
            }
        }
    }

    async function saveDnsConfig() {
        saving = true;
        const res = await fetch('/api/v1/config/candidate');
        let payload = (await res.json()).candidate;
        payload.dns = config.dns;

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

<ServiceStarter featurePath="dns.enabled" title="DNS Server" description="Domain Name System service with Adblocking and TLS Sinkhole capabilities.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-emerald-600">DNS & Adblock Shield_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Privacy, Encryption & Sinkhole Filters</p>
        </div>
        <button onclick={saveDnsConfig} disabled={saving} class="rouman-btn-primary flex items-center gap-2">
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Applying...</span>
            {:else}
                <span>💾 Apply Changes_</span>
            {/if}
        </button>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        
        <!-- Filter Engine (Adblock) -->
        <div class="rouman-card flex flex-col gap-6 !bg-white/60">
            <div class="flex justify-between items-center bg-slate-50 p-3 rounded-xl border border-slate-100">
                <span class="font-black tracking-widest text-emerald-600 uppercase">Sinkhole Engine_</span>
                <span class="text-[9px] bg-red-50 text-red-500 px-3 py-0.5 rounded-full font-black uppercase tracking-tight shadow-sm">Network-wide Shield</span>
            </div>

            <div class="flex flex-col gap-5">
                <div class="flex justify-between items-center py-3 border-b border-slate-50">
                    <div class="flex flex-col gap-0.5">
                        <span class="font-black text-slate-800 uppercase tracking-widest text-[10px]">Adblock Filtering</span>
                        <span class="text-slate-400 text-[9px] font-bold">Blocks Ads, Trackers, and Malware</span>
                    </div>
                    <button 
                        onclick={() => config.dns.adblock_enabled = !config.dns.adblock_enabled}
                        class={`text-[9px] font-black px-4 py-1.5 rounded-full transition-all shadow-sm ${config.dns.adblock_enabled ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-100 text-slate-400'}`}
                    >
                        {config.dns.adblock_enabled ? 'ACTIVE' : 'INACTIVE'}
                    </button>
                </div>
                
                <div class="flex justify-between items-center py-3 border-b border-slate-50">
                    <div class="flex flex-col gap-0.5">
                        <span class="font-black text-slate-800 uppercase tracking-widest text-[10px]">Master Resolver</span>
                        <span class="text-slate-400 text-[9px] font-bold">Routes all requests through local engine</span>
                    </div>
                    <button 
                        onclick={() => config.dns.sinkhole_enabled = !config.dns.sinkhole_enabled}
                        class={`text-[9px] font-black px-4 py-1.5 rounded-full transition-all shadow-sm ${config.dns.sinkhole_enabled ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-100 text-slate-400'}`}
                    >
                        {config.dns.sinkhole_enabled ? 'ON' : 'OFF'}
                    </button>
                </div>

                <div class="flex justify-between items-center py-3">
                    <div class="flex flex-col gap-0.5">
                        <span class="font-black text-slate-800 uppercase tracking-widest text-[10px]">Transparent Intercept</span>
                        <span class="text-slate-400 text-[9px] font-bold">Force redirect port 53 to Router</span>
                    </div>
                    <button 
                        onclick={() => config.dns.transparent_intercept = !config.dns.transparent_intercept}
                        class={`text-[9px] font-black px-4 py-1.5 rounded-full transition-all shadow-sm ${config.dns.transparent_intercept ? 'bg-orange-50 text-orange-600 border border-orange-100' : 'bg-slate-100 text-slate-400'}`}
                    >
                        {config.dns.transparent_intercept ? 'ON' : 'OFF'}
                    </button>
                </div>
            </div>
        </div>

        <!-- Privacy & Protocols -->
        <div class="rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex justify-between items-center bg-slate-50 p-3 rounded-xl border border-slate-100">
                <span class="font-black tracking-widest text-sky-600 uppercase">Protocols & Upstream_</span>
            </div>

            <div class="flex flex-col gap-6">
                
                <div class="flex flex-col gap-3">
                    <span class="text-slate-400 font-black uppercase tracking-widest text-[9px]">Active Listeners</span>
                    <div class="flex gap-3 text-[9px] font-black">
                        <button onclick={() => config.dns.udp_enabled = !config.dns.udp_enabled} class={`px-4 py-2 rounded-xl border transition-all shadow-sm flex-1 ${config.dns.udp_enabled ? 'text-blue-600 border-blue-200 bg-blue-50' : 'text-slate-300 border-slate-100 bg-slate-50'}`}>UDP 53</button>
                        <button onclick={() => config.dns.dot_enabled = !config.dns.dot_enabled} class={`px-4 py-2 rounded-xl border transition-all shadow-sm flex-1 ${config.dns.dot_enabled ? 'text-purple-600 border-purple-200 bg-purple-50' : 'text-slate-300 border-slate-100 bg-slate-50'}`}>DoT 853</button>
                        <button onclick={() => config.dns.doh_enabled = !config.dns.doh_enabled} class={`px-4 py-2 rounded-xl border transition-all shadow-sm flex-1 ${config.dns.doh_enabled ? 'text-emerald-600 border-emerald-200 bg-emerald-50' : 'text-slate-300 border-slate-100 bg-slate-50'}`}>DoH 443</button>
                    </div>
                </div>

                <div class="flex flex-col gap-2.5 pt-4 border-t border-slate-50">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Global Upstream DoH URL</label>
                    <input bind:value={config.dns.doh_url} class="rouman-input !py-2 !text-slate-800 font-black" placeholder="https://cloudflare-dns.com/dns-query" />
                    <span class="text-[9px] text-slate-300 font-black italic mt-1 text-right">EXAMPLE: https://dns.quad9.net/dns-query</span>
                </div>
            </div>
        </div>
    </div>
</div>
</ServiceStarter>
