<script lang="ts">
    import { onMount } from 'svelte';
    
    let { featurePath, title, description, children } = $props();

    let enabled = $state(false);
    let loading = $state(true);
    let error = $state("");

    async function checkStatus() {
        loading = true;
        try {
            const res = await fetch('/api/v1/config/active', { credentials: 'same-origin' });
            if (res.ok) {
                const cfg = (await res.json()).active;
                let val = cfg;
                for (const part of featurePath.split('.')) {
                    if (val == null) break;
                    val = val[part];
                }
                enabled = !!val;
            } else {
                error = `Config fetch failed: ${res.status}`;
            }
        } catch(e: any) {
            error = e.toString();
        }
        loading = false;
    }

    async function enableService() {
        loading = true;
        error = "";
        try {
            const res = await fetch('/api/v1/config/candidate', { credentials: 'same-origin' });
            if (!res.ok) { error = `Failed to load candidate: ${res.status}`; loading = false; return; }
            let candidate = (await res.json()).candidate;

            let target = candidate;
            const parts = featurePath.split('.');
            for (let i = 0; i < parts.length - 1; i++) {
                if (target[parts[i]] == null) {
                    target[parts[i]] = {};
                }
                target = target[parts[i]];
            }
            target[parts[parts.length - 1]] = true;

            const putRes = await fetch('/api/v1/config/candidate', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(candidate),
                credentials: 'same-origin'
            });
            if (!putRes.ok) { error = `Failed to update candidate: ${putRes.status}`; loading = false; return; }

            const commitRes = await fetch('/api/v1/config/commit', { method: 'POST', credentials: 'same-origin' });
            if (!commitRes.ok) { error = `Failed to commit: ${commitRes.status}`; loading = false; return; }

            enabled = true;
        } catch(e: any) {
            error = e.toString();
        }
        loading = false;
    }

    onMount(() => {
        checkStatus();
    });
</script>

{#if loading}
    <div class="w-full h-[60vh] flex items-center justify-center">
        <div class="flex flex-col items-center gap-4">
            <div class="w-8 h-8 border-4 border-[#1E88E5] border-t-transparent rounded-full animate-spin"></div>
            <div class="animate-pulse text-[#1E88E5] text-[10px] font-bold tracking-[0.2em] uppercase">Validating Service Policy...</div>
        </div>
    </div>
{:else if enabled}
    {@render children()}
{:else}
    <div class="flex flex-col items-center justify-center p-12 text-center h-[70vh] bg-white rounded-xl shadow-sm border border-gray-100 m-4">
        <div class="w-20 h-20 rounded-full bg-gray-50 flex items-center justify-center mb-6 shadow-inner">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-800 tracking-wide mb-3">{title} is Disabled</h2>
        <p class="text-sm text-gray-500 max-w-md mb-8 leading-relaxed">
            {description} <br/><br/>
            <span class="inline-block px-3 py-1 bg-blue-50 text-blue-600 rounded text-[10px] uppercase tracking-widest font-bold">Zero-Trust Policy</span><br/>
            <span class="text-xs mt-2 inline-block">You must explicitly enable this service to grant it system access.</span>
        </p>
        
        <button onclick={enableService} class="px-8 py-3.5 bg-[#1E88E5] text-white rounded-lg font-bold text-xs tracking-[0.2em] uppercase hover:bg-blue-600 hover:-translate-y-0.5 active:translate-y-0 transition-all shadow-lg shadow-blue-500/30 flex items-center gap-3">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            Initialize Service
        </button>
        {#if error}
            <div class="mt-6 text-red-500 text-[10px] font-mono tracking-wider p-3 bg-red-50 rounded border border-red-100">{error}</div>
        {/if}
    </div>
{/if}
