<script lang="ts">
    import { onMount } from 'svelte';

    let config = $state({ automation: { netwatch: [], scheduler: [], scripts: [] } } as any);
    let saving = $state(false);

    async function loadConfig() {
        const res = await fetch('/api/v1/config/candidate');
        if (res.ok) {
            const data = await res.json();
            if (data.candidate && data.candidate.automation) {
                config.automation = data.candidate.automation;
            }
            if (!config.automation) config.automation = { netwatch: [], scheduler: [], scripts: [] };
            if (!config.automation.netwatch) config.automation.netwatch = [];
            if (!config.automation.scheduler) config.automation.scheduler = [];
            if (!config.automation.scripts) config.automation.scripts = [];
        }
    }

    function addNetwatch() {
        config.automation.netwatch = [...config.automation.netwatch, {
            name: `Rule ${config.automation.netwatch.length + 1}`,
            target: '8.8.8.8',
            interval_secs: 60,
            script_up: '',
            script_down: '',
            enabled: true
        }];
    }

    function addScheduler() {
        config.automation.scheduler = [...config.automation.scheduler, {
            name: `Task ${config.automation.scheduler.length + 1}`,
            interval_secs: 3600,
            script: '',
            enabled: true
        }];
    }
    
    function addScript() {
        config.automation.scripts = [...config.automation.scripts, {
            name: `custom_script_${config.automation.scripts.length + 1}`,
            content: '#!/bin/sh\n\necho "Running router automation script"\n\n'
        }];
    }

    async function saveAutomation() {
        saving = true;
        const res = await fetch('/api/v1/config/candidate');
        let payload = (await res.json()).candidate;
        payload.automation = config.automation;

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

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <!-- Standardized Header -->
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-slate-800">Automation & Logic_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Netwatch Monitoring & Scheduled Tasks</p>
        </div>
        <button onclick={saveAutomation} disabled={saving} class="rouman-btn-primary flex items-center gap-2">
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Applying...</span>
            {:else}
                <span>💾 Apply Changes_</span>
            {/if}
        </button>
    </div>

    <!-- Main Content Grid -->
    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        
        <!-- Netwatch Section -->
        <div class="xl:col-span-7 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-100 pb-5">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">
                        📡
                    </div>
                    <div>
                        <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Netwatch Tools</h4>
                        <p class="text-[9px] text-slate-400 font-bold uppercase tracking-tight">Real-time Ping Monitoring</p>
                    </div>
                </div>
                <button onclick={addNetwatch} class="px-4 py-1.5 bg-emerald-50 text-emerald-600 rounded-full font-black text-[10px] uppercase hover:bg-emerald-100 transition-all border border-emerald-100 shadow-sm">
                    + Add Monitor
                </button>
            </div>
            
            <div class="flex flex-col gap-5 overflow-y-auto max-h-[600px] pr-2 custom-scrollbar">
                {#each config.automation.netwatch as rule, i}
                    <div class="bg-slate-50/50 border border-slate-100 p-6 rounded-2xl flex flex-col gap-6 group relative hover:border-emerald-200 hover:bg-white transition-all shadow-sm">
                        <button onclick={() => config.automation.netwatch.splice(i, 1)} class="absolute right-5 top-5 text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                        </button>
                        
                        <div class="flex items-center gap-4">
                            <input bind:value={rule.name} class="bg-transparent border-none text-slate-800 font-black text-[13px] w-1/2 outline-none focus:text-emerald-600 transition-colors" placeholder="Monitor Name" />
                            <div class="flex-1"></div>
                            <button 
                                onclick={() => rule.enabled = !rule.enabled} 
                                class={`px-4 py-1 rounded-full text-[9px] font-black tracking-widest transition-all shadow-sm ${rule.enabled ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-100 text-slate-400 border border-slate-200'}`}
                            >
                                {rule.enabled ? 'ACTIVE' : 'DISABLED'}
                            </button>
                        </div>
                        
                        <div class="grid grid-cols-2 gap-6">
                            <div class="flex flex-col gap-2">
                                <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Target Address</label>
                                <input bind:value={rule.target} class="rouman-input !py-1.5 !text-emerald-600 font-black" placeholder="e.g. 8.8.8.8" />
                            </div>
                            <div class="flex flex-col gap-2">
                                <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Check Interval (s)</label>
                                <input type="number" bind:value={rule.interval_secs} class="rouman-input !py-1.5 !text-sky-600 font-black" />
                            </div>
                        </div>
                        
                        <div class="grid grid-cols-1 gap-4">
                            <div class="flex flex-col gap-2">
                                <label class="text-[9px] text-emerald-600 uppercase font-black tracking-widest flex items-center gap-2">
                                    <span class="w-1.5 h-1.5 bg-emerald-500 rounded-full"></span> On Up Event
                                </label>
                                <input bind:value={rule.script_up} class="rouman-input !py-1.5 !text-emerald-600 !bg-white/40 font-black" placeholder="Run script when reachable..." />
                            </div>
                            <div class="flex flex-col gap-2">
                                <label class="text-[9px] text-red-600 uppercase font-black tracking-widest flex items-center gap-2">
                                    <span class="w-1.5 h-1.5 bg-red-500 rounded-full"></span> On Down Event
                                </label>
                                <input bind:value={rule.script_down} class="rouman-input !py-1.5 !text-red-600 !bg-white/40 font-black" placeholder="Run script when timeout..." />
                            </div>
                        </div>
                    </div>
                {/each}
                {#if config.automation.netwatch?.length === 0}
                    <div class="py-20 flex flex-col items-center gap-4 border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                        <span class="text-4xl grayscale opacity-20">🔍</span>
                        <span class="text-[10px] text-slate-300 font-black uppercase tracking-[0.3em]">No Active Monitors Configured</span>
                    </div>
                {/if}
            </div>
        </div>

        <!-- Right Side: Scheduler & Scripts -->
        <div class="xl:col-span-5 flex flex-col gap-8">
            
            <!-- Scheduler Card -->
            <div class="rouman-card flex flex-col gap-8 !bg-white/60">
                <div class="flex justify-between items-center border-b border-slate-100 pb-5">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">
                            ⏰
                        </div>
                        <div>
                            <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Cron Scheduler</h4>
                            <p class="text-[9px] text-slate-400 font-bold uppercase tracking-tight">Interval-based Tasks</p>
                        </div>
                    </div>
                    <button onclick={addScheduler} class="px-4 py-1.5 bg-sky-50 text-sky-600 rounded-full font-black text-[10px] uppercase hover:bg-sky-100 transition-all border border-sky-100 shadow-sm">
                        + New Task
                    </button>
                </div>
                
                <div class="flex flex-col gap-4 overflow-y-auto max-h-[250px] pr-2 custom-scrollbar">
                    {#each config.automation.scheduler as task, i}
                        <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-4 group relative hover:border-sky-200 hover:bg-white transition-all shadow-sm">
                            <div class="flex items-center gap-4">
                                <input bind:value={task.name} class="bg-transparent border-none text-slate-800 font-black flex-1 outline-none focus:text-sky-600 transition-colors text-[12px]" />
                                <button onclick={() => task.enabled = !task.enabled} class={`text-[9px] font-black px-3 py-1 rounded-full shadow-sm ${task.enabled ? 'bg-sky-50 text-sky-600' : 'bg-slate-100 text-slate-400'}`}>{task.enabled ? 'ACTIVE' : 'PAUSED'}</button>
                                <button onclick={() => config.automation.scheduler.splice(i, 1)} class="text-slate-300 hover:text-red-500 transition-colors p-1.5 hover:bg-red-50 rounded-lg">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                    </svg>
                                </button>
                            </div>
                            
                            <div class="grid grid-cols-12 gap-4 items-end">
                                <div class="col-span-4 flex flex-col gap-2">
                                    <label class="text-[8px] text-slate-400 uppercase font-black tracking-widest">Every (sec)</label>
                                    <input type="number" bind:value={task.interval_secs} class="rouman-input !py-1.5 !text-slate-800 font-black" />
                                </div>
                                <div class="col-span-8 flex flex-col gap-2">
                                    <label class="text-[8px] text-slate-400 uppercase font-black tracking-widest">Target Script</label>
                                    <input bind:value={task.script} class="rouman-input !py-1.5 !text-sky-600 font-black" placeholder="Script name..." />
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Library Card -->
            <div class="rouman-card flex flex-col gap-8 flex-1 !bg-white/60">
                <div class="flex justify-between items-center border-b border-slate-100 pb-5">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-600 shadow-sm">
                            📜
                        </div>
                        <div>
                            <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Script Library</h4>
                            <p class="text-[9px] text-slate-400 font-bold uppercase tracking-tight">Reusable Automation Units</p>
                        </div>
                    </div>
                    <button onclick={addScript} class="px-4 py-1.5 bg-purple-50 text-purple-600 rounded-full font-black text-[10px] uppercase hover:bg-purple-100 transition-all border border-purple-100 shadow-sm">
                        + New Script
                    </button>
                </div>
                
                <div class="flex flex-col gap-5 overflow-y-auto max-h-[350px] pr-2 custom-scrollbar">
                    {#each config.automation.scripts as script, i}
                        <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-4 group relative hover:border-purple-200 hover:bg-white transition-all shadow-sm">
                            <div class="flex justify-between items-center mb-1">
                                <input bind:value={script.name} class="bg-transparent border-b border-slate-100 text-purple-600 font-black uppercase w-2/3 outline-none focus:border-purple-500 transition-all pb-1 text-[12px]" />
                                <button onclick={() => config.automation.scripts.splice(i, 1)} class="text-slate-300 hover:text-red-500 text-[9px] font-black tracking-widest p-1.5 hover:bg-red-50 rounded-lg">REMOVE</button>
                            </div>
                            <textarea 
                                bind:value={script.content} 
                                rows="5" 
                                class="bg-white/80 border border-slate-100 rounded-xl p-4 text-slate-700 font-mono text-[11px] w-full outline-none focus:border-purple-300 transition-all shadow-inner"
                                placeholder="# Write your script here..."
                            ></textarea>
                        </div>
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
    .custom-scrollbar::-webkit-scrollbar-track {
        background: rgba(0, 0, 0, 0.02);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(0, 0, 0, 0.08);
        border-radius: 10px;
    }
</style>
