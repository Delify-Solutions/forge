import { invoke } from '@tauri-apps/api/core';
import { Channel } from '@tauri-apps/api/core';
import type {
    BundleEntry,
    InstallProgress,
    ProcessStatus,
    Site,
    SystemReport,
} from '@/types';

export const tauri = {
    scanSystem: () => invoke<SystemReport>('scan_system'),
    setupDnsResolver: () => invoke<void>('setup_dns_resolver'),
    startDnsmasq: () => invoke<number>('start_dnsmasq'),
    stopDnsmasq: () => invoke<void>('stop_dnsmasq'),
    startNginx: () => invoke<number>('start_nginx'),
    stopNginx: () => invoke<void>('stop_nginx'),
    reloadNginx: () => invoke<void>('reload_nginx'),
    startPhpFpm: () => invoke<number>('start_php_fpm'),
    stopPhpFpm: () => invoke<void>('stop_php_fpm'),
    servicesStatus: () => invoke<ProcessStatus[]>('services_status'),
    listSites: () => invoke<Site[]>('list_sites'),
    addSite: (name: string, path: string, phpVersion?: string) =>
        invoke<Site>('add_site', { req: { name, path, phpVersion } }),
    removeSite: (id: number) => invoke<void>('remove_site', { id }),
    updateSitePhp: (id: number, phpVersion: string) =>
        invoke<Site>('update_site_php', { id, phpVersion }),
    listBundles: () => invoke<BundleEntry[]>('list_bundles'),
    installBundle: (
        engine: string,
        version: string | null,
        onProgress: (p: InstallProgress) => void,
    ) => {
        const channel = new Channel<InstallProgress>();
        channel.onmessage = onProgress;
        return invoke<BundleEntry>('install_bundle', {
            engine,
            version,
            onProgress: channel,
        });
    },
    uninstallBundle: (engine: string, version: string) =>
        invoke<void>('uninstall_bundle', { engine, version }),
};
