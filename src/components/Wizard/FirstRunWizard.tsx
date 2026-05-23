import { useEffect, useState } from 'react';
import {
    CheckCircle2,
    XCircle,
    AlertTriangle,
    Loader2,
    ArrowRight,
    ArrowLeft,
    RefreshCw,
} from 'lucide-react';

import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { tauri } from '@/lib/tauri';
import type { SystemReport } from '@/types';

type Step = 'welcome' | 'scan' | 'choose' | 'conflicts' | 'dns' | 'done';

const STEPS: { id: Step; label: string }[] = [
    { id: 'welcome', label: 'Welcome' },
    { id: 'scan', label: 'System scan' },
    { id: 'choose', label: 'Choose source' },
    { id: 'conflicts', label: 'Resolve conflicts' },
    { id: 'dns', label: 'Setup DNS' },
    { id: 'done', label: 'Done' },
];

type ScanRow = {
    label: string;
    status: 'ok' | 'warn' | 'error';
    detail: string;
};

function buildScanRows(report: SystemReport): ScanRow[] {
    const rows: ScanRow[] = [];

    rows.push({
        label: 'Homebrew',
        status: report.homebrew.installed ? 'ok' : 'error',
        detail: report.homebrew.installed
            ? (report.homebrew.prefix ?? 'detected')
            : 'Not detected — install from https://brew.sh',
    });

    for (const key of ['nginx', 'php', 'phpFpm'] as const) {
        const engine = report[key];
        const label =
            key === 'phpFpm'
                ? 'PHP-FPM'
                : key.charAt(0).toUpperCase() + key.slice(1);
        rows.push({
            label,
            status: engine.found ? 'ok' : 'error',
            detail: engine.found
                ? `${engine.version ?? 'unknown'} · ${engine.binary ?? ''}`
                : `Not found — run: brew install ${
                      key === 'phpFpm' ? 'php' : key
                  }`,
        });
    }

    for (const port of report.ports) {
        rows.push({
            label: `Port ${port.port}`,
            status: port.inUse ? 'warn' : 'ok',
            detail: port.inUse
                ? `In use${port.usedBy ? ` by ${port.usedBy}` : ''}`
                : 'Available',
        });
    }

    let resolverStatus: ScanRow['status'];
    let resolverDetail: string;
    if (!report.resolver.exists) {
        resolverStatus = 'error';
        resolverDetail = 'Not present';
    } else if (report.resolver.correct) {
        resolverStatus = 'ok';
        resolverDetail = 'Correct';
    } else {
        resolverStatus = 'warn';
        resolverDetail = 'Exists but content does not match';
    }
    rows.push({
        label: '/etc/resolver/test',
        status: resolverStatus,
        detail: resolverDetail,
    });

    return rows;
}

interface FirstRunWizardProps {
    open: boolean;
    onComplete: () => void;
}

export function FirstRunWizard({ open, onComplete }: FirstRunWizardProps) {
    const [stepIndex, setStepIndex] = useState(0);
    const [report, setReport] = useState<SystemReport | null>(null);
    const [scanError, setScanError] = useState<string | null>(null);
    const [scanning, setScanning] = useState(false);
    const step = STEPS[stepIndex].id;
    const canGoBack = stepIndex > 0 && step !== 'done';

    const next = () => setStepIndex((i) => Math.min(i + 1, STEPS.length - 1));
    const back = () => setStepIndex((i) => Math.max(i - 1, 0));
    const finish = () => onComplete();

    const runScan = async () => {
        setScanning(true);
        setScanError(null);
        try {
            const result = await tauri.scanSystem();
            setReport(result);
        } catch (err) {
            setScanError(err instanceof Error ? err.message : 'Scan failed.');
        } finally {
            setScanning(false);
        }
    };

    useEffect(() => {
        if (open && step === 'scan' && !report && !scanning) {
            void runScan();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [open, step]);

    return (
        <Dialog open={open}>
            <DialogContent
                showCloseButton={false}
                className="max-w-2xl"
                onEscapeKeyDown={(e) => e.preventDefault()}
                onPointerDownOutside={(e) => e.preventDefault()}
                onInteractOutside={(e) => e.preventDefault()}
            >
                <DialogHeader>
                    <DialogTitle>Welcome to Delify Forge</DialogTitle>
                    <DialogDescription>
                        Let&apos;s get your machine ready in a minute or two.
                    </DialogDescription>
                </DialogHeader>

                <Stepper currentStep={stepIndex} />

                <div className="min-h-[260px] py-2">
                    {step === 'welcome' && <WelcomeStep />}
                    {step === 'scan' && (
                        <ScanStep
                            scanning={scanning}
                            report={report}
                            error={scanError}
                            onRescan={runScan}
                        />
                    )}
                    {step === 'choose' && <ChooseStep />}
                    {step === 'conflicts' && <ConflictsStep report={report} />}
                    {step === 'dns' && <DnsStep />}
                    {step === 'done' && <DoneStep />}
                </div>

                <DialogFooter className="flex items-center justify-between">
                    <div>
                        {canGoBack && (
                            <Button variant="ghost" size="sm" onClick={back}>
                                <ArrowLeft />
                                Back
                            </Button>
                        )}
                    </div>
                    {step === 'done' ? (
                        <Button onClick={finish}>Open Delify Forge</Button>
                    ) : (
                        <Button onClick={next}>
                            Continue
                            <ArrowRight />
                        </Button>
                    )}
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

function Stepper({ currentStep }: { currentStep: number }) {
    return (
        <ol className="flex items-center gap-2 text-xs text-muted-foreground">
            {STEPS.map((s, i) => (
                <li key={s.id} className="flex items-center gap-2">
                    <span
                        className={cn(
                            'flex h-6 w-6 items-center justify-center rounded-full border text-[11px] font-medium',
                            i < currentStep &&
                                'border-primary bg-primary text-primary-foreground',
                            i === currentStep && 'border-primary text-primary',
                            i > currentStep && 'border-border',
                        )}
                    >
                        {i + 1}
                    </span>
                    <span
                        className={cn(
                            i === currentStep && 'text-foreground font-medium',
                        )}
                    >
                        {s.label}
                    </span>
                    {i < STEPS.length - 1 && (
                        <span className="text-border">›</span>
                    )}
                </li>
            ))}
        </ol>
    );
}

function WelcomeStep() {
    return (
        <div className="space-y-3 text-sm">
            <p>
                Delify Forge serves your local PHP, Node, and other web projects
                on friendly{' '}
                <code className="rounded bg-muted px-1.5 py-0.5 text-xs">
                    *.test
                </code>{' '}
                domains.
            </p>
            <p>This wizard will:</p>
            <ul className="ml-5 list-disc space-y-1 text-muted-foreground">
                <li>Scan your machine for Homebrew, Nginx, and PHP.</li>
                <li>
                    Help you pick which binary Forge should use for each engine.
                </li>
                <li>Resolve conflicts on ports 80 and 5353 if any.</li>
                <li>
                    Ask you once for admin access to route <code>.test</code>{' '}
                    domains to <code>127.0.0.1</code>.
                </li>
            </ul>
        </div>
    );
}

function ScanStep({
    scanning,
    report,
    error,
    onRescan,
}: {
    scanning: boolean;
    report: SystemReport | null;
    error: string | null;
    onRescan: () => void;
}) {
    return (
        <div className="space-y-2 text-sm">
            <div className="mb-3 flex items-center justify-between">
                <p className="text-muted-foreground">
                    Detecting Homebrew, Nginx, PHP, and port availability.
                </p>
                <Button
                    size="sm"
                    variant="ghost"
                    onClick={onRescan}
                    disabled={scanning}
                >
                    <RefreshCw className={cn(scanning && 'animate-spin')} />
                    Rescan
                </Button>
            </div>

            {scanning && !report && (
                <div className="flex items-center gap-2 text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Scanning system...
                </div>
            )}

            {error && (
                <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5">
                    <XCircle className="mt-0.5 h-4 w-4 text-destructive" />
                    <div className="flex-1">
                        <p className="font-medium">Scan failed</p>
                        <p className="mt-1 text-xs text-muted-foreground">
                            {error}
                        </p>
                    </div>
                </div>
            )}

            {report && (
                <ul className="space-y-1.5">
                    {buildScanRows(report).map((row) => (
                        <li
                            key={row.label}
                            className="flex items-center justify-between rounded-md border border-border bg-background px-3 py-2"
                        >
                            <span className="flex items-center gap-2">
                                <StatusIcon status={row.status} />
                                <span className="font-medium">{row.label}</span>
                            </span>
                            <span className="ml-3 text-right text-xs text-muted-foreground">
                                {row.detail}
                            </span>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}

function ChooseStep() {
    return (
        <div className="space-y-3 text-sm">
            <p>
                For each engine, pick where Forge should get the binary from. In
                MVP, the system Homebrew install is the recommended choice.
            </p>
            <div className="space-y-2">
                {['Nginx', 'PHP', 'PHP-FPM'].map((engine) => (
                    <label
                        key={engine}
                        className="flex items-center justify-between rounded-md border border-border bg-background px-3 py-2"
                    >
                        <span className="font-medium">{engine}</span>
                        <select
                            disabled
                            className="rounded-md border border-input bg-muted/50 px-2 py-1 text-xs"
                            defaultValue="brew"
                        >
                            <option value="brew">Use Homebrew binary</option>
                            <option value="bundle">
                                Download Forge bundle (V0.3)
                            </option>
                            <option value="path">Use binary on PATH</option>
                        </select>
                    </label>
                ))}
            </div>
        </div>
    );
}

function ConflictsStep({ report }: { report: SystemReport | null }) {
    const conflicts = report?.ports.filter((p) => p.inUse) ?? [];

    if (!report) {
        return (
            <p className="text-sm text-muted-foreground">
                Run the system scan first.
            </p>
        );
    }

    if (conflicts.length === 0) {
        return (
            <div className="flex items-center gap-2 text-sm">
                <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                No port conflicts detected.
            </div>
        );
    }

    return (
        <div className="space-y-3 text-sm">
            {conflicts.map((c) => (
                <div
                    key={c.port}
                    className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5"
                >
                    <AlertTriangle className="mt-0.5 h-4 w-4 text-destructive" />
                    <div className="flex-1">
                        <p className="font-medium">
                            Port {c.port} is in use
                            {c.usedBy ? ` by ${c.usedBy}` : ''}
                        </p>
                        <p className="mt-1 text-xs text-muted-foreground">
                            Stop the offending process before continuing, or
                            choose an alternative port.
                        </p>
                    </div>
                </div>
            ))}
        </div>
    );
}

function DnsStep() {
    const [busy, setBusy] = useState(false);
    const [done, setDone] = useState(false);
    const [err, setErr] = useState<string | null>(null);

    const run = async () => {
        setBusy(true);
        setErr(null);
        try {
            await tauri.setupDnsResolver();
            await tauri.startDnsmasq();
            await tauri.startPhpFpm();
            await tauri.startNginx();
            setDone(true);
        } catch (e) {
            setErr(e instanceof Error ? e.message : 'Setup failed.');
        } finally {
            setBusy(false);
        }
    };

    return (
        <div className="space-y-3 text-sm">
            <p>
                Forge needs to write{' '}
                <code className="rounded bg-muted px-1.5 py-0.5 text-xs">
                    /etc/resolver/test
                </code>{' '}
                so macOS routes <code>*.test</code> domains to{' '}
                <code>127.0.0.1</code>. This requires admin access.
            </p>
            <div className="rounded-md border border-border bg-background p-3 text-xs">
                <p className="text-muted-foreground">
                    macOS will show a native password dialog. Forge does not
                    store your password — it is used once and discarded.
                    dnsmasq, PHP-FPM, and Nginx start automatically afterward.
                </p>
            </div>
            {done ? (
                <div className="flex items-center gap-2 text-emerald-500">
                    <CheckCircle2 className="h-4 w-4" />
                    Resolver installed and engines running.
                </div>
            ) : (
                <div className="flex gap-2">
                    <Button size="sm" onClick={run} disabled={busy}>
                        {busy ? <Loader2 className="animate-spin" /> : null}
                        {busy ? 'Setting up...' : 'Setup DNS now'}
                    </Button>
                </div>
            )}
            {err && <p className="text-xs text-destructive">{err}</p>}
        </div>
    );
}

function DoneStep() {
    return (
        <div className="flex h-full flex-col items-center justify-center gap-3 py-8 text-center">
            <CheckCircle2 className="h-10 w-10 text-emerald-500" />
            <div>
                <p className="text-base font-semibold">All set</p>
                <p className="mt-1 text-sm text-muted-foreground">
                    Open Delify Forge to add your first site.
                </p>
            </div>
        </div>
    );
}

function StatusIcon({ status }: { status: ScanRow['status'] }) {
    if (status === 'ok')
        return <CheckCircle2 className="h-4 w-4 text-emerald-500" />;
    if (status === 'warn')
        return <AlertTriangle className="h-4 w-4 text-amber-500" />;
    return <XCircle className="h-4 w-4 text-destructive" />;
}
