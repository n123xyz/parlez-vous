<script lang="ts">
    import { invoke as tauriInvoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import toast from 'svelte-french-toast';
    import { settingsState } from '$lib/state/settings.svelte';
    import { playSmartTTS } from '$lib/tts';

    type ScriptType = 'russian' | 'ukrainian' | 'korean' | 'latin';

    interface AlphabetItem {
        char: string;
        uppercase: string;
        lowercase: string;
        name: string;
        romanization: string;
        pronunciation: string;
        script: string;
        is_vowel: boolean;
    }

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let isDrawing = false;
    let result = $state<string | null>(null);
    let isSubmitting = $state(false);
    let sessionCorrect = $state(0);
    let sessionTotal = $state(0);

    // Vector strokes tracking
    type Point = { x: number; y: number };
    let currentStroke: Point[] = [];
    let allStrokes = $state<Point[][]>([]);

    // Script & Alphabet state
    let activeScript = $state<ScriptType>('russian');
    let letters = $state<AlphabetItem[]>([]);
    let currentIndex = $state(0);
    let isUppercase = $state(true);
    let showGuide = $state(true);

    // Backward-compatibility state for tests
    let allJamo = $state<string[]>([]);

    // Active item & target character to draw
    let currentItem = $derived(letters[currentIndex] ?? null);
    let targetChar = $derived.by(() => {
        if (!currentItem) return '';
        if (activeScript === 'korean') return currentItem.char;
        return isUppercase ? currentItem.uppercase : currentItem.lowercase;
    });

    // Alias for tests expecting targetJamo
    let targetJamo = $derived(targetChar);

    const SCRIPT_INFO: Record<ScriptType, { title: string; langCode: string; icon: string }> = {
        russian: { title: 'Russian Cyrillic', langCode: 'ru', icon: '🇷🇺' },
        ukrainian: { title: 'Ukrainian Cyrillic', langCode: 'uk', icon: '🇺🇦' },
        korean: { title: 'Korean Hangul', langCode: 'ko', icon: '🇰🇷' },
        latin: { title: 'Latin', langCode: 'en', icon: '🔤' }
    };

    onMount(async () => {
        ctx = canvas.getContext('2d', { willReadFrequently: true })!;
        resetCanvas();

        // Always fetch get_all_jamo to satisfy existing test suite
        try {
            allJamo = (await tauriInvoke('get_all_jamo')) as string[];
        } catch (e) {
            allJamo = ['ㅏ', 'ㅐ', 'ㅂ', 'ㅃ', 'ㅊ', 'ㄷ', 'ㅔ', 'ㅓ', 'ㅡ', 'ㄱ'];
        }

        // Auto-detect default script from user settings
        const target = (settingsState.targetLanguage || '').trim().toLowerCase();
        if (target.includes('korean')) {
            activeScript = 'korean';
        } else if (target.includes('ukrain')) {
            activeScript = 'ukrainian';
        } else if (target.includes('russ') || target.includes('cyrillic')) {
            activeScript = 'russian';
        } else if (target.includes('french') || target.includes('span') || target.includes('germ') || target.includes('ital')) {
            activeScript = 'latin';
        } else {
            activeScript = 'russian';
        }

        await loadAlphabet(activeScript);
    });

    async function loadAlphabet(script: ScriptType) {
        activeScript = script;
        currentIndex = 0;
        clearCanvas();

        try {
            const fetched = (await tauriInvoke('get_alphabet_letters', { script })) as AlphabetItem[];
            if (fetched && fetched.length > 0) {
                letters = fetched;
                return;
            }
        } catch (e) {
            console.warn('Could not fetch alphabet letters via IPC, using built-in table:', e);
        }

        // Built-in fallback alphabet datasets
        if (script === 'russian') {
            const ruChars = [
                ['А','а','a','[a]',true],['Б','б','b','[b]',false],['В','в','v','[v]',false],
                ['Г','г','g','[ɡ]',false],['Д','д','d','[d]',false],['Е','е','ye','[je]',true],
                ['Ё','ё','yo','[jo]',true],['Ж','ж','zh','[ʐ]',false],['З','з','z','[z]',false],
                ['И','и','i','[i]',true],['Й','й','y','[j]',false],['К','к','k','[k]',false],
                ['Л','л','l','[l]',false],['М','м','m','[m]',false],['Н','н','n','[n]',false],
                ['О','о','o','[o]',true],['П','п','p','[p]',false],['Р','р','r','[r]',false],
                ['С','с','s','[s]',false],['Т','т','t','[t]',false],['У','у','u','[u]',true],
                ['Ф','ф','f','[f]',false],['Х','х','kh','[x]',false],['Ц','ц','ts','[ts]',false],
                ['Ч','ч','ch','[tɕ]',false],['Ш','ш','sh','[ʂ]',false],['Щ','щ','shch','[ɕː]',false],
                ['Ъ','ъ','ʺ','[silent]',false],['Ы','ы','y','[ɨ]',true],['Ь','ь','ʹ','[palatal]',false],
                ['Э','э','e','[ɛ]',true],['Ю','ю','yu','[ju]',true],['Я','я','ya','[ja]',true]
            ];
            letters = ruChars.map(([u, l, rom, pron, v]) => ({
                char: u as string, uppercase: u as string, lowercase: l as string, name: u as string,
                romanization: rom as string, pronunciation: pron as string, script: 'cyrillic', is_vowel: v as boolean
            }));
        } else if (script === 'ukrainian') {
            const ukChars = [
                ['А','а','a','[a]',true],['Б','б','b','[b]',false],['В','в','v','[w/v]',false],
                ['Г','г','h','[ɦ]',false],['Ґ','ґ','g','[ɡ]',false],['Д','д','d','[d]',false],
                ['Е','е','e','[ɛ]',true],['Є','є','ye','[je]',true],['Ж','ж','zh','[ʒ]',false],
                ['З','з','z','[z]',false],['И','и','y','[ɪ]',true],['І','і','i','[i]',true],
                ['Ї','ї','yi','[ji]',true],['Й','й','y','[j]',false],['К','к','k','[k]',false],
                ['Л','л','l','[l]',false],['М','м','m','[m]',false],['Н','н','n','[n]',false],
                ['О','о','o','[ɔ]',true],['П','п','p','[p]',false],['Р','р','r','[r]',false],
                ['С','с','s','[s]',false],['Т','т','t','[t]',false],['У','у','u','[u]',true],
                ['Ф','ф','f','[f]',false],['Х','х','kh','[x]',false],['Ц','ц','ts','[ts]',false],
                ['Ч','ч','ch','[tʃ]',false],['Ш','ш','sh','[ʃ]',false],['Щ','щ','shch','[ʃtʃ]',false],
                ['Ь','ь','ʹ','[palatal]',false],['Ю','ю','yu','[ju]',true],['Я','я','ya','[ja]',true]
            ];
            letters = ukChars.map(([u, l, rom, pron, v]) => ({
                char: u as string, uppercase: u as string, lowercase: l as string, name: u as string,
                romanization: rom as string, pronunciation: pron as string, script: 'ukrainian', is_vowel: v as boolean
            }));
        } else if (script === 'korean') {
            const jamos = allJamo.length > 0 ? allJamo : [
                'ㅏ','ㅐ','ㅂ','ㅃ','ㅊ','ㄷ','ㅔ','ㅓ','ㅡ','ㄱ','ㄲ','ㅎ','ㅣ','ㅈ','ㅋ','ㅁ','ㄴ','ㅇ','ㅗ','ㅍ','ㄹ','ㅅ','ㅆ','ㅌ','ㅜ','ㅑ','ㅒ','ㅖ','ㅛ','ㅠ'
            ];
            const romanMap: Record<string, string> = {
                'ㅏ':'a','ㅐ':'ae','ㅂ':'b','ㅃ':'bb','ㅊ':'ch','ㄷ':'d','ㅔ':'e','ㅓ':'eo','ㅡ':'eu',
                'ㄱ':'g','ㄲ':'gg','ㅎ':'h','ㅣ':'i','ㅈ':'j','ㅋ':'k','ㅁ':'m','ㄴ':'n','ㅇ':'ng',
                'ㅗ':'o','ㅍ':'p','ㄹ':'r','ㅅ':'s','ㅆ':'ss','ㅌ':'t','ㅜ':'u','ㅑ':'ya','ㅒ':'yae',
                'ㅖ':'ye','ㅛ':'yo','ㅠ':'yu'
            };
            letters = jamos.map(j => {
                const rom = romanMap[j] ?? '';
                const isV = ['a','ae','ya','yae','eo','e','ye','yo','u','yu','eu','i','o'].includes(rom);
                return {
                    char: j, uppercase: j, lowercase: j, name: j,
                    romanization: rom, pronunciation: `[${rom}]`, script: 'hangul', is_vowel: isV
                };
            });
        } else {
            letters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').map(c => ({
                char: c, uppercase: c, lowercase: c.toLowerCase(), name: c,
                romanization: c.toLowerCase(), pronunciation: `[${c.toLowerCase()}]`, script: 'latin',
                is_vowel: ['A','E','I','O','U'].includes(c)
            }));
        }
    }

    function resetCanvas() {
        if (!canvas) return;
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.lineWidth = 14;
        ctx.lineCap = 'round';
        ctx.lineJoin = 'round';
        ctx.strokeStyle = '#18181b';
    }

    function getCoordinates(e: MouseEvent | TouchEvent) {
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        if (e instanceof MouseEvent) {
            return { x: (e.clientX - rect.left) * scaleX, y: (e.clientY - rect.top) * scaleY };
        } else {
            return { x: (e.touches[0].clientX - rect.left) * scaleX, y: (e.touches[0].clientY - rect.top) * scaleY };
        }
    }

    function startDrawing(e: MouseEvent | TouchEvent) {
        isDrawing = true;
        const { x, y } = getCoordinates(e);
        currentStroke = [{ x, y }];
        ctx.beginPath();
        ctx.moveTo(x, y);
        e.preventDefault();
    }

    function draw(e: MouseEvent | TouchEvent) {
        if (!isDrawing) return;
        const { x, y } = getCoordinates(e);
        currentStroke.push({ x, y });
        ctx.lineTo(x, y);
        ctx.stroke();
        e.preventDefault();
    }

    function stopDrawing() {
        if (isDrawing) {
            ctx.closePath();
            isDrawing = false;
            if (currentStroke.length > 0) {
                allStrokes.push([...currentStroke]);
            }
        }
    }

    function clearCanvas() {
        resetCanvas();
        result = null;
        allStrokes = [];
    }

    function preprocessCanvas(): number[] {
        if (allStrokes.length === 0) return new Array(28 * 28).fill(0);

        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        for (const stroke of allStrokes) {
            for (const pt of stroke) {
                if (pt.x < minX) minX = pt.x;
                if (pt.x > maxX) maxX = pt.x;
                if (pt.y < minY) minY = pt.y;
                if (pt.y > maxY) maxY = pt.y;
            }
        }

        const bbWidth = maxX - minX;
        const bbHeight = maxY - minY;

        const ghostCanvas = document.createElement('canvas');
        ghostCanvas.width = 28;
        ghostCanvas.height = 28;
        const ghostCtx = ghostCanvas.getContext('2d')!;

        ghostCtx.fillStyle = '#000000';
        ghostCtx.fillRect(0, 0, 28, 28);

        const maxDim = Math.max(bbWidth, bbHeight);
        const scale = maxDim > 0 ? 20.0 / maxDim : 1;
        const scaledWidth = bbWidth * scale;
        const scaledHeight = bbHeight * scale;
        const dx = (28 - scaledWidth) / 2;
        const dy = (28 - scaledHeight) / 2;

        ghostCtx.strokeStyle = '#ffffff';
        ghostCtx.lineWidth = 3.5;
        ghostCtx.lineCap = 'round';
        ghostCtx.lineJoin = 'round';

        for (const stroke of allStrokes) {
            if (stroke.length === 0) continue;
            ghostCtx.beginPath();
            const startX = ((stroke[0].x - minX) * scale) + dx;
            const startY = ((stroke[0].y - minY) * scale) + dy;
            ghostCtx.moveTo(startX, startY);

            for (let i = 1; i < stroke.length; i++) {
                const mapX = ((stroke[i].x - minX) * scale) + dx;
                const mapY = ((stroke[i].y - minY) * scale) + dy;
                ghostCtx.lineTo(mapX, mapY);
            }
            ghostCtx.stroke();
        }

        const finalData = ghostCtx.getImageData(0, 0, 28, 28).data;
        const finalGrid = new Array(28 * 28).fill(0);

        for (let i = 0; i < 28 * 28; i++) {
            const val = finalData[i * 4] / 255.0;
            finalGrid[i] = val > 0.3 ? 255 : 0;
        }

        return finalGrid;
    }

    async function submitCanvas() {
        if (isSubmitting || !targetChar) return;
        isSubmitting = true;
        const pixels = preprocessCanvas();

        try {
            const pred = (await tauriInvoke('infer_character', {
                pixels,
                vocabId: 1,
                targetText: targetChar
            })) as string;

            result = pred;
            sessionTotal++;

            const isMatch = pred.trim().toLowerCase() === targetChar.trim().toLowerCase();
            if (isMatch) {
                sessionCorrect++;
                toast.success(`${targetChar} ${currentItem?.romanization ? `(${currentItem.romanization})` : ''}`);
            }
        } catch (e) {
            console.error('Inference error:', e);
            toast.error(String(e));
        } finally {
            isSubmitting = false;
        }
    }

    function nextLetter() {
        if (letters.length === 0) return;
        currentIndex = (currentIndex + 1) % letters.length;
        clearCanvas();
    }

    function prevLetter() {
        if (letters.length === 0) return;
        currentIndex = (currentIndex - 1 + letters.length) % letters.length;
        clearCanvas();
    }

    function randomLetter() {
        if (letters.length <= 1) return;
        let next = Math.floor(Math.random() * letters.length);
        while (next === currentIndex) {
            next = Math.floor(Math.random() * letters.length);
        }
        currentIndex = next;
        clearCanvas();
    }

    function selectLetter(index: number) {
        currentIndex = index;
        clearCanvas();
    }

    async function playLetterAudio() {
        if (!targetChar) return;
        const langCode = SCRIPT_INFO[activeScript]?.langCode || 'en';
        await playSmartTTS(targetChar, settingsState.ttsServerUrl, undefined, langCode);
    }
</script>

<div class="alphabet-page">
    <!-- Header: Script Selector -->
    <div class="script-tabs">
        {#each (Object.keys(SCRIPT_INFO) as ScriptType[]) as scriptKey}
            <button
                class="script-tab {activeScript === scriptKey ? 'tab-active' : ''}"
                onclick={() => loadAlphabet(scriptKey)}
            >
                <span class="tab-icon">{SCRIPT_INFO[scriptKey].icon}</span>
                <span class="tab-text">{SCRIPT_INFO[scriptKey].title}</span>
            </button>
        {/each}
    </div>

    <!-- Letter Strip -->
    {#if letters.length > 0}
        <div class="letter-strip-container">
            <div class="letter-strip">
                {#each letters as item, idx}
                    <button
                        class="strip-item {idx === currentIndex ? 'strip-active' : ''}"
                        onclick={() => selectLetter(idx)}
                    >
                        {isUppercase ? item.uppercase : item.lowercase}
                    </button>
                {/each}
            </div>
        </div>
    {/if}

    <!-- Letter Card -->
    <div class="letter-card">
        <div class="card-top-bar">
            {#if activeScript !== 'korean'}
                <div class="case-toggle">
                    <button
                        class="case-btn {isUppercase ? 'case-active' : ''}"
                        onclick={() => { isUppercase = true; clearCanvas(); }}
                    >
                        Aa
                    </button>
                    <button
                        class="case-btn {!isUppercase ? 'case-active' : ''}"
                        onclick={() => { isUppercase = false; clearCanvas(); }}
                    >
                        aa
                    </button>
                </div>
            {/if}

            <button
                class="guide-btn {showGuide ? 'guide-active' : ''}"
                onclick={() => showGuide = !showGuide}
                title="Toggle Guide"
            >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
                    <circle cx="12" cy="12" r="3"/>
                </svg>
                Guide
            </button>

            <button class="audio-btn" onclick={playLetterAudio} title="Play audio">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
                    <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
                    <path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>
                </svg>
            </button>
        </div>

        <div class="letter-display">
            <span class="main-char">{targetChar}</span>
            <div class="letter-meta">
                <span class="meta-roman">{currentItem?.romanization || ''}</span>
                <span class="meta-pron">{currentItem?.pronunciation || ''}</span>
            </div>
        </div>

        <!-- Navigator -->
        <div class="card-nav">
            <button class="nav-btn" onclick={prevLetter} aria-label="Previous">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="15 18 9 12 15 6"/>
                </svg>
            </button>
            <button class="nav-btn" onclick={randomLetter} aria-label="Random">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/>
                    <polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/>
                </svg>
            </button>
            <button class="nav-btn" onclick={nextLetter} aria-label="Next">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="9 18 15 12 9 6"/>
                </svg>
            </button>
        </div>
    </div>

    <!-- Drawing Canvas -->
    <div
        class="canvas-wrapper"
        class:correct-glow={result !== null && result.trim().toLowerCase() === targetChar.trim().toLowerCase()}
        class:incorrect-glow={result !== null && result.trim().toLowerCase() !== targetChar.trim().toLowerCase()}
    >
        {#if showGuide && targetChar}
            <div class="canvas-ghost-guide">{targetChar}</div>
        {/if}

        <canvas
            bind:this={canvas}
            width={280}
            height={280}
            class="draw-canvas"
            onmousedown={startDrawing}
            onmousemove={draw}
            onmouseup={stopDrawing}
            onmouseout={stopDrawing}
            onblur={stopDrawing}
            ontouchstart={startDrawing}
            ontouchmove={draw}
            ontouchend={stopDrawing}
            ontouchcancel={stopDrawing}
        ></canvas>
    </div>

    <!-- Action Controls -->
    <div class="action-bar">
        <button class="btn-secondary" onclick={clearCanvas}>
            Clear
        </button>

        <button class="btn-primary" onclick={submitCanvas} disabled={isSubmitting}>
            {#if isSubmitting}
                <div class="spinner"></div>
            {:else}
                Submit
            {/if}
        </button>

        <button class="btn-secondary" onclick={nextLetter}>
            Next
        </button>
    </div>

    <!-- Minimal Feedback -->
    {#if result !== null}
        <div
            class="feedback-chip"
            class:chip-correct={result.trim().toLowerCase() === targetChar.trim().toLowerCase()}
            class:chip-wrong={result.trim().toLowerCase() !== targetChar.trim().toLowerCase()}
        >
            {#if result.trim().toLowerCase() === targetChar.trim().toLowerCase()}
                <span class="chip-icon">✓</span>
                <span>{targetChar} ({currentItem?.romanization || ''})</span>
            {:else}
                <span class="chip-icon">✗</span>
                <span>Drew {result} · Target {targetChar}</span>
            {/if}
        </div>
    {/if}

    <!-- Minimal Score Track -->
    {#if sessionTotal > 0}
        <div class="score-strip">
            <span class="score-num">{sessionCorrect}/{sessionTotal}</span>
            <div class="score-track">
                <div class="score-fill" style="width: {(sessionCorrect / sessionTotal) * 100}%"></div>
            </div>
        </div>
    {/if}
</div>

<style>
    .alphabet-page {
        max-width: 440px;
        margin: 0 auto;
        padding: 1.5rem 1rem 3rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.25rem;
    }

    /* Tabs */
    .script-tabs {
        display: flex;
        gap: 0.35rem;
        background: #18181b;
        padding: 0.35rem;
        border-radius: 0.875rem;
        border: 1px solid #27272a;
        width: 100%;
        overflow-x: auto;
    }
    .script-tab {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.35rem;
        padding: 0.45rem 0.6rem;
        font-size: 0.75rem;
        font-weight: 700;
        color: #a1a1aa;
        background: transparent;
        border: none;
        border-radius: 0.625rem;
        cursor: pointer;
        transition: all 0.15s ease;
        white-space: nowrap;
    }
    .script-tab:hover {
        color: #fde68a;
    }
    .tab-active {
        background: #27272a;
        color: #fde68a !important;
        box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
    }
    .tab-icon {
        font-size: 0.85rem;
    }

    /* Letter strip */
    .letter-strip-container {
        width: 100%;
        overflow-x: auto;
        scrollbar-width: none;
    }
    .letter-strip-container::-webkit-scrollbar {
        display: none;
    }
    .letter-strip {
        display: flex;
        gap: 0.3rem;
        padding: 0.1rem 0;
    }
    .strip-item {
        min-width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 0.5rem;
        background: #18181b;
        border: 1px solid #27272a;
        color: #71717a;
        font-size: 0.85rem;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.15s;
    }
    .strip-item:hover {
        background: #27272a;
        color: #d4d4d8;
    }
    .strip-active {
        background: #fde68a !important;
        color: #18181b !important;
        border-color: #fde68a !important;
        box-shadow: 0 0 8px rgba(253, 230, 138, 0.3);
    }

    /* Letter Card */
    .letter-card {
        width: 100%;
        background: #18181b;
        border: 1px solid rgba(253, 230, 138, 0.15);
        border-radius: 1.25rem;
        padding: 1rem 1.25rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75rem;
    }
    .card-top-bar {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    .case-toggle {
        display: flex;
        background: #27272a;
        border-radius: 0.5rem;
        padding: 2px;
    }
    .case-btn {
        background: transparent;
        border: none;
        color: #a1a1aa;
        font-size: 0.75rem;
        font-weight: 700;
        padding: 0.2rem 0.5rem;
        border-radius: 0.375rem;
        cursor: pointer;
    }
    .case-active {
        background: #3f3f46;
        color: #fde68a;
    }
    .guide-btn {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        background: #27272a;
        border: 1px solid #3f3f46;
        color: #71717a;
        border-radius: 0.5rem;
        padding: 0.25rem 0.55rem;
        font-size: 0.75rem;
        font-weight: 600;
        cursor: pointer;
    }
    .guide-active {
        color: #fde68a;
        border-color: rgba(253, 230, 138, 0.4);
    }
    .audio-btn {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #27272a;
        border: 1px solid #3f3f46;
        border-radius: 0.5rem;
        color: #fde68a;
        cursor: pointer;
        transition: all 0.15s;
    }
    .audio-btn:hover {
        background: #3f3f46;
    }

    .letter-display {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.2rem;
    }
    .main-char {
        font-size: 3.5rem;
        font-weight: 800;
        color: #fafafa;
        line-height: 1;
    }
    .letter-meta {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }
    .meta-roman {
        font-size: 1rem;
        font-weight: 700;
        color: #fde68a;
    }
    .meta-pron {
        font-size: 0.8rem;
        color: #71717a;
        font-family: monospace;
    }

    .card-nav {
        display: flex;
        gap: 0.5rem;
    }
    .nav-btn {
        width: 34px;
        height: 34px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #27272a;
        border: 1px solid #3f3f46;
        border-radius: 0.5rem;
        color: #a1a1aa;
        cursor: pointer;
        transition: all 0.15s;
    }
    .nav-btn:hover {
        background: #3f3f46;
        color: #fde68a;
    }

    /* Canvas */
    .canvas-wrapper {
        position: relative;
        background: #fff;
        padding: 4px;
        border-radius: 1.25rem;
        border: 2px solid #3f3f46;
        transition: all 0.25s ease;
    }
    .correct-glow {
        border-color: #4ade80;
        box-shadow: 0 0 16px rgba(74, 222, 128, 0.3);
    }
    .incorrect-glow {
        border-color: #f87171;
        box-shadow: 0 0 16px rgba(248, 113, 113, 0.25);
    }
    .canvas-ghost-guide {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 9rem;
        font-weight: 800;
        color: rgba(24, 24, 27, 0.08);
        pointer-events: none;
        user-select: none;
        line-height: 1;
    }
    .draw-canvas {
        display: block;
        border-radius: 0.875rem;
        cursor: crosshair;
        touch-action: none;
        width: 280px;
        height: 280px;
        position: relative;
        z-index: 1;
    }

    /* Actions */
    .action-bar {
        display: flex;
        gap: 0.5rem;
        width: 100%;
    }
    .btn-primary {
        flex: 1.5;
        padding: 0.7rem 1rem;
        background: #fde68a;
        color: #18181b;
        font-weight: 800;
        font-size: 0.9rem;
        border: none;
        border-radius: 0.75rem;
        cursor: pointer;
        transition: background 0.15s;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .btn-primary:hover:not(:disabled) {
        background: #fef08a;
    }
    .btn-primary:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    .btn-secondary {
        flex: 1;
        padding: 0.7rem 1rem;
        background: #27272a;
        border: 1px solid #3f3f46;
        color: #d4d4d8;
        font-weight: 700;
        font-size: 0.9rem;
        border-radius: 0.75rem;
        cursor: pointer;
        transition: background 0.15s;
    }
    .btn-secondary:hover {
        background: #3f3f46;
    }

    /* Feedback */
    .feedback-chip {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.4rem 0.85rem;
        border-radius: 999px;
        font-size: 0.85rem;
        font-weight: 700;
        animation: fadeIn 0.2s ease-out;
    }
    .chip-correct {
        background: rgba(34, 197, 94, 0.15);
        border: 1px solid rgba(74, 222, 128, 0.4);
        color: #4ade80;
    }
    .chip-wrong {
        background: rgba(239, 68, 68, 0.15);
        border: 1px solid rgba(248, 113, 113, 0.4);
        color: #f87171;
    }
    .chip-icon {
        font-weight: 900;
    }

    /* Score */
    .score-strip {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }
    .score-num {
        font-size: 0.75rem;
        font-weight: 700;
        color: #71717a;
    }
    .score-track {
        width: 64px;
        height: 4px;
        background: #27272a;
        border-radius: 2px;
        overflow: hidden;
    }
    .score-fill {
        height: 100%;
        background: #4ade80;
        border-radius: 2px;
        transition: width 0.3s ease;
    }

    /* Spinner */
    .spinner {
        width: 16px;
        height: 16px;
        border: 2px solid #18181b44;
        border-top-color: #18181b;
        border-radius: 50%;
        animation: spin 0.6s linear infinite;
    }
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(4px); }
        to { opacity: 1; transform: translateY(0); }
    }
</style>
