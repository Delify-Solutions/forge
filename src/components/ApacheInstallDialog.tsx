import { useState } from 'react';
import { AlertTriangle, Loader2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { tauri } from '@/lib/tauri';

export function ApacheInstallDialog({
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
        setProgress(t('sites.engineApacheInstalling'));
        try {
            await tauri.installBundle('apache', null, (p) => {
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
                    setProgress(t('sites.engineApacheInstalling'));
                }
            });
            setProgress(null);
            await onInstalled();
        } catch (e) {
            const msg =
                e instanceof Error
                    ? e.message
                    : typeof e === 'string'
                        ? e
                        : 'Install failed.';
            setErr(msg);
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
                    <DialogTitle>{t('sites.engineApacheInstallTitle')}</DialogTitle>
                    <DialogDescription>
                        {t('sites.engineApacheInstallDescription')}
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
                            {t('sites.engineApacheNeedsInstall')}
                        </p>
                    )}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={onClose} disabled={installing}>
                        {t('sites.cancel')}
                    </Button>
                    <Button onClick={startInstall} disabled={installing}>
                        {installing && <Loader2 className="animate-spin" />}
                        {installing ? progress ?? t('sites.engineApacheInstalling') : t('common.install')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
