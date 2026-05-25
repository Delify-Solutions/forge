import { useEffect, useState } from 'react';
import { Loader2, Play, RefreshCw, Square, Wand2 } from 'lucide-react';

import { PageHeader } from '@/components/PageHeader';
import { Button } from '@/components/ui/button';
import { tauri } from '@/lib/tauri';
import type { ProcessStatus } from '@/types';

const STATE_DOT: Record<ProcessStatus['state'], string> = {
    running: 'bg-emerald-500',
    stopped: 'bg-muted-foreground/40',
    crashed: 'bg-destructive',
};

interface ManagedService {
    name: string;
    label: string;
    start: () => Promise<unknown>;
    stop: () => Promise<unknown>;
}

const SERVICES: ManagedService[] = [
    {
        name: 'dnsmasq',
        label: 'dnsmasq',
        start: () => tauri.startDnsmasq(),
        stop: () => tauri.stopDnsmasq(),
    },
    {
        name: 'nginx',
        label: 'Nginx',
        start: () => tauri.startNginx(),
        stop: () => tauri.stopNginx(),
    },
    {
        name: 'php-fpm',
        label: 'PHP-FPM',
        start: () => tauri.startPhpFpm(),
        stop: () => tauri.stopPhpFpm(),
    },
];

interface ServicesProps {
    onOpenWizard: () => void;
}

export function Services({ onOpenWizard }: ServicesProps) {
    const [statuses, setStatuses] = useState<ProcessStatus[]>([]);
    const [busy, setBusy] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const refresh = async () => {
        try {
            setStatuses(await tauri.servicesStatus());
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Refresh failed.');
        }
    };

    useEffect(() => {
        void refresh();
        const id = window.setInterval(() => {
            void refresh();
        }, 2000);
        return () => window.clearInterval(id);
    }, []);

    const stateOf = (name: string): ProcessStatus['state'] =>
        statuses.find((s) => s.name === name)?.state ?? 'stopped';
    const pidOf = (name: string): number | undefined =>
        statuses.find((s) => s.name === name)?.pid;

    const run = async (svc: ManagedService, action: 'start' | 'stop') => {
        setBusy(svc.name);
        setError(null);
        try {
            if (action === 'start') await svc.start();
            else await svc.stop();
            await refresh();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Operation failed.');
        } finally {
            setBusy(null);
        }
    };

    return (
        <div>
            <div className="mb-6 flex items-start justify-between">
                <PageHeader
                    title="Services"
                    description="Engines Forge currently supervises."
                />
                <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" onClick={onOpenWizard}>
                        <Wand2 />
                        Open wizard
                    </Button>
                    <Button variant="outline" size="sm" onClick={refresh}>
                        <RefreshCw />
                        Refresh
                    </Button>
                </div>
            </div>

            {error && (
                <div className="mb-4 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm">
                    {error}
                </div>
            )}

            <div className="overflow-hidden rounded-lg border border-border bg-card">
                <table className="w-full text-sm">
                    <thead className="bg-muted/50 text-xs uppercase tracking-wide text-muted-foreground">
                        <tr>
                            <th className="px-4 py-2 text-left font-medium">
                                Service
                            </th>
                            <th className="px-4 py-2 text-left font-medium">
                                State
                            </th>
                            <th className="px-4 py-2 text-left font-medium">
                                PID
                            </th>
                            <th className="px-4 py-2 text-right font-medium">
                                Actions
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {SERVICES.map((svc) => {
                            const state = stateOf(svc.name);
                            const pid = pidOf(svc.name);
                            const isBusy = busy === svc.name;
                            return (
                                <tr
                                    key={svc.name}
                                    className="border-t border-border last:border-b-0"
                                >
                                    <td className="px-4 py-2.5 font-medium">
                                        {svc.label}
                                    </td>
                                    <td className="px-4 py-2.5">
                                        <span className="inline-flex items-center gap-2">
                                            <span
                                                className={`h-2 w-2 rounded-full ${STATE_DOT[state]}`}
                                            />
                                            {state}
                                        </span>
                                    </td>
                                    <td className="px-4 py-2.5 text-muted-foreground">
                                        {pid ?? '—'}
                                    </td>
                                    <td className="px-4 py-2.5 text-right">
                                        {state === 'running' ? (
                                            <Button
                                                size="sm"
                                                variant="ghost"
                                                onClick={() => run(svc, 'stop')}
                                                disabled={isBusy}
                                            >
                                                {isBusy ? (
                                                    <Loader2 className="animate-spin" />
                                                ) : (
                                                    <Square />
                                                )}
                                                Stop
                                            </Button>
                                        ) : (
                                            <Button
                                                size="sm"
                                                onClick={() =>
                                                    run(svc, 'start')
                                                }
                                                disabled={isBusy}
                                            >
                                                {isBusy ? (
                                                    <Loader2 className="animate-spin" />
                                                ) : (
                                                    <Play />
                                                )}
                                                Start
                                            </Button>
                                        )}
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
