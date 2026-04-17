<script lang="ts">
    import { onMount, onDestroy } from 'svelte';

    let interfaces = $state([]);
    let errorMsg = $state('');
    let loading = $state(true);

    async function loadInterfaces() {
        try {
            const res = await fetch('/api/v1/network/interfaces');
            const data = await res.json();
            if (res.ok && data.interfaces) {
                interfaces = data.interfaces;
            } else {
                errorMsg = data.error || 'Failed to load interfaces';
            }
        } catch (e) {
            errorMsg = 'Network Error';
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadInterfaces();
    });
</script>

<svelte:head>
    <title>NETWORK // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-5xl mx-auto mt-6 mb-10 flex flex-col gap-6">
    <div class="flex items-center justify-between mb-8">
        <h2 class="text-3xl text-white font-bold tracking-[0.2em]">NETWORK INTERFACES</h2>
        <div class="px-3 py-1 rounded text-xs font-bold tracking-widest uppercase border border-neon-green/40 text-neon-green bg-neon-green/10 shadow-[0_0_15px_rgba(0,255,196,0.05)]">
            LAYER 2 OAM
        </div>
    </div>

    <!-- Interface Grid -->
    {#if loading}
        <div class="text-neon-green/60 p-8 text-center animate-pulse tracking-widest text-sm flex items-center justify-center gap-3">
            <svg class="animate-spin h-5 w-5 text-neon-green" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
            SCANNING KERNEL NETLINK...
        </div>
    {:else if errorMsg}
        <div class="mb-6 p-4 border border-red-500/50 bg-red-950/30 text-red-400 text-sm rounded">
            {errorMsg}
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {#each interfaces as iface}
                <!-- Card Interface -->
                <div class="p-6 rounded-xl border {iface.is_up ? 'border-neon-green/40 shadow-[0_0_20px_rgba(0,255,196,0.07)] bg-gradient-to-br from-black/40 to-neon-green/5' : 'border-gray-800 bg-black/80'} backdrop-blur-md transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <div class="flex flex-col">
                            <span class="text-2xl font-bold tracking-widest {iface.is_up ? 'text-white drop-shadow-[0_0_5px_rgba(0,255,196,0.2)]' : 'text-gray-500'}">
                                {iface.name}
                            </span>
                            <span class="text-[10px] text-gray-500 tracking-widest uppercase mt-1">IF_INDEX: {iface.index}</span>
                        </div>
                        
                        <div class="px-2 py-1 rounded text-[10px] font-bold tracking-widest uppercase border {iface.is_up ? 'border-neon-green/80 text-neon-green bg-transparent drop-shadow-[0_0_5px_rgba(0,255,196,0.5)]' : 'border-gray-700 text-gray-500 bg-transparent'}">
                            {iface.oper_state}
                        </div>
                    </div>

                    <div class="flex flex-col gap-3 mt-8">
                        <div class="flex justify-between items-center pb-2 border-b border-white/5 group transition-colors hover:border-white/10">
                            <span class="text-[10px] text-gray-500 uppercase tracking-widest">MAC ADDR</span>
                            <span class="text-xs font-mono text-white/80 {iface.is_up ? 'group-hover:text-neon-green' : ''} transition-colors uppercase">{iface.mac || '00:00:00:00:00:00'}</span>
                        </div>
                        <div class="flex justify-between items-center pb-2 border-b border-white/5 group transition-colors hover:border-white/10">
                            <span class="text-[10px] text-gray-500 uppercase tracking-widest">MTU</span>
                            <span class="text-xs font-mono text-white/80 {iface.is_up ? 'group-hover:text-neon-green' : ''} transition-colors">{iface.mtu} BYTES</span>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
