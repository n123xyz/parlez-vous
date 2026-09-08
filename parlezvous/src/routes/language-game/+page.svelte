<script lang="ts">
    import SpeedrunGame from '$lib/components/SpeedrunGame.svelte';
    import { languageSpeedrunState, initLanguageQueue, popLanguagePuzzle, stopLanguageQueue } from '$lib/state/languageSpeedrun.svelte';
</script>

<SpeedrunGame 
    store={languageSpeedrunState}
    initQueue={initLanguageQueue}
    popPuzzle={popLanguagePuzzle}
    stopQueue={stopLanguageQueue}
    isLanguageGame={true}
/>
