<script lang="ts">
    import ConjugationGame from './components/ConjugationGame.svelte';
    import LanguageCarousel from './components/LanguageCarousel.svelte';
    import AboutSection from './components/AboutSection.svelte';
    import DocsSection from './components/DocsSection.svelte';
    import SupportSection from './components/SupportSection.svelte';

    type Tab = 'welcome' | 'about' | 'docs' | 'support';
    let activeTab = $state<Tab>('welcome');

    function setTab(tab: Tab) {
        activeTab = tab;
        // Reset scroll position
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    const videoUrl = `${import.meta.env.BASE_URL}parlezvous.mp4`;

    // Shared state for game interaction
    let selectedLangCode = $state('fr');
    let startTrigger = $state(0);
    let isMuted = $state(true);

    function selectLanguageAndStart(code: string) {
        selectedLangCode = code;
        startTrigger += 1;
        
        // Smooth scroll up to the conjugation game card
        const gameElement = document.querySelector('.game-card');
        if (gameElement) {
            gameElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
    }
</script>

<div class="app-layout">
    <!-- Header Navigation -->
    <header class="app-header">
        <div class="logo-container" onclick={() => setTab('welcome')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && setTab('welcome')}>
            <div class="logo-icon">P</div>
            <span class="logo-text">Parlez<span class="yellow-text">Vous</span></span>
        </div>
        <nav class="nav-links">
            <button class="nav-btn" class:active={activeTab === 'welcome'} onclick={() => setTab('welcome')}>Welcome</button>
            <button class="nav-btn" class:active={activeTab === 'about'} onclick={() => setTab('about')}>About</button>
            <button class="nav-btn" class:active={activeTab === 'docs'} onclick={() => setTab('docs')}>Docs</button>
            <button class="nav-btn" class:active={activeTab === 'support'} onclick={() => setTab('support')}>Support</button>
        </nav>
        <div class="header-actions">
            <a href="https://github.com/n123xyz/parlez-vous" target="_blank" rel="noopener noreferrer" class="github-icon-link" aria-label="GitHub Repository">
                <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
            </a>
        </div>
    </header>

    <!-- Main Content Area -->
    <main class="main-content">
        {#if activeTab === 'welcome'}
            <div class="welcome-container">
                <!-- Two-Panel Section -->
                <div class="two-panel-grid">
                    <!-- Left Panel: Phone Mockup / Demo Placeholder -->
                    <div class="panel panel-left">
                        <div class="phone-mockup">
                            <div class="phone-speaker"></div>
                            <div class="phone-screen">
                                <!-- svelte-ignore a11y_media_has_caption -->
                                <video class="phone-video" src={videoUrl} autoplay loop playsinline bind:muted={isMuted} controls></video>
                                <div class="screen-content overlay-content">
                                    <button class="unmute-overlay-btn" onclick={(e) => { e.stopPropagation(); isMuted = !isMuted; }} aria-label={isMuted ? "Unmute video" : "Mute video"}>
                                        {#if isMuted}
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="1" y1="1" x2="23" y2="23"></line><path d="M9 9v6a3 3 0 0 0 3 3h1.586l4.707 4.707A1 1 0 0 0 20 22V4a1 1 0 0 0-1.707-.707L13.586 8H12a3 3 0 0 0-3 3z"></path></svg>
                                            <span class="unmute-text">Tap to Unmute</span>
                                        {:else}
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path></svg>
                                        {/if}
                                    </button>

                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Right Panel: Interactive Conjugation Game -->
                    <div class="panel panel-right">
                        <div class="panel-game-container">
                            <ConjugationGame bind:selectedLangCode bind:startTrigger />
                        </div>
                    </div>
                </div>

                <!-- Carousel at the bottom of Welcome tab -->
                <div class="languages-section">
                    <span class="sec-badge">Supported Languages</span>
                    <h3>Practice in 30+ Languages</h3>
                    <LanguageCarousel onSelect={selectLanguageAndStart} />
                </div>
            </div>
        {:else if activeTab === 'about'}
            <AboutSection />
        {:else if activeTab === 'docs'}
            <DocsSection />
        {:else if activeTab === 'support'}
            <SupportSection />
        {/if}
    </main>

    <!-- Footer -->
    <footer class="app-footer">
        <p>&copy; 2026 Parlez-vous. Built with Svelte, Tauri, and Rust.</p>
    </footer>
</div>

<style>
    .app-layout {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        background: radial-gradient(circle at top, rgba(253, 253, 150, 0.05) 0%, rgba(9, 9, 11, 0) 60%), #09090b;
    }

    /* Header Styling */
    .app-header {
        height: 70px;
        border-bottom: 1px solid rgba(63, 63, 70, 0.4);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 2rem;
        background: rgba(9, 9, 11, 0.75);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        position: sticky;
        top: 0;
        z-index: 50;
    }

    .logo-container {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        cursor: pointer;
        user-select: none;
    }

    .logo-icon {
        width: 32px;
        height: 32px;
        background: #fdfd96;
        color: #09090b;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 800;
        font-size: 1.25rem;
        box-shadow: 0 0 15px rgba(253, 253, 150, 0.4);
    }

    .logo-text {
        font-size: 1.35rem;
        font-weight: 800;
        letter-spacing: -0.02em;
        color: #f4f4f5;
    }

    .yellow-text {
        color: #fdfd96;
    }

    .nav-links {
        display: flex;
        gap: 0.5rem;
        background: rgba(24, 24, 27, 0.5);
        border: 1px solid rgba(63, 63, 70, 0.4);
        padding: 0.25rem;
        border-radius: 12px;
    }

    .nav-btn {
        background: transparent;
        border: none;
        color: #a1a1aa;
        padding: 0.5rem 1.25rem;
        font-size: 0.9rem;
        font-weight: 600;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        font-family: inherit;
    }

    .nav-btn:hover {
        color: #f4f4f5;
    }

    .nav-btn.active {
        background: #fdfd96;
        color: #09090b;
        box-shadow: 0 2px 10px rgba(253, 253, 150, 0.15);
    }

    .github-icon-link {
        color: #a1a1aa;
        transition: color 0.2s;
    }

    .github-icon-link:hover {
        color: #fdfd96;
    }

    /* Main Content */
    .main-content {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    /* Welcome Tab Styles */
    .welcome-container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 3.5rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 5rem;
        width: 100%;
        box-sizing: border-box;
    }

    .two-panel-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 3rem;
        align-items: center;
    }

    @media (min-width: 992px) {
        .two-panel-grid {
            grid-template-columns: 1.1fr 1fr;
        }
    }

    .panel {
        display: flex;
        justify-content: center;
        align-items: center;
        width: 100%;
    }

    /* Left Panel: Phone Mockup */
    .phone-mockup {
        width: 310px;
        height: 620px;
        background: #18181b;
        border: 12px solid #27272a;
        border-radius: 44px;
        position: relative;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(63, 63, 70, 0.4);
        box-sizing: border-box;
    }

    .phone-speaker {
        width: 60px;
        height: 5px;
        background: #3f3f46;
        border-radius: 99px;
        position: absolute;
        top: 15px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 10;
    }

    .phone-screen {
        width: 100%;
        height: 100%;
        border-radius: 32px;
        overflow: hidden;
        position: relative;
        background: #09090b;
        border: 2px solid #09090b;
        box-sizing: border-box;
    }

    .phone-video {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        object-fit: cover;
        z-index: 1;
    }

    .screen-content {
        height: 100%;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        padding: 40px 1.5rem 20px 1.5rem;
        box-sizing: border-box;
        position: relative;
    }

    .overlay-content {
        position: relative;
        z-index: 2;
        pointer-events: none;
        height: 100%;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
    }

    .unmute-overlay-btn {
        position: absolute;
        top: 20px;
        right: 20px;
        z-index: 10;
        background: rgba(24, 24, 27, 0.75);
        border: 1px solid rgba(253, 253, 150, 0.3);
        color: #fdfd96;
        padding: 0.5rem 0.85rem;
        border-radius: 99px;
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.75rem;
        font-weight: 700;
        cursor: pointer;
        pointer-events: auto;
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        font-family: inherit;
    }

    .unmute-overlay-btn:hover {
        background: rgba(253, 253, 150, 0.15);
        border-color: #fdfd96;
        transform: scale(1.03);
    }

    .unmute-text {
        font-size: 0.7rem;
        letter-spacing: 0.02em;
        text-transform: uppercase;
    }



    .languages-section {
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .sec-badge {
        color: #fdfd96;
        font-weight: 600;
        text-transform: uppercase;
        font-size: 0.8rem;
        letter-spacing: 0.1em;
        margin-bottom: 0.5rem;
    }

    .languages-section h3 {
        font-size: 2rem;
        font-weight: 800;
        color: #f4f4f5;
        margin: 0 0 2rem 0;
        letter-spacing: -0.02em;
    }

    /* Footer */
    .app-footer {
        padding: 2.5rem;
        border-top: 1px solid rgba(63, 63, 70, 0.4);
        text-align: center;
        color: #71717a;
        font-size: 0.875rem;
        background: rgba(9, 9, 11, 0.5);
    }

    /* Mobile responsive header & welcome */
    @media (max-width: 768px) {
        .app-header {
            padding: 1rem;
            flex-direction: column;
            height: auto;
            gap: 1rem;
            box-sizing: border-box;
        }

        .nav-links {
            width: 100%;
            justify-content: center;
            box-sizing: border-box;
            flex-wrap: wrap;
            gap: 0.25rem;
        }

        .nav-btn {
            padding: 0.5rem 0.75rem;
            font-size: 0.85rem;
        }

        .header-actions {
            display: none;
        }

        .welcome-container {
            padding: 2rem 1rem;
            gap: 3.5rem;
        }

        .two-panel-grid {
            gap: 2.5rem;
        }

        .phone-mockup {
            width: 290px;
            height: 570px;
        }
    }
</style>
