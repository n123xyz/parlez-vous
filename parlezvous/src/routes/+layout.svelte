<script lang="ts">
    import '../app.css';
    import { onMount } from 'svelte';
    import { checkOllamaHealth, ollamaState } from '$lib/state/ollama.svelte.ts';
    import { loadSettings } from '$lib/state/settings.svelte.ts';
    import { Toaster } from 'svelte-french-toast';
    import { page } from '$app/stores';

    let { children } = $props();
    let currentPath = $derived($page.url.pathname);

    onMount(() => {
        checkOllamaHealth();
        loadSettings();
    });
</script>


<Toaster />

<div class="h-[100dvh] w-screen overflow-hidden bg-zinc-950 text-zinc-100 selection:bg-yellow-200/30 flex flex-col-reverse md:flex-col">
    <!-- Navigation Header -->
    <header class="h-16 shrink-0 border-t md:border-t-0 md:border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md flex items-center px-2 md:px-6 justify-between z-40 overflow-x-auto">
        <div class="hidden md:flex items-center gap-2">
            <div class="w-8 h-8 rounded-lg bg-yellow-200 flex items-center justify-center font-bold text-zinc-900 text-xl shadow-[0_0_10px_rgba(253,253,150,0.3)]">
                P
            </div>
            <span class="text-xl font-bold tracking-tight text-zinc-100">Parlez<span class="text-yellow-200">Vous</span></span>
        </div>
        
        <nav class="flex items-center gap-4 md:gap-6 w-full md:w-auto justify-around md:justify-end shrink-0">
            <a href="/journal" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/journal' ? 'text-yellow-200' : ''}">Journal</a>
            <a href="/flashcards" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/flashcards' ? 'text-yellow-200' : ''}">Flashcards</a>
            <a href="/conjugator" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/conjugator' ? 'text-yellow-200' : ''}">Conjugator</a>
            <a href="/calendar" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/calendar' ? 'text-yellow-200' : ''}">Calendar</a>
            <a href="/canvas" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/canvas' ? 'text-yellow-200' : ''}">Canvas</a>
            <a href="/avatar" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/avatar' ? 'text-yellow-200' : ''}">Avatar</a>
            <a href="/map" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/map' ? 'text-yellow-200' : ''}">Map</a>
            <a href="/vision" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/vision' ? 'text-yellow-200' : ''}">Vision</a>
            <a href="/stats" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/stats' ? 'text-yellow-200' : ''}">Stats</a>
            <a href="/settings" class="text-zinc-400 hover:text-yellow-200 font-medium transition-colors text-sm md:text-base {currentPath === '/settings' ? 'text-yellow-200' : ''}">Settings</a>
        </nav>
    </header>

    <main class="flex-1 overflow-y-auto relative flex flex-col">
        {@render children()}
    </main>
</div>
