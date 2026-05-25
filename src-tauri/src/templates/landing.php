<?php
$forge_site_name = '__SITE_NAME__';
$forge_domain = $forge_site_name . '.test';
$forge_php_version = PHP_VERSION;
$forge_document_root = __DIR__;
$forge_server = $_SERVER['SERVER_SOFTWARE'] ?? 'Unknown';

if (isset($_GET['phpinfo'])) {
    phpinfo();
    exit;
}

function forge_e($value) {
    return htmlspecialchars((string) $value, ENT_QUOTES, 'UTF-8');
}
?>
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title><?= forge_e($forge_domain) ?> is ready — Delify Forge</title>
<style>
    :root {
        color-scheme: light;
        --ink: #0f172a;
        --muted: #475569;
        --line: #d8e4f5;
        --blue-900: #123f85;
        --blue-700: #1857b8;
        --blue-50: #f7fbff;
        --green: #16a34a;
        --paper: #ffffff;
    }

    * {
        box-sizing: border-box;
    }

    body {
        margin: 0;
        min-height: 100vh;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        color: var(--ink);
        background:
            radial-gradient(circle at 18% 0%, rgba(24, 87, 184, 0.09), transparent 30rem),
            linear-gradient(180deg, #ffffff 0%, #f7fbff 100%);
    }

    a {
        color: inherit;
    }

    .page {
        width: min(960px, calc(100% - 48px));
        margin: 0 auto;
        padding: 64px 0;
    }

    .shell {
        overflow: hidden;
        border: 1px solid var(--line);
        border-radius: 24px;
        background: rgba(255, 255, 255, 0.92);
        box-shadow: 0 22px 70px rgba(18, 63, 133, 0.11);
    }

    .topbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 18px 24px;
        border-bottom: 1px solid var(--line);
    }

    .brand,
    .status {
        display: inline-flex;
        align-items: center;
        gap: 10px;
        font-weight: 750;
    }

    .brand {
        color: var(--blue-900);
        letter-spacing: -0.02em;
    }

    .mark {
        width: 28px;
        height: 28px;
        border-radius: 9px;
        background: var(--blue-700);
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.32), 0 10px 22px rgba(24, 87, 184, 0.18);
    }

    .status {
        color: var(--muted);
        font-size: 14px;
    }

    .dot {
        width: 9px;
        height: 9px;
        border-radius: 999px;
        background: var(--green);
        box-shadow: 0 0 0 4px rgba(22, 163, 74, 0.12);
    }

    .hero {
        display: grid;
        grid-template-columns: minmax(0, 1.08fr) minmax(300px, 0.92fr);
        gap: 42px;
        padding: 58px;
    }

    .eyebrow {
        margin: 0 0 16px;
        color: var(--blue-700);
        font-size: 12px;
        font-weight: 850;
        letter-spacing: 0.12em;
        text-transform: uppercase;
    }

    h1 {
        margin: 0;
        color: var(--ink);
        font-size: clamp(34px, 5vw, 56px);
        line-height: 1.04;
        letter-spacing: -0.055em;
    }

    .domain {
        color: var(--blue-700);
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        letter-spacing: -0.05em;
    }

    .lead {
        margin: 22px 0 0;
        color: var(--muted);
        font-size: 16px;
        line-height: 1.7;
    }

    .actions {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        margin-top: 30px;
    }

    .button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-height: 46px;
        padding: 0 18px;
        border-radius: 999px;
        font-size: 15px;
        font-weight: 800;
        text-decoration: none;
        transition: transform 160ms ease, box-shadow 160ms ease, background 160ms ease;
    }

    .button:focus-visible {
        outline: 2px solid rgba(24, 87, 184, 0.35);
        outline-offset: 3px;
    }

    .button.primary {
        color: #ffffff;
        background: var(--blue-700);
        box-shadow: 0 10px 24px rgba(24, 87, 184, 0.18);
    }

    .button.secondary {
        color: var(--blue-900);
        border: 1px solid var(--line);
        background: #ffffff;
    }

    .button:hover {
        transform: translateY(-1px);
    }

    .button.primary:hover {
        background: var(--blue-900);
    }

    .panel {
        align-self: start;
        overflow: hidden;
        border: 1px solid var(--line);
        border-radius: 18px;
        background: var(--paper);
    }

    .panel h2 {
        margin: 0;
        padding: 18px 20px;
        border-bottom: 1px solid var(--line);
        color: var(--blue-900);
        font-size: 15px;
        letter-spacing: -0.01em;
    }

    .facts {
        display: grid;
        gap: 1px;
        background: var(--line);
    }

    .fact {
        display: grid;
        gap: 7px;
        padding: 16px 20px;
        background: var(--blue-50);
    }

    .label {
        color: var(--muted);
        font-size: 11px;
        font-weight: 850;
        letter-spacing: 0.09em;
        text-transform: uppercase;
    }

    .value {
        overflow-wrap: anywhere;
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        font-size: 13px;
        line-height: 1.55;
    }

    .note {
        margin: 0;
        padding: 18px 20px;
        color: var(--muted);
        font-size: 14px;
        line-height: 1.65;
    }

    code {
        border: 1px solid var(--line);
        border-radius: 7px;
        padding: 0.1em 0.36em;
        color: var(--blue-900);
        background: var(--blue-50);
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        font-size: 0.92em;
    }

    .footer {
        display: flex;
        justify-content: space-between;
        gap: 18px;
        padding: 20px 58px 26px;
        border-top: 1px solid var(--line);
        color: var(--muted);
        font-size: 14px;
        line-height: 1.6;
    }

    @media (prefers-reduced-motion: reduce) {
        .button {
            transition: none;
        }
    }

    @media (max-width: 820px) {
        .page {
            width: min(100% - 24px, 960px);
            padding: 24px 0;
        }

        .hero {
            grid-template-columns: 1fr;
            padding: 34px 24px;
        }

        .topbar,
        .footer {
            align-items: flex-start;
            flex-direction: column;
            padding-left: 24px;
            padding-right: 24px;
        }
    }
</style>
</head>
<body>
    <main class="page">
        <section class="shell" aria-label="Delify Forge default site page">
            <header class="topbar">
                <div class="brand">
                    <span class="mark" aria-hidden="true"></span>
                    <span>Delify Forge</span>
                </div>
                <div class="status">
                    <span class="dot" aria-hidden="true"></span>
                    <span>Running</span>
                </div>
            </header>

            <div class="hero">
                <div>
                    <p class="eyebrow">Local PHP environment</p>
                    <h1><span class="domain"><?= forge_e($forge_domain) ?></span> is ready.</h1>
                    <p class="lead">
                        Forge created this page to confirm your <code>.test</code> domain,
                        PHP runtime, and web server are wired correctly. Replace this
                        <code>index.php</code> file with your app when you are ready.
                    </p>
                    <nav class="actions" aria-label="Quick actions">
                        <a class="button primary" href="?phpinfo=1">View phpinfo()</a>
                        <a class="button secondary" href="https://github.com/Delify-Solutions/forge" target="_blank" rel="noopener">
                            Source on GitHub
                        </a>
                    </nav>
                </div>

                <aside class="panel" aria-label="Environment information">
                    <h2>Environment info</h2>
                    <div class="facts">
                        <div class="fact">
                            <span class="label">Domain</span>
                            <span class="value"><?= forge_e($forge_domain) ?></span>
                        </div>
                        <div class="fact">
                            <span class="label">PHP</span>
                            <span class="value"><?= forge_e($forge_php_version) ?></span>
                        </div>
                        <div class="fact">
                            <span class="label">Document root</span>
                            <span class="value"><?= forge_e($forge_document_root) ?></span>
                        </div>
                        <div class="fact">
                            <span class="label">Web server</span>
                            <span class="value"><?= forge_e($forge_server) ?></span>
                        </div>
                    </div>
                    <p class="note">
                        This page is self-contained and works offline, so your first local
                        site stays useful even before a framework is installed.
                    </p>
                </aside>
            </div>

            <footer class="footer">
                <span>Open-source local development for PHP, Nginx, DNS, and more.</span>
                <span>Built by Delify Solutions.</span>
            </footer>
        </section>
    </main>
</body>
</html>
