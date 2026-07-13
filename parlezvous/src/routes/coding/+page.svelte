<script lang="ts">
    import { onMount } from 'svelte';
    import { codingState, initCodingQueue, popCodingPuzzle, stopCodingQueue } from '$lib/state/coding.svelte';
    import toast from 'svelte-french-toast';

    let currentPuzzle = $state<any>(null);
    let puzzleType = $state<'keystone' | 'speedrun' | null>(null);
    let isLoading = $state(true);
    let answerInput = $state('');
    let options = $state<string[]>([]);
    
    // For speedrun
    function shuffleArray(array: string[]) {
        const newArr = [...array];
        for (let i = newArr.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [newArr[i], newArr[j]] = [newArr[j], newArr[i]];
        }
        return newArr;
    }

    async function nextPuzzle() {
        isLoading = true;
        answerInput = '';
        const rawPuzzle = await popCodingPuzzle();
        if (rawPuzzle) {
            try {
                currentPuzzle = JSON.parse(rawPuzzle.question_data);
                puzzleType = rawPuzzle.question_type as any;
                
                if (puzzleType === 'speedrun') {
                    options = shuffleArray([currentPuzzle.correct_answer, ...currentPuzzle.distractors]);
                }
            } catch(e) {
                console.error("Invalid JSON from backend", e);
                // skip and get next
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
            await initCodingQueue();
            await nextPuzzle();
        })();
        
        // Polling if queue was empty but generating
        const pollInterval = setInterval(() => {
            if (!currentPuzzle && !isLoading && codingState.queue.length > 0) {
                nextPuzzle();
            }
        }, 1000);
        
        return () => {
            clearInterval(pollInterval);
            stopCodingQueue();
        };
    });

    function checkKeystone() {
        if (!currentPuzzle) return;
        const correct = currentPuzzle.exact_answer.trim() === answerInput.trim();
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
        
        <!-- Header -->
        <div class="mb-12 text-center">
            <h1 class="text-4xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-yellow-200 to-amber-500 mb-4">
                Parlez-Code
            </h1>
            <p class="text-zinc-400">Master programming through interactive puzzles</p>
        </div>

        <!-- Content Area -->
        <div class="flex-1 flex flex-col justify-center">
            {#if isLoading || (!currentPuzzle && codingState.isGenerating)}
                <div class="flex flex-col items-center justify-center space-y-6">
                    <div class="w-12 h-12 border-4 border-zinc-800 border-t-yellow-300 rounded-full animate-spin"></div>
                    <p class="text-zinc-400 animate-pulse">Generating your next puzzle...</p>
                </div>
            {:else if currentPuzzle}
                <div class="bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl p-6 md:p-10 transform transition-all">
                    
                    <!-- Language Badge -->
                    <div class="flex items-center justify-between mb-6">
                        <span class="px-3 py-1 bg-zinc-800 text-yellow-300 text-sm font-medium rounded-full uppercase tracking-wider">
                            {currentPuzzle.language}
                        </span>
                        <span class="text-zinc-500 text-sm font-medium uppercase tracking-widest">
                            {puzzleType === 'keystone' ? 'Keystone' : 'Speed Run'}
                        </span>
                    </div>

                    <!-- Code Block -->
                    <div class="bg-[#1e1e1e] rounded-xl p-6 mb-8 overflow-x-auto shadow-inner border border-zinc-800/50 relative group">
                        <!-- Mac window dots just for aesthetic -->
                        <div class="flex gap-2 absolute top-3 left-3 opacity-30">
                            <div class="w-3 h-3 rounded-full bg-red-500"></div>
                            <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
                            <div class="w-3 h-3 rounded-full bg-green-500"></div>
                        </div>
                        <pre class="font-mono text-[15px] leading-relaxed text-zinc-300 mt-6 whitespace-pre-wrap"><code class="block">{puzzleType === 'keystone' ? currentPuzzle.code_with_blank : currentPuzzle.code}</code></pre>
                    </div>

                    <!-- Interaction Area -->
                    {#if puzzleType === 'keystone'}
                        <div class="space-y-4">
                            <!-- svelte-ignore a11y_label_has_associated_control -->
                            <label class="block text-sm font-medium text-zinc-400 ml-1">Fill in the blank (___BLANK___):</label>
                            <div class="flex gap-4">
                                <input 
                                    type="text" 
                                    bind:value={answerInput} 
                                    onkeydown={(e) => e.key === 'Enter' && checkKeystone()}
                                    class="flex-1 bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 font-mono focus:outline-none focus:border-yellow-300 transition-colors"
                                    placeholder="Type your code here..."
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
                            <p class="text-sm font-medium text-zinc-400 mb-4 text-center">What does this code do?</p>
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
                            Skip this puzzle
                        </button>
                    </div>
                </div>
            {:else}
                <div class="text-center space-y-6">
                    <p class="text-zinc-500">No puzzles available. Please check your LLM settings.</p>
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
