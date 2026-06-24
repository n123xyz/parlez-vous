<script lang="ts">
    import verbData from '../data/verb.json';

    // Type declarations
    interface VerbConjugation {
        present: string;
        past: string;
    }
    
    interface LanguageData {
        language: string;
        verbs: {
            speak: VerbConjugation;
            eat: VerbConjugation;
            learn: VerbConjugation;
        };
    }

    const typedVerbData = verbData as Record<string, LanguageData>;

    // List of languages for selection
    const languages = Object.entries(typedVerbData).map(([code, data]) => ({
        code,
        name: data.language
    })).sort((a, b) => a.name.localeCompare(b.name));

    // Game state variables
    let { 
        selectedLangCode = $bindable('fr'),
        startTrigger = $bindable(0)
    } = $props<{ 
        selectedLangCode?: string; 
        startTrigger?: number;
    }>();

    $effect(() => {
        if (startTrigger > 0) {
            generateGame();
        }
    });

    let gameState = $state<'intro' | 'playing' | 'completed'>('intro');
    
    let currentQuestionIndex = $state(0);
    let score = $state(0);
    let selectedAnswer = $state<string | null>(null);
    let isAnswered = $state(false);

    // Question setup
    type VerbKey = 'speak' | 'eat' | 'learn';
    type TenseKey = 'present' | 'past';

    interface Question {
        verb: VerbKey;
        tense: TenseKey;
        correctAnswer: string;
        options: string[];
    }

    let questions = $state<Question[]>([]);

    function generateGame() {
        const langData = typedVerbData[selectedLangCode];
        if (!langData) return;

        const verbKeys: VerbKey[] = ['speak', 'eat', 'learn'];
        // Generate a question for each of the 3 verbs
        const generated: Question[] = verbKeys.map((verb, index) => {
            // Alternate tenses to cover both
            const tense: TenseKey = index % 2 === 0 ? 'present' : 'past';
            const correctAnswer = langData.verbs[verb][tense];
            
            // Gather all possible answers for this language to create distractors
            const allPossibleAnswers = new Set<string>();
            Object.values(langData.verbs).forEach(v => {
                allPossibleAnswers.add(v.present);
                allPossibleAnswers.add(v.past);
            });

            // Remove the correct answer from distractors pool
            allPossibleAnswers.delete(correctAnswer);

            const distractors = Array.from(allPossibleAnswers);
            
            // Shuffle distractors and select 3
            const selectedDistractors = distractors
                .sort(() => 0.5 - Math.random())
                .slice(0, 3);

            // Fill up with placeholders if we don't have enough distractors (rare)
            while (selectedDistractors.length < 3) {
                selectedDistractors.push("—");
            }

            // Combine correct answer and distractors, then shuffle
            const options = [correctAnswer, ...selectedDistractors].sort(() => 0.5 - Math.random());

            return {
                verb,
                tense,
                correctAnswer,
                options
            };
        });

        questions = generated;
        currentQuestionIndex = 0;
        score = 0;
        selectedAnswer = null;
        isAnswered = false;
        gameState = 'playing';
    }

    function selectOption(option: string) {
        if (isAnswered) return;
        selectedAnswer = option;
        isAnswered = true;
        
        if (option === questions[currentQuestionIndex].correctAnswer) {
            score++;
        }
    }

    function nextQuestion() {
        selectedAnswer = null;
        isAnswered = false;
        
        if (currentQuestionIndex + 1 < questions.length) {
            currentQuestionIndex++;
        } else {
            gameState = 'completed';
        }
    }

    function resetGame() {
        gameState = 'intro';
    }
</script>

<div class="game-card">
    {#if gameState === 'intro'}
        <div class="intro-screen">
            <div class="badge">Static Demo Game</div>
            <h2>Conjugation Practice</h2>
            <p>Select a language below to test your conjugation knowledge before downloading the app.</p>
            
            <div class="select-container">
                <label for="language-select">Target Language</label>
                <select id="language-select" bind:value={selectedLangCode}>
                    {#each languages as lang}
                        <option value={lang.code}>{lang.name}</option>
                    {/each}
                </select>
            </div>
            
            <button class="btn btn-primary" onclick={generateGame}>
                Start Quiz
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>
            </button>
        </div>
    {:else if gameState === 'playing'}
        {@const currentQuestion = questions[currentQuestionIndex]}
        <div class="quiz-screen">
            <div class="quiz-header">
                <span class="progress-indicator">Question {currentQuestionIndex + 1} of {questions.length}</span>
                <span class="score-indicator">Score: {score}</span>
            </div>
            
            <div class="progress-bar-container">
                <div class="progress-bar" style="width: {((currentQuestionIndex + (isAnswered ? 1 : 0)) / questions.length) * 100}%"></div>
            </div>

            <div class="question-box">
                <span class="sub">Conjugate the verb</span>
                <h3>{currentQuestion.verb.toUpperCase()}</h3>
                <span class="tense-tag">{currentQuestion.tense.toUpperCase()} TENSE</span>
                <p class="question-text">
                    What is the correct form in <strong>{typedVerbData[selectedLangCode].language}</strong>?
                </p>
            </div>

            <div class="options-grid">
                {#each currentQuestion.options as option}
                    {@const isCorrect = option === currentQuestion.correctAnswer}
                    {@const isSelected = option === selectedAnswer}
                    <button 
                        class="option-btn" 
                        class:correct={isAnswered && isCorrect}
                        class:incorrect={isAnswered && isSelected && !isCorrect}
                        class:disabled={isAnswered}
                        onclick={() => selectOption(option)}
                        disabled={isAnswered}
                    >
                        <span class="option-text">{option}</span>
                        {#if isAnswered && isCorrect}
                            <svg class="status-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                        {:else if isAnswered && isSelected && !isCorrect}
                            <svg class="status-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                        {/if}
                    </button>
                {/each}
            </div>

            {#if isAnswered}
                <button class="btn btn-primary next-btn" onclick={nextQuestion}>
                    {currentQuestionIndex + 1 === questions.length ? 'Show Results' : 'Next Question'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>
                </button>
            {/if}
        </div>
    {:else if gameState === 'completed'}
        <div class="completed-screen">
            <div class="completed-icon">
                {#if score === questions.length}
                    🏆
                {:else}
                    🎉
                {/if}
            </div>
            <h2>Quiz Completed!</h2>
            <div class="score-card">
                <span class="score-num">{score} / {questions.length}</span>
                <p>
                    {#if score === questions.length}
                        Perfect score! You're ready to learn.
                    {:else if score > 0}
                        Great effort! Keep practicing.
                    {:else}
                        Ready to start your language learning journey?
                    {/if}
                </p>
            </div>

            <p class="cta-desc">
                Download the complete Parlez-vous application to unlock interactive roleplaying, custom journaling, visual description games, and native handwriting support.
            </p>

            <div class="completed-actions">
                <a href="https://github.com/n123xyz/parlez-vous/releases" target="_blank" rel="noopener noreferrer" class="btn btn-primary download-apk-btn">
                    Download Android APK
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                </a>
                
                <div class="desktop-promo">
                    <span>Prefer desktop? Get the <strong>Windows / Linux / macOS</strong> version:</span>
                    <a href="https://github.com/n123xyz/parlez-vous/releases" target="_blank" rel="noopener noreferrer" class="desktop-link">
                        Download Desktop App &rarr;
                    </a>
                </div>

                <button class="btn btn-secondary" onclick={resetGame}>
                    Try Another Language
                </button>
            </div>
        </div>
    {/if}
</div>

<style>
    .game-card {
        background: rgba(24, 24, 27, 0.45);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid rgba(63, 63, 70, 0.4);
        border-radius: 24px;
        padding: 2.25rem;
        height: 100%;
        display: flex;
        flex-direction: column;
        justify-content: center;
        box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3), 0 10px 10px -5px rgba(0, 0, 0, 0.3);
        max-width: 500px;
        margin: 0 auto;
        width: 100%;
        box-sizing: border-box;
    }

    .badge {
        display: inline-block;
        padding: 0.35rem 0.75rem;
        background: rgba(253, 253, 150, 0.1);
        border: 1px solid rgba(253, 253, 150, 0.3);
        color: #fdfd96;
        border-radius: 9999px;
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        margin-bottom: 1.25rem;
    }

    h2 {
        font-size: 1.85rem;
        font-weight: 700;
        margin: 0 0 0.75rem 0;
        color: #f4f4f5;
        letter-spacing: -0.02em;
    }

    p {
        color: #a1a1aa;
        font-size: 0.95rem;
        line-height: 1.5;
        margin: 0 0 1.75rem 0;
    }

    .select-container {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        margin-bottom: 2rem;
        text-align: left;
    }

    .select-container label {
        color: #d4d4d8;
        font-size: 0.85rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.02em;
    }

    select {
        background: #18181b;
        border: 1px solid #3f3f46;
        border-radius: 12px;
        color: #f4f4f5;
        padding: 0.85rem 1rem;
        font-size: 1rem;
        font-family: inherit;
        outline: none;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    select:focus {
        border-color: #fdfd96;
        box-shadow: 0 0 0 2px rgba(253, 253, 150, 0.15);
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.85rem 1.75rem;
        font-size: 0.95rem;
        font-weight: 600;
        border-radius: 12px;
        border: none;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        text-decoration: none;
        width: 100%;
        box-sizing: border-box;
    }

    .btn-primary {
        background: #fdfd96;
        color: #09090b;
        box-shadow: 0 4px 14px rgba(253, 253, 150, 0.3);
    }

    .btn-primary:hover {
        background: #fcfca8;
        transform: translateY(-1px);
        box-shadow: 0 6px 20px rgba(253, 253, 150, 0.4);
    }

    .btn-secondary {
        background: rgba(39, 39, 42, 0.6);
        color: #f4f4f5;
        border: 1px solid rgba(63, 63, 70, 0.6);
        margin-top: 0.75rem;
    }

    .btn-secondary:hover {
        background: rgba(63, 63, 70, 0.8);
        border-color: rgba(113, 113, 122, 0.8);
        color: #ffffff;
    }

    /* Quiz Screen styles */
    .quiz-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.85rem;
        font-weight: 600;
        color: #a1a1aa;
        margin-bottom: 0.75rem;
    }

    .progress-bar-container {
        width: 100%;
        height: 6px;
        background: rgba(63, 63, 70, 0.3);
        border-radius: 9999px;
        margin-bottom: 1.75rem;
        overflow: hidden;
    }

    .progress-bar {
        height: 100%;
        background: #fdfd96;
        border-radius: 9999px;
        transition: width 0.3s ease;
    }

    .question-box {
        text-align: center;
        background: rgba(39, 39, 42, 0.4);
        border: 1px solid rgba(63, 63, 70, 0.3);
        border-radius: 16px;
        padding: 1.5rem;
        margin-bottom: 1.75rem;
    }

    .question-box .sub {
        font-size: 0.8rem;
        color: #a1a1aa;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .question-box h3 {
        font-size: 2.2rem;
        font-weight: 850;
        color: #f4f4f5;
        margin: 0.25rem 0;
        letter-spacing: -0.03em;
    }

    .tense-tag {
        display: inline-block;
        font-size: 0.75rem;
        font-weight: 700;
        padding: 0.2rem 0.6rem;
        background: rgba(253, 253, 150, 0.1);
        border: 1px solid rgba(253, 253, 150, 0.2);
        color: #fdfd96;
        border-radius: 6px;
        margin-bottom: 1rem;
    }

    .question-text {
        font-size: 0.95rem;
        color: #d4d4d8;
        margin: 0;
    }

    .options-grid {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        margin-bottom: 1.5rem;
    }

    .option-btn {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: rgba(39, 39, 42, 0.5);
        border: 1px solid rgba(63, 63, 70, 0.6);
        border-radius: 12px;
        padding: 1rem 1.25rem;
        color: #e4e4e7;
        font-size: 1.05rem;
        font-weight: 550;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        text-align: left;
        width: 100%;
        box-sizing: border-box;
    }

    .option-btn:not(.disabled):hover {
        background: rgba(63, 63, 70, 0.6);
        border-color: #a1a1aa;
        transform: scale(1.01);
    }

    .option-btn.correct {
        background: rgba(16, 185, 129, 0.15);
        border-color: rgb(16, 185, 129);
        color: rgb(52, 211, 153);
    }

    .option-btn.incorrect {
        background: rgba(239, 68, 68, 0.15);
        border-color: rgb(239, 68, 68);
        color: rgb(248, 113, 113);
    }

    .status-icon {
        width: 18px;
        height: 18px;
        flex-shrink: 0;
    }

    .next-btn {
        margin-top: 0.5rem;
        animation: pulse 2s infinite;
    }

    /* Completed screen */
    .completed-screen {
        text-align: center;
    }

    .completed-icon {
        font-size: 3.5rem;
        margin-bottom: 1rem;
    }

    .score-card {
        background: rgba(253, 253, 150, 0.05);
        border: 1px solid rgba(253, 253, 150, 0.15);
        border-radius: 16px;
        padding: 1.5rem;
        margin-bottom: 1.75rem;
    }

    .score-num {
        font-size: 2.5rem;
        font-weight: 800;
        color: #fdfd96;
        display: block;
        margin-bottom: 0.25rem;
    }

    .score-card p {
        margin: 0;
        font-weight: 500;
        color: #d4d4d8;
    }

    .cta-desc {
        color: #a1a1aa;
        margin-bottom: 2rem;
    }

    .completed-actions {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .download-apk-btn {
        font-size: 1rem;
        padding: 1rem 2rem;
    }

    .desktop-promo {
        margin-top: 1rem;
        margin-bottom: 0.5rem;
        font-size: 0.85rem;
        color: #a1a1aa;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        align-items: center;
        text-align: center;
    }

    .desktop-link {
        color: #fdfd96;
        text-decoration: underline;
        font-weight: 600;
        transition: color 0.2s;
    }

    .desktop-link:hover {
        color: #fcfca8;
    }

    @keyframes pulse {
        0% {
            box-shadow: 0 4px 14px rgba(253, 253, 150, 0.3);
        }
        50% {
            box-shadow: 0 4px 22px rgba(253, 253, 150, 0.55);
        }
        100% {
            box-shadow: 0 4px 14px rgba(253, 253, 150, 0.3);
        }
    }
</style>
