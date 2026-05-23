import { invoke } from '@tauri-apps/api/core';
import type { Site, SystemReport } from '@/types';

export const tauri = {
    scanSystem: () => invoke<SystemReport>('scan_system'),
    setupDnsResolver: () => invoke<void>('setup_dns_resolver'),
    listSites: () => invoke<Site[]>('list_sites'),
    addSite: (name: string, path: string) =>
        invoke<Site>('add_site', { req: { name, path } }),
    removeSite: (id: number) => invoke<void>('remove_site', { id }),
};
