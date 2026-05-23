import { useEffect, useState } from 'react';
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
                    title="Sites"
                    description="Local projects served on .test domains."
                />
                <Button onClick={() => setDialogOpen(true)}>
                    <Plus />
                    Add Site
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
                    Loading sites...
                </div>
            ) : sites.length === 0 ? (
                <EmptyState onAdd={() => setDialogOpen(true)} />
            ) : (
                <SiteTable sites={sites} onRemove={onRemove} />
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
    return (
        <div className="rounded-lg border border-dashed border-border bg-card p-10 text-center">
            <FolderOpen className="mx-auto mb-3 h-8 w-8 text-muted-foreground" />
            <p className="text-sm font-medium">No sites yet</p>
            <p className="mt-1 text-sm text-muted-foreground">
                Add a folder to start serving it on a .test domain.
            </p>
            <Button className="mt-4" onClick={onAdd}>
                <Plus />
                Add your first site
            </Button>
        </div>
    );
}

function SiteTable({
    sites,
    onRemove,
}: {
    sites: Site[];
    onRemove: (id: number) => void;
}) {
    return (
        <div className="overflow-hidden rounded-lg border border-border bg-card">
            <table className="w-full text-sm">
                <thead className="bg-muted/50 text-xs uppercase tracking-wide text-muted-foreground">
                    <tr>
                        <th className="px-4 py-2 text-left font-medium">
                            Domain
                        </th>
                        <th className="px-4 py-2 text-left font-medium">
                            Path
                        </th>
                        <th className="px-4 py-2 text-left font-medium">PHP</th>
                        <th className="px-4 py-2 text-right font-medium">
                            Actions
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
                                {site.phpVersion}
                            </td>
                            <td className="px-4 py-2.5 text-right">
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    onClick={() => onRemove(site.id)}
                                >
                                    <Trash2 />
                                    Remove
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
    const [name, setName] = useState('');
    const [path, setPath] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [err, setErr] = useState<string | null>(null);

    const reset = () => {
        setName('');
        setPath('');
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
            await tauri.addSite(name, path);
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
                    <DialogTitle>Add Site</DialogTitle>
                    <DialogDescription>
                        Pick a folder and pick a name. The site will be served
                        at <code>&lt;name&gt;.test</code>.
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-3">
                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            Folder
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
                                Browse
                            </Button>
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">
                            Name (kebab-case, becomes
                            <code className="ml-1 rounded bg-muted px-1 text-[10px]">
                                {name || 'name'}.test
                            </code>
                            )
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

                    {err && <p className="text-xs text-destructive">{err}</p>}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={() => onOpenChange(false)}>
                        Cancel
                    </Button>
                    <Button
                        onClick={submit}
                        disabled={submitting || !name || !path}
                    >
                        {submitting && <Loader2 className="animate-spin" />}
                        Save
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
