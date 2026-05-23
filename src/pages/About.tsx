import { PageHeader } from '@/components/PageHeader';

export function About() {
    return (
        <div>
            <PageHeader title="About" />
            <div className="space-y-3 rounded-lg border border-border bg-card p-6 text-sm">
                <div>
                    <span className="text-muted-foreground">App:</span>{' '}
                    <span className="font-medium">Delify Forge</span>
                </div>
                <div>
                    <span className="text-muted-foreground">Version:</span>{' '}
                    <span className="font-medium">0.0.0 · pre-MVP</span>
                </div>
                <div>
                    <span className="text-muted-foreground">License:</span>{' '}
                    <span className="font-medium">AGPL-3.0-or-later</span>
                </div>
                <div>
                    <span className="text-muted-foreground">Repo:</span>{' '}
                    <a
                        className="font-medium underline"
                        href="https://github.com/Delify-Solutions/forge"
                        target="_blank"
                        rel="noreferrer"
                    >
                        Delify-Solutions/forge
                    </a>
                </div>
            </div>
        </div>
    );
}
