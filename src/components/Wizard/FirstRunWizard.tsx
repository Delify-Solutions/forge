import { useState } from 'react';
import {
    CheckCircle2,
    XCircle,
    AlertTriangle,
    Loader2,
    ArrowRight,
    ArrowLeft,
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

const MOCK_SCAN: ScanRow[] = [
    { label: 'Homebrew', status: 'ok', detail: '/opt/homebrew' },
    {
        label: 'Nginx',
        status: 'ok',
        detail: '1.27.3 · /opt/homebrew/bin/nginx',
    },
    { label: 'PHP', status: 'ok', detail: '8.3.14 · /opt/homebrew/bin/php' },
    { label: 'PHP-FPM', status: 'ok', detail: '/opt/homebrew/sbin/php-fpm' },
    { label: 'Port 80', status: 'warn', detail: 'In use by httpd (Apache)' },
    { label: 'Port 5353 (dnsmasq)', status: 'ok', detail: 'Available' },
    { label: '/etc/resolver/test', status: 'error', detail: 'Not present' },
];

interface FirstRunWizardProps {
    open: boolean;
    onComplete: () => void;
}

export function FirstRunWizard({ open, onComplete }: FirstRunWizardProps) {
    const [stepIndex, setStepIndex] = useState(0);
    const step = STEPS[stepIndex].id;
    const canGoBack = stepIndex > 0 && step !== 'done';

    const next = () => setStepIndex((i) => Math.min(i + 1, STEPS.length - 1));
    const back = () => setStepIndex((i) => Math.max(i - 1, 0));
    const finish = () => onComplete();

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
                    {step === 'scan' && <ScanStep />}
                    {step === 'choose' && <ChooseStep />}
                    {step === 'conflicts' && <ConflictsStep />}
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

function ScanStep() {
    return (
        <div className="space-y-1.5 text-sm">
            <p className="mb-3 text-muted-foreground">
                Scanning your system (mocked data — wired to the real{' '}
                <code>scan_system</code> Tauri command in Bước 6).
            </p>
            <ul className="space-y-1.5">
                {MOCK_SCAN.map((row) => (
                    <li
                        key={row.label}
                        className="flex items-center justify-between rounded-md border border-border bg-background px-3 py-2"
                    >
                        <span className="flex items-center gap-2">
                            <StatusIcon status={row.status} />
                            <span className="font-medium">{row.label}</span>
                        </span>
                        <span className="text-xs text-muted-foreground">
                            {row.detail}
                        </span>
                    </li>
                ))}
            </ul>
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

function ConflictsStep() {
    return (
        <div className="space-y-3 text-sm">
            <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5">
                <AlertTriangle className="mt-0.5 h-4 w-4 text-destructive" />
                <div className="flex-1">
                    <p className="font-medium">
                        Port 80 is in use by{' '}
                        <code className="rounded bg-muted px-1 text-xs">
                            httpd
                        </code>{' '}
                        (Apache)
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                        Nginx will not be able to bind. Stop the offending
                        process before continuing.
                    </p>
                    <div className="mt-2 flex gap-2">
                        <Button size="sm" variant="outline" disabled>
                            Stop Apache (Bước 6)
                        </Button>
                        <Button size="sm" variant="ghost" disabled>
                            Use port 8080 instead
                        </Button>
                    </div>
                </div>
            </div>
            <p className="text-xs text-muted-foreground">
                No conflicts detected on port 5353 or 443.
            </p>
        </div>
    );
}

function DnsStep() {
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
                </p>
            </div>
            <div className="flex gap-2">
                <Button size="sm" disabled>
                    <Loader2 className="animate-spin" />
                    Wired in Bước 8
                </Button>
            </div>
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
