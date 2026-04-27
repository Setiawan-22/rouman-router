<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import ServiceStarter from '$lib/components/ServiceStarter.svelte';

    let tunnelToken = $state('');
    let status = $state('offline');
    let isSubmitting = $state(false);
    let statusInterval: number | ReturnType<typeof setInterval>;
    let errorMessage = $state('');

    async function checkStatus() {
        try {
            const res = await fetch('/api/v1/services/cloudflare/status');
            if (res.ok) {
                const data = await res.json();
                status = data.status;
            }
        } catch { }
    }

    async function handleToggle() {
        if (isSubmitting) return;
        isSubmitting = true;
        errorMessage = '';

        const action = status === 'online' ? 'stop' : 'start';
        
        try {
            if (action === 'start' && !tunnelToken.trim()) {
                errorMessage = "Tunnel Token is required";
                isSubmitting = false;
                return;
            }

            const res = await fetch(`/api/v1/services/cloudflare/${action}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ token: tunnelToken.trim() })
            });

            if (!res.ok) {
                errorMessage = `Failed to ${action} Cloudflare Tunnel`;
            } else {
                // Wait for daemon to reconcile
                setTimeout(checkStatus, 2000);
            }
        } catch {
            errorMessage = "Control Plane Error";
        } finally {
            isSubmitting = false;
        }
    }

    onMount(() => {
        checkStatus();
        statusInterval = setInterval(checkStatus, 5000);
    });

    onDestroy(() => {
        if (statusInterval) clearInterval(statusInterval);
    });
</script>

<ServiceStarter featurePath="cloudflare.enabled" title="Cloudflare Tunnels" description="Zero Trust Tunnels for exposing your network safely without public IPs.">
<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Cloudflare Tunnel_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">L7 Zero Trust Orchestrator</p>
        </div>
        
        <div class={`px-4 py-1.5 rounded-full text-[9px] font-black tracking-widest uppercase shadow-sm border transition-all duration-500 ${status === 'online' ? 'border-emerald-100 bg-emerald-50 text-emerald-600' : 'border-rose-100 bg-rose-50 text-rose-500'}`}>
            {status === 'online' ? '● ONLINE' : '○ DORMANT'}
        </div>
    </div>

    <div class="rouman-card flex flex-col gap-8 !bg-white/60">
        <div class="flex flex-col gap-3">
            <label class="text-[9px] text-slate-400 uppercase tracking-widest font-black px-1">Tunnel Authorization Token (ey...)</label>
            <textarea 
                bind:value={tunnelToken}
                disabled={status === 'online'}
                placeholder="Paste your Cloudflare Argo Tunnel token here..."
                class="w-full bg-slate-50 border border-slate-200 rounded-2xl p-5 text-slate-800 font-mono text-[11px] focus:outline-none focus:border-sky-400 min-h-[120px] transition-all disabled:opacity-50 disabled:bg-slate-50/50 shadow-inner leading-relaxed"
            ></textarea>
        </div>

        {#if errorMessage}
            <div class="p-3 border border-rose-100 bg-rose-50 text-rose-500 text-[9px] text-center uppercase tracking-widest font-black rounded-xl shadow-sm">
                {errorMessage}
            </div>
        {/if}

        <button
            onclick={handleToggle}
            disabled={isSubmitting}
            class={`w-full py-3.5 rounded-xl font-black tracking-[0.3em] uppercase transition-all duration-500 border shadow-lg ${
                status === 'online' 
                    ? 'border-rose-200 bg-rose-50 text-rose-600 hover:bg-rose-100 shadow-rose-500/10' 
                    : 'border-sky-200 bg-sky-50 text-sky-600 hover:bg-sky-100 shadow-sky-600/10'
            } disabled:opacity-50 text-[10px]`}
        >
            {#if isSubmitting}
                <div class="flex items-center justify-center gap-3">
                    <div class="w-3.5 h-3.5 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
                    <span>Processing Cluster...</span>
                </div>
            {:else}
                <span>{status === 'online' ? 'Terminate Secure Tunnel_' : 'Initialize Argo Engine_'}</span>
            {/if}
        </button>
    </div>

    <div class="rouman-card !bg-slate-50/50 border-slate-100 p-6 flex flex-col gap-4 shadow-sm">
        <div class="flex items-center gap-3 border-b border-slate-200/50 pb-3">
            <div class="w-1.5 h-1.5 bg-slate-300 rounded-full"></div>
            <h4 class="text-[9px] font-black text-slate-400 tracking-[0.3em] uppercase">Operational Console_</h4>
        </div>
        <div class="font-mono text-[10px] text-slate-600 flex flex-col gap-2 leading-relaxed">
            {#if status === 'online'}
                <div class="flex gap-3"><span class="text-emerald-500 font-black tracking-widest">[LIVE]</span> <span>Cloudflare edge established via token authorization.</span></div>
                <div class="flex gap-3"><span class="text-sky-500 font-black tracking-widest">[INFO]</span> <span>Tunneling L3 traffic to Argo Network Fabric.</span></div>
            {:else if status === 'error'}
                <div class="flex gap-3"><span class="text-rose-500 font-black tracking-widest">[FAIL]</span> <span>Tunnel initialization failed. Check token validity.</span></div>
            {:else}
                <div class="flex gap-3"><span class="text-slate-300 font-black tracking-widest">[IDLE]</span> <span>Tunnel daemon is currently dormant. Waiting for trigger.</span></div>
            {/if}
        </div>
    </div>
</div>
</ServiceStarter>
