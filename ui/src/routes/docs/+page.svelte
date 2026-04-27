<script lang="ts">
    import { fade, fly } from 'svelte/transition';
    import { page } from '$app/state';

    const sections = [
        {
            id: 'intro',
            title: 'Introduction',
            icon: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
            content: `Rouman OS is an advanced, high-performance router operating system designed for modern infrastructure. 
            Built on a solid foundation of Rust and Linux eBPF, it provides enterprise-grade networking features with a focus on speed, security, and developer-centric automation.
            \n\n### Core Philosophy\n- **Security by Default**: Every packet is inspected at the driver level using eBPF.\n- **Performance First**: Minimal overhead through kernel-space processing.\n- **Everything is an API**: Full control over your router via RESTful endpoints.\n- **Integrated Cloud**: Native support for MicroVMs and Containers.`
        },
        {
            id: 'firewall',
            title: 'Firewall & Security',
            icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04M12 21a8.966 8.966 0 01-5.917-2.24L6 18c.633-1.037 1.258-2.001 2.5-2.001h7c1.242 0 1.867.964 2.5 2.001l.083.136A8.966 8.966 0 0112 21z',
            content: `The Rouman Firewall is powered by **XDP (Express Data Path)**. This technology allows the router to process packets before they even reach the Linux kernel's networking stack, significantly reducing latency and CPU usage during high-traffic attacks.
            \n\n### Tutorial: Managing Rules\n1. Navigate to the **Firewall** application in the sidebar.\n2. Use the **Real-time Monitor** to see blocked packets.\n3. Add new rules by specifying source/destination IPs or using **SNI Filtering** to block specific domains even on encrypted HTTPS traffic.\n\n### Security Features\n- **Anti-DDOS**: Automatic rate-limiting and SYN-flood protection.\n- **Geo-Blocking**: Block traffic from specific regions (requires configuration).\n- **SNI Inspector**: Deep Packet Inspection (DPI) Lite for blocking TLS hostnames.`
        },
        {
            id: 'network',
            title: 'WAN & Routing',
            icon: 'M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9',
            content: `Rouman OS supports advanced Multi-WAN configurations, ensuring your internet stays online even if one provider fails.
            \n\n### Failover Setup\n1. Connect multiple internet lines to your router's WAN ports.\n2. In the **Interfaces** app, enable each WAN and assign a **Distance** (Priority).\n3. The **WAN Manager** will automatically ping gateways every 10 seconds. If the primary link fails, it will switch the default route to the backup link and notify you via the Topbar.\n\n### Routing Options\n- **Static Routes**: Manually define paths for internal subnets.\n- **PPPoE**: Native support for fiber/DSL connections with automatic reconnection.`
        },
        {
            id: 'services',
            title: 'DHCP & DNS',
            icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
            content: `Manage local IP assignments and secure your DNS queries with ease.
            \n\n### DHCP Server\n- **Leases**: View and manage all connected devices in the **DHCP** app.\n- **Static Leases**: Reserve IPs for specific MAC addresses to ensure devices (like NAS or Servers) always get the same IP.\n\n### DNS Security (DoT/DoH)\nRouman OS provides a built-in DNS Sinkhole. It supports **DNS over TLS (DoT)** and **DNS over HTTPS (DoH)** to encrypt your browsing habits and block ads/malware at the network level.\n\n**Tutorial**: Simply enable "DNS Filtering" in the **DNS** app to start blocking known tracking domains globally.`
        },
        {
            id: 'hotspot',
            title: 'Hotspot & RADIUS',
            icon: 'M12 11c0 3.517-1.009 6.799-2.753 9.571m-3.44-2.04l.054-.09A10.003 10.003 0 0012 21a10.003 10.003 0 008.139-4.187l.054.09A10.003 10.003 0 0112 21c-4.47 0-8.268-2.943-9.542-7',
            content: `Create professional guest Wi-Fi or ISP-style voucher systems using the integrated RADIUS server.
            \n\n### Creating Vouchers\n1. Go to **Hotspot > Profiles** and create a speed limit (e.g., 5Mbps Download).\n2. Go to **Hotspot > Vouchers** and click **Generate**.\n3. Users can then log in via the captive portal using these codes.\n\n### Billing Integration\nRouman OS tracks data usage (bytes in/out) and uptime for each session, allowing you to build complex billing systems on top of the router.`
        },
        {
            id: 'compute',
            title: 'Compute & Cloud',
            icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2',
            content: `Why just route traffic when you can run apps? Rouman OS features a built-in **MicroVM** and **Container** engine.
            \n\n### Running MicroVMs (Firecracker)\nFirecracker allows you to run secure, isolated virtual machines with near-zero overhead. Ideal for running a local DNS server, Home Assistant, or custom proxy services.\n\n### Cloudflare Tunnels\nConnect your local services to the internet without opening ports. Integrate your **Cloudflare Zero Trust** token in the **Cloudflare** app to instantly tunnel traffic to your local compute instances.`
        },
        {
            id: 'automation',
            title: 'Automation (Rhai)',
            icon: 'M13 10V3L4 14h7v7l9-11h-7z',
            content: `Program your router using **Rhai**, a tiny and fast scripting language.
            \n\n### Example Script\n\`\`\`rust\n// Block a suspicious IP if usage exceeds 1GB\nlet usage = get_usage("192.168.1.50");\nif usage > 1024 * 1024 * 1024 {\n    firewall::block("192.168.1.50");\n    notify("Admin", "IP 192.168.1.50 blocked due to high usage");\n}\n\`\`\`\n\nTutorial: Open the **Automation** app, write your script, and set a **Trigger** (e.g., Every 5 minutes or on WAN Down).`
        },
        {
            id: 'api',
            title: 'Developer API',
            icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
            content: `The entire router is controllable via JSON-over-HTTP.
            \n\n### Authentication\nInclude your API key in the header: \`Authorization: Bearer <your_token>\`.\n\n### Common Endpoints\n- \`GET /api/v1/system/status\`: Get CPU/RAM and Network stats.\n- \`POST /api/v1/firewall/rules\`: Add a new firewall rule programmatically.\n- \`GET /api/v1/notifications\`: Fetch the latest system alerts.`
        }
    ];

    let activeSection = $derived(page.url.searchParams.get('s') || 'intro');
</script>

<div class="max-w-4xl mx-auto py-12 px-4 min-h-screen" in:fade>
    <!-- Main Content -->
    <div class="flex-1">
        {#each sections as section}
            {#if activeSection === section.id}
                <div in:fly={{ y: 20, duration: 400 }} class="flex flex-col gap-8">
                    <!-- Section Header -->
                    <div class="flex items-center gap-5">
                        <div class="w-14 h-14 rounded-2xl bg-[#1E88E5]/10 flex items-center justify-center text-[#1E88E5] shadow-sm">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-7 w-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={section.icon} />
                            </svg>
                        </div>
                        <div class="flex flex-col">
                             <span class="text-[10px] font-black text-[#1E88E5] uppercase tracking-[0.25em] mb-1">Documentation Module</span>
                             <h1 class="text-4xl font-black text-slate-800 tracking-tight">{section.title}</h1>
                        </div>
                    </div>
                    
                    <!-- Content -->
                    <div class="prose prose-slate max-w-none">
                        <p class="text-[17px] text-slate-600 leading-relaxed whitespace-pre-line font-medium">
                            {section.content}
                        </p>
                    </div>

                    <!-- Tutorial Card -->
                    <div class="mt-12 p-10 rounded-[2.5rem] bg-slate-900 text-white shadow-2xl relative overflow-hidden group">
                        <div class="absolute top-0 right-0 p-8 opacity-10 group-hover:scale-110 transition-transform duration-700">
                             <svg xmlns="http://www.w3.org/2000/svg" class="h-32 w-32" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={section.icon} />
                            </svg>
                        </div>
                        <div class="relative z-10 flex flex-col gap-6">
                            <div class="flex flex-col gap-2">
                                <h3 class="text-2xl font-black tracking-tight">Need expert assistance?</h3>
                                <p class="text-slate-400 text-sm max-w-md font-medium">Our enterprise support engineers are available 24/7 to help you optimize your Rouman OS deployment for maximum security and performance.</p>
                            </div>
                            <div class="flex items-center gap-4">
                                <button class="px-8 py-4 bg-[#1E88E5] text-white rounded-2xl text-xs font-black uppercase tracking-widest hover:bg-[#1565C0] transition-all shadow-lg shadow-blue-500/20">
                                    Open Support Ticket
                                </button>
                                <button class="px-8 py-4 bg-white/10 text-white rounded-2xl text-xs font-black uppercase tracking-widest hover:bg-white/20 transition-all backdrop-blur-md">
                                    Community Forum
                                </button>
                            </div>
                        </div>
                    </div>

                    <!-- Footer Info -->
                    <div class="mt-12 pt-8 border-t border-slate-100 flex items-center justify-between">
                         <div class="flex items-center gap-2">
                             <div class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></div>
                             <span class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Documentation Updated Today</span>
                         </div>
                         <span class="text-[10px] font-bold text-slate-300 uppercase tracking-widest">v2.4.0 Production Build</span>
                    </div>
                </div>
            {/if}
        {/each}
    </div>
</div>
