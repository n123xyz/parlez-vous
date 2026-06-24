<script lang="ts">
</script>

<div class="support-container">
    <div class="support-header">
        <span class="eyebrow">Get Involved</span>
        <h2>Support & Contribute</h2>
        <p class="subtitle">Parlez-vous? is powered by open source contributors and community support.</p>
    </div>

    <div class="support-grid">
        <!-- Developer Contribution Card -->
        <div class="support-card glassmorphic">
            <div class="card-icon">💻</div>
            <h3>For Developers</h3>
            <p>
                Whether you want to fix bugs, optimize on-device inference, improve Svelte interfaces, or extend database migrations, we welcome contributions!
            </p>
            
            <div class="guide-list">
                <div class="guide-item">
                    <span class="num">1</span>
                    <div class="text">
                        <strong>Fork & Clone</strong>
                        <span>Head over to our GitHub repository and pull the source branch.</span>
                    </div>
                </div>
                <div class="guide-item">
                    <span class="num">2</span>
                    <div class="text">
                        <strong>Tauri Setup</strong>
                        <span>Follow the compilation guide in our <a href="https://github.com/n123xyz/parlez-vous/blob/main/docs/installation.md" target="_blank" rel="noopener noreferrer">docs</a> to set up Rust, pnpm, and Android Studio NDK paths.</span>
                    </div>
                </div>
                <div class="guide-item">
                    <span class="num">3</span>
                    <div class="text">
                        <strong>Submit a PR</strong>
                        <span>Create a feature branch, run local tests, and open a Pull Request. We review contributions weekly.</span>
                    </div>
                </div>
            </div>

            <a href="https://github.com/n123xyz/parlez-vous" target="_blank" rel="noopener noreferrer" class="btn btn-github">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
                View GitHub Repository
            </a>
        </div>

        <!-- Financial Support Card -->
        <div class="support-card glassmorphic highlighted-card">
            <div class="card-icon">☕</div>
            <h3>Buy Us a Coffee</h3>
            <p>
                Our project runs entirely on zero ads and zero paywalls. 100% of the funds go towards testing Android hardware, purchasing API keys for external evaluation, and general development infrastructure.
            </p>
            
            <p class="donation-desc">
                If <b>Parlez-vous?</b> has helped you learn a language, saved you subscription fees, or made your development workflows easier, please consider supporting the project.
            </p>

            <div class="kofi-section">
                <!-- A sleek Ko-fi widget lookalike -->
                <a href="https://ko-fi.com/parlezvous" target="_blank" rel="noopener noreferrer" class="kofi-button">
                    <img src="https://storage.ko-fi.com/cdn/cup-border.png" alt="Cup icon" class="kofi-cup" />
                    Support us on Ko-fi
                </a>
            </div>
            
            <div class="other-ways">
                <h4>Other ways to support:</h4>
                <ul>
                    <li>Give us a ⭐ on GitHub</li>
                    <li>Share the app with other language learners</li>
                    <li>Provide feedback by opening issues</li>
                </ul>
            </div>
        </div>
    </div>
</div>

<style>
    .support-container {
        max-width: 1100px;
        margin: 0 auto;
        padding: 2rem 1rem;
        box-sizing: border-box;
        overflow-wrap: break-word;
    }

    @media (min-width: 768px) {
        .support-container {
            padding: 4rem 1.5rem;
        }
    }

    .support-header {
        text-align: center;
        margin-bottom: 2rem;
    }

    @media (min-width: 768px) {
        .support-header {
            margin-bottom: 4rem;
        }
    }

    .eyebrow {
        color: #fdfd96;
        font-weight: 600;
        text-transform: uppercase;
        font-size: 0.85rem;
        letter-spacing: 0.1em;
        display: block;
        margin-bottom: 0.5rem;
    }

    h2 {
        font-size: 2rem;
        font-weight: 850;
        margin: 0 0 1rem 0;
        color: #f4f4f5;
        letter-spacing: -0.03em;
    }

    @media (min-width: 768px) {
        h2 {
            font-size: 2.5rem;
        }
    }

    .subtitle {
        color: #a1a1aa;
        font-size: 1.15rem;
        max-width: 600px;
        margin: 0 auto;
    }

    .support-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 2.5rem;
        align-items: stretch;
    }

    @media (min-width: 768px) {
        .support-grid {
            grid-template-columns: 1fr 1fr;
        }
    }

    .support-card {
        background: rgba(24, 24, 27, 0.45);
        border: 1px solid rgba(63, 63, 70, 0.4);
        border-radius: 24px;
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        text-align: left;
        min-width: 0;
        box-sizing: border-box;
    }

    @media (min-width: 768px) {
        .support-card {
            padding: 2.5rem;
        }
    }

    .highlighted-card {
        background: linear-gradient(135deg, rgba(24, 24, 27, 0.5) 0%, rgba(39, 39, 42, 0.2) 100%);
        border-color: rgba(253, 253, 150, 0.15);
    }

    .card-icon {
        font-size: 2.5rem;
        margin-bottom: 1.25rem;
    }

    h3 {
        font-size: 1.6rem;
        color: #f4f4f5;
        margin: 0 0 1rem 0;
        font-weight: 750;
        letter-spacing: -0.02em;
    }

    p {
        color: #d4d4d8;
        line-height: 1.6;
        margin: 0 0 1.5rem 0;
        font-size: 0.975rem;
    }

    .guide-list {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        margin-bottom: 2rem;
    }

    .guide-item {
        display: flex;
        gap: 1rem;
        align-items: flex-start;
    }

    .num {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        background: rgba(253, 253, 150, 0.1);
        border: 1px solid rgba(253, 253, 150, 0.3);
        color: #fdfd96;
        font-size: 0.85rem;
        font-weight: 700;
        border-radius: 50%;
        flex-shrink: 0;
    }

    .text {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .text strong {
        color: #f4f4f5;
        font-size: 0.95rem;
    }

    .text span {
        color: #a1a1aa;
        font-size: 0.875rem;
        line-height: 1.4;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.9rem 1rem;
        font-size: 0.95rem;
        font-weight: 600;
        border-radius: 12px;
        border: none;
        cursor: pointer;
        transition: all 0.2s ease;
        text-decoration: none;
        margin-top: auto;
        width: 100%;
        box-sizing: border-box;
    }

    @media (min-width: 768px) {
        .btn {
            width: auto;
            padding: 0.9rem 1.75rem;
        }
    }

    .btn-github {
        background: rgba(39, 39, 42, 0.8);
        color: #f4f4f5;
        border: 1px solid rgba(63, 63, 70, 0.8);
    }

    .btn-github:hover {
        background: rgba(63, 63, 70, 1);
        border-color: #a1a1aa;
        transform: translateY(-1px);
    }

    .donation-desc {
        font-size: 0.95rem;
        color: #a1a1aa;
        font-style: italic;
    }

    .kofi-section {
        margin: 1.5rem 0 2rem 0;
        text-align: center;
    }

    .kofi-button {
        display: inline-flex;
        align-items: center;
        gap: 0.75rem;
        background: #ff5e5b;
        color: white;
        text-decoration: none;
        font-weight: 700;
        font-size: 1.05rem;
        padding: 0.95rem 1rem;
        border-radius: 12px;
        box-shadow: 0 10px 15px -3px rgba(255, 94, 91, 0.3);
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        width: 100%;
        box-sizing: border-box;
        justify-content: center;
    }

    @media (min-width: 768px) {
        .kofi-button {
            padding: 0.95rem 2rem;
        }
    }

    .kofi-button:hover {
        background: #ff7875;
        transform: translateY(-2px) scale(1.01);
        box-shadow: 0 15px 25px -3px rgba(255, 94, 91, 0.45);
    }

    .kofi-cup {
        height: 22px;
        animation: bob 1.5s ease-in-out infinite alternate;
    }

    .other-ways {
        border-top: 1px solid rgba(63, 63, 70, 0.4);
        padding-top: 1.5rem;
    }

    .other-ways h4 {
        color: #e4e4e7;
        font-size: 0.9rem;
        margin: 0 0 0.75rem 0;
        font-weight: 600;
    }

    .other-ways ul {
        margin: 0;
        padding-left: 1.25rem;
        color: #a1a1aa;
        font-size: 0.85rem;
        line-height: 1.5;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    @keyframes bob {
        0% {
            transform: translateY(0);
        }
        100% {
            transform: translateY(-3px);
        }
    }
</style>
