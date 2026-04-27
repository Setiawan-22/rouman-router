<script lang="ts">
    import { onMount } from 'svelte';
    import Microvms from './Microvms.svelte';
    import Containers from './Containers.svelte';
    import Stacks from './Stacks.svelte';
    import Storage from './Storage.svelte';
    import Nodes from './Nodes.svelte';

    let activeTab = $state('instances');

    const tabs = [
        { id: 'nodes', label: 'Infrastructure Nodes' },
        { id: 'instances', label: 'Running Instances' },
        { id: 'stacks', label: 'Compute Stacks' },
        { id: 'images', label: 'Image Management' },
        { id: 'storage', label: 'Network Storage' }
    ];

    // Mock Image Management data
    let images = $state([
        { name: 'alpine-3.18-x86_64.ext4', type: 'RootFS', size: '128 MB', status: 'Ready' },
        { name: 'ubuntu-22.04-minimal.qcow2', type: 'Cloud Image', size: '450 MB', status: 'Downloading (45%)' },
        { name: 'vmlinux-5.10.bin', type: 'Kernel', size: '12 MB', status: 'Ready' }
    ]);
</script>

<div class="flex flex-col gap-8 text-[#1E293B] text-[11px] max-w-7xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-slate-200 pb-6 mb-2">
        <div class="flex flex-col gap-1.5">
            <h3 class="rouman-header !border-none !mb-0 !pb-0 text-sky-600">Edge Compute Center_</h3>
            <p class="text-[10px] text-slate-400 tracking-[0.2em] uppercase font-bold">Unified Orchestration & Virtualization Platform</p>
        </div>
        
        <!-- Premium Tabs Navigation -->
        <div class="flex items-center gap-1.5 bg-slate-50 p-1.5 rounded-2xl border border-slate-100 shadow-inner">
            {#each tabs as tab}
                <button 
                    onclick={() => activeTab = tab.id}
                    class={`px-5 py-2 rounded-xl text-[10px] font-black tracking-widest uppercase transition-all shadow-sm ${activeTab === tab.id ? 'bg-white text-sky-600 border border-slate-100' : 'text-slate-400 hover:text-slate-600'}`}
                >
                    {tab.label}
                </button>
            {/each}
        </div>
    </div>

    <!-- Tab Content -->
    <div class="flex flex-col gap-8">
        {#if activeTab === 'nodes'}
            <Nodes />
        {:else if activeTab === 'instances'}
            <div class="flex flex-col gap-12">
                <Microvms />
                <div class="border-t border-slate-100"></div>
                <Containers />
            </div>
        {:else if activeTab === 'stacks'}
            <Stacks />
        {:else if activeTab === 'images'}
            <div class="rouman-card flex flex-col gap-8 !bg-white/60">
                <div class="flex justify-between items-center border-b border-slate-50 pb-5">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-sky-50 flex items-center justify-center text-sky-600 shadow-sm">📀</div>
                        <h4 class="font-black text-xs text-slate-800 tracking-widest uppercase">Global Image Registry</h4>
                    </div>
                    <button class="px-5 py-2 bg-sky-600 text-white rounded-xl text-[10px] font-black tracking-widest uppercase hover:bg-sky-700 transition-all shadow-lg">+ Upload Image_</button>
                </div>
                
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {#each images as img}
                        <div class="p-6 rounded-3xl border border-slate-100 bg-white/40 shadow-sm flex flex-col gap-4 group hover:border-sky-300 transition-all">
                            <div class="flex justify-between items-start">
                                <div class="flex flex-col gap-1">
                                    <span class="font-black text-slate-800 truncate max-w-[150px] text-[12px]">{img.name}</span>
                                    <span class="text-[9px] text-slate-400 uppercase font-black tracking-tight">{img.type}</span>
                                </div>
                                <span class={`px-3 py-1 rounded-full text-[9px] font-black shadow-sm ${img.status.includes('Ready') ? 'bg-emerald-50 text-emerald-600' : 'bg-blue-50 text-blue-600'}`}>
                                    {img.status}
                                </span>
                            </div>
                            <div class="flex justify-between items-center text-[10px] border-t border-slate-50 pt-4">
                                <span class="text-slate-400 font-black tracking-tighter">{img.size}</span>
                                <div class="flex gap-4">
                                    <button class="text-sky-600 font-black hover:underline uppercase text-[9px]">Edit</button>
                                    <button class="text-red-400 font-black hover:underline uppercase text-[9px]">Delete</button>
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {:else if activeTab === 'storage'}
            <Storage />
        {/if}
    </div>
</div>
