import { PageHeader } from '@/components/PageHeader';

export function General() {
    return (
        <div>
            <PageHeader
                title="General"
                description="Application preferences, paths, and theme settings."
            />
            <div className="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground">
                General settings will land in V0.2.
            </div>
        </div>
    );
}
