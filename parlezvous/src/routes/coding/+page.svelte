<script lang="ts">
    import SpeedrunGame from '$lib/components/SpeedrunGame.svelte';
    import { codingState, initCodingQueue, popCodingPuzzle, stopCodingQueue } from '$lib/state/coding.svelte';
</script>

<SpeedrunGame 
    store={codingState}
    initQueue={initCodingQueue}
    popPuzzle={popCodingPuzzle}
    stopQueue={stopCodingQueue}
/>
