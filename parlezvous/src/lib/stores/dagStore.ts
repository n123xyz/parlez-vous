import { writable } from 'svelte/store';

interface DagState {
    unlockedTracks: number[];
    currentTrack: number;
    completedNodes: string[];
}

export const dagStore = writable<DagState>({
    unlockedTracks: [1], // Track 1 is open by default
    currentTrack: 1,
    completedNodes: []
});
