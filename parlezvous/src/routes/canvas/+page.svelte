<script lang="ts">
    import { invoke as tauriInvoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import toast from 'svelte-french-toast';

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let isDrawing = false;
    let result = $state<string | null>(null);
    let confidence = $state<number>(0);
    let isSubmitting = $state(false);
    let sessionCorrect = $state(0);
    let sessionTotal = $state(0);

    // --- NEW: Track vector strokes ---
    type Point = { x: number, y: number };
    let currentStroke: Point[] = [];
    let allStrokes = $state<Point[][]>([]);

    // Jamo practice state
    let allJamo = $state<string[]>([]);
    let currentIndex = $state(0);
    let targetJamo = $derived(allJamo[currentIndex] ?? '');

    // Romanization map for display
    const romanMap: Record<string, string> = {
        'ㅏ': 'a', 'ㅐ': 'ae', 'ㅂ': 'b', 'ㅃ': 'bb', 'ㅊ': 'ch',
        'ㄷ': 'd', 'ㅔ': 'e', 'ㅓ': 'eo', 'ㅡ': 'eu', 'ㄱ': 'g',
        'ㄲ': 'gg', 'ㅎ': 'h', 'ㅣ': 'i', 'ㅈ': 'j', 'ㅋ': 'k',
        'ㅁ': 'm', 'ㄴ': 'n', 'ㅇ': 'ng', 'ㅗ': 'o', 'ㅍ': 'p',
        'ㄹ': 'r', 'ㅅ': 's', 'ㅆ': 'ss', 'ㅌ': 't', 'ㅜ': 'u',
        'ㅑ': 'ya', 'ㅒ': 'yae', 'ㅖ': 'ye', 'ㅛ': 'yo', 'ㅠ': 'yu'
    };

    onMount(async () => {
        ctx = canvas.getContext('2d', { willReadFrequently: true })!;
        resetCanvas();

        try {
            allJamo = (await tauriInvoke('get_all_jamo')) as string[];
            // Shuffle for varied practice
            allJamo = allJamo.sort(() => Math.random() - 0.5);
        } catch (e) {
            console.error('Failed to load jamo:', e);
            // Fallback set
            allJamo = ['ㅏ', 'ㅂ', 'ㄱ', 'ㅁ', 'ㄴ', 'ㅅ', 'ㅇ', 'ㅎ', 'ㅗ', 'ㅜ'];
        }
    });

    function resetCanvas() {
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.lineWidth = 14;
        ctx.lineCap = 'round';
        ctx.lineJoin = 'round';
        ctx.strokeStyle = '#1a1a1a';
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
        currentStroke = [{ x, y }]; // Start a new stroke
        
        ctx.beginPath();
        ctx.moveTo(x, y);
        e.preventDefault();
    }

    function draw(e: MouseEvent | TouchEvent) {
        if (!isDrawing) return;
        const { x, y } = getCoordinates(e);
        currentStroke.push({ x, y }); // Record the movement
        
        ctx.lineTo(x, y);
        ctx.stroke();
        e.preventDefault();
    }

    function stopDrawing() {
        if (isDrawing) {
            ctx.closePath();
            isDrawing = false;
            if (currentStroke.length > 0) {
                allStrokes.push([...currentStroke]); // Save the completed stroke
            }
        }
    }

    function clearCanvas() {
        resetCanvas();
        result = null;
        confidence = 0;
        allStrokes = []; // Clear the memory
    }

    function preprocessCanvas(): number[] {
        if (allStrokes.length === 0) return new Array(28 * 28).fill(0);

        // 1. Find the Bounding Box of the Vector Coordinates
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

        // 2. Setup the hidden 28x28 Ghost Canvas
        const ghostCanvas = document.createElement('canvas');
        ghostCanvas.width = 28;
        ghostCanvas.height = 28;
        const ghostCtx = ghostCanvas.getContext('2d')!;
        
        // Fill with black (assuming model expects white lines on black bg)
        ghostCtx.fillStyle = '#000000';
        ghostCtx.fillRect(0, 0, 28, 28);

        // 3. Calculate scaling to fit within a 20x20 safe zone (leaving 4px padding)
        const maxDim = Math.max(bbWidth, bbHeight);
        const scale = maxDim > 0 ? 20.0 / maxDim : 1;
        
        const scaledWidth = bbWidth * scale;
        const scaledHeight = bbHeight * scale;
        
        // Center offsets
        const dx = (28 - scaledWidth) / 2;
        const dy = (28 - scaledHeight) / 2;

        // 4. Redraw the strokes cleanly at 28x28 resolution
        ghostCtx.strokeStyle = '#ffffff'; 
        
        // FIX 1: Thicken the lines to survive Max Pooling!
        ghostCtx.lineWidth = 3.5; 
        ghostCtx.lineCap = 'round';
        ghostCtx.lineJoin = 'round';

        for (const stroke of allStrokes) {
            if (stroke.length === 0) continue;
            ghostCtx.beginPath();
            
            // Map original coordinates to the new 28x28 scaled/centered grid
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

        // 5. Extract the pristine 28x28 array
        const finalData = ghostCtx.getImageData(0, 0, 28, 28).data;
        const finalGrid = new Array(28 * 28).fill(0);

        for (let i = 0; i < 28 * 28; i++) {
            // Grab the Red channel (0-255) and normalize to 0.0-1.0
            const val = finalData[i * 4] / 255.0; 
            
            // FIX 2: Apply the exact same binarization threshold as PyTorch
            // If the pixel is even slightly grey (> 0.3), snap it to pure white (255).
            finalGrid[i] = val > 0.3 ? 255 : 0; 
        }

        return finalGrid;
    }

    async function submitCanvas() {
        if (isSubmitting || !targetJamo) return;
        isSubmitting = true;
        const pixels = preprocessCanvas();
        try {
            const pred = (await tauriInvoke('infer_character', {
                pixels,
                vocabId: 1,
                targetText: targetJamo
            })) as string;
            result = pred;
            sessionTotal++;
            if (pred === targetJamo) {
                sessionCorrect++;
                toast.success(`Correct! ${targetJamo} (${romanMap[targetJamo] ?? ''})`);
            }
        } catch (e) {
            console.error('Inference failed:', e);
            toast.error('Inference failed: ' + e);
        } finally {
            isSubmitting = false;
        }
    }

    function nextJamo() {
        currentIndex = (currentIndex + 1) % allJamo.length;
        clearCanvas();
    }

    function prevJamo() {
        currentIndex = (currentIndex - 1 + allJamo.length) % allJamo.length;
        clearCanvas();
    }

    function skipToRandom() {
        let next = Math.floor(Math.random() * allJamo.length);
        while (next === currentIndex && allJamo.length > 1) {
            next = Math.floor(Math.random() * allJamo.length);
        }
        currentIndex = next;
        clearCanvas();
    }
</script>

<div class="canvas-page">
    <!-- Header -->
    <div class="canvas-header">
        <h2 class="canvas-title">
            <span class="title-icon">✍</span>
            Jamo Canvas
        </h2>
        <p class="canvas-subtitle">Practice writing individual Hangul characters</p>

        <!-- Session stats -->
        {#if sessionTotal > 0}
            <div class="session-stats">
                <span class="stat-chip">
                    <span class="stat-num">{sessionCorrect}</span>/<span class="stat-den">{sessionTotal}</span>
                </span>
                <div class="stat-bar-track">
                    <div class="stat-bar-fill" style="width: {sessionTotal > 0 ? (sessionCorrect / sessionTotal) * 100 : 0}%"></div>
                </div>
            </div>
        {/if}
    </div>

    <!-- Target card -->
    <div class="target-card">
        <span class="target-label">Draw this character</span>
        <span class="target-jamo">{targetJamo}</span>
        <span class="target-roman">{romanMap[targetJamo] ?? ''}</span>

        <div class="target-nav">
            <button class="nav-btn" onclick={prevJamo} aria-label="Previous jamo">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
            </button>
            <button class="nav-btn shuffle-btn" onclick={skipToRandom} aria-label="Random jamo">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></svg>
            </button>
            <button class="nav-btn" onclick={nextJamo} aria-label="Next jamo">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
            </button>
        </div>
    </div>

    <!-- Canvas area -->
    <div class="canvas-wrapper" class:correct-glow={result !== null && result === targetJamo} class:incorrect-glow={result !== null && result !== targetJamo}>
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

    <!-- Actions -->
    <div class="canvas-actions">
        <button class="action-btn clear-btn" onclick={clearCanvas}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
            Clear
        </button>
        <button class="action-btn submit-btn" onclick={submitCanvas} disabled={isSubmitting}>
            {#if isSubmitting}
                <div class="spinner"></div>
            {:else}
                Submit
            {/if}
        </button>
        <button class="action-btn next-btn" onclick={nextJamo}>
            Next →
        </button>
    </div>

    <!-- Result display -->
    {#if result !== null}
        <div class="result-card animate-result" class:result-correct={result === targetJamo} class:result-incorrect={result !== targetJamo}>
            <div class="result-header">
                {#if result === targetJamo}
                    <span class="result-icon">✓</span>
                    <span class="result-title">Correct!</span>
                {:else}
                    <span class="result-icon result-icon-wrong">✗</span>
                    <span class="result-title">Not quite</span>
                {/if}
            </div>

            <div class="result-comparison">
                <div class="result-col">
                    <span class="result-label">You drew</span>
                    <span class="result-jamo">{result}</span>
                    <span class="result-roman">{romanMap[result] ?? '?'}</span>
                </div>
                {#if result !== targetJamo}
                    <div class="result-divider"></div>
                    <div class="result-col">
                        <span class="result-label">Target</span>
                        <span class="result-jamo">{targetJamo}</span>
                        <span class="result-roman">{romanMap[targetJamo] ?? ''}</span>
                    </div>
                {/if}
            </div>

            {#if result === targetJamo}
                <p class="result-detail">SRS reinforcement updated</p>
            {:else}
                <p class="result-detail">Try drawing it again, or skip to the next</p>
            {/if}
        </div>
    {/if}
</div>

<style>
    .canvas-page {
        max-width: 480px;
        margin: 0 auto;
        padding: 2rem 1.5rem 3rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;
    }

    /* --- Header --- */
    .canvas-header {
        text-align: center;
        width: 100%;
    }
    .canvas-title {
        font-size: 1.75rem;
        font-weight: 800;
        color: #fde68a;
        margin: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
    }
    .title-icon {
        font-size: 1.5rem;
    }
    .canvas-subtitle {
        color: #a1a1aa;
        margin: 0.35rem 0 0;
        font-size: 0.9rem;
    }

    /* --- Session stats --- */
    .session-stats {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.75rem;
        margin-top: 0.75rem;
    }
    .stat-chip {
        font-size: 0.8rem;
        font-weight: 700;
        color: #d4d4d8;
        background: #27272a;
        padding: 0.2rem 0.6rem;
        border-radius: 999px;
    }
    .stat-num { color: #4ade80; }
    .stat-den { color: #71717a; }
    .stat-bar-track {
        width: 80px;
        height: 4px;
        background: #27272a;
        border-radius: 2px;
        overflow: hidden;
    }
    .stat-bar-fill {
        height: 100%;
        background: linear-gradient(90deg, #4ade80, #22d3ee);
        border-radius: 2px;
        transition: width 0.4s ease;
    }

    /* --- Target card --- */
    .target-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        background: #18181b;
        border: 1px solid rgba(253, 230, 138, 0.15);
        border-radius: 1.25rem;
        padding: 1.25rem 2rem;
        width: 100%;
        box-shadow: 0 0 24px rgba(253, 230, 138, 0.04);
    }
    .target-label {
        font-size: 0.7rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.12em;
        color: #71717a;
    }
    .target-jamo {
        font-size: 4rem;
        font-weight: 800;
        color: #fafafa;
        line-height: 1.15;
        margin: 0.15rem 0;
    }
    .target-roman {
        font-size: 1rem;
        font-weight: 600;
        color: #fde68a;
        letter-spacing: 0.04em;
    }
    .target-nav {
        display: flex;
        gap: 0.5rem;
        margin-top: 0.75rem;
    }
    .nav-btn {
        width: 36px;
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 0.625rem;
        background: #27272a;
        border: 1px solid #3f3f46;
        color: #a1a1aa;
        cursor: pointer;
        transition: all 0.15s ease;
    }
    .nav-btn:hover {
        background: #3f3f46;
        color: #fde68a;
        border-color: #fde68a44;
    }
    .shuffle-btn {
        width: 36px;
    }

    /* --- Canvas --- */
    .canvas-wrapper {
        background: #fff;
        padding: 6px;
        border-radius: 1.25rem;
        border: 2px solid #3f3f46;
        transition: border-color 0.3s ease, box-shadow 0.3s ease;
    }
    .canvas-wrapper.correct-glow {
        border-color: #4ade80;
        box-shadow: 0 0 20px rgba(74, 222, 128, 0.25);
    }
    .canvas-wrapper.incorrect-glow {
        border-color: #f87171;
        box-shadow: 0 0 20px rgba(248, 113, 113, 0.2);
    }
    .draw-canvas {
        display: block;
        border-radius: 0.875rem;
        cursor: crosshair;
        touch-action: none;
        width: 280px;
        height: 280px;
    }

    /* --- Actions --- */
    .canvas-actions {
        display: flex;
        gap: 0.625rem;
        width: 100%;
    }
    .action-btn {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
        padding: 0.75rem 1rem;
        border-radius: 0.875rem;
        font-weight: 700;
        font-size: 0.9rem;
        cursor: pointer;
        border: 1px solid transparent;
        transition: all 0.15s ease;
    }
    .clear-btn {
        background: #27272a;
        color: #a1a1aa;
        border-color: #3f3f46;
    }
    .clear-btn:hover {
        background: #3f3f46;
        color: #d4d4d8;
    }
    .submit-btn {
        background: #fde68a;
        color: #18181b;
        flex: 1.5;
        box-shadow: 0 0 14px rgba(253, 230, 138, 0.15);
    }
    .submit-btn:hover:not(:disabled) {
        background: #fef08a;
        box-shadow: 0 0 20px rgba(253, 230, 138, 0.25);
    }
    .submit-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    .next-btn {
        background: #27272a;
        color: #a1a1aa;
        border-color: #3f3f46;
    }
    .next-btn:hover {
        background: #3f3f46;
        color: #fde68a;
        border-color: #fde68a44;
    }

    /* --- Spinner --- */
    .spinner {
        width: 18px;
        height: 18px;
        border: 3px solid #18181b33;
        border-top-color: #18181b;
        border-radius: 50%;
        animation: spin 0.6s linear infinite;
    }
    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    /* --- Result card --- */
    .result-card {
        width: 100%;
        padding: 1.25rem;
        border-radius: 1.25rem;
        text-align: center;
    }
    .result-correct {
        background: linear-gradient(135deg, #052e16, #14532d);
        border: 1px solid rgba(74, 222, 128, 0.3);
    }
    .result-incorrect {
        background: linear-gradient(135deg, #2a0a0a, #450a0a);
        border: 1px solid rgba(248, 113, 113, 0.3);
    }
    .result-header {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
        margin-bottom: 0.75rem;
    }
    .result-icon {
        font-size: 1.25rem;
        font-weight: 800;
        color: #4ade80;
    }
    .result-icon-wrong {
        color: #f87171;
    }
    .result-title {
        font-size: 1.1rem;
        font-weight: 700;
        color: #fafafa;
    }
    .result-comparison {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 1.5rem;
    }
    .result-col {
        display: flex;
        flex-direction: column;
        align-items: center;
    }
    .result-label {
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: #71717a;
    }
    .result-jamo {
        font-size: 2.5rem;
        font-weight: 800;
        color: #fafafa;
        line-height: 1.2;
    }
    .result-roman {
        font-size: 0.85rem;
        font-weight: 600;
        color: #fde68a;
    }
    .result-divider {
        width: 1px;
        height: 60px;
        background: #3f3f46;
    }
    .result-detail {
        margin: 0.5rem 0 0;
        font-size: 0.8rem;
        color: #71717a;
    }

    /* --- Animation --- */
    .animate-result {
        animation: slideUp 0.3s ease-out forwards;
    }
    @keyframes slideUp {
        from {
            opacity: 0;
            transform: translateY(12px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
</style>
