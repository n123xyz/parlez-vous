<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { ollamaState } from '$lib/state/ollama.svelte.ts';
    import { settingsState } from '$lib/state/settings.svelte.ts';
    import toast from 'svelte-french-toast';
    import { onMount, onDestroy, tick } from 'svelte';
    import { timeTracker } from '$lib/state/timeTracker.svelte.ts';

    interface ConjugationExercise {
        subject: string;
        tense: string;
        sentence: string;
        verb: string;
        answer: string;
        translation: string;
    }

    interface ConjugationResponse {
        exercise: ConjugationExercise;
        history_id: number;
    }

    interface HistoryEntry {
        exercise: ConjugationExercise;
        userAnswer: string;
        correct: boolean;
    }

    // --- Core state ---
    let isGenerating = $state(false);
    let exercise = $state<ConjugationExercise | null>(null);
    let historyId = $state<number | null>(null);
    let userAnswer = $state('');
    let hasSubmitted = $state(false);
    let isCorrect = $state(false);

    let displaySentence = $derived.by(() => {
        if (!exercise) return "";
        const escapeRegExp = (str: string) => str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        
        // 1. Try exact substring match first
        const exactRegex = new RegExp(escapeRegExp(exercise.answer), 'i');
        if (exactRegex.test(exercise.sentence)) {
            return exercise.sentence.replace(exactRegex, "___");
        }

        // 2. Try splitting words and matching with gaps (e.g., "a beaucoup réfléchi")
        const parts = exercise.answer.trim().split(/\s+/);
        if (parts.length > 1) {
            const boundLeft = `(^|[\\s.,!?;:'])`;
            const boundRight = `(?=[\\s.,!?;:']|$)`;
            
            let pattern = "";
            for (let i = 0; i < parts.length; i++) {
                pattern += boundLeft + escapeRegExp(parts[i]) + boundRight;
                if (i < parts.length - 1) {
                    pattern += `(.*?)`; // Capture anything in between (like adverbs)
                }
            }
            
            try {
                const seqRegex = new RegExp(pattern, 'i');
                const match = exercise.sentence.match(seqRegex);
                if (match) {
                    let result = "";
                    let groupIdx = 1;
                    for (let i = 0; i < parts.length; i++) {
                        const left = match[groupIdx++];
                        result += left + "___";
                        if (i < parts.length - 1) {
                            const gap = match[groupIdx++];
                            result += gap;
                        }
                    }
                    return exercise.sentence.substring(0, match.index) + result + exercise.sentence.substring(match.index! + match[0].length);
                }
            } catch (e) {
                console.error(e);
            }
        }

        // 3. Fallback: fuzzy match on substrings (handles phrases, reflexive verbs, and typos)
        const editDistance = (a: string, b: string) => {
            if (a.length === 0) return b.length;
            if (b.length === 0) return a.length;
            const matrix = Array(a.length + 1).fill(null).map(() => Array(b.length + 1).fill(null));
            for (let i = 0; i <= a.length; i++) matrix[i][0] = i;
            for (let j = 0; j <= b.length; j++) matrix[0][j] = j;
            for (let i = 1; i <= a.length; i++) {
                for (let j = 1; j <= b.length; j++) {
                    const indicator = a[i - 1] === b[j - 1] ? 0 : 1;
                    matrix[i][j] = Math.min(
                        matrix[i][j - 1] + 1,
                        matrix[i - 1][j] + 1,
                        matrix[i - 1][j - 1] + indicator
                    );
                }
            }
            return matrix[a.length][b.length];
        };

        const tokens = [];
        let currentIndex = 0;
        const splitRegex = /([ \t\n.,!?;:'"()]+)/;
        const partsList = exercise.sentence.split(splitRegex);
        
        for (const p of partsList) {
            if (p.length > 0) {
                tokens.push({ text: p, start: currentIndex, end: currentIndex + p.length });
                currentIndex += p.length;
            }
        }

        const target = exercise.answer.trim().toLowerCase();
        const maxAllowedDist = Math.max(1, Math.floor(target.length * 0.3));
        let bestDist = 999;
        let bestStart = -1;
        let bestEnd = -1;

        for (let startIdx = 0; startIdx < tokens.length; startIdx++) {
            if (!tokens[startIdx].text.match(/[a-zàâçéèêëîïôûùüÿñæœ]/i)) continue;
            
            for (let endIdx = startIdx; endIdx < tokens.length; endIdx++) {
                if (!tokens[endIdx].text.match(/[a-zàâçéèêëîïôûùüÿñæœ]/i)) continue;
                
                const subStart = tokens[startIdx].start;
                const subEnd = tokens[endIdx].end;
                const subText = exercise.sentence.substring(subStart, subEnd).trim().toLowerCase();
                
                // Fast prune: length difference must be reasonable
                if (Math.abs(subText.length - target.length) <= Math.max(3, target.length * 0.5)) {
                    const dist = editDistance(subText, target);
                    if (dist <= maxAllowedDist && dist < bestDist) {
                        bestDist = dist;
                        bestStart = subStart;
                        bestEnd = subEnd;
                    }
                }
            }
        }

        if (bestStart !== -1) {
            return exercise.sentence.substring(0, bestStart) + "___" + exercise.sentence.substring(bestEnd);
        }

        // 4. Final Fallback: replace the infinitive verb if we absolutely cannot find the answer
        const verbRegex = new RegExp(escapeRegExp(exercise.verb), 'i');
        return exercise.sentence.replace(verbRegex, "___");
    });

    // --- History & scoring ---
    let history = $state<HistoryEntry[]>([]);
    let recentScore = $derived(() => {
        const last10 = history.slice(-10);
        return last10.filter(h => h.correct).length;
    });

    // --- Timer ---
    let timerSeconds = $state(0);
    let timerMax = $state(120);   // initial default 2 min
    let timerInterval: ReturnType<typeof setInterval> | null = null;
    let timerActive = $state(false);

    // Input ref for auto-focus
    let inputEl: HTMLInputElement | undefined = $state(undefined);
    let pageEl: HTMLDivElement | undefined = $state(undefined);

    /**
     * Compute the timer duration (in seconds) based on last-10 score.
     * Score 0  → 180s (3 min)
     * Score 10 → 30s
     * Linear interpolation between.
     */
    function computeTimerDuration(score: number): number {
        const MAX_TIME = 180;
        const MIN_TIME = 30;
        const clamped = Math.max(0, Math.min(10, score));
        return Math.round(MAX_TIME - ((MAX_TIME - MIN_TIME) * clamped) / 10);
    }

    function startTimer() {
        stopTimer();
        timerMax = computeTimerDuration(recentScore());
        timerSeconds = timerMax;
        timerActive = true;
        timerInterval = setInterval(() => {
            timerSeconds--;
            if (timerSeconds <= 0) {
                // Timer expired — auto-submit if not yet done, then generate next
                if (!hasSubmitted && exercise) {
                    checkAnswer();
                }
                generateExercise();
            }
        }, 1000);
    }

    function stopTimer() {
        if (timerInterval) {
            clearInterval(timerInterval);
            timerInterval = null;
        }
        timerActive = false;
    }

    async function generateExercise() {
        if (!settingsState.activeModel) {
            toast.error('Please select an active model in settings.');
            return;
        }

        stopTimer();
        isGenerating = true;
        exercise = null;
        historyId = null;
        userAnswer = '';
        hasSubmitted = false;

        try {
            const result = (await invoke('generate_conjugation_exercise', {
                language: settingsState.targetLanguage,
                model: settingsState.activeModel
            })) as ConjugationResponse;
            exercise = result.exercise;
            historyId = result.history_id;
        } catch (e) {
            console.error('Failed to generate exercise:', e);
            toast.error('Failed to generate exercise: ' + e);
        } finally {
            isGenerating = false;
        }

        // Focus the input after the DOM updates (must be after isGenerating = false so the input is rendered)
        if (exercise) {
            await tick();
            inputEl?.focus();
            startTimer();
        }
    }

    function checkAnswer() {
        if (!exercise || hasSubmitted) return;
        hasSubmitted = true;
        isCorrect = userAnswer.trim().toLowerCase() === exercise.answer.trim().toLowerCase();

        // Record to history (keep last 10)
        history = [
            ...history,
            { exercise, userAnswer: userAnswer.trim(), correct: isCorrect }
        ].slice(-10);

        // Persist the result in the DB so tense stats stay up to date
        if (historyId !== null) {
            invoke('record_conjugation_result', { historyId, correct: isCorrect })
                .catch(e => console.error('Failed to record result:', e));
        }

        // Move focus to the outer div so page-level Enter handler is immediately active
        tick().then(() => pageEl?.focus());
    }

    /** Input-level: submit answer on Enter (guards blank) */
    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !hasSubmitted && userAnswer.trim()) {
            e.preventDefault();
            checkAnswer();
        }
    }

    /** Page-level: advance to next exercise on Enter after submission */
    function handlePageKeydown(e: KeyboardEvent) {
        // Only act when the input is NOT focused (i.e. post-submission / disabled state)
        if (e.key === 'Enter' && hasSubmitted && document.activeElement !== inputEl) {
            e.preventDefault();
            generateExercise();
        }
    }

    // Timer progress (0 → 1)
    let timerProgress = $derived(timerMax > 0 ? timerSeconds / timerMax : 0);
    let timerColor = $derived(() => {
        if (timerProgress > 0.5) return '#facc15';   // yellow
        if (timerProgress > 0.2) return '#f97316';   // orange
        return '#ef4444';                              // red
    });

    // Format mm:ss
    function fmtTime(s: number): string {
        const m = Math.floor(s / 60);
        const sec = s % 60;
        return `${m}:${sec.toString().padStart(2, '0')}`;
    }

    onMount(() => {
        timeTracker.startTracking();
    });

    onDestroy(() => {
        invoke('cancel_conjugation_generation').catch(e => console.error("Failed to cancel generation: ", e));
        timeTracker.flushTime();
        timeTracker.stopTracking();
    });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="w-full max-w-5xl mx-auto p-4 md:p-6 flex flex-col gap-6 md:gap-8 min-h-[calc(100vh-64px)] overflow-x-hidden" role="application" tabindex="0" onkeydown={handlePageKeydown} bind:this={pageEl}>

    <!-- Header row -->
    <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
            <h1 class="text-3xl font-bold text-yellow-200">Verb Conjugator</h1>
            <p class="text-zinc-400 mt-2">Practice dynamic conjugations in {settingsState.targetLanguage}.</p>
        </div>
        <div class="flex items-center gap-4">
            <!-- Score badge -->
            {#if history.length > 0}
                <div class="flex items-center gap-2 bg-zinc-900 border border-zinc-800 rounded-2xl px-5 py-2.5">
                    <span class="text-xs font-bold tracking-widest text-zinc-500 uppercase">Last 10</span>
                    <span class="text-2xl font-black tabular-nums"
                        class:text-green-400={recentScore() >= 7}
                        class:text-yellow-200={recentScore() >= 4 && recentScore() < 7}
                        class:text-red-400={recentScore() < 4}
                    >{recentScore()}</span>
                    <span class="text-zinc-500 text-sm">/10</span>
                </div>
            {/if}

            <button
                id="new-exercise-btn"
                class="bg-yellow-200 hover:bg-yellow-300 disabled:opacity-50 text-zinc-900 font-bold py-3 px-6 rounded-xl transition-all shadow-[0_0_15px_rgba(253,253,150,0.2)] active:scale-95"
                onclick={generateExercise}
                disabled={isGenerating}
            >
                {isGenerating ? 'Generating...' : 'New Exercise'}
            </button>
        </div>
    </div>

    <!-- Main content area: exercise + sidebar history -->
    <div class="flex flex-col lg:flex-row gap-6 flex-1">

        <!-- Exercise card -->
        <div class="flex-1 flex flex-col">
            {#if isGenerating}
                <div class="flex-1 flex items-center justify-center bg-zinc-900/50 rounded-3xl border border-dashed border-zinc-700 min-h-[400px]">
                    <div class="animate-pulse flex flex-col items-center">
                        <div class="w-16 h-16 border-4 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
                        <p class="mt-4 text-zinc-400 font-bold tracking-widest uppercase text-sm">Consulting Grammar...</p>
                    </div>
                </div>
            {:else if exercise}
                <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-5 sm:p-8 md:p-12 shadow-2xl relative overflow-hidden flex-1 w-full max-w-full">
                    <div class="absolute top-0 right-0 w-32 h-32 bg-yellow-200/5 rounded-bl-[100px] -z-10"></div>

                    <!-- Timer bar -->
                    {#if timerActive}
                        <div class="mb-6 flex items-center gap-3">
                            <div class="flex-1 h-2 bg-zinc-800 rounded-full overflow-hidden">
                                <div
                                    class="h-full rounded-full transition-all duration-1000 ease-linear"
                                    style="width: {timerProgress * 100}%; background: {timerColor()}"
                                ></div>
                            </div>
                            <span class="text-sm font-mono font-bold tabular-nums min-w-[50px] text-right"
                                style="color: {timerColor()}"
                            >{fmtTime(timerSeconds)}</span>
                        </div>
                    {/if}

                    <div class="flex flex-col gap-8 items-center text-center">

                        <div class="space-y-2">
                            <span class="text-xs font-bold tracking-widest text-zinc-500 uppercase">Verb to conjugate</span>
                            <h2 class="text-4xl md:text-5xl font-bold text-zinc-100">{exercise.verb}</h2>
                            <p class="text-yellow-200/80 font-medium italic">({exercise.translation})</p>
                        </div>
                        
                        <div class="text-xl md:text-2xl font-medium text-zinc-300 break-words w-full">
                            {displaySentence}
                        </div>

                        <div class="flex flex-wrap justify-center gap-3 md:gap-4 w-full">
                            <div class="bg-zinc-950 border border-zinc-800 px-4 md:px-6 py-3 rounded-2xl flex flex-col items-center flex-1 min-w-[100px] max-w-[160px]">
                                <span class="text-zinc-500 text-[10px] md:text-xs font-bold uppercase tracking-widest mb-1">Tense</span>
                                <span class="text-yellow-200 font-bold text-sm md:text-base text-center break-words">{exercise.tense}</span>
                            </div>
                            <div class="bg-zinc-950 border border-zinc-800 px-4 md:px-6 py-3 rounded-2xl flex flex-col items-center flex-1 min-w-[100px] max-w-[160px]">
                                <span class="text-zinc-500 text-[10px] md:text-xs font-bold uppercase tracking-widest mb-1">Subject</span>
                                <span class="text-yellow-200 font-bold text-sm md:text-base text-center break-words">{exercise.subject}</span>
                            </div>
                        </div>

                        <div class="w-full max-w-md mt-6 space-y-4">
                            <input
                                id="conjugation-input"
                                type="text"
                                autocomplete="new-password"
                                autocorrect="off"
                                autocapitalize="off"
                                spellcheck="false"
                                data-form-type="other"
                                data-lpignore="true"
                                bind:value={userAnswer}
                                bind:this={inputEl}
                                onkeydown={handleKeydown}
                                placeholder="Type the conjugated form..."
                                disabled={hasSubmitted}
                                class="w-full bg-zinc-950 border border-zinc-800 text-zinc-100 rounded-2xl px-6 py-4 focus:outline-none focus:border-yellow-200/50 transition-colors text-lg text-center"
                            >

                            {#if !hasSubmitted}
                                <button
                                    id="submit-answer-btn"
                                    class="w-full bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-100 font-bold py-4 px-6 rounded-2xl transition-colors"
                                    onclick={checkAnswer}
                                    disabled={!userAnswer.trim()}
                                >
                                    Submit Answer
                                </button>
                            {:else}
                                {#if isCorrect}
                                    <div class="w-full bg-green-500/10 border border-green-500/30 text-green-500 font-bold py-4 px-6 rounded-2xl text-center flex flex-col gap-1 animate-fadeIn">
                                        <span>✓ Excellent!</span>
                                        <span class="text-sm font-normal text-green-500/80">Added to your Vocabulary Flashcards.</span>
                                    </div>
                                {:else}
                                    <div class="w-full bg-red-500/10 border border-red-500/30 text-red-400 font-bold py-4 px-6 rounded-2xl text-center flex flex-col gap-2 animate-fadeIn">
                                        <span>✗ Not quite right.</span>
                                        <div class="text-sm font-normal flex flex-col">
                                            <span class="text-zinc-400">Correct answer:</span>
                                            <span class="text-lg md:text-xl text-yellow-200 break-words">{exercise.answer}</span>
                                        </div>
                                    </div>

                                {/if}
                            {/if}
                        </div>

                    </div>
                </div>
            {:else}
                <div class="flex-1 flex items-center justify-center bg-zinc-900/20 rounded-3xl border border-dashed border-zinc-800 min-h-[400px]">
                    <p class="text-zinc-500 flex flex-col items-center gap-2">
                        <span class="text-4xl opacity-50">✨</span>
                        <span>Click "New Exercise" to generate a random conjugation challenge.</span>
                    </p>
                </div>
            {/if}
        </div>

        <!-- History sidebar -->
        {#if history.length > 0}
            <div class="lg:w-72 shrink-0">
                <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-5 sticky top-6">
                    <h3 class="text-xs font-bold tracking-widest text-zinc-500 uppercase mb-4">Recent History</h3>

                    <div class="flex flex-col gap-2">
                        {#each [...history].reverse() as entry, i (history.length - 1 - i)}
                            <div class="flex items-center gap-3 px-3 py-2.5 rounded-xl transition-colors {entry.correct ? 'bg-green-950/40' : 'bg-red-950/40'}"
                            >
                                <span class="text-lg shrink-0" class:text-green-400={entry.correct} class:text-red-400={!entry.correct}>
                                    {entry.correct ? '✓' : '✗'}
                                </span>
                                <div class="flex flex-col min-w-0">
                                    <span class="text-sm font-semibold text-zinc-200 truncate">
                                        {entry.exercise.verb}
                                    </span>
                                    <span class="text-xs text-zinc-500 truncate">
                                        {entry.exercise.subject} · {entry.exercise.tense}
                                    </span>
                                </div>
                                <span class="ml-auto text-xs font-mono text-zinc-600 shrink-0">
                                    {entry.correct ? entry.userAnswer : entry.exercise.answer}
                                </span>
                            </div>
                        {/each}
                    </div>

                    <!-- Timer info -->
                    {#if timerActive}
                        <div class="mt-4 pt-4 border-t border-zinc-800 text-center">
                            <span class="text-xs text-zinc-500 uppercase tracking-widest">Next in</span>
                            <span class="text-sm font-mono font-bold ml-2" style="color: {timerColor()}">{fmtTime(timerSeconds)}</span>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(6px); }
        to   { opacity: 1; transform: translateY(0); }
    }
    :global(.animate-fadeIn) {
        animation: fadeIn 0.3s ease-out;
    }
</style>
