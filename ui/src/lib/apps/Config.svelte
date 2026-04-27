<script lang="ts">
    import { onMount } from 'svelte';

    let activeConfig = $state(null as any);
    let candidateConfig = $state(null as any);
    let editHostname = $state("");

    async function loadConfig() {
        const resAct = await fetch('/api/v1/config/active');
        if (resAct.ok) activeConfig = (await resAct.json()).active;

        const resCand = await fetch('/api/v1/config/candidate');
        if (resCand.ok) {
            candidateConfig = (await resCand.json()).candidate;
            editHostname = candidateConfig.system.hostname;
        }
    }

    async function saveCandidate() {
        if (!candidateConfig) return;
        let payload = JSON.parse(JSON.stringify(candidateConfig));
        payload.system.hostname = editHostname;

        await fetch('/api/v1/config/candidate', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        loadConfig();
    }

    async function commitConfig() {
        await fetch('/api/v1/config/commit', { method: 'POST' });
        loadConfig();
    }

    async function rollbackConfig() {
        await fetch('/api/v1/config/rollback', { method: 'POST' });
        loadConfig();
    }

    onMount(() => loadConfig());
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <!-- Premium Header -->
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Core Configuration_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Global Orchestration & Persistence Engine</p>
        </div>
        <div class="flex gap-4">
            <button onclick={rollbackConfig} class="px-5 py-2.5 rounded-xl border border-rose-100 text-rose-500 bg-rose-50 hover:bg-rose-100 transition-all font-black uppercase tracking-widest text-[9px] shadow-sm">
                Discard Staged_
            </button>
            <button onclick={commitConfig} class="rouman-btn-primary flex items-center gap-2 shadow-lg">
                <span>🚀 Apply Staged Config_</span>
            </button>
        </div>
    </div>

    <!-- Info Banner: Explain Workflow -->
    <div class="bg-sky-50 border border-sky-100 rounded-3xl p-6 flex gap-5 items-center shadow-sm">
        <div class="w-12 h-12 rounded-2xl bg-white flex items-center justify-center text-sky-600 text-2xl shadow-sm border border-sky-100">
            💡
        </div>
        <div class="flex-1">
            <h4 class="font-black text-sky-700 uppercase tracking-widest text-[10px] mb-1.5">Candidate-Commit Workflow_</h4>
            <p class="text-sky-600/80 text-[10px] leading-relaxed font-bold uppercase tracking-tight">
                Rouman OS utilizes a professional staging matrix. Changes made are stored in a <span class="text-sky-800 font-black">Candidate State</span>. 
                They only transition to <span class="text-emerald-600 font-black">Active Runtime</span> after explicit commitment. This architectural safety prevents lockout events.
            </p>
        </div>
    </div>

    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        
        <!-- Left: System Controls -->
        <div class="xl:col-span-5 flex flex-col gap-8">
            <div class="rouman-card flex flex-col gap-8 !bg-white/60">
                <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                    <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">⚙️</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">General Settings</h4>
                </div>
                
                <div class="flex flex-col gap-6">
                    <div class="flex flex-col gap-2.5">
                        <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Router Network Identity</label>
                        <div class="flex gap-3">
                            <input type="text" bind:value={editHostname} class="rouman-input flex-1 !py-1.5 !text-slate-800 font-black" placeholder="rouman-core" />
                            <button onclick={saveCandidate} class="px-5 py-1.5 bg-sky-50 text-sky-600 rounded-xl border border-sky-100 font-black text-[9px] uppercase hover:bg-sky-100 shadow-sm transition-all">Set</button>
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-4 mt-2">
                        <div class="p-4 rounded-2xl bg-slate-50 border border-slate-100 flex flex-col gap-3 shadow-sm">
                            <span class="text-[9px] text-slate-400 uppercase font-black tracking-widest">L2 Discovery</span>
                            <div class="flex items-center gap-3">
                                <button 
                                    onclick={() => { candidateConfig.rdp.enabled = !candidateConfig.rdp.enabled; saveCandidate(); }} 
                                    class={`px-4 py-1.5 rounded-full text-[9px] font-black transition-all shadow-sm border ${candidateConfig?.rdp?.enabled ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-100 text-slate-400 border-slate-200'}`}
                                >
                                    {candidateConfig?.rdp?.enabled ? 'BROADCAST' : 'DORMANT'}
                                </button>
                            </div>
                        </div>

                        <div class="p-4 rounded-2xl bg-slate-50 border border-slate-100 flex flex-col gap-3 shadow-sm">
                            <span class="text-[9px] text-slate-400 uppercase font-black tracking-widest">Config Security</span>
                            <div class="flex items-center gap-3">
                                <button 
                                    onclick={() => { candidateConfig.security.encryption_enabled = !candidateConfig.security.encryption_enabled; saveCandidate(); }} 
                                    class={`px-4 py-1.5 rounded-full text-[9px] font-black transition-all shadow-sm border ${candidateConfig?.security?.encryption_enabled ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-100 text-slate-400 border-slate-200'}`}
                                >
                                    {candidateConfig?.security?.encryption_enabled ? 'ENCRYPTED' : 'PLAINTEXT'}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="rouman-card flex flex-col gap-6 !bg-emerald-50/20 border-emerald-100 shadow-sm">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">💾</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-emerald-600">Maintenance Matrix</h4>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <button class="py-3 bg-white border border-slate-100 rounded-xl text-[9px] font-black uppercase hover:bg-emerald-50 transition-all text-slate-600 shadow-sm">Download Backup</button>
                    <button class="py-3 bg-white border border-rose-100 rounded-xl text-[9px] font-black uppercase hover:bg-rose-50 transition-all text-rose-500 shadow-sm">Factory Reset_</button>
                </div>
            </div>
        </div>

        <!-- Right: Real-time Config Sync View -->
        <div class="xl:col-span-7 rouman-card flex flex-col gap-6 overflow-hidden h-[650px] !bg-white/60">
            <div class="flex items-center justify-between border-b border-slate-50 pb-5">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-600 shadow-sm">🗂️</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Orchestration Diff</h4>
                </div>
                <span class="text-[9px] font-mono text-slate-300 font-black tracking-widest uppercase">system_manifest.json</span>
            </div>
            
            <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-6 overflow-hidden">
                <div class="flex flex-col gap-3 h-full overflow-hidden">
                    <span class="text-[9px] text-slate-400 font-black uppercase tracking-widest px-2 flex items-center gap-2">
                        <span class="w-1.5 h-1.5 bg-emerald-400 rounded-full"></span> Active Runtime
                    </span>
                    <div class="flex-1 bg-slate-900 rounded-3xl border border-slate-800 p-5 font-mono text-[10px] text-emerald-400/80 overflow-auto custom-scrollbar shadow-2xl">
                        <pre class="leading-relaxed">{JSON.stringify(activeConfig, null, 2)}</pre>
                    </div>
                </div>
                <div class="flex flex-col gap-3 h-full overflow-hidden">
                    <span class="text-[9px] text-sky-600 font-black uppercase tracking-widest px-2 flex items-center gap-2">
                        <span class="w-1.5 h-1.5 bg-sky-500 rounded-full animate-pulse"></span> Staged Candidate
                    </span>
                    <div class="flex-1 bg-slate-900 rounded-3xl border border-sky-500/30 p-5 font-mono text-[10px] text-sky-400 overflow-auto custom-scrollbar shadow-2xl">
                        <pre class="leading-relaxed">{JSON.stringify(candidateConfig, null, 2)}</pre>
                    </div>
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
        background: rgba(0, 0, 0, 0.1);
        border-radius: 10px;
    }
</style>
