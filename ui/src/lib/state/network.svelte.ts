import { browser } from '$app/environment';

class NetworkState {
    interfaces = $state([] as any[]);
    loading = $state(false);

    async refresh() {
        if (!browser) return;
        this.loading = true;
        try {
            const res = await fetch('/api/v1/network/interfaces');
            if (res.ok) {
                const data = await res.json();
                this.interfaces = data.interfaces || [];
            }
        } catch (e) {
            console.error("Failed to fetch interfaces", e);
        } finally {
            this.loading = false;
        }
    }
}

export const networkState = new NetworkState();
