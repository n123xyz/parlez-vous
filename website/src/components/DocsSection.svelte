<script lang="ts">
    import { onMount } from 'svelte';
    import { Marked } from 'marked';
    
    // Import raw markdown files using Vite's ?raw modifier
    import readmeRaw from '../../../docs/README.md?raw';
    import architectureRaw from '../../../docs/architecture.md?raw';
    import featuresRaw from '../../../docs/features.md?raw';
    import installationRaw from '../../../docs/installation.md?raw';
    import developmentRaw from '../../../docs/development.md?raw';

    const marked = new Marked();

    // Map of doc IDs to their metadata and raw contents
    const docsMap: Record<string, { title: string; content: string }> = {
        introduction: { title: "Introduction", content: readmeRaw },
        architecture: { title: "System Architecture", content: architectureRaw },
        features: { title: "Application Features", content: featuresRaw },
        installation: { title: "Installation & Setup", content: installationRaw },
        development: { title: "Developer Guide", content: developmentRaw }
    };

    let activeDocId = $state("introduction");
    let renderedHtml = $derived.by(() => {
        const raw = docsMap[activeDocId]?.content || "";
        // Replace file:/// absolute links to render nicely in the app
        const processed = raw.replace(/file:\/\/[^)\s]+/g, (match) => {
            // Extract the filename without extensions
            const parts = match.split('/');
            const filename = parts[parts.length - 1].split('.')[0];
            if (filename === 'README') return '#docs-introduction';
            return `#docs-${filename}`;
        });
        return marked.parse(processed) as string;
    });

    function selectDoc(id: string) {
        activeDocId = id;
        // Scroll doc panel to top
        const panel = document.querySelector('.doc-content-panel');
        if (panel) {
            panel.scrollTop = 0;
        }
    }

    // Handle hash links if user clicks from within markdown
    function handleHashChange() {
        const hash = window.location.hash;
        if (hash.startsWith('#docs-')) {
            const docId = hash.replace('#docs-', '');
            if (docsMap[docId]) {
                activeDocId = docId;
            } else if (docId === 'introduction' || docId === 'README') {
                activeDocId = 'introduction';
            }
        }
    }

    onMount(() => {
        window.addEventListener('hashchange', handleHashChange);
        handleHashChange(); // Run once at load
        return () => {
            window.removeEventListener('hashchange', handleHashChange);
        };
    });
</script>

<div class="docs-container">
    <!-- Sidebar -->
    <aside class="docs-sidebar">
        <div class="sidebar-header">
            <h4>Documentation</h4>
        </div>
        <nav class="sidebar-nav">
            {#each Object.entries(docsMap) as [id, doc]}
                <button 
                    class="nav-item" 
                    class:active={activeDocId === id}
                    onclick={() => selectDoc(id)}
                >
                    <span class="indicator"></span>
                    <span class="title">{doc.title}</span>
                </button>
            {/each}
        </nav>
    </aside>

    <!-- Main Content Panel -->
    <main class="doc-content-panel">
        <div class="markdown-body">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html renderedHtml}
        </div>
    </main>
</div>

<style>
    .docs-container {
        display: flex;
        flex: 1;
        width: 100%;
        max-width: 1280px;
        margin: 0 auto;
        height: calc(100vh - 70px);
        overflow: hidden;
        border-left: 1px solid rgba(63, 63, 70, 0.4);
        border-right: 1px solid rgba(63, 63, 70, 0.4);
    }

    /* Sidebar styles */
    .docs-sidebar {
        width: 260px;
        background: rgba(18, 18, 20, 0.5);
        border-right: 1px solid rgba(63, 63, 70, 0.4);
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        box-sizing: border-box;
    }

    .sidebar-header {
        padding: 1.5rem;
        border-bottom: 1px solid rgba(63, 63, 70, 0.3);
    }

    .sidebar-header h4 {
        margin: 0;
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: #fdfd96;
        font-weight: 700;
    }

    .sidebar-nav {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        overflow-y: auto;
    }

    .nav-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: transparent;
        border: none;
        color: #a1a1aa;
        padding: 0.75rem 1rem;
        text-align: left;
        font-family: inherit;
        font-size: 0.95rem;
        font-weight: 550;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        position: relative;
    }

    .nav-item:hover {
        background: rgba(253, 253, 150, 0.04);
        color: #f4f4f5;
    }

    .nav-item.active {
        background: rgba(253, 253, 150, 0.08);
        color: #fdfd96;
    }

    .indicator {
        width: 4px;
        height: 16px;
        background: transparent;
        border-radius: 99px;
        transition: background-color 0.2s ease;
    }

    .nav-item.active .indicator {
        background: #fdfd96;
        box-shadow: 0 0 8px #fdfd96;
    }

    /* Content panel styles */
    .doc-content-panel {
        flex: 1;
        padding: 3rem;
        overflow-y: auto;
        box-sizing: border-box;
        text-align: left;
        background: rgba(9, 9, 11, 0.2);
    }

    /* Responsive Mobile layout for docs */
    @media (max-width: 768px) {
        .docs-container {
            flex-direction: column;
            height: auto;
            border: none;
        }

        .docs-sidebar {
            width: 100%;
            border-right: none;
            border-bottom: 1px solid rgba(63, 63, 70, 0.4);
        }

        .sidebar-nav {
            flex-direction: row;
            overflow-x: auto;
            padding: 0.75rem;
            white-space: nowrap;
        }

        .nav-item {
            padding: 0.5rem 0.85rem;
            font-size: 0.85rem;
        }

        .indicator {
            display: none;
        }

        .doc-content-panel {
            padding: 1.5rem;
            overflow-y: visible;
        }
    }
</style>
