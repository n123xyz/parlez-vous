<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import { playSmartTTS } from '$lib/tts';
    import { settingsState } from '$lib/state/settings.svelte.ts';

    interface DbVocabItem {
        id: number;
        target_text: string;
        native_text: string;
        is_character: boolean;
    }

    let vocabulary = $state<DbVocabItem[]>([]);
    let isLoading = $state(true);

    onMount(async () => {
        try {
            vocabulary = (await invoke('get_all_vocabulary')) as DbVocabItem[];
        } catch (e) {
            console.error("Failed to load vocabulary:", e);
        } finally {
            isLoading = false;
        }
    });
</script>

<div class="max-w-6xl mx-auto p-6 flex flex-col gap-8 min-h-[calc(100vh-64px)]">
    <div class="flex items-center justify-between border-b border-zinc-800 pb-4">
        <div>
            <h1 class="text-3xl font-bold text-yellow-200">Vocabulary</h1>
            <p class="text-zinc-400 mt-2">Review the vocabulary extracted from your journal entries.</p>
        </div>
        <div class="bg-zinc-900 border border-zinc-800 px-6 py-3 rounded-2xl flex items-center gap-3">
            <span class="text-zinc-400 font-bold uppercase tracking-widest text-xs">Total Words</span>
            <span class="text-2xl font-bold text-zinc-100">{vocabulary.length}</span>
        </div>
    </div>

    {#if isLoading}
        <div class="flex-1 flex flex-col items-center justify-center gap-4">
            <div class="w-12 h-12 border-4 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
            <p class="text-zinc-500 font-bold tracking-widest uppercase text-sm">Loading Vocabulary...</p>
        </div>
    {:else if vocabulary.length === 0}
        <div class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-zinc-800 rounded-3xl p-12 bg-zinc-900/50">
            <div class="w-16 h-16 bg-zinc-800 rounded-full flex items-center justify-center mb-4">
                <span class="text-2xl text-zinc-600">?</span>
            </div>
            <h2 class="text-xl font-bold text-zinc-300 mb-2">No vocabulary yet</h2>
            <p class="text-zinc-500 text-center max-w-md">
                Head over to the <a href="/journal" class="text-yellow-200 hover:underline">Journal</a> tab and create your first entry to automatically generate vocabulary cards!
            </p>
        </div>
    {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {#each vocabulary as vocab}
                <div class="w-full bg-zinc-900 border border-zinc-800 rounded-3xl p-6 flex flex-col justify-between shadow-xl hover:border-yellow-200/30 transition-colors">
                    
                    <!-- Top: Target Language -->
                    <div class="flex flex-col items-center mb-6 mt-2 relative group">
                        <button 
                            class="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 bg-zinc-800 text-yellow-200 hover:text-yellow-400 p-2 rounded-full transition-all border border-zinc-700 hover:border-yellow-200/50"
                            onclick={() => playSmartTTS(vocab.target_text, settingsState.ttsServerUrl, undefined, settingsState.targetLanguage)}
                            title="Play Audio"
                        >
                            ▶
                        </button>
                        <span class="text-xs font-bold tracking-widest text-zinc-600 uppercase mb-2">Target</span>
                        <span class="text-3xl font-bold text-zinc-100 text-center break-words px-6">{vocab.target_text}</span>
                    </div>

                    <!-- Divider -->
                    <div class="w-full h-px bg-zinc-800 mb-6"></div>

                    <!-- Bottom: Native Translation -->
                    <div class="flex flex-col items-center mb-2">
                        <span class="text-xs font-bold tracking-widest text-yellow-600/80 uppercase mb-2">Translation</span>
                        <span class="text-xl font-bold text-yellow-200 text-center break-words">{vocab.native_text}</span>
                    </div>

                </div>
            {/each}
        </div>
    {/if}
</div>