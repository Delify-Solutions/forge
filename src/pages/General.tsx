import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Loader2 } from 'lucide-react';

import { PageHeader } from '@/components/PageHeader';
import { tauri } from '@/lib/tauri';
import type { PreferredTools, ToolCatalog, ToolCatalogEntry, ToolKind } from '@/types';

export function General() {
    const { t } = useTranslation();

    const [catalog, setCatalog] = useState<ToolCatalog | null>(null);
    const [prefs, setPrefs] = useState<PreferredTools | null>(null);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState<string | null>(null);
    const [editorError, setEditorError] = useState<string | null>(null);
    const [terminalError, setTerminalError] = useState<string | null>(null);

    const load = useCallback(async () => {
        setLoading(true);
        setLoadError(null);
        try {
            const [cat, p] = await Promise.all([
                tauri.listToolCatalog(),
                tauri.getPreferredTools(),
            ]);
            setCatalog(cat);
            setPrefs(p);
        } catch (e) {
            setLoadError(String(e));
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        load();
    }, [load]);

    async function handleChange(kind: ToolKind, slug: string) {
        if (!prefs) return;

        const prev = kind === 'editor' ? prefs.editor : prefs.terminal;

        // Optimistic update.
        setPrefs((p) => p ? { ...p, [kind]: slug } : p);
        if (kind === 'editor') setEditorError(null);
        else setTerminalError(null);

        try {
            await tauri.setPreferredTool(kind, slug);
        } catch (e) {
            // Revert on error.
            setPrefs((p) => p ? { ...p, [kind]: prev } : p);
            if (kind === 'editor') setEditorError(String(e));
            else setTerminalError(String(e));
        }
    }

    function renderOption(entry: ToolCatalogEntry) {
        const suffix = entry.installed ? '' : ` ${t('general.notInstalledSuffix')}`;
        return (
            <option key={entry.slug} value={entry.slug}>
                {entry.label}{suffix}
            </option>
        );
    }

    return (
        <div>
            <PageHeader
                title={t('general.title')}
                description={t('general.subtitle')}
            />

            {loading && (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    {t('common.loading')}
                </div>
            )}

            {!loading && loadError && (
                <div className="rounded-lg border border-destructive bg-destructive/10 p-4 text-sm text-destructive">
                    <p>{t('general.loadFailed')}: {loadError}</p>
                    <button
                        onClick={load}
                        className="mt-2 underline underline-offset-2"
                    >
                        {t('common.rescan')}
                    </button>
                </div>
            )}

            {!loading && !loadError && catalog && prefs && (
                <div className="space-y-6">
                    {/* Editor preference */}
                    <section className="rounded-lg border border-border bg-card p-6">
                        <h2 className="mb-1 text-sm font-medium">
                            {t('general.preferredEditorLabel')}
                        </h2>
                        <p className="mb-3 text-xs text-muted-foreground">
                            {t('general.subtitle')}
                        </p>
                        <select
                            value={prefs.editor}
                            onChange={(e) => handleChange('editor', e.target.value)}
                            className="h-8 rounded-md border border-input bg-muted/50 px-2 text-sm"
                        >
                            {catalog.editors.map(renderOption)}
                        </select>
                        {editorError && (
                            <p className="mt-2 text-xs text-destructive">
                                {t('general.saveFailed')}: {editorError}
                            </p>
                        )}
                    </section>

                    {/* Terminal preference */}
                    <section className="rounded-lg border border-border bg-card p-6">
                        <h2 className="mb-1 text-sm font-medium">
                            {t('general.preferredTerminalLabel')}
                        </h2>
                        <p className="mb-3 text-xs text-muted-foreground">
                            {t('general.subtitle')}
                        </p>
                        <select
                            value={prefs.terminal}
                            onChange={(e) => handleChange('terminal', e.target.value)}
                            className="h-8 rounded-md border border-input bg-muted/50 px-2 text-sm"
                        >
                            {catalog.terminals.map(renderOption)}
                        </select>
                        {terminalError && (
                            <p className="mt-2 text-xs text-destructive">
                                {t('general.saveFailed')}: {terminalError}
                            </p>
                        )}
                    </section>
                </div>
            )}
        </div>
    );
}
