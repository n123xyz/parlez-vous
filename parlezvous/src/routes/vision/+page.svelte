<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke, convertFileSrc } from '@tauri-apps/api/core';
    import { settingsState, loadSettings } from '$lib/state/settings.svelte.ts';
    import { ollamaState } from '$lib/state/ollama.svelte.ts';
    import toast from 'svelte-french-toast';
    import { timeTracker } from '$lib/state/timeTracker.svelte.ts';
    import { marked } from 'marked';

    let userDescription = $state('');
    let isGrading = $state(false);
    let feedback = $state<any>(null);
    let selectedImageUri = $state<string | null>(null);
    let displayImageUrl = $state<string | null>(null);

    onMount(async () => {
        await loadSettings();
        timeTracker.startTracking();
    });

    onDestroy(() => {
        timeTracker.flushTime();
        timeTracker.stopTracking();
    });

    async function pickImage() {
        console.log("pickImage called");
        
        try {
            const isAndroid = /Android/i.test(navigator.userAgent);
            
            if (isAndroid) {
                toast.success("Opening system gallery...");
                // Trigger the native Android intent directly!
                const response: any = await invoke('plugin:litert|pick_gallery_image');
                const imagePath = response.path;
                
                console.log("Returned absolute cache path:", imagePath);
                
                if (imagePath) {
                    // Update the UI
                    selectedImageUri = imagePath;
                    // Convert the absolute path to an asset:// URL so the WebView can render it
                    displayImageUrl = convertFileSrc(imagePath);
                    
                    toast.success("Image selected and cached!");
                    feedback = null;
                    userDescription = '';
                }
            } else {
                // Use default web file picker
                const fileInput = document.createElement('input');
                fileInput.type = 'file';
                fileInput.accept = 'image/*';
                fileInput.onchange = (e) => {
                    const file = (e.target as HTMLInputElement).files?.[0];
                    if (file) {
                        displayImageUrl = URL.createObjectURL(file);
                        
                        const reader = new FileReader();
                        reader.onloadend = () => {
                            selectedImageUri = reader.result as string; // Data URL (base64)
                            toast.success("Image selected!");
                            feedback = null;
                            userDescription = '';
                        };
                        reader.readAsDataURL(file);
                    }
                };
                fileInput.click();
            }
        } catch (e) {
            if (e !== "Image selection cancelled by user") {
                toast.error("Failed to select image: " + String(e));
            }
            console.error("pickImage error:", e);
        }
    }

    async function submitDescription() {
        if (!userDescription.trim() || !selectedImageUri) return;
        isGrading = true;
        try {
            const prompt = `I am practicing ${settingsState.targetLanguage}. I am describing the attached image. 
            Here is my description: "${userDescription}".
            Please correct my grammar and tell me if I successfully described the visual contents.`;

            const response: any = await invoke('chat_with_avatar', {
                history: [{ role: 'user', content: prompt }],
                model: settingsState.activeModel,
                language: settingsState.targetLanguage,
                activeTextbook: null,
                activePage: null,
                activeTheme: null,
                audioBase64: null,
                imageUri: selectedImageUri
            });

            feedback = {
                correction: response.idealized_correction,
                comment: response.response
            };
        } catch (e) {
            toast.error("Grading failed: " + e);
        } finally {
            isGrading = false;
        }
    }

    function resetTask() {
        selectedImageUri = null;
        displayImageUrl = null;
        userDescription = '';
        feedback = null;
    }
</script>

<div class="max-w-5xl mx-auto p-6 flex flex-col gap-8">
    <div class="flex justify-between items-end border-b border-zinc-800 pb-4">
        <div>
            <h2 class="text-3xl font-bold text-yellow-200">Describe the Chaos</h2>
            <p class="text-zinc-400 mt-2">Pick an image from your camera roll and describe what you see.</p>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div class="bg-zinc-900 rounded-3xl p-4 border border-zinc-800 shadow-xl flex flex-col min-h-[400px]">
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="flex-1 rounded-2xl overflow-hidden bg-zinc-950 flex items-center justify-center border {displayImageUrl ? 'border-zinc-700' : 'border-zinc-800 border-dashed hover:border-yellow-500/50 cursor-pointer transition-colors'} relative"
                onclick={() => { if (!displayImageUrl) pickImage(); }}
            >
                {#if displayImageUrl}
                    <img src={displayImageUrl} alt="Selected from camera roll" class="w-full h-full object-cover" />
                    <div 
                        class="absolute top-4 right-4 w-10 h-10 bg-black/60 rounded-full flex items-center justify-center text-white cursor-pointer hover:bg-black/80 transition backdrop-blur-md"
                        onclick={pickImage}
                        title="Change Image"
                    >
                        🔄
                    </div>
                {:else}
                    <div class="text-center p-8">
                        <div class="text-6xl mb-4">📸</div>
                        <h3 class="text-xl font-bold text-zinc-300">Tap to Select Image</h3>
                        <p class="text-sm text-zinc-500 mt-2">Opens your native camera roll</p>
                    </div>
                {/if}
            </div>
        </div>

        <div class="flex flex-col gap-4">
            <div class="bg-zinc-900 rounded-3xl p-6 border border-zinc-800 shadow-xl flex-1 flex flex-col {displayImageUrl ? '' : 'opacity-50 pointer-events-none'}">
                <label for="desc" class="block text-sm font-bold tracking-widest text-zinc-500 uppercase mb-4">Your Description</label>
                <textarea 
                    id="desc"
                    bind:value={userDescription}
                    class="w-full flex-1 bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl p-4 focus:outline-none focus:border-yellow-200 transition-colors resize-none mb-4"
                    placeholder="Start typing your description here..."
                ></textarea>

                {#if !feedback}
                    <button 
                        onclick={submitDescription}
                        disabled={isGrading || !userDescription.trim() || !selectedImageUri}
                        class="w-full py-4 bg-yellow-200 hover:bg-yellow-300 text-zinc-900 font-bold rounded-xl transition-colors disabled:opacity-50"
                    >
                        {isGrading ? 'Grading...' : 'Submit for Review'}
                    </button>
                {/if}
            </div>

            {#if feedback}
                <div class="bg-zinc-900 rounded-3xl p-6 border border-zinc-800 shadow-xl animate-fade-in">
                    <h3 class="text-sm font-bold tracking-widest text-green-500 uppercase mb-4">Feedback</h3>
                    
                    {#if feedback.correction && feedback.correction !== 'null'}
                        <div class="mb-4 bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                            <p class="text-xs text-zinc-500 uppercase mb-1">Idealized Correction</p>
                            <p class="text-zinc-200">{feedback.correction}</p>
                        </div>
                    {/if}

                    <div class="prose prose-invert prose-sm max-w-none text-zinc-400">
                        {@html marked.parse(feedback.comment)}
                    </div>

                    <button 
                        onclick={resetTask}
                        class="mt-6 w-full py-3 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 font-bold rounded-xl transition-colors"
                    >
                        Describe Another Image
                    </button>
                </div>
            {/if}
        </div>
    </div>
</div>
