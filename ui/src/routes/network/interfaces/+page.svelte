<script>
  import { onMount } from 'svelte';
  import { Network, Layers, Share2, Plus, Trash2, Activity, Shield, Zap, Info, Settings2, Globe, Save, RefreshCw } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let config = { 
    network: { 
        wan_interface: 'eth0',
        interfaces: { vlans: [], bridges: [], assignments: [] } 
    } 
  };
  let loading = true;
  let saving = false;
  let message = "";

  async function fetchData() {
    try {
      const res = await fetch('/api/v1/config/active');
      const data = await res.json();
      config = data.active;
    } catch (e) {
      console.error("Failed to fetch interfaces config", e);
    } finally {
      loading = false;
    }
  }

  async function saveConfig() {
    saving = true;
    message = "";
    try {
      const res = await fetch('/api/v1/config/candidate', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config)
      });
      
      if (res.ok) {
        await fetch('/api/v1/config/commit', { method: 'POST' });
        message = "Interfaces updated and applied!";
        fetchData();
      }
    } catch (e) {
      message = "Error: " + e.message;
    } finally {
      saving = false;
    }
  }

  function addVlan() {
    config.network.interfaces.vlans = [...config.network.interfaces.vlans, {
      name: `vlan${config.network.interfaces.vlans.length + 10}`,
      parent: 'eth1',
      vlan_id: 10 + config.network.interfaces.vlans.length,
      enabled: true
    }];
  }

  function addBridge() {
    config.network.interfaces.bridges = [...config.network.interfaces.bridges, {
      name: `br${config.network.interfaces.bridges.length}`,
      members: ['eth2', 'eth3'],
      enabled: true
    }];
  }

  function addAssignment() {
    config.network.interfaces.assignments = [...config.network.interfaces.assignments, {
      interface: '',
      address: '192.168.100.1/24',
      gateway: null,
      enabled: true
    }];
  }

  onMount(fetchData);
</script>

<div class="space-y-8" in:fade>
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-cyan-400 mb-2 flex items-center gap-3">
        <Layers size={32} />
        Interfaces & VLANs
      </h1>
      <p class="text-primary-light/60">Manage physical ports, virtual LANs, and software bridges.</p>
    </div>
    
    <button on:click={saveConfig} disabled={saving}
      class="px-8 py-3 bg-gradient-to-r from-cyan-600 to-cyan-500 text-white font-bold rounded-xl hover:scale-105 active:scale-95 transition-all flex items-center gap-2 shadow-lg shadow-cyan-500/20">
      {#if saving}
        <RefreshCw size={18} class="animate-spin" />
        Applying...
      {:else}
        <Save size={18} />
        Apply Config
      {/if}
    </button>
  </div>

  {#if message}
    <div class="p-4 bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 rounded-xl text-sm font-bold flex items-center gap-2" transition:slide>
      <Zap size={18} /> {message}
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
    <!-- IP Assignments Section -->
    <div class="space-y-6">
      <div class="flex justify-between items-center">
        <h2 class="text-xl font-bold text-primary-light flex items-center gap-2">
          <Globe size={20} class="text-cyan-400" />
          IP Assignments
        </h2>
        <button on:click={addAssignment} class="p-1 px-3 bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 rounded-lg text-xs font-bold hover:bg-cyan-500/20">
          + Add IP
        </button>
      </div>

      <div class="space-y-4">
        {#each config.network.interfaces.assignments as assign, i}
          <div class="bg-primary-dark/40 border border-cyan-500/10 rounded-2xl p-4 flex items-center gap-4" transition:slide>
            <div class="flex-1 space-y-4">
              <div class="flex gap-4">
                <div class="flex-1">
                   <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-tighter">Interface</label>
                   <input bind:value={assign.interface} placeholder="eth1, br0, etc." class="w-full bg-black/40 border border-cyan-500/10 rounded-lg px-3 py-1.5 text-xs text-primary-light outline-none focus:border-cyan-500/30" />
                </div>
                <div class="flex-1">
                  <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-tighter">Address (CIDR)</label>
                  <input bind:value={assign.address} placeholder="192.168.1.1/24" class="w-full bg-black/40 border border-cyan-500/10 rounded-lg px-3 py-1.5 text-xs text-primary-light outline-none focus:border-cyan-500/30" />
                </div>
              </div>
            </div>
            <button on:click={() => config.network.interfaces.assignments = config.network.interfaces.assignments.filter((_, idx) => idx !== i)} class="text-red-500/30 hover:text-red-500 p-2">
              <Trash2 size={16} />
            </button>
          </div>
        {/each}
      </div>
    </div>

    <!-- Virtual Interfaces (VLAN & Bridge) -->
    <div class="space-y-6">
      <div class="bg-primary-dark/40 border border-cyan-500/10 rounded-2xl p-6">
        <div class="flex justify-between items-center mb-6">
            <h2 class="text-lg font-bold text-primary-light flex items-center gap-2">
              <Share2 size={18} class="text-cyan-400" />
              VLANs & Bridges
            </h2>
            <div class="flex gap-2 text-[10px]">
              <button on:click={addVlan} class="px-3 py-1 bg-cyan-500/10 border border-cyan-500/20 rounded-md text-cyan-400 font-bold hover:bg-cyan-500/20">VLAN+</button>
              <button on:click={addBridge} class="px-3 py-1 bg-cyan-500/10 border border-cyan-500/20 rounded-md text-cyan-400 font-bold hover:bg-cyan-500/20">BRIDGE+</button>
            </div>
        </div>

        <div class="space-y-6">
          <!-- VLAN List -->
          {#if config.network.interfaces.vlans.length > 0}
            <div class="space-y-3">
              <h3 class="text-[10px] font-bold text-primary-light/40 uppercase">VLAN 802.1Q</h3>
              {#each config.network.interfaces.vlans as vlan, i}
                <div class="flex items-center gap-4 bg-black/20 p-3 rounded-xl border border-cyan-500/5 hover:border-cyan-500/20 transition-all">
                  <div class="p-2 bg-cyan-500/10 rounded-lg text-cyan-400">
                    <Activity size={16} />
                  </div>
                  <div class="flex-1 flex gap-4">
                    <input bind:value={vlan.name} class="bg-transparent border-none text-xs font-bold text-primary-light w-20 focus:ring-0" />
                    <div class="text-[10px] text-primary-light/40 flex flex-col">
                      <span>ID: <input type="number" bind:value={vlan.vlan_id} class="bg-transparent border-none p-0 w-8 text-cyan-400 focus:ring-0 inline" /></span>
                      <span>Link: {vlan.parent}</span>
                    </div>
                  </div>
                  <button on:click={() => config.network.interfaces.vlans = config.network.interfaces.vlans.filter((_, idx) => idx !== i)} class="text-red-500/30 hover:text-red-500">
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Bridge List -->
          {#if config.network.interfaces.bridges.length > 0}
            <div class="space-y-3 pt-4 border-t border-cyan-500/5">
              <h3 class="text-[10px] font-bold text-primary-light/40 uppercase">Bridges</h3>
              {#each config.network.interfaces.bridges as br, i}
                <div class="bg-black/20 p-3 rounded-xl border border-cyan-500/5">
                  <div class="flex justify-between items-center mb-2">
                    <input bind:value={br.name} class="bg-transparent border-none text-xs font-bold text-primary-light focus:ring-0" />
                    <button on:click={() => config.network.interfaces.bridges = config.network.interfaces.bridges.filter((_, idx) => idx !== i)} class="text-red-500/30 hover:text-red-500">
                      <Trash2 size={14} />
                    </button>
                  </div>
                  <div class="flex flex-wrap gap-2">
                    {#each br.members as port}
                      <span class="px-2 py-0.5 bg-cyan-500/10 text-cyan-400 rounded text-[10px] border border-cyan-500/10">{port}</span>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <!-- Quick Info / Docs -->
      <div class="bg-cyan-500/5 border border-cyan-500/10 rounded-2xl p-6 flex gap-4">
         <div class="shrink-0 text-cyan-400">
            <Info size={24} />
         </div>
         <div class="space-y-1">
            <h4 class="text-sm font-bold text-primary-light">Network Segmentation</h4>
            <p class="text-xs text-primary-light/60 leading-relaxed">
              VLANs allow you to separate guest traffic from management traffic. Assign multiple IP addresses to different VLANs to create unique subnets.
            </p>
         </div>
      </div>
    </div>
  </div>
</div>

<style>
  :global(.text-cyan-400) {
    text-shadow: 0 0 15px rgba(34, 211, 238, 0.3);
  }
  input[type="number"]::-webkit-inner-spin-button {
    display: none;
  }
</style>
