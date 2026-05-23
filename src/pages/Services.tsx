import { useEffect, useState } from 'react';
import { Loader2, RefreshCw } from 'lucide-react';

import { PageHeader } from '@/components/PageHeader';
import { Button } from '@/components/ui/button';
import { tauri } from '@/lib/tauri';
import type { ProcessStatus } from '@/types';

const STATE_DOT: Record<ProcessStatus['state'], string> = {
    running: 'bg-emerald-500',
    stopped: 'bg-muted-foreground/40',
    crashed: 'bg-destructive',
};

export function Services() {
    const [statuses, setStatuses] = useState<ProcessStatus[]>([]);
    const [loading, setLoading] = useState(true);

    const refresh = async () => {
        setLoading(true);
        try {
            setStatuses(await tauri.servicesStatus());
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        void refresh();
        const id = window.setInterval(() => {
            void refresh();
        }, 2000);
        return () => window.clearInterval(id);
    }, []);

    return (
        <div>
            <div className="mb-6 flex items-start justify-between">
                <PageHeader
                    title="Services"
                    description="Engines Forge currently supervises."
                />
                <Button variant="outline" size="sm" onClick={refresh}>
                    <RefreshCw />
                    Refresh
                </Button>
            </div>

            {loading && statuses.length === 0 ? (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Loading...
                </div>
            ) : statuses.length === 0 ? (
                <div className="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground">
                    No supervised processes yet. Run the first-run wizard to
                    start dnsmasq.
                </div>
            ) : (
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
                            </tr>
                        </thead>
                        <tbody>
                            {statuses.map((p) => (
                                <tr
                                    key={p.name}
                                    className="border-t border-border last:border-b-0"
                                >
                                    <td className="px-4 py-2.5 font-medium">
                                        {p.name}
                                    </td>
                                    <td className="px-4 py-2.5">
                                        <span className="inline-flex items-center gap-2">
                                            <span
                                                className={`h-2 w-2 rounded-full ${STATE_DOT[p.state]}`}
                                            />
                                            {p.state}
                                        </span>
                                    </td>
                                    <td className="px-4 py-2.5 text-muted-foreground">
                                        {p.pid ?? '—'}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
}
