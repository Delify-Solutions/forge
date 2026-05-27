<?php
// Plain PHP starter — replace this file with your application code.
$site_domain = $_SERVER['HTTP_HOST'] ?? 'localhost';
$php_version = PHP_VERSION;

if (isset($_GET['phpinfo'])) {
    phpinfo();
    exit;
}
?>
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title><?= htmlspecialchars($site_domain, ENT_QUOTES, 'UTF-8') ?> — Plain PHP</title>
<style>
    body { font-family: ui-sans-serif, system-ui, sans-serif; max-width: 640px; margin: 80px auto; padding: 0 24px; color: #0f172a; }
    h1 { font-size: 2rem; margin-bottom: 0.5rem; }
    p { color: #475569; line-height: 1.7; }
    a { color: #1857b8; }
    code { background: #f1f5f9; border-radius: 4px; padding: 0.1em 0.4em; font-size: 0.9em; }
</style>
</head>
<body>
    <h1><?= htmlspecialchars($site_domain, ENT_QUOTES, 'UTF-8') ?></h1>
    <p>Your plain PHP project is ready. PHP <?= htmlspecialchars($php_version, ENT_QUOTES, 'UTF-8') ?> is running.</p>
    <p>
        <a href="?phpinfo=1">View phpinfo()</a> &mdash;
        Replace <code>index.php</code> with your application code.
    </p>
</body>
</html>
