import { invoke } from '@tauri-apps/api/core';
import { settingsState } from '$lib/state/settings.svelte.ts';

export const ollamaState = $state({
    isHealthy: true,
    hasChecked: false,
    models: [] as string[]
});

export async function checkOllamaHealth() {
    try {
        const isHealthy = await invoke<boolean>('check_ollama_health');
        ollamaState.isHealthy = isHealthy;
        ollamaState.hasChecked = true;
        
        if (isHealthy) {
            await fetchOllamaModels();
        }
    } catch (e) {
        console.error('Failed to check Ollama health:', e);
        ollamaState.isHealthy = false;
        ollamaState.hasChecked = true;
    }
}

export async function fetchOllamaModels() {
    try {
        const rawModels = await invoke<string[]>('list_ollama_models');
        const models = rawModels.filter(m => !m.includes('litert'));
        ollamaState.models = models;
        const isAndroidTauri = (window as any).__TAURI_INTERNALS__ && navigator.userAgent.toLowerCase().includes('android');
        const isLitert = settingsState.activeModel.includes('litert');

        if (rawModels.includes(settingsState.activeModel) || (isAndroidTauri && isLitert)) {
            // Already a valid Ollama model
        } else if (models.length > 0) {
            // It's not a valid Ollama model.
            // Overwrite if it is not a Litert model, or if we are on Desktop (meaning Litert is invalid here)
            if (!isLitert || !isAndroidTauri) {
                settingsState.activeModel = models[0];
                import('$lib/state/settings.svelte.ts').then(m => m.saveSettings());
            }
        } else if (isAndroidTauri) {
            if (!isLitert) {
                settingsState.activeModel = "gemma-4-E2B-it.litertlm";
                import('$lib/state/settings.svelte.ts').then(m => m.saveSettings());
            }
        }
    } catch (e) {
        console.error('Failed to list Ollama models:', e);
    }
}
