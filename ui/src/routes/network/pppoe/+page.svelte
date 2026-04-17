<script>
  import { onMount } from 'svelte';
  import { Shield, Globe, Lock, User, Wifi, RefreshCw, AlertTriangle, CheckCircle2, Settings2 } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let config = { 
    network: { 
      pppoe: { enabled: false, username: '', password: '', interface: 'eth0', mtu: 1492 } 
    } 
  };
  let status = { connected: false, ip_address: null };
  let loading = true;
  let saving = false;
  let message = "";

  async function fetchData() {
    try {
      const res = await fetch('/api/v1/config/active');
      const data = await res.json();
      config = data.active;
      
      const statusRes = await fetch('/api/v1/network/pppoe/status');
      const statusData = await statusRes.json();
      status = statusData.status;
    } catch (e) {
      console.error("Failed to fetch PPPoE config", e);
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
        message = "Configuration applied successfully!";
        fetchData();
      }
    } catch (e) {
      message = "Error: " + e.message;
    } finally {
      saving = false;
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
      <h1 class="text-3xl font-bold text-amber-400 mb-2 flex items-center gap-3">
        <Globe size={32} />
        PPPoE WAN Setup
      </h1>
      <p class="text-primary-light/60">Configure your primary internet connection via ISP PPPoE.</p>
    </div>
    
    <div class="flex items-center gap-4">
       {#if status.connected}
         <div class="flex items-center gap-2 px-4 py-1.5 bg-green-500/10 border border-green-500/20 text-green-500 rounded-full text-xs font-bold animate-pulse">
            <Wifi size={14} /> CONNECTED: {status.ip_address}
         </div>
       {:else}
         <div class="flex items-center gap-2 px-4 py-1.5 bg-red-500/10 border border-red-500/20 text-red-500 rounded-full text-xs font-bold">
            <AlertTriangle size={14} /> DISCONNECTED
         </div>
       {/if}
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
    <div class="md:col-span-2 space-y-6">
      <div class="bg-primary-dark/40 border border-amber-500/10 rounded-2xl p-8 backdrop-blur-sm">
        <div class="flex items-center justify-between mb-8">
          <h2 class="text-xl font-bold text-primary-light flex items-center gap-2">
            <Settings2 size={20} class="text-amber-400" />
            Credentials
          </h2>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" bind:checked={config.network.pppoe.enabled} class="sr-only peer">
            <div class="w-11 h-6 bg-primary-dark/60 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-gray-400 after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-amber-500"></div>
            <span class="ml-3 text-sm font-bold text-primary-light">{config.network.pppoe.enabled ? 'Enabled' : 'Disabled'}</span>
          </label>
        </div>

        <div class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-2">
              <label class="text-xs font-bold text-primary-light/40 uppercase tracking-widest flex items-center gap-2">
                <User size={14} /> ISP Username
              </label>
              <input type="text" bind:value={config.network.pppoe.username} placeholder="user@isp"
                class="w-full bg-black/40 border border-amber-500/10 rounded-xl px-4 py-3 text-primary-light focus:border-amber-500/50 outline-none transition-all" />
            </div>

            <div class="space-y-2">
              <label class="text-xs font-bold text-primary-light/40 uppercase tracking-widest flex items-center gap-2">
                <Lock size={14} /> ISP Password
              </label>
              <input type="password" bind:value={config.network.pppoe.password} placeholder="••••••••"
                class="w-full bg-black/40 border border-amber-500/10 rounded-xl px-4 py-3 text-primary-light focus:border-amber-500/50 outline-none transition-all" />
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-amber-500/5">
            <div class="space-y-2">
              <label class="text-xs font-bold text-primary-light/40 uppercase tracking-widest">Physical Interface</label>
              <select bind:value={config.network.pppoe.interface} 
                class="w-full bg-black/40 border border-amber-500/10 rounded-xl px-4 py-3 text-primary-light focus:border-amber-500/50 outline-none">
                <option value="ens33">ens33 (Physical WAN)</option>
                <option value="eth1">eth1</option>
              </select>
            </div>

            <div class="space-y-2">
              <label class="text-xs font-bold text-primary-light/40 uppercase tracking-widest">MTU Size</label>
              <input type="number" bind:value={config.network.pppoe.mtu} 
                class="w-full bg-black/40 border border-amber-500/10 rounded-xl px-4 py-3 text-primary-light focus:border-amber-500/50 outline-none" />
            </div>
          </div>

          <div class="flex justify-end gap-3 pt-6">
            <button on:click={fetchData} class="px-6 py-3 bg-primary-dark/60 text-primary-light/60 rounded-xl hover:bg-primary-dark transition-all">
              Cancel
            </button>
            <button on:click={saveConfig} disabled={saving}
              class="px-8 py-3 bg-gradient-to-r from-amber-600 to-amber-500 text-white font-bold rounded-xl hover:scale-105 active:scale-95 transition-all flex items-center gap-2 shadow-lg shadow-amber-500/20">
              {#if saving}
                <RefreshCw size={18} class="animate-spin" />
                Applying...
              {:else}
                Save & Connect
              {/if}
            </button>
          </div>
          
          {#if message}
            <div class="text-center p-3 bg-amber-500/10 text-amber-400 rounded-lg text-sm font-bold" transition:slide>
              {message}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <div class="space-y-6">
      <div class="bg-amber-500/10 border border-amber-500/20 rounded-2xl p-6 text-center group">
         <Shield size={40} class="mx-auto text-amber-400 mb-3 group-hover:scale-110 transition-transform" />
         <h4 class="text-sm font-bold text-primary-light mb-1">Standard Encryption</h4>
         <p class="text-[10px] text-primary-light/40 leading-relaxed">
            PAP/CHAP authentication is automatically handled. Passwords are encrypted on disk.
         </p>
      </div>

      <div class="bg-primary-dark/40 border border-amber-500/10 rounded-2xl p-6">
        <h3 class="text-sm font-bold text-amber-400 uppercase mb-4 flex items-center gap-2">
          <Wifi size={16} /> Link Status
        </h3>
        <div class="space-y-4">
           <div class="flex justify-between items-center text-xs">
              <span class="text-primary-light/40">Interface</span>
              <span class="text-primary-light font-mono">ppp0</span>
           </div>
           <div class="flex justify-between items-center text-xs">
              <span class="text-primary-light/40">Status</span>
              <span class={status.connected ? "text-green-500 font-bold" : "text-red-500 font-bold"}>
                {status.connected ? 'ACTIVE' : 'IDLE'}
              </span>
           </div>
           <div class="flex justify-between items-center text-xs">
              <span class="text-primary-light/40">Local IP</span>
              <span class="text-primary-light font-mono">{status.ip_address || '--. --. --. --'}</span>
           </div>
           <div class="flex justify-between items-center text-xs pt-4 border-t border-amber-500/5">
              <span class="text-primary-light/40">MSS Clamping</span>
              <span class="text-green-500/60 font-mono">AUTO</span>
           </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  :global(.text-amber-400) {
    text-shadow: 0 0 15px rgba(251, 191, 36, 0.3);
  }
</style>
