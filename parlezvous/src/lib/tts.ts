import toast from 'svelte-french-toast';
import { invoke } from '@tauri-apps/api/core';

export let ttsAudioContext: AudioContext | null = null;
export let ttsPlaybackRate: number = 1.0;
let mediaElementSource: MediaElementAudioSourceNode | null = null;
let hiddenAudioElement: HTMLAudioElement | null = null;
export let lipsyncNodeRef: any = null;

// Initialize the shared audio context and the hidden audio element
export function initTTSAudio() {
    if (ttsAudioContext) return;
    ttsAudioContext = new AudioContext();

    hiddenAudioElement = document.createElement('audio');
    hiddenAudioElement.preservesPitch = true;
    hiddenAudioElement.style.display = 'none';
    document.body.appendChild(hiddenAudioElement);

    mediaElementSource = ttsAudioContext.createMediaElementSource(hiddenAudioElement);
    mediaElementSource.connect(ttsAudioContext.destination);
}

export function setLipSyncNode(node: any) {
    lipsyncNodeRef = node;
    if (mediaElementSource && lipsyncNodeRef) {
        mediaElementSource.connect(lipsyncNodeRef);
    }
}

export function setPlaybackRate(rate: number) {
    ttsPlaybackRate = rate;
    if (hiddenAudioElement) {
        hiddenAudioElement.playbackRate = rate;
    }
}

export function sanitizeTTSInput(text: string): string {
    // Strip markdown formatting (*, _, ~, `)
    let sanitized = text.replace(/[*_~`]/g, '');
    // Strip emojis
    sanitized = sanitized.replace(/[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F700}-\u{1F77F}\u{1F780}-\u{1F7FF}\u{1F800}-\u{1F8FF}\u{1F900}-\u{1F9FF}\u{1FA00}-\u{1FA6F}\u{1FA70}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/gu, '');
    // Replace newlines with spaces
    sanitized = sanitized.replace(/\n/g, ' ');
    // Strip LLM control tokens
    sanitized = sanitized.replace(/<start_of_turn>\s*model\s*/g, '');
    sanitized = sanitized.replace(/<start_of_turn>/g, '');
    sanitized = sanitized.replace(/<end_of_turn>/g, '');
    return sanitized;
}

// Split text into chunks using the native browser Intl API.
// Produces paired segments: 2 sentences per chunk, or 2 graphemes
// per chunk for languages where sentence segmentation yields no splits.
export function chunkSentences(text: string, locale: string = 'fr'): string[] {
    if (!text.trim()) return [];

    // Try sentence-level segmentation first
    const sentenceSegmenter = new Intl.Segmenter(locale, { granularity: 'sentence' });
    const sentences = Array.from(sentenceSegmenter.segment(text))
        .map(s => s.segment.trim())
        .filter(s => s.length > 0);

    return pairSegments(sentences);
}

function pairSegments(segments: string[]): string[] {
    const paired: string[] = [];
    for (let i = 0; i < segments.length; i += 2) {
        const chunk = i + 1 < segments.length
            ? segments[i] + ' ' + segments[i + 1]
            : segments[i];
        paired.push(chunk.trim());
    }
    return paired.filter(s => s.length > 0);
}

export async function playTTS(
    text: string,
    ttsServerUrl: string,
    onAnim?: (animCode: string, delayMs: number) => void,
    locale: string = 'fr'
) {
    if (!ttsAudioContext) initTTSAudio();

    if (ttsAudioContext?.state === 'suspended') {
        await ttsAudioContext.resume();
    }

    const sanitized = sanitizeTTSInput(text);

    // 1. Replace <anim:code> with a hidden anchor BEFORE chunking
    const animRegex = /[<\[]anim:([a-zA-Z0-9_-]+)[>\]]/g;
    const textWithAnchors = sanitized.replace(animRegex, '|||ANIM_$1|||');

    // 2. Chunk the text. The anchors safely ride along inside their specific chunk!
    const chunks = chunkSentences(textWithAnchors, locale);

    // Process chunks sequentially
    for (const chunk of chunks) {
        if (!chunk.trim()) continue;

        // 3. Extract the animation meant for THIS specific audio chunk
        const anchorRegex = /\|\|\|ANIM_([a-zA-Z0-9_-]+)\|\|\|/g;
        const animsForThisChunk: string[] = [];
        let match;

        while ((match = anchorRegex.exec(chunk)) !== null) {
            animsForThisChunk.push(match[1]);
        }

        // 4. Remove the anchors so the TTS engine gets clean text
        const cleanChunk = chunk.replace(anchorRegex, '').trim();

        if (!cleanChunk) continue; // Skip if chunk was ONLY an animation tag

        try {
            // Offload TTS request to Rust backend via the generate_tts_audio command
            const rawAudioBytes: number[] = await invoke('generate_tts_audio', {
                text: cleanChunk,
                language: locale,
                voice: 'sohee', // The backend will ignore this for Supertonic and use F1.json
                speed: ttsPlaybackRate,
                url: ttsServerUrl
            });

            const audioBytes = new Uint8Array(rawAudioBytes);
            const arrayBuffer = audioBytes.buffer;
            const blob = new Blob([arrayBuffer], { type: 'audio/wav' });
            const url = URL.createObjectURL(blob);

            if (hiddenAudioElement) {
                hiddenAudioElement.src = url;
                hiddenAudioElement.playbackRate = ttsPlaybackRate;
                hiddenAudioElement.load(); // Force the browser to evaluate the file

                try {
                    // .play() returns a Promise that instantly REJECTS if the audio is corrupted/empty
                    await hiddenAudioElement.play();

                    // 5. TRIGGER THE ANIMATION IMMEDIATELY (0ms delay)
                    if (onAnim && animsForThisChunk.length > 0) {
                        onAnim(animsForThisChunk[0], 0);
                    }

                    // Wait for the audio chunk to finish before fetching the next one
                    await new Promise<void>((resolve) => {
                        // Resolve normally when finished
                        hiddenAudioElement!.onended = () => resolve();

                        // IF the audio crashes mid-playback, resolve anyway to prevent deadlocks!
                        hiddenAudioElement!.onerror = (e) => {
                            console.warn("Audio playback interrupted", e);
                            resolve();
                        };
                    });

                } catch (playError) {
                    console.warn("Skipping invalid audio chunk:", playError);
                    // The WAV was corrupted or empty (common on the final sentence). 
                    // We just catch the error and do nothing, allowing the loop to finish gracefully!
                } finally {
                    URL.revokeObjectURL(url);
                }
            }

        } catch (error) {
            console.error("TTS Stream Error:", error);
        }
    }
}

export async function playSupertonicTTS(
    text: string,
    onAnim?: (animCode: string, delayMs: number) => void,
    locale: string = 'fr'
) {
    if (!ttsAudioContext) initTTSAudio();

    if (ttsAudioContext?.state === 'suspended') {
        await ttsAudioContext.resume();
    }

    const sanitized = sanitizeTTSInput(text);
    const animRegex = /[<\[]anim:([a-zA-Z0-9_-]+)[>\]]/g;
    const textWithAnchors = sanitized.replace(animRegex, '|||ANIM_$1|||');
    const chunks = chunkSentences(textWithAnchors, locale);

    // Tauri invoke is dynamic to avoid breaking non-Tauri web builds
    let invoke: any;
    if ((window as any).__TAURI_INTERNALS__) {
        try {
            const core = await import('@tauri-apps/api/core');
            invoke = core.invoke;
        } catch (e) {
            console.warn("Could not load tauri api", e);
            return;
        }
    } else {
        return;
    }

    for (const chunk of chunks) {
        if (!chunk.trim()) continue;

        const anchorRegex = /\|\|\|ANIM_([a-zA-Z0-9_-]+)\|\|\|/g;
        const animsForThisChunk: string[] = [];
        let match;

        while ((match = anchorRegex.exec(chunk)) !== null) {
            animsForThisChunk.push(match[1]);
        }

        const cleanChunk = chunk.replace(anchorRegex, '').trim();
        if (!cleanChunk) continue;

        try {
            const response = await invoke('generate_supertonic_tts', {
                text: cleanChunk,
                lang: locale,
                speed: ttsPlaybackRate,
                steps: 6
            }) as { audioBytes: number[]; sampleRate: number };

            // The Rust code returns an array of bytes representing WAV
            const audioBytes = new Uint8Array(response.audioBytes);

            // To create a valid WAV blob, we need to add a WAV header, but wait!
            // Supertonic Rust returns raw PCM samples, or does it return full WAV bytes?
            // The Rust code: write_wav_file writes WAV. But we returned `pcm_data`!
            // Actually, we returned `pcm_data`. So it's raw 16-bit PCM.
            // We need to convert raw PCM to an AudioBuffer, or we should return a full WAV file from Rust.
            // Since we returned raw PCM bytes from Rust without WAV header, we must decode it manually or add a WAV header!
            // Let's create an AudioBuffer directly from raw PCM bytes instead.

            const ctx = ttsAudioContext!;
            const numSamples = audioBytes.length / 2; // 16-bit
            const audioBuffer = ctx.createBuffer(1, numSamples, response.sampleRate || 22050);
            const channelData = audioBuffer.getChannelData(0);

            const dataView = new DataView(audioBytes.buffer);
            for (let i = 0; i < numSamples; i++) {
                // Read 16-bit signed integer (little-endian)
                const sample16 = dataView.getInt16(i * 2, true);
                // Convert to float [-1.0, 1.0]
                channelData[i] = sample16 / 32768.0;
            }

            // Create AudioBufferSourceNode
            const source = ctx.createBufferSource();
            source.buffer = audioBuffer;
            source.playbackRate.value = ttsPlaybackRate;

            // Connect to hidden audio element destination
            source.connect(ctx.destination);
            if (lipsyncNodeRef) {
                source.connect(lipsyncNodeRef);
            }

            source.start();

            if (onAnim && animsForThisChunk.length > 0) {
                onAnim(animsForThisChunk[0], 0);
            }

            await new Promise<void>((resolve) => {
                source.onended = () => resolve();
            });

        } catch (error) {
            toast.error('Supertonic TTS failed' + error);
            console.error("Supertonic TTS Error:", error);
        }
    }
}


export async function playSmartTTS(
    text: string,
    ttsServerUrl: string,
    onAnim?: (animCode: string, delayMs: number) => void,
    locale: string = 'fr'
) {
    const isAndroidTauri = (window as any).__TAURI_INTERNALS__ && navigator.userAgent.toLowerCase().includes('android');
    if (isAndroidTauri) {
        await playSupertonicTTS(text, onAnim, locale);
    } else {
        await playTTS(text, ttsServerUrl, onAnim, locale);
    }
}
