<script lang="ts">
</script>

<div class="about-container">
    <div class="about-header">
        <span class="eyebrow">Our Philosophy</span>
        <h2>Why Parlez-vous?</h2>
        <p class="subtitle">A local-first, open-source language learning app built for real fluency.</p>
    </div>

    <div class="about-grid">
        <div class="about-card glassmorphic">
            <div class="card-icon">🌍</div>
            <h3>Universal Connection</h3>
            <p>
                Language is the one true thing that connects all people. By speaking another language, you build deep cultural bridges and expand your neural highways. We believe everyone deserves high-quality tools to unlock this superpower without commercial barriers.
            </p>
        </div>

        <div class="about-card glassmorphic">
            <div class="card-icon">💸</div>
            <h3>Always Free, Private & Offline</h3>
            <p>
                Accessing advanced language learning support should be entirely free. <b>Parlez-vous</b> is 100% open-source, self-hosted, and designed to run entirely locally. Your data remains on your device (using SQLite and local AI execution via LiteRT on Android), ensuring complete privacy.
            </p>
        </div>

        <div class="about-card glassmorphic highlighted-card">
            <div class="card-icon">🧠</div>
            <h3>Active vs. Inductive Learning</h3>
            <p>
                Most popular platforms (like Duolingo) rely on inductive learning — passive repetition, multiple-choice matching, and gamified loops designed to keep you on the app rather than getting you fluent. They monetize your journey with artificial paywalls.
            </p>
            <p class="highlight-text">
                <b>Parlez-vous</b> changes this by requiring <b>active production</b>. You write custom journals, practice speech directly with an interactive avatar, and draw characters by hand. You learn by doing, grading, and correcting.
            </p>
        </div>
    </div>

    <div class="mission-statement">
        <blockquote>
            "Learning a language is like learing to ride a bike. No single person is born with the ability to cycle, but with practice and hard work you'll be shocked by the roads it unlocks."
        </blockquote>
    </div>
</div>

<style>
    .about-container {
        max-width: 1100px;
        margin: 0 auto;
        padding: 4rem 1.5rem;
    }

    .about-header {
        text-align: center;
        margin-bottom: 4rem;
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
        font-size: 2.5rem;
        font-weight: 850;
        margin: 0 0 1rem 0;
        color: #f4f4f5;
        letter-spacing: -0.03em;
    }

    .subtitle {
        color: #a1a1aa;
        font-size: 1.15rem;
        max-width: 600px;
        margin: 0 auto;
    }

    .about-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 2rem;
        margin-bottom: 4rem;
    }

    @media (min-width: 768px) {
        .about-grid {
            grid-template-columns: 1fr 1fr;
        }
        
        .highlighted-card {
            grid-column: span 2;
        }
    }

    .about-card {
        background: rgba(24, 24, 27, 0.45);
        border: 1px solid rgba(63, 63, 70, 0.4);
        border-radius: 20px;
        padding: 2.5rem;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        text-align: left;
    }

    .about-card:hover {
        transform: translateY(-4px);
        border-color: rgba(253, 253, 150, 0.25);
        background: rgba(24, 24, 27, 0.55);
        box-shadow: 0 12px 20px -8px rgba(0, 0, 0, 0.4);
    }

    .card-icon {
        font-size: 2.25rem;
        margin-bottom: 1.25rem;
    }

    h3 {
        font-size: 1.45rem;
        color: #f4f4f5;
        margin: 0 0 1rem 0;
        font-weight: 700;
        letter-spacing: -0.01em;
    }

    .about-card p {
        color: #d4d4d8;
        line-height: 1.6;
        margin: 0;
        font-size: 0.975rem;
    }

    .about-card p + p {
        margin-top: 1rem;
    }

    .highlighted-card {
        background: linear-gradient(135deg, rgba(24, 24, 27, 0.5) 0%, rgba(39, 39, 42, 0.2) 100%);
        border-color: rgba(253, 253, 150, 0.15);
    }

    .highlight-text {
        color: #f4f4f5;
        font-weight: 500;
        border-left: 3px solid #fdfd96;
        padding-left: 1rem;
    }

    .mission-statement {
        text-align: center;
        max-width: 800px;
        margin: 3rem auto 0 auto;
        border-top: 1px solid rgba(63, 63, 70, 0.3);
        padding-top: 3rem;
    }

    blockquote {
        font-size: 1.45rem;
        font-style: italic;
        color: #e4e4e7;
        line-height: 1.5;
        margin: 0;
        font-weight: 500;
    }
</style>
