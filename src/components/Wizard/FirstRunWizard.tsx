// SPDX-License-Identifier: AGPL-3.0-or-later

import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
    CheckCircle2,
    XCircle,
    AlertTriangle,
    Loader2,
    ArrowRight,
    ArrowLeft,
    RefreshCw,
    Download,
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
import { SUPPORTED_LANGUAGES, setLanguage, type LanguageCode } from '@/i18n';
import type { InstallProgress, SystemReport } from '@/types';

type Step = 'welcome' | 'scan' | 'choose' | 'conflicts' | 'dns' | 'done';

const STEPS: { id: Step; labelKey: string }[] = [
    { id: 'welcome', labelKey: 'wizard.steps.welcome' },
    { id: 'scan', labelKey: 'wizard.steps.scan' },
    { id: 'choose', labelKey: 'wizard.steps.choose' },
    { id: 'conflicts', labelKey: 'wizard.steps.conflicts' },
    { id: 'dns', labelKey: 'wizard.steps.dns' },
    { id: 'done', labelKey: 'wizard.steps.done' },
];

type RowStatus = 'ok' | 'warn' | 'error';

type EngineKey = 'nginx' | 'php' | 'phpFpm';

interface ScanRow {
    key: string;
    label: string;
    status: RowStatus;
    detail: string;
    engine?: EngineKey;
}

const ENGINE_BUNDLE_NAME: Record<EngineKey, string> = {
    nginx: 'nginx',
    php: 'php',
    phpFpm: 'php-fpm',
};

interface FirstRunWizardProps {
    open: boolean;
    onComplete: () => void;
}

export function FirstRunWizard({ open, onComplete }: FirstRunWizardProps) {
    const { t } = useTranslation();
    const [stepIndex, setStepIndex] = useState(0);
    const [report, setReport] = useState<SystemReport | null>(null);
    const [scanError, setScanError] = useState<string | null>(null);
    const [scanning, setScanning] = useState(false);
    const [installState, setInstallState] = useState<
        Record<string, InstallProgress>
    >({});

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

    const installEngine = async (engine: EngineKey) => {
        const bundleEngine = ENGINE_BUNDLE_NAME[engine];
        setInstallState((s) => ({
            ...s,
            [bundleEngine]: { kind: 'started' },
        }));
        try {
            await tauri.installBundle(bundleEngine, null, (p) => {
                setInstallState((s) => ({ ...s, [bundleEngine]: p }));
            });
            await runScan();
        } catch (err) {
            setInstallState((s) => ({
                ...s,
                [bundleEngine]: {
                    kind: 'failed',
                    message:
                        err instanceof Error
                            ? err.message
                            : 'Install failed.',
                },
            }));
        }
    };

    useEffect(() => {
        if (open && step === 'scan' && !report && !scanning) {
            void runScan();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [open, step]);

    const rows = useMemo<ScanRow[]>(
        () => (report ? buildScanRows(report, t) : []),
        [report, t],
    );

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
                    <DialogTitle>{t('wizard.title')}</DialogTitle>
                    <DialogDescription>
                        {t('wizard.subtitle')}
                    </DialogDescription>
                </DialogHeader>

                <Stepper currentStep={stepIndex} />

                <div className="min-h-[260px] py-2">
                    {step === 'welcome' && <WelcomeStep />}
                    {step === 'scan' && (
                        <ScanStep
                            scanning={scanning}
                            rows={rows}
                            report={report}
                            error={scanError}
                            installState={installState}
                            onRescan={runScan}
                            onInstall={installEngine}
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
                                {t('common.back')}
                            </Button>
                        )}
                    </div>
                    {step === 'done' ? (
                        <Button onClick={finish}>{t('common.open')}</Button>
                    ) : (
                        <Button onClick={next}>
                            {t('common.continue')}
                            <ArrowRight />
                        </Button>
                    )}
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

function buildScanRows(
    report: SystemReport,
    t: (k: string, opts?: Record<string, unknown>) => string,
): ScanRow[] {
    const rows: ScanRow[] = [];

    rows.push({
        key: 'homebrew',
        label: t('wizard.scan.labels.homebrew'),
        status: report.homebrew.installed ? 'ok' : 'warn',
        detail: report.homebrew.installed
            ? t('wizard.scan.details.homebrewOk', {
                  prefix: report.homebrew.prefix ?? 'detected',
              })
            : t('wizard.scan.details.homebrewMissing'),
    });

    const engines: { key: EngineKey; labelKey: string }[] = [
        { key: 'nginx', labelKey: 'wizard.scan.labels.nginx' },
        { key: 'php', labelKey: 'wizard.scan.labels.php' },
        { key: 'phpFpm', labelKey: 'wizard.scan.labels.phpFpm' },
    ];
    for (const { key, labelKey } of engines) {
        const engine = report[key];
        rows.push({
            key,
            label: t(labelKey),
            status: engine.found ? 'ok' : 'error',
            detail: engine.found
                ? t('wizard.scan.details.engineOk', {
                      version: engine.version ?? '',
                  })
                : t('wizard.scan.details.engineMissing'),
            engine: engine.found ? undefined : key,
        });
    }

    for (const port of report.ports) {
        rows.push({
            key: `port-${port.port}`,
            label: t('wizard.scan.labels.port', { port: port.port }),
            status: port.inUse ? 'warn' : 'ok',
            detail: port.inUse
                ? port.usedBy
                    ? t('wizard.scan.details.portInUseBy', {
                          name: port.usedBy,
                      })
                    : t('wizard.scan.details.portInUse')
                : t('wizard.scan.details.portFree'),
        });
    }

    let resolverStatus: RowStatus;
    let resolverDetail: string;
    if (!report.resolver.exists) {
        resolverStatus = 'error';
        resolverDetail = t('wizard.scan.details.resolverMissing');
    } else if (report.resolver.correct) {
        resolverStatus = 'ok';
        resolverDetail = t('wizard.scan.details.resolverOk');
    } else {
        resolverStatus = 'warn';
        resolverDetail = t('wizard.scan.details.resolverWrong');
    }
    rows.push({
        key: 'resolver',
        label: t('wizard.scan.labels.resolver'),
        status: resolverStatus,
        detail: resolverDetail,
    });

    return rows;
}

function Stepper({ currentStep }: { currentStep: number }) {
    const { t } = useTranslation();
    return (
        <ol className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
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
                        {t(s.labelKey)}
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
    const { t, i18n } = useTranslation();
    const current = (i18n.resolvedLanguage ?? 'en') as LanguageCode;
    return (
        <div className="space-y-4 text-sm">
            <div className="rounded-md border border-border bg-muted/30 p-3">
                <p className="mb-2 text-xs font-medium text-muted-foreground">
                    {t('wizard.welcome.languagePrompt')}
                </p>
                <div className="flex gap-2">
                    {SUPPORTED_LANGUAGES.map((lang) => (
                        <Button
                            key={lang.code}
                            variant={
                                current === lang.code ? 'default' : 'outline'
                            }
                            size="sm"
                            onClick={() => setLanguage(lang.code)}
                        >
                            {lang.label}
                        </Button>
                    ))}
                </div>
            </div>
            <p>{t('wizard.welcome.intro')}</p>
            <p>{t('wizard.welcome.actions')}</p>
            <ul className="ml-5 list-disc space-y-1 text-muted-foreground">
                <li>{t('wizard.welcome.scanItem')}</li>
                <li>{t('wizard.welcome.pickItem')}</li>
                <li>{t('wizard.welcome.conflictItem')}</li>
                <li>{t('wizard.welcome.adminItem')}</li>
            </ul>
        </div>
    );
}

function ScanStep({
    scanning,
    rows,
    report,
    error,
    installState,
    onRescan,
    onInstall,
}: {
    scanning: boolean;
    rows: ScanRow[];
    report: SystemReport | null;
    error: string | null;
    installState: Record<string, InstallProgress>;
    onRescan: () => void;
    onInstall: (engine: EngineKey) => void;
}) {
    const { t } = useTranslation();
    return (
        <div className="space-y-2 text-sm">
            <div className="mb-3 flex items-center justify-between">
                <p className="text-muted-foreground">
                    {t('wizard.scan.intro')}
                </p>
                <Button
                    size="sm"
                    variant="ghost"
                    onClick={onRescan}
                    disabled={scanning}
                >
                    <RefreshCw className={cn(scanning && 'animate-spin')} />
                    {t('common.rescan')}
                </Button>
            </div>

            {scanning && !report && (
                <div className="flex items-center gap-2 text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    {t('wizard.scan.scanning')}
                </div>
            )}

            {error && (
                <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2.5">
                    <XCircle className="mt-0.5 h-4 w-4 text-destructive" />
                    <div className="flex-1">
                        <p className="font-medium">
                            {t('wizard.scan.failedTitle')}
                        </p>
                        <p className="mt-1 text-xs text-muted-foreground">
                            {error}
                        </p>
                    </div>
                </div>
            )}

            {rows.length > 0 && (
                <ul className="space-y-1.5">
                    {rows.map((row) => (
                        <ScanRowView
                            key={row.key}
                            row={row}
                            progress={
                                row.engine
                                    ? installState[
                                          ENGINE_BUNDLE_NAME[row.engine]
                                      ]
                                    : undefined
                            }
                            onInstall={onInstall}
                        />
                    ))}
                </ul>
            )}
        </div>
    );
}

function ScanRowView({
    row,
    progress,
    onInstall,
}: {
    row: ScanRow;
    progress?: InstallProgress;
    onInstall: (engine: EngineKey) => void;
}) {
    const { t } = useTranslation();
    const isInstalling =
        progress &&
        progress.kind !== 'done' &&
        progress.kind !== 'failed';
    const installFailed = progress?.kind === 'failed';

    return (
        <li className="flex items-center justify-between rounded-md border border-border bg-background px-3 py-2">
            <span className="flex items-center gap-2">
                <StatusIcon status={row.status} />
                <span className="font-medium">{row.label}</span>
            </span>
            <span className="ml-3 flex items-center gap-3">
                {progress && (
                    <span className="text-right text-xs text-muted-foreground">
                        {renderProgress(progress, t)}
                    </span>
                )}
                {!progress && (
                    <span className="text-right text-xs text-muted-foreground">
                        {row.detail}
                    </span>
                )}
                {row.engine && row.status === 'error' && (
                    <Button
                        size="sm"
                        variant={installFailed ? 'destructive' : 'default'}
                        onClick={() => onInstall(row.engine!)}
                        disabled={isInstalling}
                    >
                        {isInstalling ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                        ) : (
                            <Download className="h-3 w-3" />
                        )}
                        {isInstalling
                            ? t('common.installing')
                            : t('wizard.scan.actions.install')}
                    </Button>
                )}
            </span>
        </li>
    );
}

function renderProgress(
    progress: InstallProgress,
    t: (k: string, opts?: Record<string, unknown>) => string,
): string {
    switch (progress.kind) {
        case 'started':
            return t('wizard.scan.progress.started');
        case 'downloading': {
            const { downloaded, total } = progress;
            if (total && total > 0) {
                const percent = Math.min(
                    100,
                    Math.floor((downloaded / total) * 100),
                );
                return t('wizard.scan.progress.downloading', { percent });
            }
            const mb = (downloaded / 1024 / 1024).toFixed(1);
            return `${mb} MB`;
        }
        case 'verifying':
            return t('wizard.scan.progress.verifying');
        case 'extracting':
            return t('wizard.scan.progress.extracting');
        case 'done':
            return t('common.installed');
        case 'failed':
            return progress.message || t('wizard.scan.progress.failed');
    }
}

function ChooseStep() {
    const { t } = useTranslation();
    return (
        <div className="space-y-3 text-sm">
            <p>{t('wizard.choose.intro')}</p>
            <div className="space-y-2">
                {(['Nginx', 'PHP', 'PHP-FPM'] as const).map((engine) => (
                    <label
                        key={engine}
                        className="flex items-center justify-between rounded-md border border-border bg-background px-3 py-2"
                    >
                        <span className="font-medium">{engine}</span>
                        <select
                            disabled
                            className="rounded-md border border-input bg-muted/50 px-2 py-1 text-xs"
                            defaultValue="forge"
                        >
                            <option value="forge">
                                {t('wizard.choose.options.forge')}
                            </option>
                            <option value="brew">
                                {t('wizard.choose.options.brew')}
                            </option>
                            <option value="path">
                                {t('wizard.choose.options.path')}
                            </option>
                        </select>
                    </label>
                ))}
            </div>
        </div>
    );
}

function ConflictsStep({ report }: { report: SystemReport | null }) {
    const { t } = useTranslation();
    const conflicts = report?.ports.filter((p) => p.inUse) ?? [];

    if (!report) {
        return (
            <p className="text-sm text-muted-foreground">
                {t('wizard.scan.intro')}
            </p>
        );
    }

    if (conflicts.length === 0) {
        return (
            <div className="flex items-center gap-2 text-sm">
                <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                {t('wizard.conflicts.noneTitle')}
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
                            {c.usedBy
                                ? t('wizard.conflicts.portTitleBy', {
                                      port: c.port,
                                      name: c.usedBy,
                                  })
                                : t('wizard.conflicts.portTitle', {
                                      port: c.port,
                                  })}
                        </p>
                        <p className="mt-1 text-xs text-muted-foreground">
                            {t('wizard.conflicts.portHint')}
                        </p>
                    </div>
                </div>
            ))}
        </div>
    );
}

function DnsStep() {
    const { t } = useTranslation();
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
            setErr(e instanceof Error ? e.message : t('wizard.dns.failed'));
        } finally {
            setBusy(false);
        }
    };

    return (
        <div className="space-y-3 text-sm">
            <p>{t('wizard.dns.intro')}</p>
            <div className="rounded-md border border-border bg-background p-3 text-xs">
                <p className="text-muted-foreground">{t('wizard.dns.note')}</p>
            </div>
            {done ? (
                <div className="flex items-center gap-2 text-emerald-500">
                    <CheckCircle2 className="h-4 w-4" />
                    {t('wizard.dns.doneTitle')}
                </div>
            ) : (
                <div className="flex gap-2">
                    <Button size="sm" onClick={run} disabled={busy}>
                        {busy ? <Loader2 className="animate-spin" /> : null}
                        {busy ? t('wizard.dns.running') : t('wizard.dns.action')}
                    </Button>
                </div>
            )}
            {err && <p className="text-xs text-destructive">{err}</p>}
        </div>
    );
}

function DoneStep() {
    const { t } = useTranslation();
    return (
        <div className="flex h-full flex-col items-center justify-center gap-3 py-8 text-center">
            <CheckCircle2 className="h-10 w-10 text-emerald-500" />
            <div>
                <p className="text-base font-semibold">
                    {t('wizard.done.title')}
                </p>
                <p className="mt-1 text-sm text-muted-foreground">
                    {t('wizard.done.subtitle')}
                </p>
            </div>
        </div>
    );
}

function StatusIcon({ status }: { status: RowStatus }) {
    if (status === 'ok')
        return <CheckCircle2 className="h-4 w-4 text-emerald-500" />;
    if (status === 'warn')
        return <AlertTriangle className="h-4 w-4 text-amber-500" />;
    return <XCircle className="h-4 w-4 text-destructive" />;
}
