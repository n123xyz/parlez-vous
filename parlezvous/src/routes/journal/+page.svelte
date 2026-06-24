<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
    import { onMount, onDestroy } from 'svelte';
    import { ollamaState } from '$lib/state/ollama.svelte.ts';
    import { settingsState } from '$lib/state/settings.svelte.ts';
    import toast from 'svelte-french-toast';
    import { playSmartTTS } from '$lib/tts';
    import { marked } from 'marked';
    import { timeTracker } from '$lib/state/timeTracker.svelte.ts';

    let selectedMoods = $state(['Happy']);
    let selectedWeathers = $state(['Sunny']);
    let selectedActivities = $state(['Studying']);
    
    let isCustomMode = $state(false);
    let customEntry = $state('');
    
    let isGenerating = $state(false);

    function toggleSelection(currentSelection: string[], item: string) {
        if (currentSelection.includes(item)) {
            return currentSelection.filter(i => i !== item);
        } else {
            return [...currentSelection, item];
        }
    }

    const moods = ['Happy', 'Tired', 'Excited', 'Sad', 'Calm'];
    const weathers = ['Sunny', 'Raining', 'Cloudy', 'Snowing', 'Windy'];
    const activities = ['Studying', 'Working', 'Relaxing', 'Traveling', 'Exercising'];

    let journalResult = $state<{ generated_target_text: string; native_translation: string; feedback?: string } | null>(null);
    let vocabChips = $state<any[]>([]);
    let unlisten: () => void;
    
    let activeThemeId = $state<string | null>(null);

    onMount(async () => {
        unlisten = await listen('vocabulary_extracted', (event) => {
            vocabChips = event.payload as any[];
        });

        try {
            const curr: any = await invoke('get_curriculum', { language: settingsState.targetLanguage });
            activeThemeId = curr.active_theme_id;
        } catch (e) {
            console.error("No curriculum found", e);
        }

        timeTracker.startTracking();
    });

    onDestroy(() => {
        timeTracker.flushTime();
        timeTracker.stopTracking();
        if (unlisten) unlisten();
    });

    async function generateJournal() {
        if (!settingsState.activeModel) {
            toast.error('Please select an AI model in settings (or wait for it to load).');
            return;
        }
        
        isGenerating = true;
        journalResult = null;
        vocabChips = [];

        try {
            const payload = {
                mood: selectedMoods.length ? selectedMoods.join(', ') : 'Neutral',
                weather: selectedWeathers.length ? selectedWeathers.join(', ') : 'Neutral',
                activity: selectedActivities.length ? selectedActivities.join(', ') : 'Neutral',
                model: settingsState.activeModel,
                language: settingsState.targetLanguage,
                activeTheme: activeThemeId
            };

            const result = await invoke('generate_journal', payload);
            journalResult = result as any;
        } catch (e) {
            console.error('Failed to generate journal:', e);
            toast.error('Failed to generate journal: ' + e);
        } finally {
            isGenerating = false;
        }
    }

    async function gradeJournal() {
        if (!settingsState.activeModel) {
            toast.error('Please select an AI model in settings (or wait for it to load).');
            return;
        }
        
        isGenerating = true;
        journalResult = null;
        vocabChips = [];

        try {
            const payload = {
                entry: customEntry,
                model: settingsState.activeModel,
                language: settingsState.targetLanguage,
            };

            const result = await invoke('grade_journal', payload);
            journalResult = result as any;
        } catch (e) {
            console.error('Failed to grade journal:', e);
            toast.error('Failed to grade journal: ' + e);
        } finally {
            isGenerating = false;
        }
    }

    async function addToSrs(vocabId: number) {
        try {
            await invoke('add_to_srs', { vocabId });
            // Remove the chip from UI to indicate success
            vocabChips = vocabChips.filter(chip => chip.id !== vocabId);
            toast.success('Added to SRS!');
        } catch (e) {
            console.error('Failed to add to SRS:', e);
            toast.error('Failed to add to SRS: ' + e);
        }
    }
</script>

<div class="max-w-6xl mx-auto p-6 flex flex-col md:flex-row gap-8">
    <!-- Left Column: Controls -->
    <div class="w-full md:w-1/3 space-y-6 bg-zinc-900 p-6 rounded-2xl border border-zinc-800 shadow-xl flex flex-col">
        <div>
            <h2 class="text-2xl font-bold text-yellow-200">Journal Builder</h2>
            <p class="text-sm text-zinc-400 mt-1">Targeting {settingsState.skillLevel} level in {settingsState.targetLanguage}</p>
        </div>
        
        <div class="flex gap-2 p-1 bg-zinc-950 rounded-xl border border-zinc-800">
            <button 
                class="flex-1 py-2 text-sm font-bold rounded-lg transition-colors {!isCustomMode ? 'bg-zinc-800 text-yellow-200' : 'text-zinc-500 hover:text-zinc-300'}"
                onclick={() => isCustomMode = false}
            >
                Scaffold
            </button>
            <button 
                class="flex-1 py-2 text-sm font-bold rounded-lg transition-colors {isCustomMode ? 'bg-zinc-800 text-yellow-200' : 'text-zinc-500 hover:text-zinc-300'}"
                onclick={() => isCustomMode = true}
            >
                Custom Entry
            </button>
        </div>

        {#if isCustomMode}
            <div class="flex-1 flex flex-col min-h-[300px]">
                <label for="customEntry" class="text-sm font-medium text-zinc-300 mb-2">Write your entry</label>
                <textarea 
                    id="customEntry"
                    bind:value={customEntry}
                    placeholder="Write a journal entry in your target language. The AI will judge, correct, and extract vocabulary from it."
                    class="flex-1 w-full bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl p-4 focus:outline-none focus:border-yellow-200 transition-colors resize-none"
                ></textarea>
            </div>
        {:else}
            <div class="space-y-6 flex-1">
                <div>
                    <h3 class="text-sm font-medium text-zinc-300 mb-2 uppercase tracking-widest">Moods</h3>
                    <div class="flex flex-wrap gap-2">
                        {#each moods as m}
                            <button 
                                class="px-4 py-2 rounded-full text-sm font-medium transition-colors {selectedMoods.includes(m) ? 'bg-yellow-200 text-zinc-900 shadow-[0_0_10px_rgba(253,253,150,0.3)]' : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                                onclick={() => selectedMoods = toggleSelection(selectedMoods, m)}
                            >
                                {m}
                            </button>
                        {/each}
                    </div>
                </div>

                <div>
                    <h3 class="text-sm font-medium text-zinc-300 mb-2 uppercase tracking-widest">Weather</h3>
                    <div class="flex flex-wrap gap-2">
                        {#each weathers as w}
                            <button 
                                class="px-4 py-2 rounded-full text-sm font-medium transition-colors {selectedWeathers.includes(w) ? 'bg-yellow-200 text-zinc-900 shadow-[0_0_10px_rgba(253,253,150,0.3)]' : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                                onclick={() => selectedWeathers = toggleSelection(selectedWeathers, w)}
                            >
                                {w}
                            </button>
                        {/each}
                    </div>
                </div>

                <div>
                    <h3 class="text-sm font-medium text-zinc-300 mb-2 uppercase tracking-widest">Activities</h3>
                    <div class="flex flex-wrap gap-2">
                        {#each activities as a}
                            <button 
                                class="px-4 py-2 rounded-full text-sm font-medium transition-colors {selectedActivities.includes(a) ? 'bg-yellow-200 text-zinc-900 shadow-[0_0_10px_rgba(253,253,150,0.3)]' : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                                onclick={() => selectedActivities = toggleSelection(selectedActivities, a)}
                            >
                                {a}
                            </button>
                        {/each}
                    </div>
                </div>
            </div>
        {/if}

        <button 
            class="w-full mt-4 bg-yellow-200 hover:bg-yellow-300 disabled:opacity-50 text-zinc-900 font-bold py-3 px-6 rounded-xl transition-all shadow-[0_0_15px_rgba(253,253,150,0.2)]"
            onclick={isCustomMode ? gradeJournal : generateJournal}
            disabled={isGenerating || (isCustomMode && !customEntry.trim()) || (!isCustomMode && selectedMoods.length === 0 && selectedWeathers.length === 0 && selectedActivities.length === 0)}
        >
            {isGenerating ? 'Processing...' : (isCustomMode ? 'Submit for Grading' : 'Generate Entry')}
        </button>
    </div>

    <!-- Right Column: Results -->
    <div class="w-full md:w-2/3 flex flex-col gap-6">
        {#if isGenerating}
            <div class="flex-1 flex items-center justify-center bg-zinc-900/50 rounded-2xl border border-dashed border-zinc-700">
                <div class="animate-pulse flex flex-col items-center">
                    <div class="w-12 h-12 border-4 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
                    <p class="mt-4 text-zinc-400 font-medium">Consulting the model...</p>
                </div>
            </div>
        {:else if journalResult}
            <div class="bg-zinc-900 p-8 rounded-2xl border border-zinc-800 shadow-xl prose prose-invert max-w-none relative">
                <h3 class="text-xl font-medium text-yellow-200 mb-4 border-b border-zinc-800 pb-2">Target Language</h3>
                <button 
                    class="absolute top-8 right-8 text-yellow-200 hover:text-yellow-400 bg-zinc-800 p-2 rounded-full transition-colors border border-zinc-700 hover:border-yellow-200/50"
                    onclick={() => playSmartTTS(journalResult!.generated_target_text, settingsState.ttsServerUrl, undefined, settingsState.targetLanguage)}
                    title="Play Audio"
                >
                    ▶
                </button>
                <p class="text-lg text-zinc-100 leading-relaxed pr-10">{journalResult.generated_target_text}</p>
                
                <h3 class="text-xl font-medium text-zinc-400 mt-8 mb-4 border-b border-zinc-800 pb-2">Native Translation</h3>
                <p class="text-zinc-300 italic">{journalResult.native_translation}</p>

                {#if isCustomMode && journalResult.feedback}
                    <div class="mt-8 p-6 bg-blue-500/10 border border-blue-500/20 rounded-2xl">
                        <h3 class="text-sm font-bold text-blue-400 uppercase tracking-widest mb-3 flex items-center gap-2">
                            <span class="text-lg">👨‍🏫</span> Teacher's Feedback
                        </h3>
                        <div class="prose prose-invert prose-sm max-w-none prose-p:my-1 prose-headings:my-2 prose-a:text-yellow-200 text-zinc-200 leading-relaxed italic">
                            {@html marked.parse(journalResult.feedback)}
                        </div>
                    </div>
                {/if}
            </div>

            {#if vocabChips.length > 0}
                <div class="bg-zinc-900/50 p-6 rounded-2xl border border-zinc-800">
                    <h3 class="text-sm uppercase tracking-widest text-zinc-500 mb-4">Extracted Vocabulary (Click to add to SRS)</h3>
                    <div class="flex flex-wrap gap-3">
                        {#each vocabChips as chip}
                            <div class="group flex items-center bg-zinc-800 border border-zinc-700 hover:border-yellow-200/50 rounded-lg transition-all overflow-hidden">
                                <button 
                                    class="px-4 py-2 hover:bg-zinc-700 flex items-center gap-2"
                                    onclick={() => addToSrs(chip.id)}
                                    title="Add to SRS"
                                >
                                    <span class="font-bold text-yellow-200">{chip.target_text}</span>
                                    <span class="text-zinc-500 group-hover:text-zinc-300 transition-colors">— {chip.native_text}</span>
                                </button>
                                <button 
                                    class="px-3 py-2 bg-zinc-700/50 hover:bg-zinc-600 text-yellow-200/70 hover:text-yellow-200 transition-colors border-l border-zinc-700/50"
                                    onclick={() => playSmartTTS(chip.target_text, settingsState.ttsServerUrl, undefined, settingsState.targetLanguage)}
                                    title="Play Audio"
                                >
                                    🔊
                                </button>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        {:else}
            <div class="flex-1 flex items-center justify-center bg-zinc-900/20 rounded-2xl border border-dashed border-zinc-800">
                <p class="text-zinc-500">Select your parameters and generate to see the result here.</p>
            </div>
        {/if}
    </div>
</div>
