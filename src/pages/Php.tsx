import { PageHeader } from '@/components/PageHeader';

export function Php() {
    return (
        <div>
            <PageHeader
                title="PHP"
                description="Detected PHP runtimes and active version."
            />
            <div className="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground">
                PHP detection lands in Bước 6. Multi-version support arrives in
                V0.2 via mise.
            </div>
        </div>
    );
}
