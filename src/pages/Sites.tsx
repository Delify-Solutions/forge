import { PageHeader } from '@/components/PageHeader';

export function Sites() {
    return (
        <div>
            <PageHeader
                title="Sites"
                description="Local projects served on .test domains."
            />
            <div className="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground">
                Site list and Add Site flow land in Bước 7.
            </div>
        </div>
    );
}
