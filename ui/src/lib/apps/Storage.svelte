<script lang="ts">
    import { onMount } from 'svelte';

    let mounts = $state([] as any[]);
    let loading = $state(false);
    
    // New mount form
    let protocol = $state("nfs");
    let remotePath = $state("");
    let localMount = $state("/mnt/storage");
    let options = $state("rw,soft");

    async function fetchMounts() {
        loading = true;
        try {
            const res = await fetch('/api/v1/compute/storage');
            if (res.ok) mounts = (await res.json()).mounts || [];
        } catch {}
        loading = false;
    }

    async function addMount() {
        if(!remotePath || !localMount) return;
        try {
            const res = await fetch('/api/v1/compute/storage', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ protocol, remote: remotePath, local: localMount, options })
            });
            if (res.ok) {
                remotePath = "";
                await fetchMounts();
            }
        } catch (e) {
            console.error(e);
        }
    }

    onMount(fetchMounts);
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-amber-600">Network Storage_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Distributed Volume & Disk Management</p>
        </div>
        <div class="px-4 py-2 bg-amber-50 border border-amber-100 text-amber-600 rounded-2xl text-[10px] font-black tracking-widest uppercase shadow-sm">
            Cluster Filesystem
        </div>
    </div>

    <div class="grid grid-cols-1 xl:grid-cols-12 gap-8">
        <!-- Mount Form -->
        <div class="xl:col-span-4 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex items-center gap-3 border-b border-slate-50 pb-4">
                <div class="w-8 h-8 rounded-lg bg-amber-50 flex items-center justify-center text-amber-600 shadow-sm">🔌</div>
                <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Mount Parameters</h4>
            </div>
            
            <div class="flex flex-col gap-5">
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Source Protocol</label>
                    <select bind:value={protocol} class="rouman-input !py-1.5 !text-amber-600 font-black appearance-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3Ryb2tlPSIjZjk3MzE2Ij48cGF0aCBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTE5IDlsLTcgNy03LTciLz48L3N2Zz4=')] bg-[length:12px] bg-[right_10px_center] bg-no-repeat pr-10">
                        <option value="nfs">NFS (Network File System)</option>
                        <option value="cifs">SMB/CIFS (Windows Share)</option>
                        <option value="iscsi">iSCSI Block Storage</option>
                    </select>
                </div>
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Remote Endpoint</label>
                    <input bind:value={remotePath} placeholder="10.0.0.50:/exports/data" class="rouman-input !py-1.5 font-mono !text-emerald-600 font-black" />
                </div>
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Local Target Path</label>
                    <input bind:value={localMount} placeholder="/mnt/remote_storage" class="rouman-input !py-1.5 font-mono !text-sky-600 font-black" />
                </div>
                <div class="flex flex-col gap-2.5">
                    <label class="text-[9px] text-slate-400 uppercase font-black tracking-widest px-1">Advanced Mount Options</label>
                    <input bind:value={options} placeholder="rw,soft,vers=4" class="rouman-input !py-1.5 font-mono !text-slate-400 font-black" />
                </div>
                
                <button onclick={addMount} class="mt-6 rouman-btn-primary !bg-amber-50 border !border-amber-100 !text-amber-600 hover:!bg-white !py-3 text-[10px] font-black tracking-[0.2em] shadow-sm">
                    Establish Connection_
                </button>
            </div>
        </div>

        <!-- Active Mounts -->
        <div class="xl:col-span-8 rouman-card flex flex-col gap-8 !bg-white/60">
            <div class="flex justify-between items-center border-b border-slate-50 pb-4">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 shadow-sm">📦</div>
                    <h4 class="font-black text-xs uppercase tracking-widest text-slate-800">Active Volumes</h4>
                </div>
                <span class="text-[9px] text-slate-300 font-black uppercase tracking-widest">OPERATIONAL</span>
            </div>
            
            <div class="flex-1 overflow-y-auto max-h-[450px] custom-scrollbar">
                {#if mounts.length === 0}
                    <div class="py-24 text-center text-slate-300 font-black uppercase tracking-[0.3em] border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
                        No external storage assets mapped
                    </div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="w-full text-left border-collapse">
                            <thead class="text-slate-400 text-[9px] uppercase tracking-widest border-b border-slate-50">
                                <tr>
                                    <th class="p-4 px-2">Local Map</th>
                                    <th class="p-4 px-2">Remote Origin</th>
                                    <th class="p-4 px-2">Protocol</th>
                                    <th class="p-4 text-right pr-2">Status</th>
                                </tr>
                            </thead>
                            <tbody class="text-slate-600 font-mono">
                                {#each mounts as m}
                                    <tr class="border-b border-slate-50 hover:bg-slate-50/50 transition-all">
                                        <td class="p-4 px-2 font-black text-sky-600 text-[11px]">{m.local}</td>
                                        <td class="p-4 px-2 text-slate-400 text-[10px] lowercase">{m.remote}</td>
                                        <td class="p-4 px-2">
                                            <span class="px-3 py-1 bg-amber-50 text-amber-600 border border-amber-100 rounded-full text-[9px] font-black shadow-sm">{m.protocol}</span>
                                        </td>
                                        <td class="p-4 text-right pr-2">
                                            <button class="text-red-300 hover:text-red-500 font-black text-[9px] tracking-widest transition-colors p-2 hover:bg-red-50 rounded-lg">UNMOUNT_</button>
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
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

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: rgba(255, 255, 255, 0.02);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 10px;
    }
</style>
