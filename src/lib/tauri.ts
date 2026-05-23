import { invoke } from '@tauri-apps/api/core';
import type { ProcessStatus, Site, SystemReport } from '@/types';

export const tauri = {
    scanSystem: () => invoke<SystemReport>('scan_system'),
    setupDnsResolver: () => invoke<void>('setup_dns_resolver'),
    startDnsmasq: () => invoke<number>('start_dnsmasq'),
    stopDnsmasq: () => invoke<void>('stop_dnsmasq'),
    startNginx: () => invoke<number>('start_nginx'),
    stopNginx: () => invoke<void>('stop_nginx'),
    reloadNginx: () => invoke<void>('reload_nginx'),
    servicesStatus: () => invoke<ProcessStatus[]>('services_status'),
    listSites: () => invoke<Site[]>('list_sites'),
    addSite: (name: string, path: string) =>
        invoke<Site>('add_site', { req: { name, path } }),
    removeSite: (id: number) => invoke<void>('remove_site', { id }),
};
