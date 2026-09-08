<script lang="ts">
    import { onMount } from 'svelte';
    import toast from 'svelte-french-toast';
    import { playSmartTTS, initTTSAudio } from '$lib/tts';
    import { settingsState } from '$lib/state/settings.svelte';

    let {
        title,
        subtitle,
        isLanguageGame,
        store,
        initQueue,
        popPuzzle,
        stopQueue,
    } = $props<{
        title?: string;
        subtitle?: string;
        isLanguageGame?: boolean;
        store: { isGenerating: boolean; queue: any[] };
        initQueue: () => Promise<void>;
        popPuzzle: () => Promise<any>;
        stopQueue: () => void;
    }>();

    let currentPuzzle = $state<any>(null);
    let puzzleType = $state<'keystone' | 'speedrun' | null>(null);
    let isLoading = $state(true);
    let answerInput = $state('');
    let options = $state<string[]>([]);
    
    function shuffleArray(array: string[]) {
        const newArr = [...array];
        for (let i = newArr.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [newArr[i], newArr[j]] = [newArr[j], newArr[i]];
        }
        return newArr;
    }

    async function handlePlayTTS() {
        if (!currentPuzzle) return;
        initTTSAudio();
        const textToPlay = currentPuzzle.code_with_blank || currentPuzzle.code;
        // Clean blank tokens for TTS so it sounds better
        const cleanText = textToPlay.replace('___BLANK___', 'blank');
        const langMap: Record<string, string> = {
            english: 'en', korean: 'ko', japanese: 'ja', arabic: 'ar',
            bulgarian: 'bg', czech: 'cs', danish: 'da', german: 'de',
            greek: 'el', spanish: 'es', estonian: 'et', finnish: 'fi',
            french: 'fr', hindi: 'hi', croatian: 'hr', hungarian: 'hu',
            indonesian: 'id', italian: 'it', lithuanian: 'lt', latvian: 'lv',
            dutch: 'nl', polish: 'pl', portuguese: 'pt', romanian: 'ro',
            russian: 'ru', slovak: 'sk', slovenian: 'sl', swedish: 'sv',
            turkish: 'tr', ukrainian: 'uk', vietnamese: 'vi'
        };
        const langName = currentPuzzle.language.toLowerCase();
        const lang = langMap[langName] || langName;
        await playSmartTTS(cleanText, settingsState.ttsServerUrl, undefined, lang);
    }

    async function nextPuzzle() {
        isLoading = true;
        answerInput = '';
        const rawPuzzle = await popPuzzle();
        if (rawPuzzle) {
            try {
                currentPuzzle = JSON.parse(rawPuzzle.question_data);
                puzzleType = rawPuzzle.question_type as any;
                
                if (puzzleType === 'speedrun') {
                    options = shuffleArray([currentPuzzle.correct_answer, ...currentPuzzle.distractors]);
                }
            } catch(e) {
                console.error("Invalid JSON from backend", e);
                nextPuzzle();
                return;
            }
        } else {
            currentPuzzle = null;
            puzzleType = null;
        }
        isLoading = false;
    }

    onMount(() => {
        (async () => {
            await initQueue();
            await nextPuzzle();
        })();
        
        const pollInterval = setInterval(() => {
            if (!currentPuzzle && !isLoading && store.queue.length > 0) {
                nextPuzzle();
            }
        }, 1000);
        
        return () => {
            clearInterval(pollInterval);
            stopQueue();
        };
    });

    function checkKeystone() {
        if (!currentPuzzle) return;
        const correct = currentPuzzle.exact_answer.trim().toLowerCase() === answerInput.trim().toLowerCase();
        if (correct) {
            toast.success("Correct!");
            nextPuzzle();
        } else {
            toast.error("Incorrect. Try again.");
        }
    }

    function checkSpeedrun(selected: string) {
        if (!currentPuzzle) return;
        if (selected === currentPuzzle.correct_answer) {
            toast.success("Correct!");
            nextPuzzle();
        } else {
            toast.error("Incorrect. Try again.");
        }
    }
</script>

<div class="min-h-screen bg-zinc-950 text-zinc-100 flex flex-col p-4 md:p-8">
    <div class="max-w-3xl w-full mx-auto flex-1 flex flex-col pt-12 md:pt-20">
        
        {#if title || subtitle}
        <div class="mb-12 text-center">
            {#if title}
            <h1 class="text-4xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-yellow-200 to-amber-500 mb-4">
                {title}
            </h1>
            {/if}
            {#if subtitle}
            <p class="text-zinc-400">{subtitle}</p>
            {/if}
        </div>
        {/if}

        <div class="flex-1 flex flex-col justify-center">
            {#if isLoading || (!currentPuzzle && store.isGenerating)}
                <div class="flex flex-col items-center justify-center space-y-6">
                    <div class="w-12 h-12 border-4 border-zinc-800 border-t-yellow-300 rounded-full animate-spin"></div>
                    <p class="text-zinc-400 animate-pulse">Generating your next puzzle...</p>
                </div>
            {:else if currentPuzzle}
                <div class="bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl p-6 md:p-10 transform transition-all">
                    
                    <div class="flex items-center justify-between mb-6">
                        <span class="px-3 py-1 bg-zinc-800 text-yellow-300 text-sm font-medium rounded-full uppercase tracking-wider">
                            {currentPuzzle.language}
                        </span>
                        <span class="text-zinc-500 text-sm font-medium uppercase tracking-widest">
                            {puzzleType === 'keystone' ? 'Keystone' : 'Speed Run'}
                        </span>
                    </div>

                    <div class="bg-[#1e1e1e] rounded-xl p-6 mb-8 overflow-x-auto shadow-inner border border-zinc-800/50 relative group">
                        <div class="flex gap-2 absolute top-3 left-3 opacity-30">
                            <div class="w-3 h-3 rounded-full bg-red-500"></div>
                            <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
                            <div class="w-3 h-3 rounded-full bg-green-500"></div>
                        </div>
                        {#if isLanguageGame}
                            <button 
                                onclick={handlePlayTTS}
                                class="absolute top-2 right-2 p-2 bg-zinc-800/80 hover:bg-zinc-700 text-yellow-300 rounded-lg transition-colors border border-zinc-700 hover:border-yellow-300/50 flex items-center justify-center group-hover:opacity-100 opacity-60"
                                aria-label="Play pronunciation"
                                title="Play audio"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                  <path fill-rule="evenodd" d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.707.707L4.586 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.586l3.707-3.707a1 1 0 011.09-.217zM14.657 2.929a1 1 0 011.414 0A9.972 9.972 0 0119 10a9.972 9.972 0 01-2.929 7.071 1 1 0 01-1.414-1.414A7.971 7.971 0 0017 10c0-2.21-.894-4.208-2.343-5.657a1 1 0 010-1.414zm-2.829 2.828a1 1 0 011.415 0A5.983 5.983 0 0115 10a5.984 5.984 0 01-1.757 4.243 1 1 0 01-1.415-1.415A3.984 3.984 0 0013 10a3.983 3.983 0 00-1.172-2.828 1 1 0 010-1.415z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        {/if}
                        <pre class="font-mono text-[15px] leading-relaxed text-zinc-300 mt-6 whitespace-pre-wrap"><code class="block">{puzzleType === 'keystone' ? currentPuzzle.code_with_blank : currentPuzzle.code}</code></pre>
                    </div>

                    {#if puzzleType === 'keystone'}
                        <div class="space-y-3">
                            <!-- svelte-ignore a11y_label_has_associated_control -->
                            <label class="block text-xs font-bold uppercase tracking-wider text-zinc-500 ml-1">Fill in blank</label>
                            <div class="flex gap-4">
                                <input 
                                    type="text" 
                                    bind:value={answerInput} 
                                    onkeydown={(e) => e.key === 'Enter' && checkKeystone()}
                                    class="flex-1 bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 font-mono focus:outline-none focus:border-yellow-300 transition-colors"
                                    placeholder="Answer..."
                                    autocomplete="off"
                                    spellcheck="false"
                                />
                                <button 
                                    onclick={checkKeystone}
                                    class="bg-yellow-300 hover:bg-yellow-400 text-zinc-900 font-bold px-8 py-3 rounded-xl transition-all shadow-lg hover:shadow-yellow-300/20 active:scale-95"
                                >
                                    Submit
                                </button>
                            </div>
                        </div>
                    {:else if puzzleType === 'speedrun'}
                        <div class="space-y-4">
                            <p class="text-xs font-bold uppercase tracking-wider text-zinc-500 mb-4 text-center">Select answer</p>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {#each options as opt}
                                    <button 
                                        onclick={() => checkSpeedrun(opt)}
                                        class="text-left bg-zinc-800/50 hover:bg-zinc-800 border border-zinc-700 hover:border-yellow-300/50 text-zinc-200 rounded-xl p-4 transition-all hover:shadow-lg active:scale-[0.98]"
                                    >
                                        <span class="block text-[15px] leading-snug">{opt}</span>
                                    </button>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    <div class="mt-8 flex justify-center">
                        <button 
                            onclick={nextPuzzle}
                            class="text-sm font-medium text-zinc-500 hover:text-yellow-300/80 transition-colors"
                        >
                            Skip
                        </button>
                    </div>
                </div>
            {:else}
                <div class="text-center space-y-6">
                    <p class="text-zinc-500 text-sm">No puzzles available.</p>
                    <button 
                        onclick={nextPuzzle}
                        class="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-full transition-colors"
                    >
                        Retry
                    </button>
                </div>
            {/if}
        </div>
    </div>
</div>
