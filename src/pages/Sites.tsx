import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { FolderOpen, Plus, Trash2, AlertTriangle, Loader2 } from 'lucide-react';

import { PageHeader } from '@/components/PageHeader';
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
import type { Site } from '@/types';

export function Sites() {
    const { t } = useTranslation();
    const [sites, setSites] = useState<Site[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dialogOpen, setDialogOpen] = useState(false);

    const refresh = async () => {
        setLoading(true);
        setError(null);
        try {
            setSites(await tauri.listSites());
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load.');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        void refresh();
    }, []);

    const onRemove = async (id: number) => {
        try {
            await tauri.removeSite(id);
            await refresh();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to remove.');
        }
    };
    return (
        <div>
            <div className="mb-6 flex items-start justify-between">
                <PageHeader
                    title={t('sites.title')}
                    description={t('sites.subtitle')}
                />
                <Button onClick={() => setDialogOpen(true)}>
                    <Plus />
                    {t('sites.addButton')}
                </Button>
            </div>

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
            ) : (
                <SiteTable sites={sites} onRemove={onRemove} onRefresh={refresh} />
            )}

            <AddSiteDialog
                open={dialogOpen}
                onOpenChange={setDialogOpen}
                onAdded={refresh}
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
}: {
    sites: Site[];
    onRemove: (id: number) => void;
    onRefresh: () => void;
}) {
    const { t } = useTranslation();
    const [phpLines, setPhpLines] = useState<string[]>([]);

    useEffect(() => {
        tauri.scanSystem().then((r) => setPhpLines(r.installedPhpLines ?? []));
    }, []);

    const handlePhpChange = async (siteId: number, newVersion: string) => {
        try {
            await tauri.updateSitePhp(siteId, newVersion);
            onRefresh();
        } catch {
            // silently fail — row will keep old value
        }
    };

    return (
        <div className="overflow-hidden rounded-lg border border-border bg-card">
            <table className="w-full text-sm">
                <thead className="bg-muted/50 text-xs uppercase tracking-wide text-muted-foreground">
                    <tr>
                        <th className="px-4 py-2 text-left font-medium">
                            {t('sites.domainHeader')}
                        </th>
                        <th className="px-4 py-2 text-left font-medium">
                            {t('sites.pathHeader')}
                        </th>
                        <th className="px-4 py-2 text-left font-medium">
                            {t('sites.phpHeader')}
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
                                    href={`http://${site.domain}`}
                                    target="_blank"
                                    rel="noreferrer"
                                    className="hover:underline"
                                >
                                    {site.domain}
                                </a>
                            </td>
                            <td className="px-4 py-2.5 text-muted-foreground">
                                {site.path}
                            </td>
                            <td className="px-4 py-2.5 text-muted-foreground">
                                {phpLines.length > 0 ? (
                                    <select
                                        value={site.phpVersion}
                                        onChange={(e) =>
                                            handlePhpChange(site.id, e.target.value)
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
                            <td className="px-4 py-2.5 text-right">
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    onClick={() => onRemove(site.id)}
                                >
                                    <Trash2 />
                                    {t('sites.removeAction')}
                                </Button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
function AddSiteDialog({
    open,
    onOpenChange,
    onAdded,
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onAdded: () => void;
}) {
    const { t } = useTranslation();
    const [name, setName] = useState('');
    const [path, setPath] = useState('');
    const [phpVersion, setPhpVersion] = useState('');
    const [phpLines, setPhpLines] = useState<string[]>([]);
    const [submitting, setSubmitting] = useState(false);
    const [err, setErr] = useState<string | null>(null);

    useEffect(() => {
        if (open) {
            tauri.scanSystem().then((r) => {
                const lines = r.installedPhpLines ?? [];
                setPhpLines(lines);
                if (lines.length > 0) {
                    setPhpVersion((prev) => prev || lines[0]);
                }
            });
        }
    }, [open]);

    const reset = () => {
        setName('');
        setPath('');
        setPhpVersion(phpLines.length > 0 ? phpLines[0] : '');
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
            await tauri.addSite(name, path, phpVersion || undefined);
            reset();
            onOpenChange(false);
            onAdded();
        } catch (e) {
            setErr(e instanceof Error ? e.message : 'Failed to add site.');
        } finally {
            setSubmitting(false);
        }
    };

    return (
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
                        {t('sites.save')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
