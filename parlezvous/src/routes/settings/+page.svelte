<script lang="ts">
    import { ollamaState, fetchOllamaModels } from '$lib/state/ollama.svelte.ts';
    import { settingsState, loadSettings, saveSettings } from '$lib/state/settings.svelte.ts';
    import { onMount } from 'svelte';
    import toast from 'svelte-french-toast';
    import { invoke } from '@tauri-apps/api/core';

    import { listen } from '@tauri-apps/api/event';

    let isSaving = $state(false);
    let litertModelExists = $state(false);
    let isDownloadingLitert = $state(false);
    let downloadProgress = $state(-1);

    let isAndroidTauri = $state(false);

    let supertonicReady = $state(false);
    let isDownloadingSupertonic = $state(false);
    let supertonicProgress = $state(-1);

    let tokenizerExists = $state(false);
    let isDownloadingTokenizer = $state(false);

    async function checkLitertModel() {
        try {
            const result: { exists: boolean, isDownloading: boolean } = await invoke('plugin:litert|check_model_exists', { payload: { modelPath: "gemma-4-E2B-it.litertlm" }});
            litertModelExists = result.exists;
            if (result.isDownloading) {
                isDownloadingLitert = true;
            }
        } catch(e) {
            console.error('Failed to check LiteRT model:', e);
        }
    }

    async function downloadLitertModel() {

        try {
            await saveSettings();
        } catch(e) {
            console.warn("Failed to auto-save HF token:", e);
        }

        isDownloadingLitert = true;
        downloadProgress = 0;
        try {
            await invoke('plugin:litert|download_model', { payload: { modelPath: "gemma-4-E2B-it.litertlm", token: settingsState.huggingFaceToken }});
            toast.success('Download queued in background!');
        } catch(e) {
            isDownloadingLitert = false;
            toast.error('Failed to start LiteRT download: ' + e);
        }
    }

    async function purgeLitertModel() {
        try {
            await invoke('plugin:litert|purge_model', { payload: { modelPath: "gemma-4-E2B-it.litertlm" }});
            toast.success('Model purged successfully');
            await checkLitertModel();
        } catch(e) {
            toast.error('Failed to purge model: ' + e);
        }
    }

    async function checkSupertonicModel() {
        try {
            const result: { exists: boolean, isDownloading: boolean } = await invoke('plugin:supertonic|is_supertonic_ready');
            supertonicReady = result.exists;
            if (result.isDownloading) {
                isDownloadingSupertonic = true;
            }
        } catch(e) {
            console.error('Failed to check Supertonic model:', e);
        }
    }

    async function downloadSupertonicModel() {
        isDownloadingSupertonic = true;
        supertonicProgress = 0;
        try {
            await invoke('plugin:supertonic|download_supertonic_models', { payload: { modelPath: "", downloadUrl: "", token: null }});
            toast.success('Supertonic models download queued!');
        } catch(e) {
            isDownloadingSupertonic = false;
            toast.error('Failed to start Supertonic download: ' + e);
        }
    }

    async function purgeSupertonicModel() {
        try {
            await invoke('plugin:supertonic|purge_supertonic_models');
            toast.success('Supertonic models purged successfully');
            await checkSupertonicModel();
        } catch(e) {
            toast.error('Failed to purge Supertonic models: ' + e);
        }
    }

    async function checkTokenizer() {
        try {
            tokenizerExists = await invoke('check_tokenizer_exists');
        } catch(e) {
            console.error('Failed to check tokenizer:', e);
        }
    }

    async function downloadTokenizer() {
        isDownloadingTokenizer = true;
        try {
            await invoke('download_tokenizer');
            toast.success('Tokenizer downloaded successfully!');
            await checkTokenizer();
        } catch(e) {
            toast.error('Failed to download tokenizer: ' + e);
        } finally {
            isDownloadingTokenizer = false;
        }
    }

    async function purgeTokenizer() {
        try {
            await invoke('delete_tokenizer');
            toast.success('Tokenizer purged successfully');
            await checkTokenizer();
        } catch(e) {
            toast.error('Failed to purge tokenizer: ' + e);
        }
    }

    onMount(() => {
        isAndroidTauri = (window as any).__TAURI_INTERNALS__ && navigator.userAgent.toLowerCase().includes('android');
        
        loadSettings().then(() => {
            fetchOllamaModels();
            checkLitertModel();
            checkSupertonicModel();
            checkTokenizer();
        });

        const unlisten = listen<{ downloaded: number, total: number, state?: string }>('download_progress', (event) => {
            const { downloaded, total, state } = event.payload;
            isDownloadingLitert = true;
            if (state) {
                if (state === 'SUCCEEDED') {
                    isDownloadingLitert = false;
                    downloadProgress = -1;
                    checkLitertModel();
                    toast.success('LiteRT model downloaded successfully!');
                } else if (state === 'FAILED' || state === 'CANCELLED') {
                    isDownloadingLitert = false;
                    downloadProgress = -1;
                    toast.error(`Download ${state.toLowerCase()}.`);
                }
            } else {
                if (total > 0) {
                    downloadProgress = Math.round((downloaded / total) * 100);
                } else {
                    downloadProgress = -1;
                }
            }
        });

        const unlistenSupertonic = listen<{ downloaded: number, total: number, state?: string }>('supertonic_download_progress', (event) => {
            const { downloaded, total, state } = event.payload;
            isDownloadingSupertonic = true;
            if (state) {
                if (state === 'SUCCEEDED') {
                    isDownloadingSupertonic = false;
                    supertonicProgress = -1;
                    checkSupertonicModel();
                    toast.success('Supertonic models downloaded successfully!');
                } else if (state === 'FAILED' || state === 'CANCELLED') {
                    isDownloadingSupertonic = false;
                    supertonicProgress = -1;
                    toast.error(`Download ${state.toLowerCase()}.`);
                }
            } else {
                if (total > 0) {
                    supertonicProgress = Math.round((downloaded / total) * 100);
                } else {
                    supertonicProgress = -1;
                }
            }
        });

        return () => {
            unlisten.then(f => f());
            unlistenSupertonic.then(f => f());
        };
    });

    async function handleSave() {
        isSaving = true;
        try {
            await saveSettings();
            toast.success('Settings saved successfully!');
        } catch (e) {
            toast.error('Failed to save settings: ' + e);
        } finally {
            isSaving = false;
        }
    }
</script>

<div class="max-w-3xl mx-auto p-6">
    <h2 class="text-3xl font-bold text-yellow-200 mb-8 border-b border-zinc-800 pb-4">Settings</h2>

    <div class="bg-zinc-900 p-8 rounded-2xl border border-zinc-800 shadow-xl space-y-8">
        <div>
            <h3 class="text-xl font-medium text-zinc-200 mb-2">Local LLM Configuration</h3>
            <p class="text-zinc-400 text-sm mb-6">Select the Ollama model to power Parlez-Vous. A capable instruct model like Gemma or Llama is recommended.</p>
            
            {#if ollamaState.models.length === 0}
                <div class="bg-zinc-800 p-4 rounded-xl border border-zinc-700 flex items-center justify-between mb-4">
                    <span class="text-zinc-400">No models found in local Ollama.</span>
                    <button 
                        class="text-yellow-200 text-sm hover:underline"
                        onclick={fetchOllamaModels}
                    >
                        Refresh
                    </button>
                </div>
            {/if}

            {#if ollamaState.models.length > 0 || isAndroidTauri}
                <div class="flex flex-col gap-2">
                    <label for="modelSelect" class="text-sm font-medium text-zinc-400">Active Model</label>
                    <div class="relative">
                        <select 
                            id="modelSelect"
                            bind:value={settingsState.activeModel}
                            onchange={handleSave}
                            class="w-full bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors cursor-pointer appearance-none"
                        >
                            {#if ollamaState.models.length > 0}
                                <optgroup label="Ollama Models">
                                    {#each ollamaState.models as model}
                                        <option value={model}>{model}</option>
                                    {/each}
                                </optgroup>
                            {/if}
                            {#if isAndroidTauri}
                                <optgroup label="LiteRT (On-Device)">
                                    <option value="gemma-4-E2B-it.litertlm">gemma-4-E2B-it.litertlm</option>
                                </optgroup>
                            {/if}
                        </select>
                        <div class="absolute inset-y-0 right-0 flex items-center px-4 pointer-events-none text-zinc-500">
                            ▼
                        </div>
                    </div>
                </div>
            {/if}
        </div>

        {#if isAndroidTauri}
        <div>
            <h3 class="text-xl font-medium text-zinc-200 mb-2">On-Device Inference (Android LiteRT)</h3>
            <p class="text-zinc-400 text-sm mb-6">Manage the high-performance on-device LiteRT LLM. This model powers offline interactions on Android.</p>
            
            <div class="bg-zinc-800 p-4 rounded-xl border border-zinc-700 flex flex-col gap-3">
                <div class="flex items-center justify-between">
                    <span class="text-zinc-300">
                        Gemma 4 E2B IT Model
                        <span class="block text-sm {litertModelExists ? 'text-green-400' : 'text-zinc-500'}">
                            {litertModelExists ? 'Downloaded and ready' : 'Not downloaded'}
                        </span>
                    </span>
                    
                    {#if isDownloadingLitert}
                        <span class="text-yellow-200 text-sm animate-pulse">Downloading... {downloadProgress >= 0 ? downloadProgress + '%' : ''}</span>
                    {:else if !litertModelExists}
                        <button 
                            class="bg-yellow-200 text-zinc-900 px-4 py-2 rounded-lg text-sm font-bold hover:bg-yellow-300 transition-colors"
                            onclick={downloadLitertModel}
                        >
                            Download Model
                        </button>
                    {:else}
                        <div class="flex items-center gap-4">
                            <button 
                                class="text-red-400 text-sm hover:underline"
                                onclick={purgeLitertModel}
                            >
                                Purge
                            </button>
                            <button 
                                class="text-zinc-400 text-sm hover:underline"
                                onclick={checkLitertModel}
                            >
                                Verify
                            </button>
                        </div>
                    {/if}
                </div>

                {#if isDownloadingLitert && downloadProgress >= 0}
                    <div class="w-full bg-zinc-900 rounded-full h-2.5 overflow-hidden">
                        <div class="bg-yellow-200 h-2.5 rounded-full transition-all duration-300" style="width: {downloadProgress}%"></div>
                    </div>
                {/if}
            </div>

            <div class="bg-zinc-800 p-4 rounded-xl border border-zinc-700 flex flex-col gap-3 mt-4">
                <div class="flex items-center justify-between">
                    <span class="text-zinc-300">
                        Supertonic TTS Models
                        <span class="block text-sm {supertonicReady ? 'text-green-400' : 'text-zinc-500'}">
                            {supertonicReady ? 'Downloaded and ready' : 'Not downloaded'}
                        </span>
                    </span>
                    
                    {#if isDownloadingSupertonic}
                        <span class="text-yellow-200 text-sm animate-pulse">Downloading... {supertonicProgress >= 0 ? supertonicProgress + '%' : ''}</span>
                    {:else if !supertonicReady}
                        <button 
                            class="bg-yellow-200 text-zinc-900 px-4 py-2 rounded-lg text-sm font-bold hover:bg-yellow-300 transition-colors"
                            onclick={downloadSupertonicModel}
                        >
                            Download Models
                        </button>
                    {:else}
                        <div class="flex items-center gap-4">
                            <button 
                                class="text-red-400 text-sm hover:underline"
                                onclick={purgeSupertonicModel}
                            >
                                Purge
                            </button>
                            <button 
                                class="text-zinc-400 text-sm hover:underline"
                                onclick={checkSupertonicModel}
                            >
                                Verify
                            </button>
                        </div>
                    {/if}
                </div>

                {#if isDownloadingSupertonic && supertonicProgress >= 0}
                    <div class="w-full bg-zinc-900 rounded-full h-2.5 overflow-hidden">
                        <div class="bg-yellow-200 h-2.5 rounded-full transition-all duration-300" style="width: {supertonicProgress}%"></div>
                    </div>
                {/if}
            </div>

            <div class="bg-zinc-800 p-4 rounded-xl border border-zinc-700 flex flex-col gap-3 mt-4">
                <div class="flex items-center justify-between">
                    <span class="text-zinc-300">
                        Gemma Tokenizer (Context Truncation)
                        <span class="block text-sm {tokenizerExists ? 'text-green-400' : 'text-zinc-500'}">
                            {tokenizerExists ? 'Downloaded and ready' : 'Not downloaded'}
                        </span>
                    </span>
                    
                    {#if isDownloadingTokenizer}
                        <span class="text-yellow-200 text-sm animate-pulse">Downloading...</span>
                    {:else if !tokenizerExists}
                        <button 
                            class="bg-yellow-200 text-zinc-900 px-4 py-2 rounded-lg text-sm font-bold hover:bg-yellow-300 transition-colors"
                            onclick={downloadTokenizer}
                        >
                            Download Tokenizer
                        </button>
                    {:else}
                        <div class="flex items-center gap-4">
                            <button 
                                class="text-red-400 text-sm hover:underline"
                                onclick={purgeTokenizer}
                            >
                                Purge
                            </button>
                            <button 
                                class="text-zinc-400 text-sm hover:underline"
                                onclick={checkTokenizer}
                            >
                                Verify
                            </button>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
        {/if}

        <div>
            <h3 class="text-xl font-medium text-zinc-200 mb-2">Application Preferences</h3>
            <p class="text-zinc-400 text-sm mb-6">Configure target language and TTS integration.</p>

            <div class="flex flex-col gap-4">
                <div class="flex flex-col gap-2">
                    <label for="languageInput" class="text-sm font-medium text-zinc-400">Target Language</label>
                    <input 
                        id="languageInput"
                        type="text"
                        bind:value={settingsState.targetLanguage}
                        placeholder="e.g. French, Spanish, Korean"
                        class="bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors"
                    >
                </div>

                <div class="flex flex-col gap-2">
                    <label for="skillLevelSelect" class="text-sm font-medium text-zinc-400">Skill Level</label>
                    <div class="relative">
                        <select 
                            id="skillLevelSelect"
                            bind:value={settingsState.skillLevel}
                            class="w-full bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors cursor-pointer appearance-none"
                        >
                            <option value="Beginner">Beginner</option>
                            <option value="Intermediate">Intermediate</option>
                            <option value="Advanced">Advanced</option>
                            <option value="Fluent">Fluent</option>
                        </select>
                        <div class="absolute inset-y-0 right-0 flex items-center px-4 pointer-events-none text-zinc-500">
                            ▼
                        </div>
                    </div>
                </div>
                <div class="flex flex-col gap-2">
                    <label for="asrInput" class="text-sm font-medium text-zinc-400">ASR Server URL</label>
                    <input 
                        id="asrInput"
                        type="text"
                        bind:value={settingsState.asrServerUrl}
                        placeholder="http://localhost:8000/v1/audio/transcriptions"
                        class="bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors"
                    >
                </div>
                <div class="flex flex-col gap-2">
                    <label for="ttsInput" class="text-sm font-medium text-zinc-400">TTS Server URL</label>
                    <input 
                        id="ttsInput"
                        type="text"
                        bind:value={settingsState.ttsServerUrl}
                        placeholder="http://localhost:5050/v1/audio/speech"
                        class="bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors"
                    >
                </div>

                <div class="flex flex-col gap-2">
                    <label for="ollamaInput" class="text-sm font-medium text-zinc-400">Ollama Server URL</label>
                    <input 
                        id="ollamaInput"
                        type="text"
                        bind:value={settingsState.ollamaServerUrl}
                        placeholder="http://localhost:11434"
                        class="bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors"
                    >
                </div>

                <div class="flex flex-col gap-2">
                    <label for="embeddingSelect" class="text-sm font-medium text-zinc-400">Embedding Model (RAG)</label>
                    <div class="relative">
                        <select 
                            id="embeddingSelect"
                            bind:value={settingsState.embeddingModel}
                            class="w-full bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors cursor-pointer appearance-none"
                        >
                            {#each ollamaState.models as model}
                                <option value={model}>{model}</option>
                            {/each}
                        </select>
                        <div class="absolute inset-y-0 right-0 flex items-center px-4 pointer-events-none text-zinc-500">
                            ▼
                        </div>
                    </div>
                </div>

                <div class="flex flex-col gap-2 mt-4 pt-4 border-t border-zinc-800">
                    <label for="hfTokenInput" class="text-sm font-medium text-zinc-400">HuggingFace Access Token</label>
                    <p class="text-xs text-zinc-500 mb-1">Optional: Provide a token to improve download speed and reliability.</p>
                    <input 
                        id="hfTokenInput"
                        type="password"
                        bind:value={settingsState.huggingFaceToken}
                        onchange={handleSave}
                        placeholder="hf_..."
                        class="bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors"
                    >
                </div>

                <div class="flex flex-col gap-2 mt-4">
                    <label for="litertAcceleratorSelect" class="text-sm font-medium text-zinc-400">LiteRT Hardware Accelerator</label>
                    <p class="text-xs text-zinc-500 mb-1">Forces on-device inference to run on specific hardware if supported.</p>
                    <div class="relative">
                        <select 
                            id="litertAcceleratorSelect"
                            bind:value={settingsState.litertAccelerator}
                            onchange={handleSave}
                            class="w-full bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors cursor-pointer appearance-none"
                        >
                            <option value="Auto">Auto (NPU → GPU → CPU)</option>
                            <option value="CPU">CPU</option>
                            <option value="GPU">GPU</option>
                            <option value="NPU">NPU</option>
                        </select>
                        <div class="absolute inset-y-0 right-0 flex items-center px-4 pointer-events-none text-zinc-500">
                            ▼
                        </div>
                    </div>
                </div>

                <div class="flex flex-col gap-2 mt-4 border-t border-zinc-800 pt-4">
                    <label for="maxTokensInput" class="text-sm font-medium text-zinc-400">LiteRT Max Tokens</label>
                    <p class="text-xs text-zinc-500 mb-1">Maximum length of response context. Reduce if experiencing memory crashes on mobile.</p>
                    <input 
                        id="maxTokensInput"
                        type="number"
                        bind:value={settingsState.litertMaxTokens}
                        onchange={handleSave}
                        min="256"
                        max="4096"
                        step="128"
                        class="bg-zinc-950 border border-zinc-700 text-zinc-100 rounded-xl px-4 py-3 focus:outline-none focus:border-yellow-200 transition-colors"
                    >
                </div>
            </div>
            
            <button 
                class="mt-6 bg-yellow-200 hover:bg-yellow-300 disabled:opacity-50 text-zinc-900 font-bold py-3 px-6 rounded-xl transition-all shadow-[0_0_15px_rgba(253,253,150,0.2)]"
                onclick={handleSave}
                disabled={isSaving}
            >
                {isSaving ? 'Saving...' : 'Save Preferences'}
            </button>
        </div>

        <div>
            <h3 class="text-xl font-medium text-zinc-200 mb-2">System Status</h3>
            <div class="bg-zinc-800 p-4 rounded-xl border border-zinc-700 flex flex-col gap-3">
                <div class="flex justify-between items-center">
                    <span class="text-zinc-400">Ollama Backend</span>
                    <span class="flex items-center gap-2">
                        <div class="w-2 h-2 rounded-full {ollamaState.isHealthy ? 'bg-green-500' : 'bg-red-500'}"></div>
                        {ollamaState.isHealthy ? 'Connected' : 'Disconnected'}
                    </span>
                </div>
            </div>
        </div>
    </div>
</div>
