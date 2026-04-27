<script lang="ts">
    import { windowStore, type WindowState } from '$lib/state/windows.svelte';
    
    let { window: win }: { window: WindowState } = $props();

    let dragging = $state(false);
    let startX = 0;
    let startY = 0;

    function onMouseDown(e: MouseEvent) {
        if (win.isMaximized) return;
        if ((e.target as HTMLElement).closest('.window-controls')) return;
        
        dragging = true;
        startX = e.clientX - win.x;
        startY = e.clientY - win.y;
        windowStore.focus(win.id);
        
        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
        
        e.preventDefault();
    }

    function onMouseMove(e: MouseEvent) {
        if (!dragging) return;
        win.x = e.clientX - startX;
        win.y = e.clientY - startY;
    }

    function onMouseUp() {
        dragging = false;
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
    }
</script>

<div 
    class="absolute flex flex-col border bg-black/80 backdrop-blur-xl shadow-[0_10px_30px_rgba(0,0,0,0.5)] overflow-hidden 
    {dragging ? 'transition-none' : 'transition-all duration-200'} 
    {win.isMinimized ? 'opacity-0 pointer-events-none scale-95' : 'opacity-100'} 
    {win.isMaximized ? 'rounded-none border-transparent inset-0 !w-full !h-full !left-0 !top-0' : 'rounded-lg border-neon-green/30'}"
    style="left: {win.x}px; top: {win.y}px; width: {win.width}px; height: {win.height}px; z-index: {win.zIndex}"
    onmousedown={() => windowStore.focus(win.id)}
>
    <!-- WINDOW HEADER -->
    <div 
        class="h-10 bg-white/5 border-b border-white/10 flex items-center justify-between px-4 {win.isMaximized ? 'cursor-default' : 'cursor-move'} select-none active:bg-white/10"
        onmousedown={onMouseDown}
    >
        <span class="text-[10px] font-bold tracking-[0.2em] text-neon-green uppercase flex items-center gap-2">
            <div class="w-1.5 h-1.5 rounded-full bg-neon-green shadow-[0_0_5px_rgba(0,255,196,0.8)]"></div>
            {win.title}_
        </span>

        <div class="flex items-center gap-2 window-controls">
            <button 
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-white/10 text-gray-400"
                onclick={() => windowStore.minimize(win.id)}
            >
                <div class="w-2.5 h-0.5 bg-current translate-y-1"></div>
            </button>
            <button 
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-white/10 text-gray-400"
                onclick={() => windowStore.toggleMaximize(win.id)}
            >
                {#if win.isMaximized}
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H5a2 2 0 01-2-2V5a2 2 0 012-2h9a2 2 0 012 2v3m-1 1h3a2 2 0 012 2v9a2 2 0 01-2 2h-9a2 2 0 01-2-2v-3" />
                    </svg>
                {:else}
                    <div class="w-3 h-3 border-2 border-current rounded-sm"></div>
                {/if}
            </button>
            <button 
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-red-500/80 text-gray-400 hover:text-white transition-colors"
                onclick={() => windowStore.close(win.id)}
            >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>
    </div>

    <!-- WINDOW CONTENT -->
    <div class="flex-1 overflow-auto p-6 scrollbar-thin scrollbar-thumb-neon-green/20">
        <win.component {...win.props} />
    </div>
</div>

<style>
    /* Styling khusus konten dalam window agar konsisten */
    :global(.window-content) {
        height: 100%;
    }
</style>
