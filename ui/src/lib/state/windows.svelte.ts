import { type Component } from 'svelte';

export interface WindowState {
    id: string;
    title: string;
    component: Component<any>;
    props?: Record<string, any>;
    x: number;
    y: number;
    width: number;
    height: number;
    zIndex: number;
    isMinimized: boolean;
    isMaximized: boolean;
}

class WindowStore {
    windows = $state<WindowState[]>([]);
    nextZIndex = $state(100);

    open(id: string, title: string, component: Component<any>, props?: Record<string, any>) {
        const existing = this.windows.find(w => w.id === id);
        if (existing) {
            existing.isMinimized = false;
            this.focus(id);
            return;
        }

        const newWindow: WindowState = {
            id,
            title,
            component,
            props,
            x: 50 + (this.windows.length * 20),
            y: 50 + (this.windows.length * 20),
            width: 800,
            height: 500,
            zIndex: this.nextZIndex++,
            isMinimized: false,
            isMaximized: false
        };

        this.windows.push(newWindow);
    }

    close(id: string) {
        this.windows = this.windows.filter(w => w.id !== id);
    }

    minimize(id: string) {
        const win = this.windows.find(w => w.id === id);
        if (win) win.isMinimized = true;
    }

    focus(id: string) {
        const win = this.windows.find(w => w.id === id);
        if (win && win.zIndex < this.nextZIndex - 1) {
            win.zIndex = this.nextZIndex++;
        }
    }

    toggleMaximize(id: string) {
        const win = this.windows.find(w => w.id === id);
        if (win) {
            win.isMaximized = !win.isMaximized;
            this.focus(id);
        }
    }

    updatePosition(id: string, x: number, y: number) {
        const index = this.windows.findIndex(w => w.id === id);
        if (index !== -1) {
            this.windows[index].x = x;
            this.windows[index].y = y;
        }
    }
}

export const windowStore = new WindowStore();
