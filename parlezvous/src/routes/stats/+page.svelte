<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import { settingsState } from '$lib/state/settings.svelte.ts';

    interface TenseStat {
        tense: string;
        total: number;
        correct: number;
    }

    interface Curriculum {
        active_seconds: number;
    }

    let tenseStats = $state<TenseStat[]>([]);
    let activeSeconds = $state(0);
    let isLoading = $state(true);

    let activeHours = $derived(Math.floor(activeSeconds / 3600));
    let activeMinutes = $derived(Math.floor((activeSeconds % 3600) / 60));

    onMount(async () => {
        if (!settingsState.isLoaded) {
            await new Promise(r => setTimeout(r, 500));
        }
        
        try {
            const lang = settingsState.targetLanguage;
            
            const curriculum = await invoke<Curriculum>('get_curriculum', { language: lang });
            activeSeconds = curriculum.active_seconds || 0;

            tenseStats = await invoke<TenseStat[]>('get_all_tense_stats', { language: lang });
        } catch (e) {
            console.error("Failed to load stats:", e);
        } finally {
            isLoading = false;
        }
    });
</script>

<div class="h-full w-full flex flex-col items-center justify-start p-4 md:p-8 overflow-y-auto">
    <div class="w-full max-w-4xl flex flex-col gap-8">
        <div class="flex flex-col gap-2">
            <h1 class="text-3xl md:text-5xl font-extrabold tracking-tight text-white drop-shadow-md">
                Your <span class="text-yellow-400 bg-clip-text text-transparent bg-gradient-to-r from-yellow-300 to-yellow-500">Stats</span>
            </h1>
            <p class="text-zinc-400 text-sm md:text-base">
                Track your progress in {settingsState.targetLanguage}.
            </p>
        </div>

        {#if isLoading}
            <div class="flex items-center justify-center py-20">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-yellow-200"></div>
            </div>
        {:else}
            <!-- Time Spent Card -->
            <div class="bg-zinc-900 border border-zinc-800 rounded-2xl p-6 shadow-xl flex items-center gap-6">
                <div class="w-16 h-16 rounded-full bg-yellow-400/20 flex items-center justify-center text-yellow-400 text-3xl">
                    ⏱️
                </div>
                <div class="flex flex-col">
                    <span class="text-zinc-400 text-sm font-medium uppercase tracking-wider">Total Time Learning</span>
                    <div class="flex items-baseline gap-2">
                        <span class="text-4xl font-bold text-zinc-100">{activeHours}</span>
                        <span class="text-zinc-500 font-medium">hrs</span>
                        <span class="text-4xl font-bold text-zinc-100">{activeMinutes}</span>
                        <span class="text-zinc-500 font-medium">mins</span>
                    </div>
                </div>
            </div>

            <!-- Tense Accuracy Section -->
            <div class="flex flex-col gap-4">
                <h2 class="text-xl font-bold text-zinc-100">Tense Accuracy</h2>
                
                {#if tenseStats.length === 0}
                    <div class="bg-zinc-900 border border-zinc-800 rounded-xl p-8 text-center text-zinc-500">
                        No conjugation exercises completed yet.
                    </div>
                {:else}
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {#each tenseStats as stat}
                            {@const pct = stat.total > 0 ? Math.round((stat.correct / stat.total) * 100) : 0}
                            <div class="bg-zinc-900 border border-zinc-800 rounded-xl p-5 shadow-lg flex flex-col gap-3">
                                <div class="flex items-center justify-between">
                                    <h3 class="text-lg font-bold text-zinc-200 capitalize">{stat.tense}</h3>
                                    <span class="text-sm font-bold {pct >= 80 ? 'text-green-400' : pct >= 50 ? 'text-yellow-400' : 'text-red-400'}">
                                        {pct}%
                                    </span>
                                </div>
                                <div class="w-full bg-zinc-800 rounded-full h-2.5 overflow-hidden">
                                    <div class="h-2.5 rounded-full transition-all duration-500 {pct >= 80 ? 'bg-green-400' : pct >= 50 ? 'bg-yellow-400' : 'bg-red-400'}" style="width: {pct}%"></div>
                                </div>
                                <div class="text-xs text-zinc-500 font-medium text-right mt-1">
                                    {stat.correct} / {stat.total} correct
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>
