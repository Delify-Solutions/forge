import { PageHeader } from '@/components/PageHeader';

export function Services() {
    return (
        <div>
            <PageHeader
                title="Services"
                description="Status of Nginx, dnsmasq, and PHP-FPM."
            />
            <div className="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground">
                Service status surfaces in Bước 8 (dnsmasq) and Bước 9 (Nginx).
            </div>
        </div>
    );
}
