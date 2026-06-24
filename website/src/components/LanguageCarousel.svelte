<script lang="ts">
    import verbData from '../data/verb.json';
    
    // Extracted list of language objects with flag emojis
    const flags: Record<string, string> = {
        en: "🇺🇸", ko: "🇰🇷", ja: "🇯🇵", ar: "🇸🇦", bg: "🇧🇬", cs: "🇨🇿",
        da: "🇩🇰", de: "🇩🇪", el: "🇬🇷", es: "🇪🇸", et: "🇪🇪", fi: "🇫🇮",
        fr: "🇫🇷", hi: "🇮🇳", hr: "🇭🇷", hu: "🇭🇺", id: "🇮🇩", it: "🇮🇹",
        lt: "🇱🇹", lv: "🇱🇻", nl: "🇳🇱", pl: "🇵🇱", pt: "🇵🇹", ro: "🇷🇴",
        ru: "🇷🇺", sk: "🇸🇰", sl: "🇸🇮", sv: "🇸🇪", tr: "🇹🇷", uk: "🇺🇦",
        vi: "🇻🇳"
    };

    const languagesList = Object.entries(verbData).map(([code, data]) => ({
        code,
        name: data.language,
        flag: flags[code] || "🌐"
    }));

    // Svelte 5 Callback Prop
    let { onSelect } = $props<{ onSelect: (code: string) => void }>();

    // Duplicate list to ensure seamless infinite looping
    const carouselItems = [...languagesList, ...languagesList, ...languagesList];
</script>

<div class="carousel-container">
    <div class="carousel-track">
        {#each carouselItems as item}
            <button class="carousel-item" onclick={() => onSelect(item.code)}>
                <span class="flag">{item.flag}</span>
                <span class="name">{item.name}</span>
            </button>
        {/each}
    </div>
</div>

<style>
    .carousel-container {
        width: 100%;
        overflow: hidden;
        position: relative;
        padding: 2.5rem 0;
        background: linear-gradient(to right, rgba(9, 9, 11, 0) 0%, rgba(9, 9, 11, 0.8) 15%, rgba(9, 9, 11, 0.8) 85%, rgba(9, 9, 11, 0) 100%);
        border-top: 1px solid rgba(63, 63, 70, 0.4);
        border-bottom: 1px solid rgba(63, 63, 70, 0.4);
        margin-top: auto;
    }

    .carousel-container::before,
    .carousel-container::after {
        content: "";
        position: absolute;
        top: 0;
        width: 150px;
        height: 100%;
        z-index: 2;
        pointer-events: none;
    }

    .carousel-container::before {
        left: 0;
        background: linear-gradient(to right, #09090b, transparent);
    }

    .carousel-container::after {
        right: 0;
        background: linear-gradient(to left, #09090b, transparent);
    }

    .carousel-track {
        display: flex;
        gap: 1.5rem;
        width: max-content;
        animation: marquee 45s linear infinite;
    }

    .carousel-track:hover {
        animation-play-state: paused;
    }

    .carousel-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem 1.5rem;
        background: rgba(24, 24, 27, 0.6);
        border: 1px solid rgba(63, 63, 70, 0.5);
        border-radius: 12px;
        color: #f4f4f5;
        font-weight: 500;
        font-size: 0.95rem;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
        user-select: none;
        font-family: inherit;
    }

    .carousel-item:hover {
        transform: translateY(-3px);
        background: rgba(253, 253, 150, 0.08);
        border-color: rgba(253, 253, 150, 0.4);
        box-shadow: 0 10px 15px -3px rgba(253, 253, 150, 0.05), 0 4px 6px -4px rgba(253, 253, 150, 0.05);
    }

    .flag {
        font-size: 1.35rem;
    }

    .name {
        letter-spacing: -0.01em;
    }

    @keyframes marquee {
        0% {
            transform: translateX(0);
        }
        100% {
            transform: translateX(calc(-33.333% - 0.5rem));
        }
    }
</style>
