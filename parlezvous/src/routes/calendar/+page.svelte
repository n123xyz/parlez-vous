<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import { playSmartTTS } from '$lib/tts';
    import { settingsState } from '$lib/state/settings.svelte.ts';

    interface JournalEntryDTO {
        id: number;
        language_code: string;
        date: string;
        mood_input: string;
        weather_input: string;
        activity_input: string;
        generated_target_text: string;
        native_translation: string;
    }

    let entries = $state<JournalEntryDTO[]>([]);
    let isLoading = $state(true);
    let expandedEntry = $state<JournalEntryDTO | null>(null);

    onMount(async () => {
        try {
            entries = (await invoke('get_journal_entries')) as JournalEntryDTO[];
        } catch (e) {
            console.error('Failed to load journal entries:', e);
        } finally {
            isLoading = false;
        }
    });
</script>

<div class="max-w-6xl mx-auto p-6 flex flex-col gap-8 min-h-[calc(100vh-64px)]">
    <div>
        <h1 class="text-3xl font-bold text-yellow-200">Journal Calendar</h1>
        <p class="text-zinc-400 mt-2">Review your past entries and track your language journey.</p>
    </div>

    {#if isLoading}
        <div class="flex-1 flex items-center justify-center">
            <div class="w-12 h-12 border-4 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
        </div>
    {:else if entries.length === 0}
        <div class="flex-1 flex items-center justify-center bg-zinc-900/20 rounded-3xl border border-dashed border-zinc-800">
            <p class="text-zinc-500">No journal entries found. Go create one!</p>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {#each entries as entry}
                <button 
                    class="bg-zinc-900 border border-zinc-800 rounded-2xl p-6 flex flex-col gap-4 shadow-xl hover:border-yellow-200/30 transition-colors text-left"
                    onclick={() => expandedEntry = entry}
                >
                    <div class="flex justify-between items-start border-b border-zinc-800 pb-4 w-full">
                        <div class="flex flex-col">
                            <span class="text-xs font-bold tracking-widest text-zinc-500 uppercase">{entry.language_code}</span>
                            <span class="text-lg font-bold text-yellow-200">{entry.date}</span>
                        </div>
                        <div class="flex flex-col items-end gap-1 text-xs text-zinc-400">
                            <span class="bg-zinc-800 px-2 py-1 rounded-md">{entry.mood_input}</span>
                            <span class="bg-zinc-800 px-2 py-1 rounded-md">{entry.weather_input}</span>
                        </div>
                    </div>
                    <div class="flex flex-col gap-3 flex-1 w-full">
                        <p class="text-zinc-100 text-sm leading-relaxed line-clamp-4">{entry.generated_target_text}</p>
                        <p class="text-zinc-500 text-xs italic line-clamp-3 mt-auto">{entry.native_translation}</p>
                    </div>
                </button>
            {/each}
        </div>
    {/if}

    {#if expandedEntry}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-sm" onclick={() => expandedEntry = null}>
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="bg-zinc-900 border border-zinc-700 rounded-3xl p-8 max-w-3xl w-full shadow-2xl relative max-h-[90vh] overflow-y-auto" onclick={(e) => e.stopPropagation()}>
                <button 
                    class="absolute top-6 right-6 text-zinc-500 hover:text-white bg-zinc-800 rounded-full w-8 h-8 flex items-center justify-center"
                    onclick={() => expandedEntry = null}
                >✕</button>

                <div class="flex flex-col mb-6 border-b border-zinc-800 pb-4">
                    <span class="text-sm font-bold tracking-widest text-zinc-500 uppercase">{expandedEntry.language_code}</span>
                    <span class="text-2xl font-bold text-yellow-200">{expandedEntry.date}</span>
                </div>

                <div class="space-y-8">
                    <div class="relative">
                        <h3 class="text-sm font-bold tracking-widest text-zinc-600 uppercase mb-3">Target Entry</h3>
                        <p class="text-zinc-100 text-lg leading-relaxed whitespace-pre-wrap">{expandedEntry.generated_target_text}</p>
                        <button 
                            class="absolute top-0 right-0 text-yellow-200 hover:text-yellow-400 bg-zinc-800 p-2 rounded-full transition-colors border border-zinc-700 hover:border-yellow-200/50"
                            onclick={() => playSmartTTS(expandedEntry!.generated_target_text, settingsState.ttsServerUrl, undefined, settingsState.targetLanguage)}
                            title="Play Audio"
                        >
                            ▶
                        </button>
                    </div>

                    <div>
                        <h3 class="text-sm font-bold tracking-widest text-zinc-600 uppercase mb-3">Native Translation</h3>
                        <p class="text-zinc-400 italic whitespace-pre-wrap">{expandedEntry.native_translation}</p>
                    </div>
                </div>
            </div>
        </div>
    {/if}
</div>
