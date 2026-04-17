<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { goto } from '$app/navigation';

    let tunnelToken = $state('');
    let isRunning = $state(false);
    let isSubmitting = $state(false);
    let statusInterval: number | ReturnType<typeof setInterval>;
    let errorMessage = $state('');

    async function checkStatus() {
        try {
            const res = await fetch('/api/v1/services/cloudflare/status');
            if (res.ok) {
                const data = await res.json();
                isRunning = data.running;
            }
        } catch { }
    }

    async function handleToggle() {
        if (isSubmitting) return;
        isSubmitting = true;
        errorMessage = '';

        try {
            if (isRunning) {
                await fetch('/api/v1/services/cloudflare/stop', { method: 'POST' });
                isRunning = false;
            } else {
                if (!tunnelToken.trim()) {
                    errorMessage = "Tunnel Token is required";
                    return;
                }
                const res = await fetch('/api/v1/services/cloudflare/start', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ token: tunnelToken.trim() })
                });

                if (res.ok) {
                    isRunning = true;
                } else {
                    errorMessage = "Failed to start Cloudflare Tunnel";
                }
            }
        } catch {
            errorMessage = "Control Plane Error";
        } finally {
            isSubmitting = false;
        }
    }

    onMount(() => {
        checkStatus();
        statusInterval = setInterval(checkStatus, 3000);
    });

    onDestroy(() => {
        if (statusInterval) clearInterval(statusInterval);
    });
</script>

<svelte:head>
    <title>ZERO TRUST // ROUMAN</title>
</svelte:head>

<div class="w-full max-w-2xl mx-auto mt-16 mb-10 flex flex-col gap-6">
    
    <div class="flex items-center justify-between mb-2">
        <h2 class="text-3xl text-white font-bold tracking-[0.2em]">ZERO TRUST</h2>
        <div class={`px-3 py-1 rounded text-xs font-bold tracking-widest uppercase border ${isRunning ? 'border-neon-green text-neon-green shadow-[0_0_10px_rgba(0,255,196,0.5)]' : 'border-red-500/50 text-red-500/80 bg-red-950/20'}`}>
            {isRunning ? 'ACTIVE' : 'INACTIVE'}
        </div>
    </div>

    <!-- Main Card -->
    <div class="w-full p-8 rounded-xl border {isRunning ? 'border-neon-green/40 bg-neon-green/5' : 'border-neon-green/10 bg-black/40'} backdrop-blur-md shadow-[0_0_30px_rgba(0,255,196,0.02)] transition-colors duration-500">
        
        <div class="mb-8">
            <h3 class="text-xl font-bold tracking-widest text-white">CLOUDFLARED</h3>
            <p class="text-xs uppercase tracking-widest text-neon-green/50 mt-1">Orchestrator Module (Layer 4)</p>
        </div>

        {#if errorMessage}
            <div class="mb-6 p-3 border border-red-500/50 bg-red-950/30 text-red-400 text-xs rounded text-center">
                {errorMessage}
            </div>
        {/if}

        <div class="flex flex-col gap-6">
            <div class="flex flex-col gap-2">
                <label for="token" class="text-xs uppercase tracking-widest text-neon-green/70">Tunnel Token</label>
                <input 
                    type="password" 
                    id="token" 
                    bind:value={tunnelToken}
                    disabled={isRunning}
                    placeholder={isRunning ? "•••••••••••••••••••••••••••••" : "ey... (Paste token here)"}
                    class="w-full bg-black/60 border border-neon-green/20 rounded p-3 text-neon-green focus:outline-none focus:border-neon-green transition-all disabled:opacity-30 font-mono text-sm"
                />
            </div>

            <button
                onclick={handleToggle}
                disabled={isSubmitting}
                class={`w-full py-4 rounded font-bold tracking-widest uppercase transition-all duration-300 border ${
                    isRunning 
                        ? 'border-red-500/50 bg-red-950/20 text-red-400 hover:bg-red-900/40 hover:border-red-500 shadow-[0_0_15px_rgba(255,0,0,0.1)]' 
                        : 'border-neon-green bg-neon-green/10 text-neon-green hover:bg-neon-green/20 hover:shadow-[0_0_15px_rgba(0,255,196,0.4)]'
                } focus:outline-none disabled:opacity-50`}
            >
                {isSubmitting ? 'PROCESSING...' : isRunning ? 'TERMINATE TUNNEL' : 'INITIALIZE TUNNEL'}
            </button>
        </div>
    </div>
</div>
