<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import { settingsState } from '$lib/state/settings.svelte.ts';
    import { CURRICULUM_TIERS, getTierFromXP, XP_THRESHOLDS, type Tier, type Theme } from '$lib/curriculum';
    import toast from 'svelte-french-toast';

    let userTier = $state(1);
    let totalXp = $state(0);
    let activeThemeId = $state('greetings');
    let isLoading = $state(true);

    onMount(async () => {
        // Wait for settings to load if not already
        if (!settingsState.isLoaded) {
            await new Promise(r => setTimeout(r, 100)); // basic await, assuming layout triggers it
        }

        try {
            const curr: any = await invoke('get_curriculum', { language: settingsState.targetLanguage });
            // Recalculate tier based on XP instead of trusting DB tier if it hasn't caught up
            userTier = getTierFromXP(curr.total_xp);
            totalXp = curr.total_xp;
            activeThemeId = curr.active_theme_id;
        } catch (e) {
            console.error("Failed to load curriculum", e);
        } finally {
            isLoading = false;
        }
    });

    function isLocked(tier: number) {
        const skill = settingsState.skillLevel.toLowerCase();
        let maxAllowedTier = userTier + 1; // Default beginner behavior

        if (skill === 'intermediate') {
            maxAllowedTier = Math.max(maxAllowedTier, 3);
        } else if (skill === 'advanced') {
            maxAllowedTier = Math.max(maxAllowedTier, 5);
        } else if (skill === 'fluent') {
            return false;
        }

        return tier > maxAllowedTier;
    }

    async function setActiveTheme(themeId: string) {
        try {
            await invoke('set_active_theme', { language: settingsState.targetLanguage, themeId: themeId });
            activeThemeId = themeId;
            toast.success("Active Theme Updated!");
        } catch (e) {
            toast.error("Failed to update theme");
        }
    }

    function calculateProgress(tierLevel: number) {
        if (tierLevel < userTier) return 100;
        if (tierLevel > userTier) return 0;
        
        const currentTierThreshold = XP_THRESHOLDS[tierLevel as keyof typeof XP_THRESHOLDS] || 0;
        const nextTierThreshold = XP_THRESHOLDS[(tierLevel + 1) as keyof typeof XP_THRESHOLDS] || currentTierThreshold + 100000;
        
        const xpInCurrentTier = totalXp - currentTierThreshold;
        const xpNeeded = nextTierThreshold - currentTierThreshold;
        
        return Math.min(100, Math.max(0, (xpInCurrentTier / xpNeeded) * 100));
    }
</script>

<div class="w-full max-w-6xl mx-auto p-4 md:p-6 flex flex-col gap-6 md:gap-8 min-h-[calc(100vh-64px)] mb-20 overflow-x-hidden">
    <div>
        <h1 class="text-3xl font-bold text-yellow-200">Thematic Progression Map</h1>
        <p class="text-zinc-400 mt-2">Your journey through {settingsState.targetLanguage}. Total XP: <span class="font-bold text-yellow-200">{totalXp.toLocaleString()}</span></p>
    </div>

    {#if isLoading}
        <div class="flex-1 flex items-center justify-center">
            <div class="w-12 h-12 border-4 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
        </div>
    {:else}
        <div class="flex flex-col gap-6 md:gap-8 relative before:content-[''] before:absolute before:left-12 before:top-4 before:bottom-4 before:w-1 before:bg-zinc-800 w-full">
            {#each CURRICULUM_TIERS as tier}
                <div class="flex items-start gap-4 md:gap-8 relative z-10 w-full">
                    <!-- Node -->
                    <div class="w-24 shrink-0 flex flex-col items-center gap-2">
                        <div class="w-16 h-16 rounded-full border-4 flex items-center justify-center text-xl font-black transition-all
                            {isLocked(tier.level) ? 'bg-zinc-900 border-zinc-700 text-zinc-600' : tier.color + ' border-white/20 text-white shadow-[0_0_20px_rgba(255,255,255,0.2)]'}">
                            {tier.level}
                        </div>
                        <span class="text-xs font-bold uppercase tracking-widest {isLocked(tier.level) ? 'text-zinc-600' : 'text-zinc-400'}">{tier.cefr}</span>
                    </div>

                    <!-- Content -->
                    <div class="flex-1 min-w-0 bg-zinc-900 border {isLocked(tier.level) ? 'border-zinc-800/50 opacity-50' : 'border-zinc-700'} rounded-2xl p-4 sm:p-6 shadow-xl transition-all hover:border-yellow-200/30">
                        <div class="flex flex-col xl:flex-row justify-between items-start gap-3 mb-4">
                            <div>
                                <h2 class="text-xl font-bold {isLocked(tier.level) ? 'text-zinc-500' : 'text-zinc-100'}">{tier.name}</h2>
                                <p class="text-sm {isLocked(tier.level) ? 'text-zinc-600' : 'text-zinc-400'}">{tier.description}</p>
                            </div>
                            {#if isLocked(tier.level)}
                                <span class="bg-zinc-950 text-zinc-600 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-widest border border-zinc-800 flex items-center gap-1">
                                    <span>🔒</span> Locked
                                </span>
                            {:else if tier.level === userTier}
                                <span class="bg-yellow-500/20 text-yellow-200 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-widest border border-yellow-500/30 flex items-center gap-1">
                                    <span>📍</span> Current Tier
                                </span>
                            {/if}
                        </div>

                        {#if !isLocked(tier.level)}
                            <!-- Progress Bar -->
                            {#if tier.level === userTier}
                                <div class="w-full bg-zinc-800 rounded-full h-2.5 mb-6 overflow-hidden">
                                    <div class="bg-yellow-400 h-2.5 rounded-full transition-all duration-1000" style="width: {calculateProgress(tier.level)}%"></div>
                                </div>
                            {/if}

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                                {#each tier.themes as theme}
                                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                                    <div 
                                        class="p-4 rounded-xl border transition-all cursor-pointer {activeThemeId === theme.id ? 'bg-yellow-500/10 border-yellow-500/50 shadow-[0_0_15px_rgba(253,253,150,0.1)]' : 'bg-zinc-950 border-zinc-800 hover:border-zinc-600'}"
                                        onclick={() => setActiveTheme(theme.id)}
                                    >
                                        <div class="flex items-center justify-between mb-1">
                                            <h3 class="font-bold {activeThemeId === theme.id ? 'text-yellow-200' : 'text-zinc-200'}">{theme.name}</h3>
                                            {#if activeThemeId === theme.id}
                                                <div class="w-2 h-2 rounded-full bg-yellow-400 shadow-[0_0_8px_rgba(250,204,21,0.8)] animate-pulse"></div>
                                            {/if}
                                        </div>
                                        <p class="text-xs text-zinc-500 leading-relaxed">{theme.description}</p>
                                    </div>
                                {/each}
                            </div>

                            <div class="mt-6 pt-6 border-t border-zinc-800 flex flex-col sm:flex-row gap-3 sm:gap-4">
                                <a href="/journal" class="px-4 py-2 text-center bg-zinc-800 hover:bg-zinc-700 text-sm font-bold text-zinc-200 rounded-lg transition-colors border border-zinc-700">Practice Journal</a>
                                <a href="/conjugator" class="px-4 py-2 text-center bg-zinc-800 hover:bg-zinc-700 text-sm font-bold text-zinc-200 rounded-lg transition-colors border border-zinc-700">Drill Grammar</a>
                            </div>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
