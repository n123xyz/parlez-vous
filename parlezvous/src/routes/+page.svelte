<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { settingsState } from '$lib/state/settings.svelte.ts';
    import { ollamaState } from '$lib/state/ollama.svelte.ts';
    import { 
        BookOpen, 
        Layers, 
        Brain, 
        Calendar, 
        PenTool, 
        Code2, 
        Gamepad2, 
        Bot, 
        MapPin, 
        Eye, 
        BarChart2, 
        Settings, 
        ArrowRight, 
        Sparkles, 
        Clock, 
        CheckCircle2, 
        Flame,
        Coffee,
        Utensils,
        ShoppingBag
    } from 'lucide-svelte';

    interface UserProfile {
        skill_level: string;
        tier: number;
        active_seconds: number;
    }

    interface Curriculum {
        active_seconds: number;
    }

    let activeSeconds = $state(0);
    let userTier = $state(1);

    let activeHours = $derived(Math.floor(activeSeconds / 3600));
    let activeMinutes = $derived(Math.floor((activeSeconds % 3600) / 60));

    onMount(async () => {
        try {
            const profile = await invoke<UserProfile>('get_profile');
            if (profile) {
                userTier = profile.tier || 1;
                activeSeconds = profile.active_seconds || 0;
            }
        } catch {
            // Web fallback or if backend not running
        }

        try {
            const lang = settingsState.targetLanguage;
            if (lang) {
                const curriculum = await invoke<Curriculum>('get_curriculum', { language: lang });
                if (curriculum?.active_seconds) {
                    activeSeconds = curriculum.active_seconds;
                }
            }
        } catch {
            // Web fallback
        }
    });

    const practiceModules = [
        {
            title: 'Daily Journal',
            description: 'Reflect on your day with AI grammar corrections, mood & weather prompts, and instant vocabulary extraction.',
            href: '/journal',
            icon: BookOpen,
            badge: 'Core Practice',
            color: 'from-amber-500/20 to-yellow-500/10 border-amber-500/30 text-amber-300',
            iconColor: 'text-amber-400 bg-amber-400/10'
        },
        {
            title: 'SRS Flashcards',
            description: 'Spaced repetition decks generated automatically from your journal vocabulary to lock in long-term memory.',
            href: '/flashcards',
            icon: Layers,
            badge: 'Memory Review',
            color: 'from-emerald-500/20 to-teal-500/10 border-emerald-500/30 text-emerald-300',
            iconColor: 'text-emerald-400 bg-emerald-400/10'
        },
        {
            title: 'Verb Conjugator',
            description: 'Master irregular and regular verb tenses across past, present, and future with interactive quiz drills.',
            href: '/conjugator',
            icon: Brain,
            badge: 'Grammar Trainer',
            color: 'from-sky-500/20 to-blue-500/10 border-sky-500/30 text-sky-300',
            iconColor: 'text-sky-400 bg-sky-400/10'
        }
    ];

    const interactiveModules = [
        {
            title: '3D Avatar & Roleplay',
            description: 'Engage in immersive spoken roleplay with 3D personas: order coffee, dine at a bistro, or negotiate prices at a bazaar.',
            href: '/avatar',
            icon: Bot,
            badge: 'Personas & Roleplay',
            color: 'from-purple-500/20 to-violet-500/10 border-purple-500/30 text-purple-300',
            iconColor: 'text-purple-400 bg-purple-400/10'
        },
        {
            title: 'Visual Vision',
            description: 'Capture or inspect real-world objects to uncover their names and descriptions in your target language.',
            href: '/vision',
            icon: Eye,
            badge: 'Object Recognition',
            color: 'from-rose-500/20 to-pink-500/10 border-rose-500/30 text-rose-300',
            iconColor: 'text-rose-400 bg-rose-400/10'
        },
        {
            title: 'Speedrun Dash',
            description: 'Race against the clock in a high-intensity arcade word game designed to build rapid mental recall.',
            href: '/language-game',
            icon: Gamepad2,
            badge: 'Arcade Game',
            color: 'from-orange-500/20 to-amber-500/10 border-orange-500/30 text-orange-300',
            iconColor: 'text-orange-400 bg-orange-400/10'
        }
    ];

    const skillsModules = [
        {
            title: 'Alphabet & Canvas',
            description: 'Master stroke order and character recognition through interactive handwriting drawing recognition.',
            href: '/canvas',
            icon: PenTool,
            badge: 'Handwriting',
            color: 'from-teal-500/20 to-emerald-500/10 border-teal-500/30 text-teal-300',
            iconColor: 'text-teal-400 bg-teal-400/10'
        },
        {
            title: 'Bilingual Coding',
            description: 'Sharpen both programming and language skills with multilingual code challenges and syntax puzzles.',
            href: '/coding',
            icon: Code2,
            badge: 'Polyglot Code',
            color: 'from-indigo-500/20 to-cyan-500/10 border-indigo-500/30 text-indigo-300',
            iconColor: 'text-indigo-400 bg-indigo-400/10'
        },
        {
            title: 'World Dialect Map',
            description: 'Explore regional dialects, accent variations, and cultural nuances across countries on a global map.',
            href: '/map',
            icon: MapPin,
            badge: 'Cultural Context',
            color: 'from-lime-500/20 to-green-500/10 border-lime-500/30 text-lime-300',
            iconColor: 'text-lime-400 bg-lime-400/10'
        }
    ];

    const trackingModules = [
        {
            title: 'Study Calendar',
            description: 'Track daily study streaks, review scheduled practice sessions, and maintain continuous momentum.',
            href: '/calendar',
            icon: Calendar,
            badge: 'Habits & Streaks',
            color: 'from-yellow-500/20 to-amber-500/10 border-yellow-500/30 text-yellow-300',
            iconColor: 'text-yellow-400 bg-yellow-400/10'
        },
        {
            title: 'Learning Stats',
            description: 'Detailed metrics on your study hours, tense accuracies, and vocabulary acquisition milestones.',
            href: '/stats',
            icon: BarChart2,
            badge: 'Progress Analytics',
            color: 'from-cyan-500/20 to-blue-500/10 border-cyan-500/30 text-cyan-300',
            iconColor: 'text-cyan-400 bg-cyan-400/10'
        },
        {
            title: 'App Settings',
            description: 'Manage target language, Ollama and LiteRT AI models, voice styles, and reminder notifications.',
            href: '/settings',
            icon: Settings,
            badge: 'Configuration',
            color: 'from-zinc-500/20 to-zinc-700/10 border-zinc-500/30 text-zinc-300',
            iconColor: 'text-zinc-300 bg-zinc-700/40'
        }
    ];
</script>

<div class="min-h-full w-full bg-zinc-950 text-zinc-100 flex flex-col items-center justify-start p-4 md:p-8 lg:p-12 overflow-y-auto">
    <div class="w-full max-w-6xl flex flex-col gap-10">

        <!-- Hero Header -->
        <div class="relative overflow-hidden rounded-3xl border border-zinc-800 bg-gradient-to-br from-zinc-900 via-zinc-900/90 to-zinc-950 p-6 md:p-10 shadow-2xl">
            <!-- Subtle background decorative glow -->
            <div class="absolute -right-16 -top-16 w-80 h-80 rounded-full bg-yellow-400/10 blur-3xl pointer-events-none"></div>
            <div class="absolute -left-16 -bottom-16 w-80 h-80 rounded-full bg-amber-500/10 blur-3xl pointer-events-none"></div>

            <div class="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
                <div class="flex flex-col gap-3 max-w-2xl">
                    <div class="flex flex-wrap items-center gap-2">
                        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-yellow-400/10 text-yellow-300 border border-yellow-400/20">
                            <Sparkles class="w-3.5 h-3.5" />
                            AI-Powered Language Immersion
                        </span>
                        
                        {#if ollamaState.isHealthy}
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                                AI Engine Connected
                            </span>
                        {:else}
                            <a href="/settings" class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-zinc-800 text-zinc-400 border border-zinc-700 hover:text-yellow-200 transition-colors">
                                <span class="w-1.5 h-1.5 rounded-full bg-zinc-500"></span>
                                Local AI Offline • Configure
                            </a>
                        {/if}
                    </div>

                    <h1 class="text-3xl md:text-5xl font-extrabold tracking-tight text-zinc-100">
                        Welcome to <span class="text-yellow-200 drop-shadow-[0_0_20px_rgba(253,253,150,0.3)]">Parlez-Vous</span>
                    </h1>

                    <p class="text-zinc-400 text-base md:text-lg leading-relaxed">
                        Practice speaking, writing, and vocabulary acquisition through interactive AI immersion, spaced repetition, and real-time feedback.
                    </p>
                </div>

                <!-- Learning Focus & Quick Actions -->
                <div class="flex flex-col sm:flex-row md:flex-col gap-3 shrink-0">
                    <div class="bg-zinc-950/80 border border-zinc-800/80 rounded-2xl p-4 flex items-center justify-between gap-4 backdrop-blur-sm">
                        <div class="flex items-center gap-3">
                            <div class="w-10 h-10 rounded-xl bg-yellow-200/10 border border-yellow-200/20 flex items-center justify-center text-yellow-300 font-bold text-lg">
                                {settingsState.targetLanguage ? settingsState.targetLanguage[0] : '🌐'}
                            </div>
                            <div class="flex flex-col">
                                <span class="text-xs text-zinc-400 uppercase tracking-wider font-semibold">Target Language</span>
                                <span class="text-base font-bold text-zinc-100">{settingsState.targetLanguage || 'Language'}</span>
                            </div>
                        </div>

                        <div class="px-2.5 py-1 rounded-lg bg-zinc-800/90 text-xs font-medium text-zinc-300 border border-zinc-700">
                            {settingsState.skillLevel || 'Beginner'}
                        </div>
                    </div>

                    <div class="flex items-center gap-2">
                        <a 
                            href="/journal" 
                            class="flex-1 inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl font-bold bg-yellow-200 hover:bg-yellow-300 text-zinc-950 shadow-[0_0_15px_rgba(253,253,150,0.3)] transition-all hover:scale-[1.02] active:scale-[0.98] text-sm md:text-base"
                        >
                            <BookOpen class="w-4 h-4" />
                            <span>Daily Journal</span>
                            <ArrowRight class="w-4 h-4" />
                        </a>

                        <a 
                            href="/flashcards" 
                            class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl font-medium bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition-colors text-sm md:text-base"
                            title="Flashcards"
                        >
                            <Layers class="w-4 h-4" />
                            <span class="hidden sm:inline">Flashcards</span>
                        </a>
                    </div>
                </div>
            </div>

            <!-- Learning Quick Stats Strip -->
            <div class="mt-8 pt-6 border-t border-zinc-800/80 grid grid-cols-2 md:grid-cols-4 gap-4 text-center md:text-left">
                <div class="flex flex-col">
                    <span class="text-xs text-zinc-500 font-medium uppercase tracking-wider flex items-center justify-center md:justify-start gap-1.5">
                        <Clock class="w-3.5 h-3.5 text-yellow-300" />
                        Study Time
                    </span>
                    <span class="text-xl font-bold text-zinc-200 mt-0.5">
                        {activeHours}h {activeMinutes}m
                    </span>
                </div>

                <div class="flex flex-col">
                    <span class="text-xs text-zinc-500 font-medium uppercase tracking-wider flex items-center justify-center md:justify-start gap-1.5">
                        <Flame class="w-3.5 h-3.5 text-orange-400" />
                        Current Tier
                    </span>
                    <span class="text-xl font-bold text-zinc-200 mt-0.5">
                        Tier {userTier}
                    </span>
                </div>

                <div class="flex flex-col">
                    <span class="text-xs text-zinc-500 font-medium uppercase tracking-wider flex items-center justify-center md:justify-start gap-1.5">
                        <Bot class="w-3.5 h-3.5 text-purple-400" />
                        Active Model
                    </span>
                    <span class="text-sm font-semibold text-zinc-300 truncate mt-1" title={settingsState.activeModel}>
                        {settingsState.activeModel || 'Auto'}
                    </span>
                </div>

                <div class="flex flex-col">
                    <span class="text-xs text-zinc-500 font-medium uppercase tracking-wider flex items-center justify-center md:justify-start gap-1.5">
                        <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
                        Status
                    </span>
                    <span class="text-sm font-semibold text-zinc-300 mt-1">
                        Ready to Learn
                    </span>
                </div>
            </div>
        </div>

        <!-- Section 1: Daily Practice & Core Fluency -->
        <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl md:text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <span class="w-2 h-6 rounded-full bg-yellow-300"></span>
                        Daily Practice & Fluency
                    </h2>
                    <p class="text-zinc-400 text-sm mt-0.5">Essential core daily exercises to build habits and active retention.</p>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                {#each practiceModules as mod}
                    {@const Icon = mod.icon}
                    <a 
                        href={mod.href}
                        class="group relative overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/60 p-6 flex flex-col justify-between hover:border-zinc-700 hover:bg-zinc-800/50 hover:shadow-xl transition-all duration-200 hover:-translate-y-0.5"
                    >
                        <div class="flex flex-col gap-4">
                            <div class="flex items-center justify-between">
                                <div class="w-12 h-12 rounded-xl flex items-center justify-center {mod.iconColor}">
                                    <Icon class="w-6 h-6" />
                                </div>
                                <span class="text-xs font-semibold px-2.5 py-1 rounded-full border {mod.color}">
                                    {mod.badge}
                                </span>
                            </div>

                            <div class="flex flex-col gap-1.5">
                                <h3 class="text-lg font-bold text-zinc-100 group-hover:text-yellow-200 transition-colors flex items-center gap-1.5">
                                    {mod.title}
                                </h3>
                                <p class="text-sm text-zinc-400 leading-relaxed">
                                    {mod.description}
                                </p>
                            </div>
                        </div>

                        <div class="mt-6 pt-4 border-t border-zinc-800/80 flex items-center justify-between text-xs font-medium text-zinc-400 group-hover:text-yellow-200 transition-colors">
                            <span>Open Activity</span>
                            <ArrowRight class="w-4 h-4 transform group-hover:translate-x-1 transition-transform" />
                        </div>
                    </a>
                {/each}
            </div>
        </div>

        <!-- Section 2: AI Companions & Interactive Learning -->
        <div class="flex flex-col gap-4">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                <div>
                    <h2 class="text-xl md:text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <span class="w-2 h-6 rounded-full bg-purple-400"></span>
                        AI Companions & Gamified Immersion
                    </h2>
                    <p class="text-zinc-400 text-sm mt-0.5">Spoken conversational avatars, situational roleplays, object vision, and arcade challenges.</p>
                </div>

                <div class="flex items-center gap-2 overflow-x-auto pb-1 sm:pb-0 scrollbar-none">
                    <span class="text-xs font-semibold text-zinc-500 uppercase tracking-wider hidden md:inline">Scenarios:</span>
                    <a href="/avatar?scenario=ordering_coffee" class="px-2.5 py-1 rounded-lg bg-zinc-900/90 border border-zinc-800 hover:border-amber-500/40 text-xs font-medium text-amber-300 transition-colors flex items-center gap-1.5 shrink-0">
                        <Coffee class="w-3.5 h-3.5" />
                        <span>Café</span>
                    </a>
                    <a href="/avatar?scenario=ordering_food" class="px-2.5 py-1 rounded-lg bg-zinc-900/90 border border-zinc-800 hover:border-rose-500/40 text-xs font-medium text-rose-300 transition-colors flex items-center gap-1.5 shrink-0">
                        <Utensils class="w-3.5 h-3.5" />
                        <span>Bistro</span>
                    </a>
                    <a href="/avatar?scenario=market_negotiation" class="px-2.5 py-1 rounded-lg bg-zinc-900/90 border border-zinc-800 hover:border-emerald-500/40 text-xs font-medium text-emerald-300 transition-colors flex items-center gap-1.5 shrink-0">
                        <ShoppingBag class="w-3.5 h-3.5" />
                        <span>Market</span>
                    </a>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                {#each interactiveModules as mod}
                    {@const Icon = mod.icon}
                    <a 
                        href={mod.href}
                        class="group relative overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/60 p-6 flex flex-col justify-between hover:border-zinc-700 hover:bg-zinc-800/50 hover:shadow-xl transition-all duration-200 hover:-translate-y-0.5"
                    >
                        <div class="flex flex-col gap-4">
                            <div class="flex items-center justify-between">
                                <div class="w-12 h-12 rounded-xl flex items-center justify-center {mod.iconColor}">
                                    <Icon class="w-6 h-6" />
                                </div>
                                <span class="text-xs font-semibold px-2.5 py-1 rounded-full border {mod.color}">
                                    {mod.badge}
                                </span>
                            </div>

                            <div class="flex flex-col gap-1.5">
                                <h3 class="text-lg font-bold text-zinc-100 group-hover:text-yellow-200 transition-colors flex items-center gap-1.5">
                                    {mod.title}
                                </h3>
                                <p class="text-sm text-zinc-400 leading-relaxed">
                                    {mod.description}
                                </p>
                            </div>
                        </div>

                        <div class="mt-6 pt-4 border-t border-zinc-800/80 flex items-center justify-between text-xs font-medium text-zinc-400 group-hover:text-yellow-200 transition-colors">
                            <span>Launch Tool</span>
                            <ArrowRight class="w-4 h-4 transform group-hover:translate-x-1 transition-transform" />
                        </div>
                    </a>
                {/each}
            </div>
        </div>

        <!-- Section 3: Immersion & Specialized Skills -->
        <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl md:text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <span class="w-2 h-6 rounded-full bg-teal-400"></span>
                        Handwriting, Coding & Culture
                    </h2>
                    <p class="text-zinc-400 text-sm mt-0.5">Explore character drawing stroke order, programming challenges, and dialects.</p>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                {#each skillsModules as mod}
                    {@const Icon = mod.icon}
                    <a 
                        href={mod.href}
                        class="group relative overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/60 p-6 flex flex-col justify-between hover:border-zinc-700 hover:bg-zinc-800/50 hover:shadow-xl transition-all duration-200 hover:-translate-y-0.5"
                    >
                        <div class="flex flex-col gap-4">
                            <div class="flex items-center justify-between">
                                <div class="w-12 h-12 rounded-xl flex items-center justify-center {mod.iconColor}">
                                    <Icon class="w-6 h-6" />
                                </div>
                                <span class="text-xs font-semibold px-2.5 py-1 rounded-full border {mod.color}">
                                    {mod.badge}
                                </span>
                            </div>

                            <div class="flex flex-col gap-1.5">
                                <h3 class="text-lg font-bold text-zinc-100 group-hover:text-yellow-200 transition-colors flex items-center gap-1.5">
                                    {mod.title}
                                </h3>
                                <p class="text-sm text-zinc-400 leading-relaxed">
                                    {mod.description}
                                </p>
                            </div>
                        </div>

                        <div class="mt-6 pt-4 border-t border-zinc-800/80 flex items-center justify-between text-xs font-medium text-zinc-400 group-hover:text-yellow-200 transition-colors">
                            <span>Explore</span>
                            <ArrowRight class="w-4 h-4 transform group-hover:translate-x-1 transition-transform" />
                        </div>
                    </a>
                {/each}
            </div>
        </div>

        <!-- Section 4: Tracking & Configuration -->
        <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl md:text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <span class="w-2 h-6 rounded-full bg-cyan-400"></span>
                        Tracking & Configuration
                    </h2>
                    <p class="text-zinc-400 text-sm mt-0.5">Manage schedules, analyze statistics, and tune models and voices.</p>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                {#each trackingModules as mod}
                    {@const Icon = mod.icon}
                    <a 
                        href={mod.href}
                        class="group relative overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/60 p-6 flex flex-col justify-between hover:border-zinc-700 hover:bg-zinc-800/50 hover:shadow-xl transition-all duration-200 hover:-translate-y-0.5"
                    >
                        <div class="flex flex-col gap-4">
                            <div class="flex items-center justify-between">
                                <div class="w-12 h-12 rounded-xl flex items-center justify-center {mod.iconColor}">
                                    <Icon class="w-6 h-6" />
                                </div>
                                <span class="text-xs font-semibold px-2.5 py-1 rounded-full border {mod.color}">
                                    {mod.badge}
                                </span>
                            </div>

                            <div class="flex flex-col gap-1.5">
                                <h3 class="text-lg font-bold text-zinc-100 group-hover:text-yellow-200 transition-colors flex items-center gap-1.5">
                                    {mod.title}
                                </h3>
                                <p class="text-sm text-zinc-400 leading-relaxed">
                                    {mod.description}
                                </p>
                            </div>
                        </div>

                        <div class="mt-6 pt-4 border-t border-zinc-800/80 flex items-center justify-between text-xs font-medium text-zinc-400 group-hover:text-yellow-200 transition-colors">
                            <span>Manage</span>
                            <ArrowRight class="w-4 h-4 transform group-hover:translate-x-1 transition-transform" />
                        </div>
                    </a>
                {/each}
            </div>
        </div>

        <!-- Footer / Quick Help -->
        <div class="mt-4 mb-8 flex flex-col md:flex-row items-center justify-between gap-4 p-4 rounded-2xl border border-zinc-800/60 bg-zinc-900/30 text-xs text-zinc-500">
            <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-yellow-300/60"></span>
                <span>Tip: You can return to this Welcome page at any time by clicking the <span class="text-zinc-300 font-semibold">ParlezVous</span> logo or <span class="text-zinc-300 font-semibold">Home</span> in the header.</span>
            </div>

            <div class="flex items-center gap-4">
                <a href="/settings" class="hover:text-yellow-200 transition-colors underline-offset-4 hover:underline">Change Language</a>
                <span>•</span>
                <a href="/stats" class="hover:text-yellow-200 transition-colors underline-offset-4 hover:underline">View All Stats</a>
            </div>
        </div>

    </div>
</div>
