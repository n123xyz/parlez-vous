<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import toast from 'svelte-french-toast';

    let wsUrl = $state('ws://localhost:8080');
    let ws: WebSocket | null = null;
    let status = $state<'Disconnected' | 'Connecting' | 'Connected'>('Disconnected');
    let messages = $state<{ type: 'sent' | 'received' | 'system'; content: string; time: string }[]>([]);
    let currentInput = $state('');

    onDestroy(() => {
        if (ws) {
            ws.close();
        }
    });

    function connect() {
        if (!wsUrl.trim()) return;
        
        if (ws) {
            ws.close();
        }

        try {
            status = 'Connecting';
            ws = new WebSocket(wsUrl);

            ws.onopen = () => {
                status = 'Connected';
                addMessage('system', 'Connected to ' + wsUrl);
                toast.success('WebSocket connected!');
            };

            ws.onmessage = (event) => {
                addMessage('received', event.data);
            };

            ws.onclose = () => {
                status = 'Disconnected';
                addMessage('system', 'Disconnected from ' + wsUrl);
                ws = null;
            };

            ws.onerror = (error) => {
                console.error('WebSocket Error:', error);
                toast.error('WebSocket Error. Check console.');
                // Will also trigger onclose
            };
        } catch (e) {
            console.error(e);
            status = 'Disconnected';
            toast.error('Failed to connect: ' + e);
        }
    }

    function disconnect() {
        if (ws) {
            ws.close();
        }
    }

    function sendMessage() {
        if (!currentInput.trim() || status !== 'Connected' || !ws) return;

        try {
            ws.send(currentInput);
            addMessage('sent', currentInput);
            currentInput = '';
        } catch (e) {
            console.error('Failed to send:', e);
            toast.error('Failed to send message.');
        }
    }

    function addMessage(type: 'sent' | 'received' | 'system', content: string) {
        const time = new Date().toLocaleTimeString();
        messages = [...messages, { type, content, time }];
    }

    function clearMessages() {
        messages = [];
    }
</script>

<div class="max-w-5xl mx-auto p-6 h-[calc(100vh-64px)] flex flex-col gap-6">
    <!-- Header Controls -->
    <div class="bg-zinc-900 p-6 rounded-2xl border border-zinc-800 shadow-xl flex flex-col md:flex-row gap-4 items-center justify-between">
        <div class="flex-1 w-full flex items-center gap-4">
            <div class="flex flex-col">
                <span class="text-sm font-bold text-zinc-500 uppercase tracking-widest">Status</span>
                <div class="flex items-center gap-2 mt-1">
                    <div class="w-3 h-3 rounded-full 
                        {status === 'Connected' ? 'bg-green-500 shadow-[0_0_10px_#22c55e]' : 
                         status === 'Connecting' ? 'bg-yellow-200 animate-pulse' : 'bg-red-500'}">
                    </div>
                    <span class="font-medium {status === 'Connected' ? 'text-green-400' : 'text-zinc-400'}">{status}</span>
                </div>
            </div>

            <div class="w-px h-8 bg-zinc-800 mx-2"></div>

            <input 
                type="text" 
                bind:value={wsUrl}
                placeholder="ws://localhost:8080"
                class="flex-1 bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors font-mono text-sm"
                disabled={status === 'Connected' || status === 'Connecting'}
            >
        </div>

        <div class="flex items-center gap-3 w-full md:w-auto">
            {#if status === 'Connected'}
                <button 
                    class="w-full md:w-auto px-6 py-3 bg-red-500/10 hover:bg-red-500/20 text-red-400 font-bold rounded-xl transition-colors border border-red-500/20"
                    onclick={disconnect}
                >
                    Disconnect
                </button>
            {:else}
                <button 
                    class="w-full md:w-auto px-6 py-3 bg-yellow-200 hover:bg-yellow-300 disabled:opacity-50 text-zinc-900 font-bold rounded-xl transition-colors shadow-[0_0_15px_rgba(253,253,150,0.3)]"
                    onclick={connect}
                    disabled={status === 'Connecting' || !wsUrl.trim()}
                >
                    Connect
                </button>
            {/if}
            <button 
                class="w-full md:w-auto px-4 py-3 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 font-bold rounded-xl transition-colors"
                onclick={clearMessages}
            >
                Clear
            </button>
        </div>
    </div>

    <!-- Message Viewer -->
    <div class="flex-1 bg-zinc-900 rounded-3xl border border-zinc-800 shadow-2xl overflow-hidden flex flex-col relative">
        <!-- Messages Log -->
        <div class="flex-1 overflow-y-auto p-6 space-y-4 font-mono text-sm">
            {#if messages.length === 0}
                <div class="h-full flex items-center justify-center text-zinc-500 italic">
                    No messages yet. Connect and send data.
                </div>
            {/if}

            {#each messages as msg}
                {#if msg.type === 'system'}
                    <div class="flex justify-center my-4">
                        <span class="px-4 py-1 bg-zinc-800 text-zinc-400 rounded-full text-xs">
                            {msg.time} - {msg.content}
                        </span>
                    </div>
                {:else}
                    <div class="flex flex-col {msg.type === 'sent' ? 'items-end' : 'items-start'}">
                        <div class="flex items-baseline gap-2 mb-1 px-1">
                            <span class="text-xs text-zinc-600">{msg.time}</span>
                            <span class="text-xs font-bold uppercase tracking-wide {msg.type === 'sent' ? 'text-yellow-200' : 'text-blue-400'}">
                                {msg.type === 'sent' ? 'TX' : 'RX'}
                            </span>
                        </div>
                        <div class="max-w-[85%] px-4 py-3 rounded-2xl border {msg.type === 'sent' ? 'bg-yellow-200/10 border-yellow-200/20 text-yellow-100 rounded-tr-sm' : 'bg-zinc-800 border-zinc-700 text-zinc-200 rounded-tl-sm'} break-words whitespace-pre-wrap">
                            {msg.content}
                        </div>
                    </div>
                {/if}
            {/each}
        </div>

        <!-- Input Area -->
        <div class="p-4 bg-zinc-900/80 backdrop-blur border-t border-zinc-800">
            <form class="flex gap-3" onsubmit={(e) => { e.preventDefault(); sendMessage(); }}>
                <input 
                    type="text" 
                    bind:value={currentInput}
                    placeholder="Type JSON or text message..." 
                    class="flex-1 bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200/50 transition-colors font-mono text-sm"
                    disabled={status !== 'Connected'}
                >
                <button 
                    type="submit"
                    disabled={status !== 'Connected' || !currentInput.trim()}
                    class="bg-yellow-200 hover:bg-yellow-300 disabled:opacity-50 disabled:bg-zinc-800 disabled:text-zinc-500 text-zinc-900 px-8 py-3 rounded-xl font-bold transition-colors"
                >
                    Send
                </button>
            </form>
        </div>
    </div>
</div>
