<script>
  import { onMount } from 'svelte';
  import { Cloud, Plus, Trash, ExternalLink, ShieldCheck, AlertCircle, Info, ArrowUpRight } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let config = { cloudflare: { tunnels: [] } };
  let binaryStatus = { installed: false, version: '' };
  let loading = true;
  let showAddModal = false;
  
  let newTunnel = { id: crypto.randomUUID(), name: '', token: '', enabled: true, ingress: [] };

  async function fetchData() {
    try {
      const configRes = await fetch('/api/v1/network/cloudflare/config');
      config = await configRes.json();
      
      const checkRes = await fetch('/api/v1/network/cloudflare/check');
      binaryStatus = await checkRes.json();
    } catch (e) {
      console.error("Failed to fetch Cloudflare data", e);
    } finally {
      loading = false;
    }
  }

  async function addTunnel() {
    config.cloudflare.tunnels.push({ ...newTunnel });
    // In a real implementation we would send this to the global commit API
    alert("Konfigurasi baru ditambahkan secara lokal. Silakan gunakan tombol 'Commit Changes' utama untuk menerapkan.");
    showAddModal = false;
    newTunnel = { id: crypto.randomUUID(), name: '', token: '', enabled: true, ingress: [] };
  }

  onMount(() => {
    fetchData();
  });
</script>

<div class="space-y-8" in:fade>
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-blue-400 mb-2 flex items-center gap-3">
        <Cloud size={32} />
        Cloudflare Zero Trust
      </h1>
      <p class="text-primary-light/60">Expose local services securely and connect via Cloudflare Tunnels.</p>
    </div>
    <div class="flex gap-4">
       {#if !binaryStatus.installed}
         <div class="flex items-center gap-2 px-4 py-2 bg-red-500/10 border border-red-500/20 text-red-500 rounded-lg text-sm">
           <AlertCircle size={16} />
           cloudflared not found
         </div>
       {:else}
         <div class="flex items-center gap-2 px-4 py-2 bg-blue-500/10 border border-blue-500/20 text-blue-400 rounded-lg text-sm">
           <ShieldCheck size={16} />
           {binaryStatus.version.split(' ')[2] || 'Ready'}
         </div>
       {/if}
       <button on:click={() => showAddModal = true} class="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white font-bold rounded-lg hover:shadow-[0_0_20px_rgba(59,130,246,0.5)] transition-all">
         <Plus size={18} />
         Add Tunnel
       </button>
    </div>
  </div>

  {#if config.cloudflare.tunnels.length === 0}
    <div class="bg-primary-dark/40 border border-dashed border-blue-400/20 rounded-2xl p-12 text-center" transition:slide>
      <Cloud size={48} class="mx-auto mb-4 text-blue-400/20" />
      <h3 class="text-xl font-bold text-primary-light mb-2">No Tunnels Configured</h3>
      <p class="text-primary-light/60 mb-6">Create a tunnel in Cloudflare Zero Trust dashboard, copy the token, and add it here.</p>
      <button on:click={() => showAddModal = true} class="inline-flex items-center gap-2 px-6 py-3 border border-blue-400 text-blue-400 rounded-xl hover:bg-blue-400/10 transition-all">
        <ExternalLink size={18} />
        Setup My First Tunnel
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-6">
      {#each config.cloudflare.tunnels as tunnel}
        <div class="bg-primary-dark/40 border border-blue-400/20 rounded-2xl p-6 relative overflow-hidden group">
          <div class="absolute top-0 right-0 w-32 h-32 bg-blue-500/5 blur-3xl -mr-16 -mt-16 group-hover:bg-blue-500/10 transition-all"></div>
          
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-4">
              <div class="p-3 bg-blue-500/20 rounded-xl">
                <Cloud size={24} class="text-blue-400" />
              </div>
              <div>
                <h2 class="text-xl font-bold text-primary-light">{tunnel.name}</h2>
                <div class="flex items-center gap-2 text-xs text-primary-light/40 font-mono">
                  ID: {tunnel.id}
                </div>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <span class="px-3 py-1 bg-green-500/10 text-green-500 text-xs font-bold rounded-full border border-green-500/20">
                ACTIVE
              </span>
              <button class="p-2 text-primary-light/40 hover:text-red-400 transition-colors">
                <Trash size={18} />
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-4">
              <h3 class="text-sm font-bold text-primary-light/60 uppercase tracking-widest flex items-center gap-2">
                <Info size={14} />
                Ingress Rules
              </h3>
              <div class="space-y-2">
                {#each tunnel.ingress as rule}
                  <div class="flex items-center justify-between p-3 bg-primary-dark/60 rounded-xl border border-blue-400/10">
                    <div class="flex items-center gap-3">
                      <span class="text-sm text-blue-400 font-bold">{rule.hostname}</span>
                      <ArrowUpRight size={14} class="text-primary-light/20" />
                      <span class="text-sm text-primary-light/60 font-mono">{rule.service}</span>
                    </div>
                  </div>
                {:else}
                  <p class="text-xs text-primary-light/30 italic">No custom rules. Defaulting to Cloudflare dashboard settings.</p>
                {/each}
              </div>
            </div>

            <div class="bg-blue-500/5 rounded-xl p-4 border border-blue-400/10">
              <h3 class="text-xs font-bold text-blue-400/60 uppercase mb-3">Tunnel Token</h3>
              <div class="font-mono text-[10px] break-all text-primary-light/60 bg-black/20 p-2 rounded">
                ********************************************************************************************************************************
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showAddModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm" transition:fade>
  <div class="bg-primary-dark/90 border border-blue-400/30 w-full max-w-lg rounded-2xl p-8 shadow-2xl shadow-blue-500/20" transition:slide>
    <h2 class="text-2xl font-bold text-primary-light mb-6 flex items-center gap-3">
      <Plus class="text-blue-400" />
      New Tunnel
    </h2>
    
    <div class="space-y-4">
      <div>
        <label class="block text-xs font-bold text-primary-light/40 uppercase mb-1">Tunnel Name</label>
        <input 
          bind:value={newTunnel.name}
          type="text" 
          placeholder="e.g. HomeServer-Tunnel"
          class="w-full bg-primary-dark border border-primary-light/10 rounded-xl px-4 py-3 text-primary-light focus:border-blue-500 outline-none transition-all"
        />
      </div>
      <div>
        <label class="block text-xs font-bold text-primary-light/40 uppercase mb-1">Tunnel Token</label>
        <textarea 
          bind:value={newTunnel.token}
          placeholder="Paste your token from Cloudflare dashboard..."
          rows="4"
          class="w-full bg-primary-dark border border-primary-light/10 rounded-xl px-4 py-3 text-primary-light font-mono text-xs focus:border-blue-500 outline-none transition-all"
        ></textarea>
      </div>
      
      <div class="flex gap-4 pt-4">
        <button 
          on:click={() => showAddModal = false}
          class="flex-1 py-3 text-primary-light/60 hover:text-primary-light transition-colors"
        >
          Cancel
        </button>
        <button 
          on:click={addTunnel}
          disabled={!newTunnel.name || !newTunnel.token}
          class="flex-1 py-3 bg-blue-500 text-white font-bold rounded-xl hover:bg-blue-600 disabled:opacity-50 transition-all shadow-lg shadow-blue-500/20"
        >
          Add Tunnel
        </button>
      </div>
    </div>
  </div>
</div>
{/if}

<style>
  :global(.text-blue-400) {
    color: #3b82f6;
    text-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
  }
</style>
