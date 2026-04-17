<script>
  import { onMount } from 'svelte';
  import { Users, Wifi, Clock, Fingerprint, RefreshCw, Cpu, Settings2, ShieldCheck } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let neighbors = [];
  let config = { rdp: { enabled: false, interfaces: [] } };
  let loading = true;

  async function fetchData() {
    try {
      const neighborRes = await fetch('/api/v1/network/neighbors');
      const neighborData = await neighborRes.json();
      neighbors = neighborData.neighbors;

      const configRes = await fetch('/api/v1/config/active');
      const configData = await configRes.json();
      config = configData.active;
    } catch (e) {
      console.error("Failed to fetch discovery data", e);
    } finally {
      loading = false;
    }
  }

  function formatRelativeTime(seconds) {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - seconds;
    if (diff < 5) return "Just now";
    return `${diff}s ago`;
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
      <h1 class="text-3xl font-bold text-indigo-400 mb-2 flex items-center gap-3">
        <Users size={32} />
        L2 Neighbors (RoMON)
      </h1>
      <p class="text-primary-light/60">Discovered Rouman peers on your local segment via RDP.</p>
    </div>
    <div class="flex gap-3">
       {#if config.rdp.enabled}
          <span class="flex items-center gap-2 px-3 py-1 bg-green-500/10 border border-green-500/20 text-green-500 rounded-lg text-xs font-mono">
            <ShieldCheck size={14} /> SIGNED BROADCAST ACTIVE
          </span>
       {/if}
       <button on:click={fetchData} class="p-2 bg-primary-dark/50 hover:bg-primary-dark rounded-lg transition-all text-primary-light/60 hover:text-white">
          <RefreshCw size={20} />
       </button>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
    <div class="md:col-span-3 bg-primary-dark/40 border border-indigo-500/10 rounded-2xl overflow-hidden backdrop-blur-sm">
      <table class="w-full text-left">
        <thead>
          <tr class="bg-indigo-500/5 text-primary-light/40 text-xs uppercase tracking-widest border-b border-indigo-500/10">
            <th class="px-6 py-4 font-semibold">Router Identity</th>
            <th class="px-6 py-4 font-semibold">MAC Address</th>
            <th class="px-6 py-4 font-semibold">OS Version</th>
            <th class="px-6 py-4 font-semibold text-right">Last Seen</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-indigo-500/5">
          {#each neighbors as neighbor}
            <tr class="hover:bg-indigo-500/5 transition-colors group">
              <td class="px-6 py-4">
                <div class="flex items-center gap-3">
                  <div class="p-2 bg-indigo-500/10 text-indigo-400 rounded-xl group-hover:scale-110 transition-transform">
                    <Cpu size={20} />
                  </div>
                  <div>
                    <span class="text-primary-light font-bold block">{neighbor.hostname}</span>
                    <span class="text-[10px] text-green-500 font-mono tracking-tighter">SECURE HANDSHAKE OK</span>
                  </div>
                </div>
              </td>
              <td class="px-6 py-4">
                <div class="flex items-center gap-2 text-primary-light/60 font-mono text-sm">
                  <Fingerprint size={14} class="opacity-30" />
                  {neighbor.mac}
                </div>
              </td>
              <td class="px-6 py-4 text-sm text-primary-light/40">
                v{neighbor.version}
              </td>
              <td class="px-6 py-4 text-right">
                <span class="inline-flex items-center gap-2 px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 text-[10px] font-bold">
                  <Clock size={10} />
                  {formatRelativeTime(neighbor.last_seen)}
                </span>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="4" class="px-6 py-20 text-center">
                <div class="flex flex-col items-center opacity-20">
                  <Wifi size={64} class="animate-pulse mb-4" />
                  <p class="text-xl font-bold">Scanning for Neighbors...</p>
                  <p class="text-xs uppercase tracking-widest mt-2">{config.rdp.enabled ? "Broadcasting identity on " + config.rdp.interfaces.join(", ") : "Discovery is Disabled"}</p>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="space-y-6">
      <div class="bg-primary-dark/40 border border-indigo-500/10 rounded-2xl p-6 relative overflow-hidden">
        <div class="absolute -top-10 -right-10 w-32 h-32 bg-indigo-500/10 blur-3xl rounded-full"></div>
        <h3 class="text-sm font-bold text-indigo-400 uppercase mb-4 flex items-center gap-2">
          <Settings2 size={16} />
          Discovery Logic
        </h3>
        <div class="space-y-4 text-xs">
          <div class="flex justify-between items-center bg-black/30 p-2 rounded-lg">
            <span class="text-primary-light/40">Status</span>
            <span class={config.rdp.enabled ? "text-green-500 font-bold" : "text-red-500 font-bold"}>
              {config.rdp.enabled ? "ENABLED" : "DISABLED"}
            </span>
          </div>
          <div class="space-y-2">
            <span class="text-primary-light/40 block">Listening On:</span>
            <div class="flex flex-wrap gap-1">
              {#each config.rdp.interfaces as iface}
                <span class="px-2 py-0.5 bg-indigo-500/20 text-indigo-400 rounded-md border border-indigo-500/30">
                  {iface}
                </span>
              {:else}
                <span class="text-primary-light/10 italic">No interfaces selected</span>
              {/each}
            </div>
          </div>
          <div class="p-3 bg-indigo-500/5 border border-indigo-500/10 rounded-xl text-[10px] leading-relaxed text-indigo-400/60 italic">
            "RoMON compatible Layer 2 discovery. Uses HMAC-SHA256 signature to prevent peer spoofing."
          </div>
        </div>
      </div>
    
      <div class="bg-indigo-500/10 border border-indigo-500/20 rounded-2xl p-6 text-center group">
         <ShieldCheck size={40} class="mx-auto text-indigo-400 mb-3 group-hover:scale-110 transition-transform" />
         <h4 class="text-sm font-bold text-primary-light mb-1">Signed Discovery</h4>
         <p class="text-[10px] text-primary-light/40 leading-relaxed">
            Discovery packets are signed with your Router ID. Neighbors must share the same key to be visible.
         </p>
      </div>
    </div>
  </div>
</div>

<style>
  :global(.text-indigo-400) {
    text-shadow: 0 0 15px rgba(129, 140, 248, 0.3);
  }
</style>
