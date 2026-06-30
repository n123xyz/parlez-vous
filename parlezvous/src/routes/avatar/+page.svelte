<script lang="ts">
    import { invoke, convertFileSrc } from '@tauri-apps/api/core';
    import { onMount, onDestroy, tick } from 'svelte';
    import { ollamaState } from '$lib/state/ollama.svelte.ts';
    import { settingsState } from '$lib/state/settings.svelte.ts';
    import toast from 'svelte-french-toast';
    import { marked } from 'marked';
    import { open } from '@tauri-apps/plugin-dialog';
    import { initTTSAudio, playTTS, setPlaybackRate, ttsPlaybackRate, setLipSyncNode, playSupertonicTTS, playSmartTTS } from '$lib/tts';
    import { CURRICULUM_TIERS, getTierFromXP } from '$lib/curriculum';
    import { timeTracker } from '$lib/state/timeTracker.svelte.ts';

    // Module variables for heavy libraries to avoid TDZ (Temporal Dead Zone) crashes
    let THREE: any;
    let GLTFLoader: any;
    let VRMLoaderPlugin: any;
    let VRM: any;
    let VRMAnimationLoaderPlugin: any;
    let createVRMAnimationClip: any;
    let createWLipSyncNode: any;

    let canvasContainer: HTMLDivElement = undefined as any;
    let chatScrollContainer: HTMLDivElement = undefined as any;
    let rootScrollContainer: HTMLDivElement = undefined as any;
    let chatHistory = $state<{ role: string; content: string; correction?: string | null; audioBase64?: string }[]>([]);
    let currentInput = $state('');
    let isChatting = $state(false);
    let chatInputRef: HTMLTextAreaElement | null = $state(null);

    let currentVrm = $state<any>(null); // Use any for type as VRM is loaded dynamically
    let mixer: any = null;
    let timer: any = null;

    // Animation preload cache & crossfade state
    let animationsMap = new Map<string, any>();
    let idleAction: any = null;
    let activeAction: any = null;
    let isSpeaking = false; // Tracks if TTS is currently active
    let renderer: any = undefined as any;
    let scene: any = undefined as any;
    let camera: any = undefined as any;
    let animationFrameId: number = undefined as any;
    let handleResize: () => void = undefined as any;

    let audioContext: AudioContext = undefined as any;
    let lipsyncNode: any = undefined;

    let isLoading = $state(true);
    let libsLoaded = $state(false);
    let containerWidth = $state(0);
    let containerHeight = $state(0);
    
    let isIngesting = $state(false);
    let textbooks = $state<string[]>([]);
    let activeTextbook = $state<string | null>(null);
    let textbookSrc = $state<string | null>(null);
    let textbookError = $state(false);
    let activePage = $state<number | null>(null);
    let textbookDir = $state('');

    let viewMode = $state<'split' | 'avatar' | 'chat'>('split');
    let muteTts = $state(false);

    function setViewMode(mode: 'split' | 'avatar' | 'chat') {
        viewMode = mode;
        // Trigger resize so ThreeJS fixes aspect ratio after DOM flex layout shifts
        setTimeout(() => {
            window.dispatchEvent(new Event('resize'));
        }, 50);
    }

    let mapFollowMode = $state(true);
    let activeThemeId = $state<string | null>(null);
    let reviewThemeCandidate = $state<{ id: string; name: string } | null>(null);
    let showReviewModal = $state(false);

    // --- NEW: Hangul Handwriting Keyboard State ---
    const INITIALS = ['ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
    const VOWELS = ['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'];
    const FINALS = ['', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];

    let showHandwritingPanel = $state(false);
    let currentSlot = $state<'initial' | 'vowel' | 'final'>('initial');
    let blockInitial = $state<string | null>(null);
    let blockVowel = $state<string | null>(null);
    let blockFinal = $state<string | null>(null);

    let composedHangul = $derived.by(() => {
        if (!blockInitial) return '';
        const iIndex = INITIALS.indexOf(blockInitial);
        const vIndex = blockVowel ? VOWELS.indexOf(blockVowel) : -1;
        const fIndex = blockFinal ? FINALS.indexOf(blockFinal) : -1;

        if (iIndex === -1 || vIndex === -1) {
            return blockInitial + (blockVowel || '') + (blockFinal || '');
        }

        const charCode = 44032 + (iIndex * 588) + (vIndex * 28) + Math.max(0, fIndex);
        return String.fromCharCode(charCode);
    });

    let drawingCanvas = $state<HTMLCanvasElement>(undefined as any);
    let drawingCtx = $state<CanvasRenderingContext2D>(undefined as any);
    let isDrawingCanvas = false;
    type Point = { x: number, y: number };
    let currentStroke: Point[] = [];
    let allStrokes = $state<Point[][]>([]);
    let strokeStatusMsg = $state<string | null>(null);
    let isRecognizing = $state(false);

    // --- NEW: VAD & ASR State ---
    let isMicActive = $state(false);
    let vadState = $state<'inactive' | 'speaking' | 'processing'>('inactive');
    let micStream: MediaStream | null = null;
    let vadAudioCtx: AudioContext | null = null;
    let analyser: AnalyserNode | null = null;
    let audioProcessor: ScriptProcessorNode | null = null;
    let silenceThreshold = 0;
    let silenceThresholdManual = 0;
    let silenceStart = 0;
    
    // Raw audio buffer collection for WAV encoding
    let audioBuffers: Float32Array[] = [];
    let recordingLength = 0;

    async function loadTextbooks() {
        try {
            textbooks = (await invoke('list_textbooks')) as string[];
            if (!textbookDir) {
                textbookDir = (await invoke('get_textbook_dir')) as string;
            }
        } catch (e) {
            console.error("Failed to load textbooks:", e);
        }
    }

    function resolveTextbookSrc(name: string | null) {
        if (!name || !textbookDir) {
            textbookSrc = null;
            textbookError = false;
            return;
        }
        try {
            const fullPath = `${textbookDir}/${name}`;
            textbookSrc = convertFileSrc(fullPath);
            textbookError = false;
        } catch (e) {
            console.error('Failed to resolve textbook path:', e);
            textbookSrc = null;
            textbookError = true;
        }
    }

    async function handleUpload() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'Textbooks', extensions: ['pdf'] }]
            });

            if (selected) {
                const filePath = typeof selected === 'string' ? selected : (selected as any).path;
                isIngesting = true;
                
                await invoke('upload_and_ingest_textbook', { 
                    filePath: filePath, 
                    model: 'nomic-embed-text-v2-moe'
                });
                
                await loadTextbooks(); 
                activeTextbook = typeof selected === 'string' 
                    ? selected.split(/[/\\]/).pop() || null 
                    : (selected as any).name || null;
                resolveTextbookSrc(activeTextbook);
            }
        } catch (e) {
            console.error("Upload/Ingestion failed:", e);
            alert("Failed to upload: " + e);
        } finally {
            isIngesting = false;
        }
    }

    onMount(async () => {
        // Dynamically import the heavy libraries ONLY when the component mounts.
        // This hides them from SvelteKit's hover-preload mechanism.
        THREE = await import('three');
        timer = new THREE.Timer();

        const gltfModule = await import('three/examples/jsm/loaders/GLTFLoader.js');
        GLTFLoader = gltfModule.GLTFLoader;

        const vrmModule = await import('@pixiv/three-vrm');
        VRMLoaderPlugin = vrmModule.VRMLoaderPlugin;
        VRM = vrmModule.VRM;

        const vrmAnimModule = await import('@pixiv/three-vrm-animation');
        VRMAnimationLoaderPlugin = vrmAnimModule.VRMAnimationLoaderPlugin;
        createVRMAnimationClip = vrmAnimModule.createVRMAnimationClip;

        const wlipsyncModule = await import('wlipsync');
        createWLipSyncNode = wlipsyncModule.createWLipSyncNode;

        await loadTextbooks();
        
        try {
            const curr: any = await invoke('get_curriculum', { language: settingsState.targetLanguage });
            activeThemeId = curr.active_theme_id;

            const currentTier = getTierFromXP(curr.total_xp);
            const todayStr = new Date().toDateString();
            const lastReviewPromptDate = localStorage.getItem('lastReviewPromptDate');

            if (currentTier > 1 && lastReviewPromptDate !== todayStr) {
                const pastTiers = CURRICULUM_TIERS.filter(t => t.level < currentTier);
                if (pastTiers.length > 0) {
                    const randomTier = pastTiers[Math.floor(Math.random() * pastTiers.length)];
                    const randomTheme = randomTier.themes[Math.floor(Math.random() * randomTier.themes.length)];
                    reviewThemeCandidate = randomTheme;
                    showReviewModal = true;
                    localStorage.setItem('lastReviewPromptDate', todayStr);
                }
            }
        } catch (e) {
            console.error("No curriculum found", e);
        }

        // Libraries are now ready for the effect block to take over
        libsLoaded = true;

        timeTracker.startTracking();
    });

    $effect(() => {
        // 1. Initialize ThreeJS if it hasn't been yet
        if (libsLoaded && containerWidth > 0 && containerHeight > 0 && !renderer) {
            initThree();
            loadVRM(); 
        }

        // 2. React to dynamic height/width changes caused by toggling the handwriting panel
        if (renderer && camera && containerWidth > 0 && containerHeight > 0) {
            camera.aspect = containerWidth / containerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(containerWidth, containerHeight);
        }
    });

    onDestroy(() => {
        timeTracker.flushTime();
        timeTracker.stopTracking();
        if (animationFrameId) cancelAnimationFrame(animationFrameId);
        if (renderer) renderer.dispose();
        if (handleResize) window.removeEventListener('resize', handleResize);
        stopMic(); // Ensure mic is released on unmount
    });

    async function initAudio() {
        if (audioContext) return;
        audioContext = new AudioContext();
    }

    async function initLipSync() {
        initTTSAudio();
        if (lipsyncNode) return;
        try {
            const profileRes = await fetch('/profile.json');
            if (!profileRes.ok) throw new Error('profile.json not found');
            const profile = await profileRes.json();
            // Validate that the profile has the required mfccs structure
            if (!profile.mfccs || !Array.isArray(profile.mfccs) || profile.mfccs.length === 0) {
                throw new Error('Invalid wlipsync profile: missing or empty mfccs array. Generate a valid profile using uLipSync in Unity.');
            }
            // Need audio context from tts module
            const { ttsAudioContext } = await import('$lib/tts');
            lipsyncNode = await createWLipSyncNode(ttsAudioContext, profile);
            setLipSyncNode(lipsyncNode);
        } catch (e) {
            console.warn("Could not load wlipsync profile, lip sync will not work:", e);
            lipsyncNode = undefined;
        }
    }

    // --- HANGUL DRAWING LOGIC ---
    function initDrawingCanvas() {
        if (!drawingCanvas) return;
        drawingCtx = drawingCanvas.getContext('2d', { willReadFrequently: true })!;
        clearDrawingCanvas();
    }

    function clearDrawingCanvas() {
        if (!drawingCtx) return;
        drawingCtx.fillStyle = '#ffffff';
        drawingCtx.fillRect(0, 0, drawingCanvas.width, drawingCanvas.height);
        drawingCtx.lineWidth = 14;
        drawingCtx.lineCap = 'round';
        drawingCtx.lineJoin = 'round';
        drawingCtx.strokeStyle = '#1a1a1a';
        allStrokes = [];
    }

    function getDrawingCoordinates(e: MouseEvent | TouchEvent) {
        const rect = drawingCanvas.getBoundingClientRect();
        const scaleX = drawingCanvas.width / rect.width;
        const scaleY = drawingCanvas.height / rect.height;
        if (e instanceof MouseEvent) {
            return { x: (e.clientX - rect.left) * scaleX, y: (e.clientY - rect.top) * scaleY };
        } else {
            return { x: (e.touches[0].clientX - rect.left) * scaleX, y: (e.touches[0].clientY - rect.top) * scaleY };
        }
    }

    function startDrawing(e: MouseEvent | TouchEvent) {
        isDrawingCanvas = true;
        const { x, y } = getDrawingCoordinates(e);
        currentStroke = [{ x, y }];
        
        drawingCtx.beginPath();
        drawingCtx.moveTo(x, y);
        e.preventDefault();
    }

    function draw(e: MouseEvent | TouchEvent) {
        if (!isDrawingCanvas) return;
        const { x, y } = getDrawingCoordinates(e);
        currentStroke.push({ x, y });
        
        drawingCtx.lineTo(x, y);
        drawingCtx.stroke();
        e.preventDefault();
    }

    function stopDrawing() {
        if (isDrawingCanvas) {
            drawingCtx.closePath();
            isDrawingCanvas = false;
            if (currentStroke.length > 0) {
                allStrokes.push([...currentStroke]);
            }
        }
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

    async function recognizeStroke() {
        if (allStrokes.length === 0) return;
        isRecognizing = true;
        strokeStatusMsg = null;
        
        try {
            const pixels = preprocessCanvas();
            const pred = (await invoke('infer_character', {
                pixels,
                vocabId: 1,
                targetText: ""
            })) as string;
            
            const match = pred.match(/^(\S+)/);
            if (match) {
                const char = match[1];
                if (currentSlot === 'initial') {
                    if (INITIALS.includes(char)) {
                        blockInitial = char;
                        currentSlot = 'vowel';
                        strokeStatusMsg = `Initial set to ${char}`;
                        clearDrawingCanvas();
                    } else {
                        strokeStatusMsg = `${char} is not a valid initial consonant.`;
                    }
                } else if (currentSlot === 'vowel') {
                    if (VOWELS.includes(char)) {
                        blockVowel = char;
                        currentSlot = 'final';
                        strokeStatusMsg = `Vowel set to ${char}`;
                        clearDrawingCanvas();
                    } else {
                        strokeStatusMsg = `${char} is not a valid vowel.`;
                    }
                } else if (currentSlot === 'final') {
                    if (FINALS.includes(char) && char !== '') {
                        blockFinal = char;
                        strokeStatusMsg = `Final set to ${char}`;
                        clearDrawingCanvas();
                    } else {
                        strokeStatusMsg = `${char} is not a valid final consonant.`;
                    }
                }
            }
        } catch (e) {
            strokeStatusMsg = `Recognition failed: ${e}`;
        } finally {
            isRecognizing = false;
            setTimeout(() => { if (strokeStatusMsg) strokeStatusMsg = null; }, 3000);
        }
    }

    function clearHangulBlock() {
        blockInitial = null;
        blockVowel = null;
        blockFinal = null;
        currentSlot = 'initial';
        clearDrawingCanvas();
        strokeStatusMsg = null;
    }

    function commitHangulBlock() {
        if (composedHangul) {
            currentInput += composedHangul;
            clearHangulBlock();
        }
    }

    // --- VAD & ASR LOGIC ---

    async function toggleMic() {
        if (isMicActive) {
            await stopMic();
        } else {
            await startMic();
        }
    }

    async function startMic() {
        try {
            micStream = await navigator.mediaDevices.getUserMedia({ 
                audio: {
                    echoCancellation: false,
                    noiseSuppression: false,
                    autoGainControl: false
                } 
            });
            vadAudioCtx = new AudioContext({ sampleRate: 16000 });
            audioProcessor = vadAudioCtx.createScriptProcessor(4096, 1, 1);
            const source = vadAudioCtx.createMediaStreamSource(micStream);
            
            source.connect(audioProcessor);
            const gainNode = vadAudioCtx.createGain();
            gainNode.gain.value = 0;
            audioProcessor.connect(gainNode);
            gainNode.connect(vadAudioCtx.destination);
            
            audioBuffers = [];
            recordingLength = 0;
            vadState = 'speaking';
            isMicActive = true;
            
            audioProcessor.onaudioprocess = (e) => {
                if (vadState === 'speaking') {
                    const inputData = e.inputBuffer.getChannelData(0);
                    audioBuffers.push(new Float32Array(inputData));
                    recordingLength += inputData.length;
                }
            };
        } catch (err: any) {
            console.error("Microphone access denied or failed", err);
            toast.error("Could not access microphone: " + (err.message || err.name || "Unknown error"));
        }
    }

    async function stopMic() {
        if (!isMicActive) return;
        isMicActive = false;
        vadState = 'processing';
        
        if (audioProcessor) {
            audioProcessor.disconnect();
            audioProcessor = null;
        }
        if (micStream) micStream.getTracks().forEach(t => t.stop());
        
        await processAudio();
        
        if (vadAudioCtx) {
            vadAudioCtx.close();
            vadAudioCtx = null;
        }
        vadState = 'inactive';
    }

    async function processAudio() {
        if (recordingLength === 0) {
            return;
        }

        const audioBlob = exportWAV(audioBuffers, recordingLength, vadAudioCtx!.sampleRate);
        audioBuffers = []; 
        recordingLength = 0;
        
        const isLitert = settingsState.activeModel.includes('litert');
        if (isLitert) {
            try {
                const reader = new FileReader();
                reader.readAsDataURL(audioBlob);
                reader.onloadend = async () => {
                    const base64 = (reader.result as string).split(',')[1];
                    await sendMessage(base64);
                };
            } catch (e: any) {
                console.error("LiteRT Audio Error:", e);
                toast.error("LiteRT Audio Error: " + (e.message || e));
            }
        } else {
            const formData = new FormData();
            formData.append("file", audioBlob, "speech.wav");
            formData.append("model", "whisper-1");

            try {
                const asrUrl = settingsState.asrServerUrl || 'http://10.0.0.58:8000/v1/audio/transcriptions';
                const response = await fetch(asrUrl, {
                    method: "POST",
                    body: formData
                });
                
                if (!response.ok) throw new Error("ASR request failed");
                
                const data = await response.json();
                const text = data.text?.trim();
                
                if (text && text.length > 1 && !text.includes("Thank you for watching")) {
                    currentInput = text;
                    await sendMessage(); 
                }
            } catch (e: any) {
                console.error("ASR Error:", e);
                toast.error("ASR Error: " + (e.message || e));
            }
        }
    }

    // --- WAV ENCODING UTILITIES ---
    function exportWAV(buffers: Float32Array[], length: number, sampleRate: number) {
        const buffer = new Float32Array(length);
        let offset = 0;
        for (let i = 0; i < buffers.length; i++) {
            buffer.set(buffers[i], offset);
            offset += buffers[i].length;
        }
        const dataView = encodeWAV(buffer, sampleRate);
        return new Blob([dataView], { type: 'audio/wav' });
    }

    function encodeWAV(samples: Float32Array, sampleRate: number) {
        const buffer = new ArrayBuffer(44 + samples.length * 2);
        const view = new DataView(buffer);
        writeString(view, 0, 'RIFF');
        view.setUint32(4, 36 + samples.length * 2, true);
        writeString(view, 8, 'WAVE');
        writeString(view, 12, 'fmt ');
        view.setUint32(16, 16, true);
        view.setUint16(20, 1, true);
        view.setUint16(22, 1, true);
        view.setUint32(24, sampleRate, true);
        view.setUint32(28, sampleRate * 2, true);
        view.setUint16(32, 2, true);
        view.setUint16(34, 16, true);
        writeString(view, 36, 'data');
        view.setUint32(40, samples.length * 2, true);
        floatTo16BitPCM(view, 44, samples);
        return view;
    }

    function writeString(view: DataView, offset: number, string: string) {
        for (let i = 0; i < string.length; i++) {
            view.setUint8(offset + i, string.charCodeAt(i));
        }
    }

    function floatTo16BitPCM(output: DataView, offset: number, input: Float32Array) {
        for (let i = 0; i < input.length; i++, offset += 2) {
            let s = Math.max(-1, Math.min(1, input[i]));
            output.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
        }
    }

    function initThree() {
        scene = new THREE.Scene();
        scene.background = new THREE.Color('#0a0a0a');

        const light = new THREE.DirectionalLight(0xffffff, 1.5);
        light.position.set(1, 1, 1).normalize();
        scene.add(light);
        
        const ambient = new THREE.AmbientLight(0x404040, 1.0);
        scene.add(ambient);

        camera = new THREE.PerspectiveCamera(30.0, canvasContainer.clientWidth / canvasContainer.clientHeight, 0.1, 20.0);
        camera.position.set(0.0, 1.4, 2.0);

        renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
        renderer.setSize(canvasContainer.clientWidth, canvasContainer.clientHeight);
        renderer.setPixelRatio(window.devicePixelRatio);
        canvasContainer.appendChild(renderer.domElement);

        handleResize = () => {
            if (!canvasContainer) return;
            camera.aspect = canvasContainer.clientWidth / canvasContainer.clientHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(canvasContainer.clientWidth, canvasContainer.clientHeight);
        };
        window.addEventListener('resize', handleResize);

        const animate = (timestamp: number) => {
            animationFrameId = requestAnimationFrame(animate);
            timer.update(timestamp);
            const delta = timer.getDelta();
            
            if (mixer) mixer.update(delta);
            
            // Lip Sync WebAudio updating (only while speaking)
            if (currentVrm && lipsyncNode && isSpeaking) {
                const weights = lipsyncNode.weights; 
                const expressionManager = currentVrm.expressionManager;
                if (expressionManager && weights) {
                    expressionManager.setValue('aa', weights['A'] || 0);
                    expressionManager.setValue('ih', weights['I'] || 0);
                    expressionManager.setValue('ou', weights['U'] || 0);
                    expressionManager.setValue('ee', weights['E'] || 0);
                    expressionManager.setValue('oh', weights['O'] || 0);
                }
            }

            if (currentVrm) currentVrm.update(delta);
            renderer.render(scene, camera);
        };
        requestAnimationFrame(animate);
    }

    async function preloadAnimations() {
        if (!currentVrm || !mixer) return;
        const loader = new GLTFLoader();
        loader.register((parser: any) => new VRMAnimationLoaderPlugin(parser));

        const animFiles: Record<string, string> = {
            'idle': '/vrm/idle_loop.vrma',
            'shrug': '/vrm/VRMA_03.vrma',
            'greet': '/vrm/VRMA_02.vrma',
            'peace': '/vrm/VRMA_03.vrma',
            'shoot': '/vrm/VRMA_04.vrma',
            'spin': '/vrm/VRMA_05.vrma',
            'pose': '/vrm/VRMA_06.vrma',
            'squat': '/vrm/VRMA_07.vrma',
            'full': '/vrm/VRMA_01.vrma'
        };

        // Pre-fetch all animations into memory instantly
        for (const [code, file] of Object.entries(animFiles)) {
            loader.load(file, (gltf: any) => {
                const vrmAnimations = gltf.userData.vrmAnimations;
                if (vrmAnimations && vrmAnimations.length > 0) {
                    const clip = createVRMAnimationClip(vrmAnimations[0], currentVrm!);
                    animationsMap.set(code, clip);

                    // Start the idle loop as soon as it enters the map
                    if (code === 'idle') {
                        idleAction = mixer.clipAction(clip);
                        idleAction.play();
                    }
                }
            });
        }
    }

    async function loadVRM() {
        const loader = new GLTFLoader();
        loader.register((parser: any) => new VRMLoaderPlugin(parser));
        loader.register((parser: any) => new VRMAnimationLoaderPlugin(parser));

        loader.load(
            '/vrm/avatar.vrm',
            (gltf: any) => {
                const vrm = gltf.userData.vrm as any;
                if (scene) scene.add(vrm.scene);
                currentVrm = vrm;
                vrm.scene.rotation.y = Math.PI;

                mixer = new THREE.AnimationMixer(vrm.scene);
                preloadAnimations(); // Preload all animations into the cache map
                isLoading = false;
            },
            undefined,
            (error: ErrorEvent) => {
                console.error(error);
                isLoading = false;
            }
        );
    }

    function playAnim(animCode: string) {
        if (!currentVrm || !mixer) return;

        let code = animCode.toLowerCase();
        if (!animationsMap.has(code)) code = 'pose'; // Fallback

        const clip = animationsMap.get(code);
        if (!clip) return;

        // If another action is already playing, fade it out smoothly
        if (activeAction) {
            activeAction.fadeOut(0.2);
        }

        activeAction = mixer.clipAction(clip);
        activeAction.setLoop(THREE.LoopOnce, 1);
        activeAction.clampWhenFinished = true;

        // Smoothly blend from the idle loop into the new action
        if (idleAction) {
            activeAction.crossFadeFrom(idleAction, 0.2, false);
        }

        activeAction.reset().fadeIn(0.2).play();

        // Listen for completion to smoothly blend back to idle
        const onFinished = (e: any) => {
            if (e.action === activeAction) {
                mixer.removeEventListener('finished', onFinished);
                if (idleAction) {
                    idleAction.reset().fadeIn(0.2).play();
                    activeAction.fadeOut(0.2);
                }
                activeAction = null;
            }
        };
        mixer.addEventListener('finished', onFinished);
    }

    function resetMouth() {
        if (currentVrm && currentVrm.expressionManager) {
            ['aa', 'ih', 'ou', 'ee', 'oh'].forEach(vowel => {
                currentVrm.expressionManager.setValue(vowel, 0);
            });
        }
    }

    async function generateAndPlayTTS(text: string) {
        if (muteTts) return;
        await initLipSync();
        isSpeaking = true; 
        
        await playSmartTTS(text, settingsState.ttsServerUrl, (animCode) => {
            playAnim(animCode); 
        }, settingsState.targetLanguage);
        
        isSpeaking = false; 
        resetMouth();       
    }

    async function sendMessage(audioBase64?: string) {
        if (!currentInput.trim() && !audioBase64) return;

        let displayInput = currentInput.trim();
        if (!displayInput && audioBase64) {
            displayInput = "🎤 [Audio Message]";
        }

        chatHistory = [...chatHistory, { role: 'user', content: displayInput, audioBase64: audioBase64 }];
        currentInput = '';
        isChatting = true;
        
        if (chatInputRef) {
            chatInputRef.style.height = 'auto'; // Reset auto-resize height
        }

        if (audioBase64) {
            setTimeout(() => chatInputRef?.blur(), 10);
        } else if (viewMode === 'chat') {
            setTimeout(() => chatInputRef?.focus({ preventScroll: true }), 10);
        } else if (viewMode === 'split' && window.innerWidth < 768) {
            setTimeout(() => chatInputRef?.blur(), 10);
        }

        await tick();
        if (chatScrollContainer) {
            chatScrollContainer.scrollTop = chatScrollContainer.scrollHeight;
        }

        try {
            let audioCount = 0;
            const historyPayload = chatHistory.slice().reverse().map(h => {
                let msg: any = { role: h.role, content: h.content };
                if (h.audioBase64 && audioCount < 2) {
                    msg.audio_base64 = h.audioBase64;
                    audioCount++;
                }
                return msg;
            }).reverse();

            const response = (await invoke('chat_with_avatar', {
                history: historyPayload,
                model: settingsState.activeModel,
                language: settingsState.targetLanguage,
                activeTextbook: activeTextbook,
                activePage: activePage,
                activeTheme: mapFollowMode ? activeThemeId : null,
                audioBase64: audioBase64 || null
            })) as { response: string; idealized_correction?: string; context_summary?: string };

            if (response.idealized_correction && response.idealized_correction !== "null" && response.idealized_correction.trim() !== "") {
                chatHistory[chatHistory.length - 1].correction = response.idealized_correction;
            }

            if (response.context_summary) {
                const lastUserMsg = chatHistory[chatHistory.length - 1];
                chatHistory = [
                    { role: 'system', content: `Context Compressed: ${response.context_summary}` },
                    lastUserMsg,
                    { role: 'assistant', content: response.response }
                ];
            } else {
                chatHistory = [...chatHistory, { role: 'assistant', content: response.response }];
            }
            
            await tick();
            if (chatScrollContainer) {
                chatScrollContainer.scrollTo({ top: chatScrollContainer.scrollHeight, behavior: 'smooth' });
            }

            await generateAndPlayTTS(response.response);

        } catch (e) {
            console.error('Chat failed:', e);
            chatHistory = [...chatHistory, { role: 'assistant', content: 'Sorry, I encountered an error. ' + e }];
        } finally {
            isChatting = false;
        }
    }
</script>

<div bind:this={rootScrollContainer} class="h-[100dvh] w-full flex flex-col md:flex-row p-2 md:p-6 gap-2 md:gap-6 max-w-7xl mx-auto relative overflow-hidden">
    {#snippet viewModeControls()}
        <!-- Desktop view -->
        <div class="hidden md:flex items-center bg-zinc-900/80 backdrop-blur-md rounded-xl p-1 border border-zinc-700 shadow-lg">
            <button onclick={() => setViewMode('split')} class="px-3 py-1 text-sm font-medium rounded-lg transition-colors {viewMode === 'split' ? 'bg-zinc-700 text-yellow-200' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}">Split</button>
            <button onclick={() => setViewMode('avatar')} class="px-3 py-1 text-sm font-medium rounded-lg transition-colors {viewMode === 'avatar' ? 'bg-zinc-700 text-yellow-200' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}">Avatar</button>
            <button onclick={() => setViewMode('chat')} class="px-3 py-1 text-sm font-medium rounded-lg transition-colors {viewMode === 'chat' ? 'bg-zinc-700 text-yellow-200' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}">Chat</button>
        </div>

        <!-- Mobile view -->
        <div class="md:hidden">
            <div class="relative bg-zinc-900/80 backdrop-blur-md rounded-xl border border-zinc-700 shadow-lg">
                <select 
                    class="appearance-none bg-transparent py-2 pl-3 pr-8 text-sm font-medium text-yellow-200 focus:outline-none focus:ring-2 focus:ring-yellow-200/50 rounded-xl cursor-pointer"
                    value={viewMode}
                    onchange={(e) => setViewMode(e.currentTarget.value as any)}
                >
                    <option value="split" class="bg-zinc-900 text-zinc-200">Split</option>
                    <option value="avatar" class="bg-zinc-900 text-zinc-200">Avatar</option>
                    <option value="chat" class="bg-zinc-900 text-zinc-200">Chat</option>
                </select>
                <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-zinc-400">
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>
                </div>
            </div>
        </div>
    {/snippet}

    {#if showReviewModal && reviewThemeCandidate}
        <div class="absolute inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm rounded-3xl p-4">
            <div class="bg-zinc-900 border border-zinc-700 rounded-3xl p-8 max-w-md shadow-2xl text-center">
                <div class="w-16 h-16 bg-purple-500/20 text-purple-300 rounded-full flex items-center justify-center mx-auto mb-4 text-3xl">
                    🧠
                </div>
                <h2 class="text-2xl font-bold text-zinc-100 mb-2">Daily Review Time!</h2>
                <p class="text-zinc-400 mb-6">Would you like to do a quick review session covering <strong class="text-yellow-200">"{reviewThemeCandidate.name}"</strong> before continuing your main curriculum?</p>
                <div class="flex gap-4 justify-center">
                    <button 
                        class="px-6 py-3 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 font-bold rounded-xl transition-colors"
                        onclick={() => showReviewModal = false}
                    >
                        Skip
                    </button>
                    <button 
                        class="px-6 py-3 bg-purple-600 hover:bg-purple-500 text-white font-bold rounded-xl transition-colors shadow-lg"
                        onclick={() => {
                            activeThemeId = reviewThemeCandidate!.id;
                            mapFollowMode = true;
                            showReviewModal = false;
                        }}
                    >
                        Yes, Let's Review
                    </button>
                </div>
            </div>
        </div>
    {/if}
    {#if isLoading}
        <div class="absolute inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950/80 backdrop-blur-sm rounded-3xl m-6">
            <div class="w-16 h-16 border-4 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
            <p class="mt-4 text-yellow-200 font-bold tracking-widest uppercase">Initializing Canvas...</p>
        </div>
    {/if}

    <div class="shrink-0 min-w-0 {viewMode === 'avatar' ? 'flex-1 h-full' : (showHandwritingPanel ? 'h-[22vh]' : 'h-[40vh]')} md:h-auto md:flex-1 flex flex-col gap-2 md:gap-6 min-h-0 transition-all duration-300 {viewMode === 'chat' ? 'hidden' : ''}">
        <div class="flex-1 bg-zinc-900 rounded-3xl overflow-hidden border border-zinc-800 shadow-2xl relative {activeTextbook ? 'max-h-[50%]' : ''}">
            <div bind:this={canvasContainer} bind:clientWidth={containerWidth} bind:clientHeight={containerHeight} class="w-full h-full"></div>
            <div class="absolute top-4 left-4 flex items-center gap-4">
                <div class="flex items-center gap-3 bg-zinc-950/50 p-2 rounded-xl backdrop-blur-md border border-zinc-800/50">
                    <div class="h-3 w-3 rounded-full {currentVrm ? 'bg-green-500 shadow-[0_0_10px_#22c55e]' : 'bg-yellow-200 animate-pulse'}"></div>
                    <span class="text-xs font-medium text-zinc-400 hidden sm:inline">{currentVrm ? 'Ready' : 'Loading...'}</span>
                </div>
                
                <select 
                    class="bg-zinc-800 text-xs border border-zinc-700 text-zinc-300 rounded-lg px-2 py-1 focus:outline-none focus:border-yellow-200/50 cursor-pointer"
                    onchange={(e) => setPlaybackRate(parseFloat(e.currentTarget.value))}
                >
                    <option value="1.0">1.0x Speed</option>
                    <option value="0.75">0.75x Speed</option>
                    <option value="0.5">0.5x Speed</option>
                </select>
            </div>
            
            <div class="absolute top-4 right-4 z-50">
                {@render viewModeControls()}
            </div>
            
            {#if viewMode === 'avatar'}
                <div class="absolute bottom-8 left-1/2 -translate-x-1/2 z-10 flex gap-4">
                    {#if isChatting}
                        <div class="absolute -top-10 left-1/2 -translate-x-1/2 flex items-center gap-2 bg-zinc-900/80 backdrop-blur-md px-4 py-2 rounded-full border border-zinc-700 shadow-xl">
                            <div class="w-2 h-2 bg-yellow-400 rounded-full animate-bounce"></div>
                            <div class="w-2 h-2 bg-yellow-400 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
                            <div class="w-2 h-2 bg-yellow-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
                        </div>
                    {:else if isMicActive}
                        <div class="absolute -top-6 left-1/2 -translate-x-1/2 flex items-center gap-2 whitespace-nowrap bg-zinc-900/80 px-3 py-1 rounded-full border border-zinc-700">
                            <span class="relative flex h-2.5 w-2.5">
                                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                                <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-green-500"></span>
                            </span>
                            <span class="text-[10px] font-bold text-zinc-300 uppercase tracking-wider">
                                {vadState === 'speaking' ? 'Listening...' : 'Processing...'}
                            </span>
                        </div>
                    {/if}
                    <button 
                        type="button"
                        onclick={toggleMic}
                        disabled={isChatting}
                        class="p-5 rounded-full transition-colors border-2 shadow-2xl {isChatting ? 'opacity-50 cursor-not-allowed bg-zinc-800 text-zinc-500 border-zinc-700' : (isMicActive ? 'bg-red-500/20 text-red-500 border-red-500/50 hover:bg-red-500/30' : 'bg-zinc-800/80 backdrop-blur-md text-zinc-300 border-zinc-700 hover:bg-zinc-700 hover:text-yellow-200')}"
                        title={isMicActive ? "Stop Microphone" : "Start Voice Interaction"}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            {#if isMicActive}
                                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                            {:else}
                                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"></path>
                                <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
                                <line x1="12" y1="19" x2="12" y2="22"></line>
                            {/if}
                        </svg>
                    </button>
                </div>
            {/if}
        </div>

        {#if activeTextbook}
            <div class="flex-1 bg-zinc-900 rounded-3xl overflow-hidden border border-zinc-800 shadow-2xl relative">
                {#if textbookError}
                    <div class="w-full h-full flex flex-col items-center justify-center gap-3 p-4 text-center">
                        <span class="text-3xl">📄</span>
                        <p class="text-zinc-400 text-sm">Failed to load <strong class="text-zinc-200">{activeTextbook}</strong>.</p>
                        <button 
                            class="mt-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-yellow-200 text-sm font-bold rounded-lg transition-colors"
                            onclick={() => { activeTextbook = null; textbookSrc = null; textbookError = false; }}
                        >Dismiss</button>
                    </div>
                {:else if textbookSrc}
                    <embed 
                        src={textbookSrc} 
                        class="w-full h-full" 
                        type="application/pdf"
                        onerror={() => textbookError = true}
                    />
                {:else}
                    <div class="w-full h-full flex items-center justify-center">
                        <div class="w-8 h-8 border-2 border-yellow-200 border-t-transparent rounded-full animate-spin"></div>
                    </div>
                {/if}
            </div>
        {/if}
    </div>

    <div class="flex-1 min-w-0 flex flex-col bg-zinc-900 rounded-2xl md:rounded-3xl border border-zinc-800 shadow-xl overflow-hidden md:w-1/3 min-h-0 {showHandwritingPanel ? 'xl:w-[600px]' : ''} {viewMode === 'avatar' ? 'hidden' : ''} {viewMode === 'chat' ? 'md:w-full' : ''}">
        
        <div class="p-4 md:p-6 border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-md flex justify-between items-center shrink-0">
            <div>
                <h2 class="text-lg md:text-xl font-bold text-yellow-200 leading-none">Practice</h2>
                <p class="text-xs text-zinc-400 mt-1 hidden sm:block">Roleplay and get subtle corrections.</p>
            </div>
            
            <div class="flex gap-2 items-center relative group">
                {#if viewMode === 'chat'}
                    {@render viewModeControls()}
                {/if}
                <button 
                    class="p-2 rounded-xl transition-colors font-bold text-xs uppercase flex items-center gap-1 {muteTts ? 'bg-red-500/20 text-red-400 hover:bg-red-500/30' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}"
                    onclick={() => muteTts = !muteTts}
                    title={muteTts ? "TTS is Muted" : "TTS is Active"}
                >
                    <span>{muteTts ? '🔇' : '🔊'}</span>
                    <span class="hidden sm:inline">TTS</span>
                </button>
                {#if isIngesting}
                    <div class="w-6 h-6 rounded-full border-2 border-yellow-200 border-t-transparent animate-spin"></div>
                {:else}
                    <button 
                        class="p-2 rounded-xl transition-colors font-bold text-xs uppercase flex items-center gap-1 {mapFollowMode ? 'bg-yellow-500/20 text-yellow-200 hover:bg-yellow-500/30' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}"
                        onclick={() => mapFollowMode = !mapFollowMode}
                        title={mapFollowMode ? "Curriculum Steering Active" : "Free Conversation Mode"}
                    >
                        <span>🧭</span>
                        <span class="hidden sm:inline">{mapFollowMode ? 'Map: ON' : 'Map: OFF'}</span>
                    </button>
                    
                    <button class="p-2 bg-zinc-800 hover:bg-zinc-700 text-yellow-200 rounded-xl transition-colors" title="Textbook Settings">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/></svg>
                    </button>
                    <div class="absolute right-0 top-full mt-2 w-56 bg-zinc-800 border border-zinc-700 rounded-xl shadow-2xl overflow-hidden opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-10">
                        <div class="p-2 max-h-48 overflow-y-auto">
                            <button 
                                class="w-full text-left px-3 py-2 text-sm rounded-lg hover:bg-zinc-700 transition-colors {activeTextbook === null ? 'bg-zinc-700/50 text-yellow-200' : 'text-zinc-300'}"
                                onclick={() => { activeTextbook = null; resolveTextbookSrc(null); }}
                            >
                                [ None ]
                            </button>
                            {#each textbooks as book}
                                <button 
                                    class="w-full text-left px-3 py-2 text-sm rounded-lg hover:bg-zinc-700 transition-colors {activeTextbook === book ? 'bg-zinc-700/50 text-yellow-200' : 'text-zinc-300'} truncate"
                                    onclick={() => { activeTextbook = book; resolveTextbookSrc(book); }}
                                    title={book}
                                >
                                    {book}
                                </button>
                            {/each}
                        </div>
                        <div class="p-2 border-t border-zinc-700">
                            <button 
                                class="w-full text-left px-3 py-2 text-sm text-green-400 hover:bg-zinc-700 rounded-lg transition-colors flex items-center gap-2"
                                onclick={handleUpload}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                                Upload New Textbook
                            </button>
                        </div>
                    </div>
                {/if}
            </div>
        </div>

        <div class="flex-1 overflow-y-auto p-4 md:p-6 space-y-6 flex flex-col min-h-[60px] md:min-h-0" bind:this={chatScrollContainer}>
            {#if chatHistory.length === 0}
                <div class="flex-1 flex items-center justify-center text-center">
                    <p class="text-zinc-500 text-sm">Start the conversation by saying hello in your target language!</p>
                </div>
            {:else}
                <div class="mt-auto"></div>
            {/if}

            {#each chatHistory as msg}
                {#if msg.role === 'system'}
                    <div class="flex justify-center w-full my-4">
                        <div class="px-4 py-2 bg-purple-500/10 border border-purple-500/30 text-purple-300 rounded-xl text-xs sm:text-sm max-w-[80%] text-center shadow-sm">
                            <span class="mr-2">🗜️</span>{msg.content}
                        </div>
                    </div>
                {:else}
                    <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
                        <div class="px-4 py-2 sm:px-5 sm:py-3 rounded-2xl max-w-[85%] sm:max-w-[80%] shadow-md {msg.role === 'user' ? 'bg-yellow-200 text-zinc-900 rounded-tr-sm' : 'bg-zinc-800 text-zinc-100 border border-zinc-700 rounded-tl-sm'}">
                            {#if msg.role === 'assistant'}
                                <div class="text-sm sm:text-base leading-relaxed prose prose-invert prose-sm max-w-none prose-p:my-1 prose-headings:my-2 prose-a:text-yellow-200">
                                    {@html marked.parse(msg.content.replace(/<start_of_turn>\s*model\s*/g, '').replace(/<start_of_turn>/g, '').replace(/<end_of_turn>/g, '').replace(/[<\[](?:anim:[a-zA-Z0-9_-]+|laugh|breath|sigh|surprise|sad|cough|cry|whisper|yell|gasp|sneeze|sniff)[>\]]/g, ''))}
                                </div>
                            {:else}
                                <p class="text-sm sm:text-base leading-relaxed">{msg.content}</p>
                                {#if msg.audioBase64}
                                    <audio src="data:audio/wav;base64,{msg.audioBase64}" controls class="mt-2 h-8 max-w-full min-w-[200px] opacity-90"></audio>
                                {/if}
                            {/if}
                        </div>
                        
                        {#if msg.role === 'user' && msg.correction}
                            <div class="mt-2 mr-2 max-w-[80%] bg-zinc-900/80 px-3 py-2 rounded-xl border border-green-500/30">
                                <p class="text-[10px] sm:text-xs font-semibold text-green-500 uppercase tracking-widest mb-0.5">Correction</p>
                                <p class="text-xs sm:text-sm text-zinc-400 italic">{msg.correction}</p>
                            </div>
                        {/if}
                    </div>
                {/if}
            {/each}

            {#if isChatting}
                <div class="flex items-start">
                    <div class="px-5 py-3 bg-zinc-800 rounded-2xl rounded-tl-sm border border-zinc-700 flex items-center gap-2">
                        <div class="w-2 h-2 bg-zinc-500 rounded-full animate-bounce"></div>
                        <div class="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
                        <div class="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
                    </div>
                </div>
            {/if}
        </div>

        <div class="p-3 md:p-4 border-t border-zinc-800 bg-zinc-900/80 shrink-0 shadow-[0_-10px_20px_rgba(0,0,0,0.3)] z-10">
            {#if isMicActive}
                <div class="flex items-center gap-2 mb-2 px-1">
                    <span class="relative flex h-2.5 w-2.5">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-green-500"></span>
                    </span>
                    <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">
                        {vadState === 'speaking' ? 'Listening...' : 'Processing...'}
                    </span>
                </div>
            {/if}

            <form class="flex gap-2" onsubmit={(e) => { e.preventDefault(); sendMessage(); }}>
                <button 
                    type="button"
                    onclick={() => { 
                        showHandwritingPanel = !showHandwritingPanel; 
                        if (showHandwritingPanel) { 
                            setTimeout(() => {
                                initDrawingCanvas();
                                if (chatScrollContainer) chatScrollContainer.scrollTo({ top: chatScrollContainer.scrollHeight, behavior: 'smooth' });
                            }, 50); 
                        } 
                    }}
                    class="p-3 rounded-xl transition-colors border flex items-center justify-center shrink-0
                           {showHandwritingPanel ? 'bg-yellow-200 text-zinc-900 border-yellow-200' : 'bg-zinc-800 text-zinc-400 border-zinc-700 hover:bg-zinc-700 hover:text-yellow-200'}"
                    title="Handwriting Keyboard"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path><path d="m15 5 4 4"></path></svg>
                </button>
                <button 
                    type="button"
                    onclick={toggleMic}
                    class="p-3 rounded-xl transition-colors border flex items-center justify-center shrink-0
                           {isMicActive ? 'bg-red-500/20 text-red-500 border-red-500/50 hover:bg-red-500/30' : 'bg-zinc-800 text-zinc-400 border-zinc-700 hover:bg-zinc-700 hover:text-yellow-200'}"
                    title={isMicActive ? "Stop Microphone" : "Start Voice Interaction"}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        {#if isMicActive}
                            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                        {:else}
                            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"></path>
                            <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
                            <line x1="12" y1="19" x2="12" y2="22"></line>
                        {/if}
                    </svg>
                </button>

                <textarea 
                    bind:this={chatInputRef}
                    bind:value={currentInput}
                    placeholder={isMicActive ? "Speak now..." : "Message..."}
                    class="flex-1 min-w-0 bg-zinc-950 border border-zinc-800 text-zinc-100 rounded-xl px-3 sm:px-4 py-2 sm:py-3 focus:outline-none focus:border-yellow-200/50 transition-colors text-sm sm:text-base resize-none"
                    rows="1"
                    style="max-height: 120px;"
                    oninput={(e) => {
                        e.currentTarget.style.height = 'auto';
                        e.currentTarget.style.height = (e.currentTarget.scrollHeight) + 'px';
                    }}
                    onkeydown={(e) => {
                        if (e.key === 'Enter' && !e.shiftKey) {
                            e.preventDefault();
                            if (!isChatting && (currentInput.trim() || isMicActive)) {
                                sendMessage();
                                if (viewMode === 'split' && window.innerWidth < 768) {
                                    e.currentTarget.blur();
                                }
                            }
                        }
                    }}
                ></textarea>
                <button 
                    type="submit"
                    onpointerdown={(e) => {
                        if (viewMode === 'chat' || window.innerWidth >= 768) {
                            e.preventDefault();
                        }
                    }}
                    disabled={isChatting || (!currentInput.trim() && !isMicActive)}
                    class="bg-yellow-200 hover:bg-yellow-300 disabled:opacity-50 text-zinc-900 px-4 sm:px-6 py-2 sm:py-3 rounded-xl font-bold transition-colors shrink-0 text-sm sm:text-base"
                >
                    Send
                </button>
            </form>
        </div>

        {#if showHandwritingPanel}
            <div class="p-3 md:p-4 border-t border-zinc-800 bg-zinc-900 flex flex-col gap-4 shrink-0 shadow-inner overflow-y-auto max-h-[45vh] md:max-h-none relative z-0">
                <div class="flex flex-col xl:flex-row gap-4 items-center xl:items-stretch">
                    <div class="flex flex-col items-center gap-2 w-full xl:w-auto shrink-0">
                        <canvas
                            bind:this={drawingCanvas}
                            width="280"
                            height="280"
                            class="bg-white rounded-2xl shadow-inner border-2 border-zinc-700 touch-none cursor-crosshair w-[200px] h-[200px] sm:w-[240px] sm:h-[240px]"
                            onmousedown={startDrawing}
                            onmousemove={draw}
                            onmouseup={stopDrawing}
                            onmouseleave={stopDrawing}
                            ontouchstart={startDrawing}
                            ontouchmove={draw}
                            ontouchend={stopDrawing}
                        ></canvas>
                        <div class="flex gap-2 w-[200px] sm:w-[240px]">
                            <button onclick={clearDrawingCanvas} class="flex-1 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-lg text-xs sm:text-sm font-medium transition-colors">Clear</button>
                            <button onclick={recognizeStroke} disabled={isRecognizing} class="flex-1 py-2 bg-yellow-200/20 hover:bg-yellow-200/30 text-yellow-200 border border-yellow-200/30 rounded-lg text-xs sm:text-sm font-bold transition-colors disabled:opacity-50">
                                {isRecognizing ? '...' : 'Recognize'}
                            </button>
                        </div>
                        {#if strokeStatusMsg}
                            <p class="text-[10px] sm:text-xs text-center text-zinc-400 mt-1 h-3">{strokeStatusMsg}</p>
                        {:else}
                            <div class="h-3 mt-1"></div>
                        {/if}
                    </div>

                    <div class="flex-1 flex flex-col gap-3 sm:gap-4 bg-zinc-950 rounded-2xl p-3 sm:p-4 border border-zinc-800 relative w-full min-w-0 shrink-0">
                        <div class="text-center shrink-0">
                            <h3 class="text-xs sm:text-sm font-semibold text-zinc-400 uppercase tracking-widest">Block</h3>
                            <div class="text-4xl sm:text-5xl font-black text-yellow-200 h-14 sm:h-16 flex items-center justify-center mt-1">
                                {composedHangul || '-'}
                            </div>
                        </div>

                        <div class="flex justify-between gap-1 sm:gap-2 shrink-0">
                            <button onclick={() => { currentSlot = 'initial'; clearDrawingCanvas(); }} class="flex-1 p-2 sm:p-3 rounded-xl border flex flex-col items-center transition-colors {currentSlot === 'initial' ? 'bg-yellow-200/10 border-yellow-200/50' : 'bg-zinc-900 border-zinc-800 hover:bg-zinc-800'}">
                                <span class="text-[10px] sm:text-xs text-zinc-500 uppercase">Initial</span>
                                <span class="text-lg sm:text-xl font-bold {blockInitial ? 'text-zinc-200' : 'text-zinc-600'}">{blockInitial || '-'}</span>
                            </button>
                            <button onclick={() => { currentSlot = 'vowel'; clearDrawingCanvas(); }} class="flex-1 p-2 sm:p-3 rounded-xl border flex flex-col items-center transition-colors {currentSlot === 'vowel' ? 'bg-yellow-200/10 border-yellow-200/50' : 'bg-zinc-900 border-zinc-800 hover:bg-zinc-800'}">
                                <span class="text-[10px] sm:text-xs text-zinc-500 uppercase">Vowel</span>
                                <span class="text-lg sm:text-xl font-bold {blockVowel ? 'text-zinc-200' : 'text-zinc-600'}">{blockVowel || '-'}</span>
                            </button>
                            <button onclick={() => { currentSlot = 'final'; clearDrawingCanvas(); }} class="flex-1 p-2 sm:p-3 rounded-xl border flex flex-col items-center transition-colors {currentSlot === 'final' ? 'bg-yellow-200/10 border-yellow-200/50' : 'bg-zinc-900 border-zinc-800 hover:bg-zinc-800'}">
                                <span class="text-[10px] sm:text-xs text-zinc-500 uppercase">Final</span>
                                <span class="text-lg sm:text-xl font-bold {blockFinal ? 'text-zinc-200' : 'text-zinc-600'}">{blockFinal || '-'}</span>
                            </button>
                        </div>

                        <div class="flex gap-2 mt-auto shrink-0">
                            <button onclick={clearHangulBlock} class="px-3 sm:px-4 py-2 sm:py-3 bg-zinc-800 hover:bg-red-500/20 text-zinc-300 hover:text-red-400 rounded-xl text-xs sm:text-sm font-medium transition-colors">Clear</button>
                            <button onclick={() => currentInput += " "} class="px-3 sm:px-4 py-2 sm:py-3 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-xl text-xs sm:text-sm font-medium transition-colors">Space</button>
                            <button onclick={commitHangulBlock} disabled={!composedHangul} class="flex-1 py-2 sm:py-3 bg-yellow-200 hover:bg-yellow-300 text-zinc-900 rounded-xl text-xs sm:text-sm font-bold transition-colors disabled:opacity-50 whitespace-nowrap">
                                Enter
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        {/if}
    </div>
</div>
