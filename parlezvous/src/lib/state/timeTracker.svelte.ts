import { invoke } from '@tauri-apps/api/core';
import { settingsState } from '$lib/state/settings.svelte.ts';

export const timeTracker = {
    activeSeconds: 0,
    intervalId: null as any,
    
    startTracking() {
        if (this.intervalId) return;
        this.intervalId = setInterval(() => {
            this.activeSeconds++;
        }, 1000);
    },

    stopTracking() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    },

    async flushTime() {
        if (this.activeSeconds === 0) return;
        try {
            await invoke('add_time_xp', { 
                language: settingsState.targetLanguage, 
                seconds: this.activeSeconds 
            });
            this.activeSeconds = 0;
        } catch (e) {
            console.error('Failed to flush active seconds', e);
        }
    }
};
