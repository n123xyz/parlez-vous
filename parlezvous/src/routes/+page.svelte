<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';

    onMount(() => {
        // Redirect to journal by default for MVP
        goto('/journal');
    });
</script>

<div class="flex items-center justify-center h-screen">
    <p class="text-gray-400 animate-pulse">Loading Parlez-Vous...</p>
</div>
