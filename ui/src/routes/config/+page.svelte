<script lang="ts">
    import { onMount } from 'svelte';

    let activeConfig = $state(null);
    let candidateConfig = $state(null);
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

<svelte:head>
    <title>CONFIG ENGINE // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-5xl mx-auto mt-6 mb-10 flex flex-col gap-6">
    <div class="flex items-center justify-between mb-8">
        <h2 class="text-3xl text-white font-bold tracking-[0.2em]">CONFIG ENGINE</h2>
        <div class="px-3 py-1 rounded text-[10px] font-bold tracking-widest uppercase border border-neon-green/40 text-neon-green bg-neon-green/10 shadow-[0_0_15px_rgba(0,255,196,0.05)]">
            TRANSACTIONAL STATE
        </div>
    </div>

    <!-- Panel Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        
        <!-- Active Pane -->
        <div class="p-6 rounded-xl border border-gray-800 bg-black/80 backdrop-blur-md relative overflow-hidden flex flex-col h-full">
            <div class="absolute top-0 left-0 w-full h-1 bg-gray-600"></div>
            <h3 class="text-xl font-bold text-gray-500 tracking-widest mb-6">ACTIVE CONFIG</h3>
            <div class="flex-1 bg-black/50 p-4 rounded border border-white/5 overflow-y-auto">
                <pre class="text-xs text-gray-500 font-mono leading-relaxed">{JSON.stringify(activeConfig, null, 2)}</pre>
            </div>
            <p class="text-[10px] text-gray-600 mt-4 uppercase tracking-widest text-center">Currently Running Values</p>
        </div>

        <!-- Candidate Pane -->
        <div class="p-6 rounded-xl border border-neon-green/40 shadow-[0_0_20px_rgba(0,255,196,0.05)] bg-black/40 backdrop-blur-md relative overflow-hidden flex flex-col h-full">
            <div class="absolute top-0 left-0 w-full h-1 bg-neon-green"></div>
            <h3 class="text-xl font-bold text-neon-green tracking-widest mb-6 drop-shadow-[0_0_5px_rgba(0,255,196,0.3)]">CANDIDATE DRAFT</h3>
            
            <div class="flex-1 bg-neon-green/5 p-4 rounded border border-neon-green/10 overflow-y-auto mb-6">
                <pre class="text-xs text-neon-green/80 font-mono leading-relaxed drop-shadow-[0_0_2px_rgba(0,255,196,0.2)]">{JSON.stringify(candidateConfig, null, 2)}</pre>
            </div>

            <!-- Editor -->
            <div class="flex flex-col gap-4">
                <div class="flex flex-col gap-2">
                    <label class="text-[10px] text-neon-green/60 uppercase tracking-widest">Edit Hostname (Test)</label>
                    <div class="flex gap-2">
                        <input type="text" bind:value={editHostname} class="flex-1 bg-black/80 border border-neon-green/30 rounded p-2.5 text-neon-green font-bold text-sm focus:outline-none focus:border-neon-green shadow-inner" />
                        <button onclick={saveCandidate} class="px-4 bg-neon-green/10 border border-neon-green/50 text-neon-green rounded font-bold text-xs tracking-widest hover:bg-neon-green/20 hover:border-neon-green transition-all" title="Save to Candidate">EDIT</button>
                    </div>
                </div>
                
                <div class="flex gap-3 justify-end mt-4 pt-4 border-t border-neon-green/10">
                    <button onclick={rollbackConfig} class="px-6 py-2.5 bg-red-950/20 border border-red-500/50 text-red-500 rounded font-bold text-xs tracking-widest hover:bg-red-900/40 hover:border-red-500 transition-all uppercase">Rollback</button>
                    <button onclick={commitConfig} class="px-6 py-2.5 bg-neon-green text-black rounded font-bold text-xs tracking-widest hover:bg-white hover:shadow-[0_0_20px_rgba(255,255,255,0.4)] transition-all uppercase">COMMIT CHANGES</button>
                </div>
            </div>
        </div>

    </div>
</div>
