<script>
  import { onMount } from 'svelte';
  import { ShieldAlert, ShieldCheck, Lock, Globe, Ghost, Activity, Trash, Plus, Info } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let config = { dns: {}, security: {}, firewall: { blacklist_ips: [] } };
  let stats = { total_packets: 0, dropped_packets: 0, engine: 'None' };
  let loading = true;
  let newIp = "";

  async function fetchData() {
    try {
      const configRes = await fetch('/api/v1/config/active');
      const configData = await configRes.json();
      config = configData.active;

      const statsRes = await fetch('/api/v1/firewall/stats');
      stats = await statsRes.json();
    } catch (e) {
      console.error("Failed to fetch security data", e);
    } finally {
      loading = false;
    }
  }

  async function addToBlacklist() {
    if (!newIp) return;
    try {
      const res = await fetch('/api/v1/firewall/blacklist', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ip: newIp })
      });
      if (res.ok) {
        config.firewall.blacklist_ips.push(newIp);
        newIp = "";
      }
    } catch (e) {
      console.error(e);
    }
  }

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="space-y-8" in:fade>
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-red-500 mb-2 flex items-center gap-3">
        <ShieldAlert size={32} />
        Security Center
      </h1>
      <p class="text-primary-light/60">Enterprise-grade protection engine. Hardened by eBPF & AES-GCM.</p>
    </div>
    <div class="px-4 py-2 bg-red-500/10 border border-red-500/30 text-red-500 rounded-full text-xs font-bold animate-pulse">
      SYSTEM HARDENED
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Storage Security -->
    <div class="bg-primary-dark/40 border border-red-500/10 rounded-2xl p-6 relative overflow-hidden group">
      <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
        <Lock size={64} />
      </div>
      <h3 class="text-sm font-bold text-primary-light/40 uppercase mb-4 flex items-center gap-2">
        <Lock size={14} />
        Data Encryption
      </h3>
      <div class="space-y-4">
        {#if config.security.encryption_enabled}
          <div class="flex items-center gap-3">
            <div class="p-2 bg-green-500/20 text-green-500 rounded-lg">
              <ShieldCheck size={20} />
            </div>
            <div>
              <p class="text-primary-light font-bold">AES-256-GCM Active</p>
              <p class="text-[10px] text-primary-light/40 italic">Config at rest is encrypted</p>
            </div>
          </div>
        {:else}
           <p class="text-red-400 font-bold">Encryption Disabled</p>
        {/if}
        <div class="text-[10px] bg-black/40 p-2 rounded font-mono text-primary-light/40">
          Hardware Binding: {config.security.hardware_bound ? "ENABLED (Locked to Machine-ID)" : "DISABLED"}
        </div>
      </div>
    </div>

    <!-- DNS Security -->
    <div class="bg-primary-dark/40 border border-blue-500/10 rounded-2xl p-6 relative overflow-hidden group">
      <div class="absolute top-0 right-0 p-4 opacity-10">
        <Globe size={64} />
      </div>
      <h3 class="text-sm font-bold text-primary-light/40 uppercase mb-4 flex items-center gap-2">
        <Globe size={14} />
        DNS Privacy
      </h3>
      <div class="space-y-2">
        <div class="flex justify-between items-center text-sm">
          <span class="text-primary-light/60">DoH Upstream</span>
          <span class={config.dns.doh_enabled ? "text-green-500 font-bold" : "text-primary-light/20"}>
            {config.dns.doh_enabled ? "ACTIVE" : "OFF"}
          </span>
        </div>
        <div class="flex justify-between items-center text-sm">
          <span class="text-primary-light/60">Anti-Leak Intercept</span>
          <span class={config.dns.transparent_intercept ? "text-green-500 font-bold" : "text-primary-light/20"}>
            {config.dns.transparent_intercept ? "ACTIVE" : "OFF"}
          </span>
        </div>
        <div class="flex justify-between items-center text-sm">
          <span class="text-primary-light/60">Content Filtering</span>
          <span class={config.dns.adblock_enabled ? "text-blue-400 font-bold" : "text-primary-light/20"}>
            {config.dns.adblock_enabled ? "ACTIVE" : "OFF"}
          </span>
        </div>
      </div>
    </div>

    <!-- eBPF Stats -->
    <div class="bg-primary-dark/40 border border-orange-500/10 rounded-2xl p-6 relative overflow-hidden group">
      <div class="absolute top-0 right-0 p-4 opacity-10">
        <Activity size={64} />
      </div>
      <h3 class="text-sm font-bold text-primary-light/40 uppercase mb-4 flex items-center gap-2">
        <Activity size={14} />
        Firewall Metrics
      </h3>
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-xs text-primary-light/40">Engine</span>
          <span class="text-xs font-mono text-orange-400">{stats.engine}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-xs text-primary-light/40">Processed</span>
          <span class="text-lg font-bold text-primary-light">{stats.total_packets.toLocaleString()}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-xs text-primary-light/40">Intercepted</span>
          <span class="text-lg font-bold text-red-500">{stats.dropped_packets.toLocaleString()}</span>
        </div>
      </div>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- Blacklist Manager -->
    <div class="bg-primary-dark/40 border border-red-500/10 rounded-2xl p-6">
      <h3 class="text-xl font-bold text-primary-light mb-6 flex items-center gap-3">
        <Ghost class="text-red-500" />
        IPS IP Blacklist
      </h3>
      
      <div class="flex gap-2 mb-6">
        <input 
          bind:value={newIp}
          type="text" 
          placeholder="Enter IP (e.g. 1.2.3.4)" 
          class="flex-1 bg-black/40 border border-primary-light/10 rounded-xl px-4 py-2 text-primary-light outline-none focus:border-red-500 transition-all"
        />
        <button on:click={addToBlacklist} class="p-2 bg-red-500 text-white rounded-xl hover:bg-red-600 transition-all">
          <Plus size={24} />
        </button>
      </div>

      <div class="space-y-2 max-h-[300px] overflow-y-auto pr-2 custom-scrollbar">
        {#each config.firewall.blacklist_ips as ip}
          <div class="flex items-center justify-between p-3 bg-red-500/5 border border-red-500/10 rounded-xl group" transition:slide>
            <span class="text-sm font-mono text-primary-light/60">{ip}</span>
            <button class="p-1 text-primary-light/20 hover:text-red-500 transition-colors">
              <Trash size={16} />
            </button>
          </div>
        {:else}
          <div class="text-center py-12 text-primary-light/20">
            <ShieldCheck size={48} class="mx-auto mb-2 opacity-10" />
            <p>No IPs blacklisted</p>
          </div>
        {/each}
      </div>
    </div>

    <!-- Security Logs / Info -->
    <div class="bg-primary-dark/40 border border-primary-light/5 rounded-2xl p-6 flex flex-col justify-center items-center text-center">
      <Info size={48} class="text-blue-400/20 mb-4" />
      <h3 class="text-lg font-bold text-primary-light mb-2">Enterprise Policy</h3>
      <p class="text-sm text-primary-light/40 max-w-sm">
        Managed by Rouman Core-Security Module. All packets are inspected at the Network Interface Card (NIC) level for ultra-low latency blocking.
      </p>
      <div class="mt-6 p-4 bg-orange-500/5 border border-orange-500/20 rounded-xl">
        <p class="text-[10px] text-orange-400 font-bold uppercase tracking-widest mb-1">Defense Level</p>
        <div class="flex gap-1">
          {#each Array(5) as _, i}
            <div class="h-1.5 w-6 rounded-full {i < 4 ? 'bg-orange-500 shadow-[0_0_5px_rgba(249,115,22,0.5)]' : 'bg-primary-light/10'}"></div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(239, 68, 68, 0.1);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(239, 68, 68, 0.3);
  }
  :global(.text-red-500) {
    text-shadow: 0 0 10px rgba(239, 68, 68, 0.4);
  }
</style>
