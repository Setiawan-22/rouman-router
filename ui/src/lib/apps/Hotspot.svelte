<script lang="ts">
    import { onMount } from 'svelte';

    let config = $state({ radius: { enabled: false, secret: '' }, proxy: { enabled: false } } as any);
    let loading = $state(true);
    let saving = $state(false);
    let message = $state("");

    let activeUsers = $state([] as any[]);

    async function fetchData() {
        try {
            const [cfgRes, userRes] = await Promise.all([
                fetch('/api/v1/config/active'),
                fetch('/api/v1/network/leases') // Using DHCP leases as a proxy for hotspot users if no specific hotspot API exists yet
            ]);
            
            if (cfgRes.ok) {
                const data = await cfgRes.json();
                config = data.active;
            }

            if (userRes.ok) {
                const userData = await userRes.json();
                // Filter users who are on the hotspot interface if possible, 
                // for now we show all active leases as "Connected Devices"
                activeUsers = userData.leases || [];
            }
        } catch (e) {
            console.error("Failed to fetch hotspot data", e);
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
                message = "Hotspot setting applied!";
                fetchData();
            }
        } catch (e) {
            message = "Error applying changes";
        } finally {
            saving = false;
        }
    }

    function generateVouchers() {
        message = "Generated 10 new vouchers (Simulation)";
    }

    onMount(fetchData);
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Hotspot Manager_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Captive Portal & Identity Orchestration</p>
        </div>
        
        <button 
            onclick={saveConfig}
            disabled={saving}
            class="rouman-btn-primary flex items-center gap-2"
        >
            {#if saving}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Applying...</span>
            {:else}
                <span>🚀 Save Config_</span>
            {/if}
        </button>
    </div>

    {#if message}
        <div class="p-3 bg-sky-50 border border-sky-100 text-sky-600 text-[9px] font-black uppercase tracking-widest text-center rounded-xl shadow-sm">
            {message}
        </div>
    {/if}

    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        <!-- Settings -->
        <div class="xl:col-span-5 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">⚙️</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Portal Logic</h4>
            </div>

            <div class="flex flex-col gap-6">
                <div class="flex justify-between items-center p-5 rounded-2xl bg-slate-50 border border-slate-100 group hover:border-sky-200 transition-all shadow-sm">
                    <div class="flex flex-col gap-0.5">
                        <span class="font-black tracking-widest uppercase text-slate-800 text-[10px]">Captive Portal</span>
                        <span class="text-[9px] text-slate-400 font-bold uppercase tracking-tight">RADIUS INTERCEPT ENGINE</span>
                    </div>
                    <button 
                        onclick={() => config.radius.enabled = !config.radius.enabled}
                        class={`px-4 py-1.5 rounded-full text-[9px] font-black transition-all border shadow-sm ${config.radius.enabled ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-100 text-slate-400 border-slate-200'}`}
                    >
                        {config.radius.enabled ? 'ONLINE' : 'DORMANT'}
                    </button>
                </div>
                
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">RADIUS Shared Secret</label>
                    <input bind:value={config.radius.secret} type="password" class="rouman-input !py-1.5 !text-sky-600 font-black" placeholder="••••••••••••" />
                </div>
            </div>
            
            <button onclick={generateVouchers} class="mt-6 py-3 bg-white border border-slate-100 rounded-xl text-[10px] font-black uppercase tracking-[0.2em] hover:bg-slate-50 transition-all text-slate-600 shadow-sm">
                Generate Session Vouchers_
            </button>
        </div>

        <!-- Active Users -->
        <div class="xl:col-span-7 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-50 pb-4">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">👥</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Live Sessions</h4>
                </div>
                <div class="px-4 py-1.5 bg-emerald-50 border border-emerald-100 rounded-full text-emerald-600 font-black text-[9px] uppercase tracking-widest shadow-sm">
                    {activeUsers.length} Devices Path
                </div>
            </div>

            <div class="flex flex-col gap-4 overflow-y-auto max-h-[400px] pr-2 custom-scrollbar">
                {#each activeUsers as user}
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex items-center justify-between group hover:border-emerald-200 hover:bg-white transition-all shadow-sm">
                        <div class="flex items-center gap-4">
                            <div class="w-2.5 h-2.5 rounded-full bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.4)]"></div>
                            <div class="flex flex-col gap-0.5">
                                <span class="text-slate-800 font-black tracking-widest text-[12px] uppercase">{user.hostname || 'UNNAMED_ENDPOINT'}</span>
                                <span class="text-[9px] text-slate-400 font-mono tracking-widest font-black">{user.mac}</span>
                            </div>
                        </div>
                        <div class="text-right flex flex-col gap-0.5">
                            <span class="text-emerald-600 font-black text-[11px] font-mono">{user.ip}</span>
                            <span class="text-[8px] text-slate-300 uppercase font-black tracking-widest">Authorized_</span>
                        </div>
                    </div>
                {/each}
                {#if activeUsers.length === 0}
                    <div class="py-24 flex flex-col items-center gap-4 border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                        <span class="text-4xl grayscale opacity-20">🛡️</span>
                        <span class="text-[10px] text-slate-300 font-black uppercase tracking-[0.3em]">No authenticated clients detected</span>
                    </div>
                {/if}
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
