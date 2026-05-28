import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import {
    Code,
    FileText,
    FolderOpen,
    Globe,
    Loader2,
    Plus,
    Search,
    Terminal,
    Trash2,
    AlertTriangle,
    Link2,
    X,
    RefreshCw,
    Play,
} from 'lucide-react';

import { PageHeader } from '@/components/PageHeader';
import { ApacheInstallDialog } from '@/components/ApacheInstallDialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { tauri } from '@/lib/tauri';
import type {
    BundleEntry,
    ComposerStatus,
    MkcertStatus,
    ProjectTemplate,
    Site,
    SiteLogsTail,
    WebServer,
} from '@/types';

export function Sites() {
    const { t } = useTranslation();
    const [sites, setSites] = useState<Site[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [mkcertStatus, setMkcertStatus] = useState<MkcertStatus | null>(null);
    const [caInstalling, setCaInstalling] = useState(false);
    const [caInstallError, setCaInstallError] = useState<string | null>(null);
    const [apacheState, setApacheState] = useState<'stopped' | 'running' | 'crashed'>('stopped');
    const [bundles, setBundles] = useState<BundleEntry[]>([]);
    const [apacheStarting, setApacheStarting] = useState(false);

    const normalizedSearch = searchQuery.trim().toLowerCase();
    const filteredSites = normalizedSearch
        ? sites.filter((site) =>
              [site.name, site.domain, site.path, ...site.aliases].some((value) =>
                  value.toLowerCase().includes(normalizedSearch),
              ),
          )
        : sites;

    const refresh = async () => {
        setLoading(true);
        setError(null);
        try {
            const [siteList, status, bundleList] = await Promise.all([
                tauri.listSites(),
                tauri.mkcertStatus(),
                tauri.listBundles(),
            ]);
            setSites(siteList);
            setMkcertStatus(status);
            setBundles(bundleList);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load.');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        void refresh();
    }, []);

    // Poll services_status every 2 seconds to track apache state.
    useEffect(() => {
        const poll = async () => {
            try {
                const statuses = await tauri.servicesStatus();
                const apacheSvc = statuses.find((s) => s.name === 'apache');
                setApacheState(apacheSvc?.state ?? 'stopped');
            } catch {
                // ignore poll errors
            }
        };
        void poll();
        const id = window.setInterval(() => void poll(), 2000);
        return () => window.clearInterval(id);
    }, []);

    const onRemove = async (id: number) => {
        try {
            await tauri.removeSite(id);
            await refresh();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to remove.');
        }
    };

    const handleInstallCa = async () => {
        setCaInstalling(true);
        setCaInstallError(null);
        try {
            await tauri.installMkcertCa();
            const status = await tauri.mkcertStatus();
            setMkcertStatus(status);
        } catch (e) {
            const message = e instanceof Error ? e.message : String(e);
            setCaInstallError(message);
        } finally {
            setCaInstalling(false);
        }
    };

    const handleStartApache = async () => {
        setApacheStarting(true);
        try {
            await tauri.startApache();
            const statuses = await tauri.servicesStatus();
            const apacheSvc = statuses.find((s) => s.name === 'apache');
            setApacheState(apacheSvc?.state ?? 'stopped');
        } catch {
            // ignore — user can retry from Services page
        } finally {
            setApacheStarting(false);
        }
    };

    const apacheSiteCount = sites.filter((s) => s.webServer === 'apache').length;
    const showApacheBanner = apacheState !== 'running' && apacheSiteCount > 0;

    return (
        <div>
            <div className="mb-6 flex items-start justify-between gap-4">
                <PageHeader
                    title={t('sites.title')}
                    description={t('sites.subtitle')}
                />
                <div className="flex items-center gap-2">
                    <div className="relative">
                        <Search className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            placeholder={t('sites.searchPlaceholder')}
                            className="w-64 pl-9 pr-8"
                        />
                        {searchQuery && (
                            <button
                                type="button"
                                onClick={() => setSearchQuery('')}
                                className="absolute right-2 top-1/2 -translate-y-1/2 rounded-sm text-muted-foreground hover:text-foreground"
                                title={t('sites.searchClear')}
                                aria-label={t('sites.searchClear')}
                            >
                                <X className="h-4 w-4" />
                            </button>
                        )}
                    </div>
                    <Button onClick={() => setDialogOpen(true)}>
                        <Plus />
                        {t('sites.addButton')}
                    </Button>
                </div>
            </div>

            {mkcertStatus && !mkcertStatus.found && (
                <div
                    className="mb-4 flex items-start gap-3 rounded-md border border-border bg-muted/40 px-3 py-2.5 text-sm"
                    role="status"
                >
                    <AlertTriangle className="mt-0.5 h-4 w-4 text-muted-foreground" />
                    <span className="text-foreground">
                        {t('sites.httpsBannerInstallMkcert')}
                    </span>
                </div>
            )}
            {mkcertStatus && mkcertStatus.found && !mkcertStatus.caInstalled && (
                <div
                    className={`mb-4 flex items-start gap-3 rounded-md border px-3 py-2.5 text-sm ${
                        caInstallError
                            ? 'border-destructive/40 bg-destructive/10'
                            : 'border-border bg-muted/40'
                    }`}
                    role={caInstallError ? 'alert' : 'status'}
                >
                    <AlertTriangle
                        className={`mt-0.5 h-4 w-4 ${
                            caInstallError ? 'text-destructive' : 'text-muted-foreground'
                        }`}
                    />
                    <div className="flex-1">
                        <p className="text-foreground">
                            {t('sites.httpsBannerInstallCa')}
                        </p>
                        {caInstallError && (
                            <p className="mt-1 text-xs text-destructive">
                                {caInstallError}
                            </p>
                        )}
                    </div>
                    <Button
                        size="sm"
                        variant="outline"
                        onClick={handleInstallCa}
                        disabled={caInstalling}
                    >
                        {caInstalling && <Loader2 className="mr-1 h-3 w-3 animate-spin" />}
                        {caInstalling
                            ? t('sites.httpsBannerInstalling')
                            : t('sites.httpsCaInstallAction')}
                    </Button>
                </div>
            )}

            {showApacheBanner && (
                <div
                    className="mb-4 flex items-start gap-3 rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2.5 text-sm"
                    role="status"
                >
                    <AlertTriangle className="mt-0.5 h-4 w-4 text-amber-500" />
                    <span className="flex-1 text-foreground">
                        {t('sites.apacheStoppedBanner', { count: apacheSiteCount })}
                    </span>
                    <Button
                        size="sm"
                        variant="outline"
                        onClick={handleStartApache}
                        disabled={apacheStarting}
                    >
                        {apacheStarting ? (
                            <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                        ) : (
                            <Play className="mr-1 h-3 w-3" />
                        )}
                        {t('sites.apacheStartButton')}
                    </Button>
                </div>
            )}

            {error && (
                <div className="mb-4 flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5 text-sm">
                    <AlertTriangle className="mt-0.5 h-4 w-4 text-destructive" />
                    <span className="text-foreground">{error}</span>
                </div>
            )}

            {loading ? (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    {t('sites.loading')}
                </div>
            ) : sites.length === 0 ? (
                <EmptyState onAdd={() => setDialogOpen(true)} />
            ) : filteredSites.length === 0 ? (
                <p className="rounded-md border border-dashed border-border bg-muted/30 px-3 py-6 text-center text-sm text-muted-foreground">
                    {t('sites.searchNoMatches', { query: searchQuery.trim() })}
                </p>
            ) : (
                <SiteTable
                    sites={filteredSites}
                    onRemove={onRemove}
                    onRefresh={refresh}
                    mkcertStatus={mkcertStatus}
                    bundles={bundles}
                />
            )}

            <AddSiteDialog
                open={dialogOpen}
                onOpenChange={setDialogOpen}
                onAdded={refresh}
                bundles={bundles}
            />
        </div>
    );
}

function EmptyState({ onAdd }: { onAdd: () => void }) {
    const { t } = useTranslation();
    return (
        <div className="rounded-lg border border-dashed border-border bg-card p-10 text-center">
            <FolderOpen className="mx-auto mb-3 h-8 w-8 text-muted-foreground" />
            <p className="text-sm font-medium">{t('sites.emptyTitle')}</p>
            <p className="mt-1 text-sm text-muted-foreground">
                {t('sites.emptySubtitle')}
            </p>
            <Button className="mt-4" onClick={onAdd}>
                <Plus />
                {t('sites.emptyAction')}
            </Button>
        </div>
    );
}

function SiteTable({
    sites,
    onRemove,
    onRefresh,
    mkcertStatus,
    bundles,
}: {
    sites: Site[];
    onRemove: (id: number) => void;
    onRefresh: () => void;
    mkcertStatus: MkcertStatus | null;
    bundles: BundleEntry[];
}) {
    const { t } = useTranslation();
    const [phpLines, setPhpLines] = useState<string[]>([]);
    const [aliasSite, setAliasSite] = useState<Site | null>(null);
    const [logsSite, setLogsSite] = useState<Site | null>(null);
    const [actionError, setActionError] = useState<string | null>(null);
    const [apacheInstallSiteId, setApacheInstallSiteId] = useState<number | null>(null);

    useEffect(() => {
        tauri.scanSystem().then((r) => setPhpLines(r.installedPhpLines ?? []));
    }, []);

    useEffect(() => {
        if (!aliasSite) return;
        const fresh = sites.find((s) => s.id === aliasSite.id);
        if (fresh && fresh !== aliasSite) {
            setAliasSite(fresh);
        }
    }, [sites, aliasSite]);

    useEffect(() => {
        if (!logsSite) return;
        const fresh = sites.find((s) => s.id === logsSite.id);
        if (fresh && fresh !== logsSite) {
            setLogsSite(fresh);
        }
    }, [sites, logsSite]);

    const handlePhpChange = async (siteId: number, newVersion: string) => {
        try {
            await tauri.updateSitePhp(siteId, newVersion);
            onRefresh();
        } catch {
            // silently fail — row will keep old value
        }
    };

    const handleEngineChange = async (siteId: number, newEngine: string) => {
        if (newEngine === 'apache') {
            const apacheBundle = bundles.find((b) => b.engine === 'apache');
            if (!apacheBundle?.installed) {
                setApacheInstallSiteId(siteId);
                return;
            }
        }
        try {
            await tauri.updateSiteWebServer(siteId, newEngine);
            onRefresh();
        } catch {
            // silently fail — row will keep old value
        }
    };

    const runAction = async (action: () => Promise<void>) => {
        setActionError(null);
        try {
            await action();
        } catch (e) {
            const message = e instanceof Error ? e.message : String(e);
            setActionError(
                message.includes('No editor found')
                    ? t('sites.editorNone')
                    : message || 'Action failed.',
            );
        }
    };

    const mkcertMissing = !mkcertStatus || !mkcertStatus.found;
    const caMissing = mkcertStatus?.found && !mkcertStatus.caInstalled;

    return (
        <div className="space-y-3">
            {actionError && (
                <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5 text-sm">
                    <AlertTriangle className="mt-0.5 h-4 w-4 text-destructive" />
                    <span className="text-foreground">{actionError}</span>
                </div>
            )}
            <div className="overflow-x-auto rounded-lg border border-border bg-card">
                <table className="w-full min-w-[860px] text-sm">
                    <thead className="bg-muted/50 text-xs uppercase tracking-wide text-muted-foreground">
                        <tr>
                            <th className="px-4 py-2 text-left font-medium">
                                {t('sites.domainHeader')}
                            </th>
                            <th className="px-4 py-2 text-left font-medium">
                                {t('sites.aliasesHeader')}
                            </th>
                            <th className="px-4 py-2 text-left font-medium">
                                {t('sites.pathHeader')}
                            </th>
                            <th className="px-4 py-2 text-left font-medium">
                                {t('sites.phpHeader')}
                            </th>
                            <th className="px-4 py-2 text-left font-medium">
                                {t('sites.engineHeader')}
                            </th>
                            <th className="px-4 py-2 text-center font-medium">
                                {t('sites.httpsHeader')}
                            </th>
                            <th className="px-4 py-2 text-right font-medium">
                                {t('sites.actionsHeader')}
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {sites.map((site) => (
                            <tr
                                key={site.id}
                                className="border-t border-border last:border-b-0"
                            >
                                <td className="px-4 py-2.5 font-medium">
                                    <a
                                        href={`${site.httpsEnabled ? 'https://' : 'http://'}${site.domain}`}
                                        target="_blank"
                                        rel="noreferrer"
                                        className="hover:underline"
                                    >
                                        {site.domain}
                                    </a>
                                </td>
                                <td className="px-4 py-2.5 text-xs text-muted-foreground">
                                    <button
                                        type="button"
                                        onClick={() => setAliasSite(site)}
                                        className="inline-flex items-center gap-1.5 rounded-md border border-input bg-muted/40 px-2 py-1 text-left hover:bg-muted"
                                    >
                                        <Link2 className="h-3 w-3" />
                                        <span>
                                            {site.aliases.length === 0
                                                ? t('sites.aliasesEmpty')
                                                : site.aliases.join(', ')}
                                        </span>
                                    </button>
                                </td>
                                <td
                                    className="max-w-[260px] truncate px-4 py-2.5 text-muted-foreground"
                                    title={site.path}
                                >
                                    {site.path}
                                </td>
                                <td className="px-4 py-2.5 text-muted-foreground">
                                    {phpLines.length > 0 ? (
                                        <select
                                            value={site.phpVersion}
                                            onChange={(e) =>
                                                handlePhpChange(
                                                    site.id,
                                                    e.target.value,
                                                )
                                            }
                                            className="h-7 rounded-md border border-input bg-muted/50 px-2 text-xs"
                                        >
                                            {phpLines.map((line) => (
                                                <option key={line} value={line}>
                                                    {line}
                                                </option>
                                            ))}
                                        </select>
                                    ) : (
                                        site.phpVersion
                                    )}
                                </td>
                                <td className="px-4 py-2.5 text-muted-foreground">
                                    <select
                                        value={site.webServer}
                                        onChange={(e) =>
                                            handleEngineChange(
                                                site.id,
                                                e.target.value,
                                            )
                                        }
                                        className="h-7 rounded-md border border-input bg-muted/50 px-2 text-xs"
                                    >
                                        <option value="nginx">
                                            {t('sites.engineNginx')}
                                        </option>
                                        <option value="apache">
                                            {t('sites.engineApache')}
                                        </option>
                                        <option value="openlitespeed" disabled>
                                            {t('sites.engineOls')}{' '}
                                            {t('sites.engineComingSoon')}
                                        </option>
                                    </select>
                                </td>
                                <td className="px-4 py-2.5 text-center">
                                    <HttpsToggle
                                        site={site}
                                        disabledReason={
                                            mkcertMissing
                                                ? t('sites.httpsBlockedNoMkcert')
                                                : caMissing
                                                  ? t('sites.httpsBlockedNoCa')
                                                  : undefined
                                        }
                                        onRefresh={onRefresh}
                                        onError={(msg) => setActionError(msg)}
                                    />
                                </td>
                                <td className="whitespace-nowrap px-4 py-2.5 text-right">
                                    <div className="inline-flex items-center justify-end gap-1">
                                        <IconActionButton
                                            label={t('sites.actionOpen')}
                                            onClick={() =>
                                                runAction(() =>
                                                    tauri.openSiteUrl(site.id),
                                                )
                                            }
                                        >
                                            <Globe />
                                        </IconActionButton>
                                        <IconActionButton
                                            label={t('sites.actionReveal')}
                                            onClick={() =>
                                                runAction(() =>
                                                    tauri.revealSitePath(site.id),
                                                )
                                            }
                                        >
                                            <FolderOpen />
                                        </IconActionButton>
                                        <IconActionButton
                                            label={t('sites.actionTerminal')}
                                            onClick={() =>
                                                runAction(() =>
                                                    tauri.openSiteTerminal(site.id),
                                                )
                                            }
                                        >
                                            <Terminal />
                                        </IconActionButton>
                                        <IconActionButton
                                            label={t('sites.actionEditor')}
                                            onClick={() =>
                                                runAction(() =>
                                                    tauri.openSiteInEditor(site.id),
                                                )
                                            }
                                        >
                                            <Code />
                                        </IconActionButton>
                                        <IconActionButton
                                            label={t('sites.actionLogs')}
                                            onClick={() => setLogsSite(site)}
                                        >
                                            <FileText />
                                        </IconActionButton>
                                        <IconActionButton
                                            label={t('sites.removeAction')}
                                            onClick={() => onRemove(site.id)}
                                        >
                                            <Trash2 />
                                        </IconActionButton>
                                    </div>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
                <AliasDialog
                    site={aliasSite}
                    onClose={() => setAliasSite(null)}
                    onChanged={onRefresh}
                />
                <LogsDialog site={logsSite} onClose={() => setLogsSite(null)} />
                <ApacheInstallDialog
                    open={apacheInstallSiteId !== null}
                    onClose={() => setApacheInstallSiteId(null)}
                    onInstalled={async () => {
                        setApacheInstallSiteId(null);
                        if (apacheInstallSiteId !== null) {
                            try {
                                await tauri.updateSiteWebServer(apacheInstallSiteId, 'apache');
                                onRefresh();
                            } catch {
                                // silently fail — row will keep old value
                            }
                        }
                    }}
                />
            </div>
        </div>
    );
}

function HttpsToggle({
    site,
    disabledReason,
    onRefresh,
    onError,
}: {
    site: Site;
    disabledReason?: string;
    onRefresh: () => void;
    onError: (msg: string) => void;
}) {
    const { t } = useTranslation();
    const [busy, setBusy] = useState(false);
    const disabled = !!disabledReason;

    const toggle = async () => {
        if (disabled || busy) return;
        setBusy(true);
        try {
            await tauri.updateSiteHttps(site.id, !site.httpsEnabled);
            onRefresh();
        } catch (e) {
            const message = e instanceof Error ? e.message : String(e);
            onError(message || t('sites.httpsCaInstallFailed'));
        } finally {
            setBusy(false);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === ' ' || e.key === 'Enter') {
            e.preventDefault();
            void toggle();
        }
    };

    const enabled = site.httpsEnabled;
    const label = busy
        ? t('sites.httpsBusy')
        : enabled
          ? t('sites.httpsDisable')
          : t('sites.httpsEnable');

    return (
        <button
            type="button"
            role="switch"
            aria-checked={enabled}
            aria-label={`${label} — ${site.domain}`}
            aria-disabled={disabled}
            title={disabled ? disabledReason : label}
            onClick={toggle}
            onKeyDown={handleKeyDown}
            tabIndex={disabled ? -1 : 0}
            className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                disabled
                    ? 'cursor-not-allowed bg-muted opacity-60'
                    : busy
                      ? 'cursor-wait bg-muted'
                      : enabled
                        ? 'bg-primary'
                        : 'bg-input'
            }`}
        >
            <span
                className={`inline-block h-3.5 w-3.5 rounded-full bg-background shadow-sm transition-transform ${
                    enabled ? 'translate-x-[18px]' : 'translate-x-0.5'
                }`}
            />
            {busy && (
                <Loader2 className="absolute left-1/2 top-1/2 h-3 w-3 -translate-x-1/2 -translate-y-1/2 animate-spin text-muted-foreground" />
            )}
        </button>
    );
}

function IconActionButton({
    label,
    onClick,
    children,
}: {
    label: string;
    onClick: () => void;
    children: React.ReactNode;
}) {
    return (
        <Button
            type="button"
            size="sm"
            variant="ghost"
            onClick={onClick}
            title={label}
            aria-label={label}
            className="h-8 w-8 p-0"
        >
            {children}
        </Button>
    );
}

function LogsDialog({
    site,
    onClose,
}: {
    site: Site | null;
    onClose: () => void;
}) {
    const { t } = useTranslation();
    const [activeTab, setActiveTab] = useState<'error' | 'access'>('error');
    const [logs, setLogs] = useState<SiteLogsTail | null>(null);
    const [loading, setLoading] = useState(false);
    const [err, setErr] = useState<string | null>(null);
    const errorTabRef = useRef<HTMLButtonElement>(null);
    const accessTabRef = useRef<HTMLButtonElement>(null);

    const loadLogs = async (siteId: number) => {
        setLoading(true);
        setErr(null);
        try {
            setLogs(await tauri.tailSiteLogs(siteId));
        } catch (e) {
            setErr(e instanceof Error ? e.message : 'Failed to load logs.');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (!site) return;
        setActiveTab('error');
        setLogs(null);
        void loadLogs(site.id);
    }, [site?.id]);

    if (!site) return null;

    const lines = activeTab === 'error' ? logs?.error : logs?.access;
    const missing = activeTab === 'error' ? logs?.errorMissing : logs?.accessMissing;

    const handleTabKeyDown = (e: React.KeyboardEvent) => {
        if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return;

        e.preventDefault();
        const nextTab = activeTab === 'error' ? 'access' : 'error';
        setActiveTab(nextTab);
        if (nextTab === 'error') {
            errorTabRef.current?.focus();
        } else {
            accessTabRef.current?.focus();
        }
    };

    return (
        <Dialog
            open={!!site}
            onOpenChange={(o) => {
                if (!o) onClose();
            }}
        >
            <DialogContent className="max-w-3xl">
                <DialogHeader>
                    <DialogTitle>
                        {t('sites.logsDialogTitle', { domain: site.domain })}
                    </DialogTitle>
                    <DialogDescription>{site.path}</DialogDescription>
                </DialogHeader>

                <div className="space-y-3">
                    <div className="flex items-center justify-between gap-3">
                        <div
                            role="tablist"
                            aria-label="Log type"
                            className="inline-flex rounded-md border border-input bg-muted/40 p-1 text-xs"
                            onKeyDown={handleTabKeyDown}
                        >
                            <button
                                ref={errorTabRef}
                                type="button"
                                role="tab"
                                id="logs-tab-error"
                                aria-selected={activeTab === 'error'}
                                tabIndex={activeTab === 'error' ? 0 : -1}
                                onClick={() => setActiveTab('error')}
                                className={`rounded px-3 py-1.5 ${
                                    activeTab === 'error'
                                        ? 'bg-background text-foreground shadow-sm'
                                        : 'text-muted-foreground hover:text-foreground'
                                }`}
                            >
                                {t('sites.logsTabError')}
                            </button>
                            <button
                                ref={accessTabRef}
                                type="button"
                                role="tab"
                                id="logs-tab-access"
                                aria-selected={activeTab === 'access'}
                                tabIndex={activeTab === 'access' ? 0 : -1}
                                onClick={() => setActiveTab('access')}
                                className={`rounded px-3 py-1.5 ${
                                    activeTab === 'access'
                                        ? 'bg-background text-foreground shadow-sm'
                                        : 'text-muted-foreground hover:text-foreground'
                                }`}
                            >
                                {t('sites.logsTabAccess')}
                            </button>
                        </div>
                        <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => loadLogs(site.id)}
                            disabled={loading}
                        >
                            {loading ? (
                                <Loader2 className="animate-spin" />
                            ) : (
                                <RefreshCw />
                            )}
                            {t('sites.logsRefresh')}
                        </Button>
                    </div>

                    {err ? (
                        <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5 text-sm">
                            <AlertTriangle className="mt-0.5 h-4 w-4 text-destructive" />
                            <span className="text-foreground">{err}</span>
                        </div>
                    ) : loading && !logs ? (
                        <div className="flex items-center gap-2 rounded-md border border-border bg-muted/30 px-3 py-8 text-sm text-muted-foreground">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            {t('sites.logsLoading')}
                        </div>
                    ) : missing ? (
                        <p
                            role="tabpanel"
                            aria-labelledby={`logs-tab-${activeTab}`}
                            tabIndex={0}
                            className="rounded-md border border-dashed border-border bg-muted/30 px-3 py-8 text-center text-sm text-muted-foreground"
                        >
                            {t('sites.logsMissing')}
                        </p>
                    ) : !lines || lines.length === 0 ? (
                        <p
                            role="tabpanel"
                            aria-labelledby={`logs-tab-${activeTab}`}
                            tabIndex={0}
                            className="rounded-md border border-dashed border-border bg-muted/30 px-3 py-8 text-center text-sm text-muted-foreground"
                        >
                            {t('sites.logsEmpty')}
                        </p>
                    ) : (
                        <pre
                            role="tabpanel"
                            aria-labelledby={`logs-tab-${activeTab}`}
                            tabIndex={0}
                            className="max-h-[420px] overflow-auto whitespace-pre-wrap rounded-md border border-border bg-muted/30 p-3 font-mono text-xs leading-relaxed text-foreground"
                        >
                            {lines.join('\n')}
                        </pre>
                    )}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={onClose}>
                        {t('sites.logsClose')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

function AddSiteDialog({
    open,
    onOpenChange,
    onAdded,
    bundles,
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onAdded: () => void;
    bundles: BundleEntry[];
}) {
    const { t } = useTranslation();
    const [name, setName] = useState('');
    const [path, setPath] = useState('');
    const [phpVersion, setPhpVersion] = useState('');
    const [webServer, setWebServer] = useState<WebServer>('nginx');
    const [template, setTemplate] = useState<ProjectTemplate>('none');
    const [phpLines, setPhpLines] = useState<string[]>([]);
    const [composerStatus, setComposerStatus] = useState<ComposerStatus | null>(null);
    const [submitting, setSubmitting] = useState(false);
    const [err, setErr] = useState<string | null>(null);
    const [apacheInstallOpen, setApacheInstallOpen] = useState(false);
    const [composerInstallOpen, setComposerInstallOpen] = useState(false);

    useEffect(() => {
        if (open) {
            tauri.scanSystem().then((r) => {
                const lines = r.installedPhpLines ?? [];
                setPhpLines(lines);
                if (lines.length > 0) {
                    setPhpVersion((prev) => prev || lines[0]);
                }
            });
            tauri.composerStatus().then(setComposerStatus);
        }
    }, [open]);

    const reset = () => {
        setName('');
        setPath('');
        setPhpVersion(phpLines.length > 0 ? phpLines[0] : '');
        setWebServer('nginx');
        setTemplate('none');
        setErr(null);
    };

    const browse = async () => {
        try {
            const selected = await openDialog({
                directory: true,
                multiple: false,
            });
            if (typeof selected === 'string') {
                setPath(selected);
                if (!name) {
                    const segments = selected.split('/');
                    const last = segments[segments.length - 1] ?? '';
                    setName(
                        last
                            .toLowerCase()
                            .replace(/[^a-z0-9-]+/g, '-')
                            .replace(/^-+|-+$/g, ''),
                    );
                }
            }
        } catch (e) {
            setErr(e instanceof Error ? e.message : 'Failed to pick folder.');
        }
    };

    const submit = async () => {
        setErr(null);
        setSubmitting(true);
        try {
            if (template === 'none') {
                await tauri.addSite(name, path, phpVersion || undefined, webServer);
            } else {
                await tauri.scaffoldAndAddSite(
                    template,
                    name,
                    path,
                    phpVersion || undefined,
                    webServer,
                );
            }
            reset();
            onOpenChange(false);
            onAdded();
        } catch (e) {
            setErr(e instanceof Error ? e.message : t('sites.scaffoldFailed'));
        } finally {
            setSubmitting(false);
        }
    };

    const composerMissing = composerStatus !== null && !composerStatus.found;

    return (
        <>
        <Dialog
            open={open}
            onOpenChange={(o) => {
                if (!o) reset();
                onOpenChange(o);
            }}
        >
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{t('sites.dialogTitle')}</DialogTitle>
                    <DialogDescription>
                        {t('sites.dialogDescription')}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-3">
                    {composerMissing && (
                        <div className="flex items-start justify-between gap-3 rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs">
                            <div>
                                <p className="font-medium">{t('sites.composerMissingTitle')}</p>
                                <p className="mt-0.5 text-muted-foreground">{t('sites.composerMissingHint')}</p>
                            </div>
                            <Button
                                size="sm"
                                variant="outline"
                                onClick={() => setComposerInstallOpen(true)}
                            >
                                {t('sites.composerInstallButton')}
                            </Button>
                        </div>
                    )}

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            {t('sites.templateLabel')}
                        </label>
                        <select
                            value={template}
                            onChange={(e) => setTemplate(e.target.value as ProjectTemplate)}
                            className="h-9 w-full rounded-md border border-input bg-muted/50 px-3 text-sm"
                        >
                            <option value="none">{t('sites.templateNone')}</option>
                            <option value="plainPhp">{t('sites.templatePlainPhp')}</option>
                            <option value="static">{t('sites.templateStatic')}</option>
                            <option value="laravel" disabled={composerMissing}>
                                {t('sites.templateLaravel')}
                                {composerMissing ? ` (${t('sites.composerRequired')})` : ''}
                            </option>
                        </select>
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            {t('sites.folderLabel')}
                        </label>
                        <div className="flex gap-2">
                            <Input
                                value={path}
                                onChange={(e) => setPath(e.target.value)}
                                placeholder="/Users/you/Code/myapp"
                            />
                            <Button
                                type="button"
                                variant="outline"
                                onClick={browse}
                            >
                                <FolderOpen />
                                {t('sites.browse')}
                            </Button>
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            {t('sites.nameLabel')}
                            <code className="ml-1 rounded bg-muted px-1 text-[10px]">
                                {name || 'name'}.test
                            </code>
                        </label>
                        <Input
                            value={name}
                            onChange={(e) =>
                                setName(
                                    e.target.value
                                        .toLowerCase()
                                        .replace(/[^a-z0-9-]+/g, '-'),
                                )
                            }
                            placeholder="myapp"
                        />
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            {t('sites.phpLabel')}
                        </label>
                        {phpLines.length > 0 ? (
                            <>
                                <select
                                    value={phpVersion}
                                    onChange={(e) => setPhpVersion(e.target.value)}
                                    className="h-9 w-full rounded-md border border-input bg-muted/50 px-3 text-sm"
                                >
                                    {phpLines.map((line) => (
                                        <option key={line} value={line}>
                                            {line}
                                        </option>
                                    ))}
                                </select>
                                <p className="text-xs text-muted-foreground">
                                    {t('sites.phpHint')}
                                </p>
                            </>
                        ) : (
                            <p className="text-xs text-muted-foreground">
                                {t('sites.phpEmpty')}
                            </p>
                        )}
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            {t('sites.engineLabel')}
                        </label>
                        <select
                            value={webServer}
                            onChange={(e) => {
                                const val = e.target.value as WebServer;
                                if (val === 'apache') {
                                    const apacheBundle = bundles.find((b) => b.engine === 'apache');
                                    if (!apacheBundle?.installed) {
                                        setApacheInstallOpen(true);
                                        return;
                                    }
                                }
                                setWebServer(val);
                            }}
                            className="h-9 w-full rounded-md border border-input bg-muted/50 px-3 text-sm"
                        >
                            <option value="nginx">
                                {t('sites.engineNginx')}
                            </option>
                            <option value="apache">
                                {t('sites.engineApache')}
                            </option>
                            <option value="openlitespeed" disabled>
                                {t('sites.engineOls')}{' '}
                                {t('sites.engineComingSoon')}
                            </option>
                        </select>
                        <p className="text-xs text-muted-foreground">
                            {t('sites.engineHelp')}
                        </p>
                    </div>

                    {err && <p className="text-xs text-destructive">{err}</p>}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={() => onOpenChange(false)}>
                        {t('sites.cancel')}
                    </Button>
                    <Button
                        onClick={submit}
                        disabled={submitting || !name || !path || phpLines.length === 0}
                    >
                        {submitting && <Loader2 className="animate-spin" />}
                        {submitting && template !== 'none'
                            ? t('sites.scaffoldingInProgress')
                            : t('sites.save')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
        <ApacheInstallDialog
            open={apacheInstallOpen}
            onClose={() => setApacheInstallOpen(false)}
            onInstalled={async () => {
                setApacheInstallOpen(false);
                setWebServer('apache');
            }}
        />
        <ComposerInstallDialog
            open={composerInstallOpen}
            onClose={() => setComposerInstallOpen(false)}
            onInstalled={async () => {
                setComposerInstallOpen(false);
                const status = await tauri.composerStatus();
                setComposerStatus(status);
            }}
        />
        </>
    );
}

function AliasDialog({
    site,
    onClose,
    onChanged,
}: {
    site: Site | null;
    onClose: () => void;
    onChanged: () => void;
}) {
    const { t } = useTranslation();
    const [draft, setDraft] = useState('');
    const [busy, setBusy] = useState(false);
    const [err, setErr] = useState<string | null>(null);

    useEffect(() => {
        if (site) {
            setDraft('');
            setErr(null);
        }
    }, [site?.id]);

    if (!site) return null;

    const submitAdd = async () => {
        const next = draft.trim().toLowerCase();
        if (!next) return;
        setBusy(true);
        setErr(null);
        try {
            await tauri.addSiteAlias(site.id, next);
            setDraft('');
            onChanged();
        } catch (e) {
            setErr(
                e instanceof Error && e.message
                    ? e.message
                    : t('sites.aliasInvalid'),
            );
        } finally {
            setBusy(false);
        }
    };

    const submitRemove = async (alias: string) => {
        setBusy(true);
        setErr(null);
        try {
            await tauri.removeSiteAlias(site.id, alias);
            onChanged();
        } catch (e) {
            setErr(e instanceof Error ? e.message : t('sites.aliasInvalid'));
        } finally {
            setBusy(false);
        }
    };

    return (
        <Dialog
            open={!!site}
            onOpenChange={(o) => {
                if (!o) onClose();
            }}
        >
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>
                        {t('sites.aliasDialogTitle', { domain: site.domain })}
                    </DialogTitle>
                    <DialogDescription>
                        {t('sites.aliasDialogDescription')}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-3">
                    <div className="flex gap-2">
                        <Input
                            value={draft}
                            onChange={(e) =>
                                setDraft(
                                    e.target.value
                                        .toLowerCase()
                                        .replace(/[^a-z0-9.-]+/g, ''),
                                )
                            }
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    void submitAdd();
                                }
                            }}
                            placeholder={t('sites.aliasAddPlaceholder')}
                        />
                        <Button onClick={submitAdd} disabled={busy || !draft}>
                            {busy && <Loader2 className="animate-spin" />}
                            <Plus />
                            {t('sites.aliasAddAction')}
                        </Button>
                    </div>

                    {err && <p className="text-xs text-destructive">{err}</p>}

                    {site.aliases.length === 0 ? (
                        <p className="rounded-md border border-dashed border-border bg-muted/30 px-3 py-4 text-center text-xs text-muted-foreground">
                            {t('sites.aliasNoneYet')}
                        </p>
                    ) : (
                        <ul className="divide-y divide-border rounded-md border border-border">
                            {site.aliases.map((alias) => (
                                <li
                                    key={alias}
                                    className="flex items-center justify-between px-3 py-2 text-sm"
                                >
                                    <a
                                        href={`http://${alias}`}
                                        target="_blank"
                                        rel="noreferrer"
                                        className="font-medium hover:underline"
                                    >
                                        {alias}
                                    </a>
                                    <Button
                                        size="sm"
                                        variant="ghost"
                                        onClick={() => submitRemove(alias)}
                                        disabled={busy}
                                    >
                                        <X />
                                        {t('sites.aliasRemove')}
                                    </Button>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={onClose}>
                        {t('sites.aliasClose')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

function ComposerInstallDialog({
    open,
    onClose,
    onInstalled,
}: {
    open: boolean;
    onClose: () => void;
    onInstalled: () => Promise<void>;
}) {
    const { t } = useTranslation();
    const [progress, setProgress] = useState<string | null>(null);
    const [err, setErr] = useState<string | null>(null);
    const [installing, setInstalling] = useState(false);

    const startInstall = async () => {
        setInstalling(true);
        setErr(null);
        setProgress(t('sites.composerInstalling'));
        try {
            await tauri.installBundle('composer', null, (p) => {
                if (p.kind === 'downloading') {
                    const pct =
                        p.total && p.total > 0
                            ? Math.round((p.downloaded / p.total) * 100)
                            : null;
                    setProgress(pct !== null ? `Downloading ${pct}%` : 'Downloading…');
                } else if (p.kind === 'verifying') {
                    setProgress('Verifying checksum…');
                } else if (p.kind === 'extracting') {
                    setProgress('Extracting…');
                } else if (p.kind === 'started') {
                    setProgress(t('sites.composerInstalling'));
                }
            });
            setProgress(null);
            await onInstalled();
        } catch (e) {
            setErr(e instanceof Error ? e.message : 'Install failed.');
            setInstalling(false);
        }
    };

    const handleOpenChange = (o: boolean) => {
        if (!o && !installing) onClose();
    };

    return (
        <Dialog open={open} onOpenChange={handleOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{t('sites.composerInstallTitle')}</DialogTitle>
                    <DialogDescription>
                        {t('sites.composerInstallDescription')}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-3">
                    {progress && (
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            {progress}
                        </div>
                    )}
                    {err && (
                        <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5 text-sm">
                            <AlertTriangle className="mt-0.5 h-4 w-4 text-destructive" />
                            <span className="text-foreground">{err}</span>
                        </div>
                    )}
                    {!installing && !err && (
                        <p className="text-sm text-muted-foreground">
                            {t('sites.composerNeedsInstall')}
                        </p>
                    )}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={onClose} disabled={installing}>
                        {t('sites.cancel')}
                    </Button>
                    <Button onClick={startInstall} disabled={installing}>
                        {installing && <Loader2 className="animate-spin" />}
                        {installing ? progress ?? t('sites.composerInstalling') : t('common.install')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
