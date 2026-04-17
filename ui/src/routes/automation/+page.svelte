<script>
  import { onMount } from 'svelte';
  import { Terminal, Activity, Clock, Play, Save, Plus, Trash2, Shield, Zap, AlertCircle, CheckCircle2 } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';

  let config = { 
    automation: { 
      netwatch: [], 
      scheduler: [], 
      scripts: [] 
    } 
  };
  let loading = true;
  let saving = false;
  let activeTab = 'netwatch';
  let message = "";

  async function fetchData() {
    try {
      const res = await fetch('/api/v1/config/active');
      const data = await res.json();
      config = data.active;
    } catch (e) {
      console.error("Failed to fetch automation config", e);
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

  function addNetwatch() {
    config.automation.netwatch = [...config.automation.netwatch, {
      name: "New Rule",
      target: "8.8.8.8",
      interval_secs: 10,
      script_up: 'log("Net is UP");',
      script_down: 'log("Net is DOWN");',
      enabled: true
    }];
  }

  function addScheduler() {
    config.automation.scheduler = [...config.automation.scheduler, {
      name: "Hourly Task",
      interval_secs: 3600,
      script: 'log("Running hourly task");',
      enabled: true
    }];
  }

  onMount(fetchData);
</script>

<div class="space-y-8" in:fade>
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-emerald-400 mb-2 flex items-center gap-3">
        <Zap size={32} />
        Automation & Scripter
      </h1>
      <p class="text-primary-light/60">Configure dynamic responses and scheduled tasks for your router.</p>
    </div>
    
    <button on:click={saveConfig} disabled={saving}
      class="px-8 py-3 bg-gradient-to-r from-emerald-600 to-emerald-500 text-white font-bold rounded-xl hover:scale-105 active:scale-95 transition-all flex items-center gap-2 shadow-lg shadow-emerald-500/20">
      {#if saving}
        <Activity size={18} class="animate-spin" />
        Saving...
      {:else}
        <Save size={18} />
        Commit Changes
      {/if}
    </button>
  </div>

  <div class="flex gap-4 p-1 bg-primary-dark/40 border border-emerald-500/10 rounded-2xl w-max">
    <button on:click={() => activeTab = 'netwatch'} 
      class="px-6 py-2 rounded-xl text-sm font-bold transition-all {activeTab === 'netwatch' ? 'bg-emerald-500 text-white' : 'text-primary-light/40 hover:text-emerald-400'}">
      Netwatch
    </button>
    <button on:click={() => activeTab = 'scheduler'} 
      class="px-6 py-2 rounded-xl text-sm font-bold transition-all {activeTab === 'scheduler' ? 'bg-emerald-500 text-white' : 'text-primary-light/40 hover:text-emerald-400'}">
      Scheduler
    </button>
  </div>

  {#if message}
    <div class="p-4 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-xl text-sm font-bold flex items-center gap-2" transition:slide>
      <CheckCircle2 size={18} /> {message}
    </div>
  {/if}

  <div class="grid grid-cols-1 gap-6">
    {#if activeTab === 'netwatch'}
      <div class="space-y-6">
        <div class="flex justify-between items-center">
          <h2 class="text-xl font-bold text-primary-light flex items-center gap-2">
            <Activity size={20} class="text-emerald-400" />
            Connectivity Monitoring
          </h2>
          <button on:click={addNetwatch} class="flex items-center gap-2 px-4 py-2 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-xl text-xs font-bold hover:bg-emerald-500/20 transition-all">
            <Plus size={14} /> Add Rule
          </button>
        </div>

        {#each config.automation.netwatch as rule, i}
          <div class="bg-primary-dark/40 border border-emerald-500/10 rounded-2xl p-6 space-y-4" transition:slide>
            <div class="flex items-center justify-between">
              <input bind:value={rule.name} class="bg-transparent border-none text-primary-light font-bold focus:ring-0 text-lg w-full" />
              <div class="flex items-center gap-4">
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" bind:checked={rule.enabled} class="sr-only peer">
                  <div class="w-11 h-6 bg-primary-dark/60 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-gray-400 after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
                </label>
                <button on:click={() => config.automation.netwatch = config.automation.netwatch.filter((_, idx) => idx !== i)} class="text-red-500/40 hover:text-red-500 transition-all">
                  <Trash2 size={18} />
                </button>
              </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-1">
                <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-widest">Ping Target</label>
                <input bind:value={rule.target} class="w-full bg-black/40 border border-emerald-500/10 rounded-xl px-4 py-2.5 text-primary-light text-sm outline-none focus:border-emerald-500/30" />
              </div>
              <div class="space-y-1">
                <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-widest">Interval (Seconds)</label>
                <input type="number" bind:value={rule.interval_secs} class="w-full bg-black/40 border border-emerald-500/10 rounded-xl px-4 py-2.5 text-primary-light text-sm outline-none focus:border-emerald-500/30" />
              </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-1">
                <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-widest flex items-center gap-2">
                  <Play size={10} class="text-green-500" /> On Up Script
                </label>
                <textarea bind:value={rule.script_up} rows="3" class="w-full bg-black/40 border border-emerald-500/10 rounded-xl px-4 py-3 text-emerald-400 font-mono text-xs outline-none focus:border-emerald-500/30 shadow-inner" spellcheck="false"></textarea>
              </div>
              <div class="space-y-1">
                <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-widest flex items-center gap-2">
                  <AlertCircle size={10} class="text-red-500" /> On Down Script
                </label>
                <textarea bind:value={rule.script_down} rows="3" class="w-full bg-black/40 border border-emerald-500/10 rounded-xl px-4 py-3 text-emerald-400 font-mono text-xs outline-none focus:border-emerald-500/30 shadow-inner" spellcheck="false"></textarea>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="space-y-6">
        <div class="flex justify-between items-center">
          <h2 class="text-xl font-bold text-primary-light flex items-center gap-2">
            <Clock size={20} class="text-emerald-400" />
            Scheduled Tasks
          </h2>
          <button on:click={addScheduler} class="flex items-center gap-2 px-4 py-2 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-xl text-xs font-bold hover:bg-emerald-500/20 transition-all">
            <Plus size={14} /> Add Task
          </button>
        </div>

        {#each config.automation.scheduler as rule, i}
          <div class="bg-primary-dark/40 border border-emerald-500/10 rounded-2xl p-6 space-y-4" transition:slide>
             <!-- Similar structure for scheduler -->
             <div class="flex items-center justify-between">
              <input bind:value={rule.name} class="bg-transparent border-none text-primary-light font-bold focus:ring-0 text-lg w-full" />
              <div class="flex items-center gap-4">
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" bind:checked={rule.enabled} class="sr-only peer">
                  <div class="w-11 h-6 bg-primary-dark/60 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-gray-400 after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
                </label>
                <button on:click={() => config.automation.scheduler = config.automation.scheduler.filter((_, idx) => idx !== i)} class="text-red-500/40 hover:text-red-500 transition-all">
                  <Trash2 size={18} />
                </button>
              </div>
            </div>

            <div class="space-y-1">
              <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-widest">Execution Interval (Seconds)</label>
              <input type="number" bind:value={rule.interval_secs} class="w-full bg-black/40 border border-emerald-500/10 rounded-xl px-4 py-2.5 text-primary-light text-sm outline-none focus:border-emerald-500/30 w-1/3" />
            </div>

            <div class="space-y-1">
              <label class="text-[10px] font-bold text-primary-light/40 uppercase tracking-widest flex items-center gap-2">
                <Terminal size={10} class="text-emerald-500" /> Rhai Script
              </label>
              <textarea bind:value={rule.script} rows="5" class="w-full bg-black/40 border border-emerald-500/10 rounded-xl px-4 py-3 text-emerald-400 font-mono text-xs outline-none focus:border-emerald-500/30 shadow-inner" spellcheck="false"></textarea>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  :global(.text-emerald-400) {
    text-shadow: 0 0 15px rgba(52, 211, 153, 0.3);
  }
  textarea {
    resize: none;
  }
</style>
