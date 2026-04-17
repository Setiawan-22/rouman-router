<script>
  import { onMount } from 'svelte';
  import { Shield, Plus, Trash, RefreshCw, Key, ArrowRight, Activity, Globe } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let config = { wireguard: { interfaces: [] } };
  let status = { status: [] };
  let loading = true;
  let newIface = { name: 'wg0', address: '10.0.0.1/24', listen_port: 51820, private_key: '', enabled: true };
  let generatedKeys = null;

  async function fetchData() {
    try {
      const configRes = await fetch('/api/v1/network/wireguard/config');
      config = await configRes.json();
      
      const statusRes = await fetch('/api/v1/network/wireguard/status');
      status = await statusRes.json();
    } catch (e) {
      console.error("Failed to fetch VPN data", e);
    } finally {
      loading = false;
    }
  }

  async function generateKeys() {
    const res = await fetch('/api/v1/network/wireguard/generate-keys', { method: 'POST' });
    generatedKeys = await res.json();
    newIface.private_key = generatedKeys.private_key;
  }

  async function saveConfig() {
    // For now we just append to interfaces and save
    // In a real app we'd have a better mutator
    // We'll call the global commit after updating config
    // (Need to implement the global config update API)
    alert("Functionality to update and commit config is being integrated.");
  }

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  });

  function getPeerStatus(pubKey) {
    for (const iface of status.status) {
      for (const peer of iface.peers) {
        if (peer.public_key === pubKey) {
          return peer;
        }
      }
    }
    return null;
  }

  function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="space-y-8" in:fade>
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-primary-glow mb-2">VPN (WireGuard)</h1>
      <p class="text-primary-light/60">Manage high-performance native VPN interfaces and peers.</p>
    </div>
    <div class="flex gap-4">
      <button 
        on:click={fetchData} 
        class="p-2 bg-primary-dark/20 border border-primary-glow/30 rounded-lg text-primary-glow hover:bg-primary-glow/10 transition-all"
        title="Refresh Status"
      >
        <RefreshCw size={20} class={loading ? 'animate-spin' : ''} />
      </button>
      <button class="flex items-center gap-2 px-4 py-2 bg-primary-glow text-primary-dark font-bold rounded-lg hover:shadow-[0_0_20px_rgba(0,255,157,0.5)] transition-all">
        <Plus size={18} />
        New Interface
      </button>
    </div>
  </div>

  {#if config.wireguard.interfaces.length === 0}
    <div class="bg-primary-dark/40 border border-dashed border-primary-glow/20 rounded-2xl p-12 text-center" transition:slide>
      <Shield size={48} class="mx-auto mb-4 text-primary-glow/40" />
      <h3 class="text-xl font-bold text-primary-light mb-2">No VPN Interfaces</h3>
      <p class="text-primary-light/60 mb-6">Start by creating your first WireGuard interface.</p>
      <button on:click={generateKeys} class="inline-flex items-center gap-2 px-6 py-3 border border-primary-glow text-primary-glow rounded-xl hover:bg-primary-glow/10 transition-all">
        <Key size={18} />
        Initialize Interface
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      {#each config.wireguard.interfaces as iface}
        <div class="bg-primary-dark/40 border border-primary-glow/20 rounded-2xl p-6 relative overflow-hidden group">
          <div class="absolute top-0 right-0 w-32 h-32 bg-primary-glow/5 blur-3xl -mr-16 -mt-16 group-hover:bg-primary-glow/10 transition-all"></div>
          
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-primary-glow/20 rounded-lg">
                <Shield size={24} class="text-primary-glow" />
              </div>
              <div>
                <h2 class="text-xl font-bold text-primary-light">{iface.name}</h2>
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full {iface.enabled ? 'bg-green-500 box-shadow-[0_0_8px_#22c55e]' : 'bg-red-500'}"></span>
                  <span class="text-xs text-primary-light/40 uppercase tracking-widest font-bold">{iface.enabled ? 'Active' : 'Disabled'}</span>
                </div>
              </div>
            </div>
            <button class="p-2 text-primary-light/40 hover:text-red-400 transition-colors">
              <Trash size={18} />
            </button>
          </div>

          <div class="grid grid-cols-2 gap-4 mb-8">
            <div class="p-4 bg-primary-dark/40 rounded-xl border border-primary-glow/5">
              <p class="text-xs text-primary-light/40 mb-1 uppercase font-bold">Address</p>
              <p class="text-lg font-mono text-primary-light">{iface.address}</p>
            </div>
            <div class="p-4 bg-primary-dark/40 rounded-xl border border-primary-glow/5">
              <p class="text-xs text-primary-light/40 mb-1 uppercase font-bold">Listen Port</p>
              <p class="text-lg font-mono text-primary-light">{iface.listen_port}</p>
            </div>
          </div>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-bold text-primary-light/60 uppercase tracking-widest">Peers ({iface.peers.length})</h3>
              <button class="text-xs text-primary-glow hover:underline">+ Add Peer</button>
            </div>
            
            <div class="space-y-3">
              {#each iface.peers as peer}
                {@const peerStats = getPeerStatus(peer.public_key)}
                <div class="p-4 bg-primary-dark/60 rounded-xl border border-primary-glow/10 hover:border-primary-glow/30 transition-all">
                  <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center gap-3">
                      <Globe size={16} class="text-primary-light/40" />
                      <span class="font-mono text-sm text-primary-light truncate w-32" title={peer.public_key}>
                        {peer.public_key.substring(0, 12)}...
                      </span>
                    </div>
                    {#if peerStats}
                      <div class="flex items-center gap-2">
                         <span class="text-[10px] text-primary-light/40 uppercase">Handshake:</span>
                         <span class="text-[10px] text-primary-glow font-bold">
                           {peerStats.last_handshake ? (Math.floor(Date.now()/1000) - peerStats.last_handshake) + 's ago' : 'Never'}
                         </span>
                      </div>
                    {/if}
                  </div>
                  
                  <div class="flex items-center justify-between text-xs">
                    <div class="flex gap-4">
                      <div class="flex items-center gap-1">
                        <ArrowRight size={12} class="rotate-90 text-primary-glow" />
                        <span class="text-primary-light/60">{peerStats ? formatBytes(peerStats.tx_bytes) : '0 B'}</span>
                      </div>
                      <div class="flex items-center gap-1">
                        <ArrowRight size={12} class="-rotate-90 text-blue-400" />
                        <span class="text-primary-light/60">{peerStats ? formatBytes(peerStats.rx_bytes) : '0 B'}</span>
                      </div>
                    </div>
                    <div class="flex flex-wrap gap-1 justify-end">
                      {#each peer.allowed_ips as ip}
                        <span class="px-2 py-0.5 bg-primary-glow/10 text-[10px] text-primary-glow rounded border border-primary-glow/20">{ip}</span>
                      {/each}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  :global(.text-primary-glow) {
    color: #00ff9d;
    text-shadow: 0 0 10px rgba(0, 255, 157, 0.4);
  }
</style>
