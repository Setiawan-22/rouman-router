<script lang="ts">
    import { onMount } from 'svelte';

    let blacklist = $state([] as string[]);
    let newDomain = $state("");
    let loading = $state(false);

    async function loadBlacklist() {
        try {
            const res = await fetch('/api/v1/dns/blacklist');
            if (res.ok) {
                blacklist = await res.json();
            }
        } catch (e) {
            console.error("Failed to load DNS blacklist", e);
        }
    }

    async function addDomain() {
        if (!newDomain) return;
        loading = true;
        try {
            const res = await fetch('/api/v1/dns/blacklist', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ domain: newDomain })
            });
            if (res.ok) {
                newDomain = "";
                await loadBlacklist();
            }
        } finally {
            loading = false;
        }
    }

    async function removeDomain(domain: string) {
        loading = true;
        try {
            const res = await fetch('/api/v1/dns/blacklist', {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ domain })
            });
            if (res.ok) {
                await loadBlacklist();
            }
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadBlacklist();
    });
</script>

<svelte:head>
    <title>DNS SINKHOLE // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-5xl mx-auto mt-6 mb-10 flex flex-col gap-6">
    <div class="flex items-center justify-between mb-8">
        <h2 class="text-3xl text-white font-bold tracking-[0.2em]">DNS <span class="text-neon-green">SINKHOLE</span></h2>
        <div class="px-3 py-1 rounded text-[10px] font-bold tracking-widest uppercase border border-neon-green/40 text-neon-green bg-neon-green/10 shadow-[0_0_15px_rgba(0,255,196,0.1)]">
            ADS & MALWARE BLOCKING
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mt-4">
        <!-- Add Domain -->
        <div class="p-8 rounded-xl border border-neon-green/30 bg-black/40 shadow-[0_0_40px_rgba(0,255,196,0.05)]">
            <h3 class="text-xl font-bold text-white tracking-widest mb-6">BLOCK DOMAIN</h3>
            <p class="text-xs text-gray-400 mb-8 leading-relaxed">
                Add a domain to the sinkhole. Any DNS requests for this domain will be resolved to 0.0.0.0.
            </p>
            
            <form onsubmit={(e) => { e.preventDefault(); addDomain(); }} class="flex flex-col gap-4">
                <input 
                    type="text" 
                    bind:value={newDomain}
                    placeholder="e.g. doubleclick.net"
                    class="bg-black/80 border border-neon-green/20 rounded p-4 text-neon-green font-mono text-sm focus:outline-none focus:border-neon-green transition-all shadow-inner"
                />
                <button 
                    type="submit"
                    disabled={loading || !newDomain}
                    class="w-full py-4 bg-neon-green/80 hover:bg-neon-green text-black rounded font-bold tracking-widest transition-all disabled:opacity-50 uppercase text-xs"
                >
                    {loading ? "PROCESSING..." : "BLOCK DOMAIN"}
                </button>
            </form>
        </div>

        <!-- Blacklist Display -->
        <div class="p-8 rounded-xl border border-white/10 bg-black/40 relative overflow-hidden flex flex-col">
             <h3 class="text-xl font-bold text-white tracking-widest mb-6 uppercase">Active Blacklist</h3>
             <div class="flex-1 overflow-y-auto max-h-[400px] pr-2">
                {#if blacklist.length === 0}
                    <div class="text-center py-10 text-gray-500 italic text-sm">No domains blocked yet.</div>
                {:else}
                    <div class="space-y-2">
                        {#each blacklist as domain}
                            <div class="flex items-center justify-between p-3 rounded bg-white/5 border border-white/5 hover:border-white/10 transition-colors group">
                                <span class="font-mono text-sm text-gray-300">{domain}</span>
                                <button 
                                    onclick={() => removeDomain(domain)}
                                    class="text-red-500 opacity-0 group-hover:opacity-100 transition-opacity hover:text-red-400"
                                    title="Unblock"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                                    </svg>
                                </button>
                            </div>
                        {/each}
                    </div>
                {/if}
             </div>
        </div>
    </div>
</div>
