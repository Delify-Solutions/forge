import { invoke } from '@tauri-apps/api/core';
import { Channel } from '@tauri-apps/api/core';
import type {
    BundleEntry,
    InstallProgress,
    MkcertStatus,
    ProcessStatus,
    Site,
    SiteLogsTail,
    SystemReport,
} from '@/types';

export const tauri = {
    scanSystem: () => invoke<SystemReport>('scan_system'),
    setDnsPort: (port: number) => invoke<void>('set_dns_port', { port }),
    setupDnsResolver: (port?: number) => invoke<void>('setup_dns_resolver', { port }),
    startDnsmasq: (port?: number) => invoke<number>('start_dnsmasq', { port }),
    stopDnsmasq: () => invoke<void>('stop_dnsmasq'),
    startNginx: () => invoke<number>('start_nginx'),
    stopNginx: () => invoke<void>('stop_nginx'),
    reloadNginx: () => invoke<void>('reload_nginx'),
    startPhpFpm: () => invoke<number>('start_php_fpm'),
    stopPhpFpm: () => invoke<void>('stop_php_fpm'),
    servicesStatus: () => invoke<ProcessStatus[]>('services_status'),
    debugResetEnvironment: () => invoke<void>('debug_reset_environment'),
    openDevtools: () => invoke<void>('open_devtools'),
    listSites: () => invoke<Site[]>('list_sites'),
    addSite: (name: string, path: string, phpVersion?: string, webServer?: string) =>
        invoke<Site>('add_site', { req: { name, path, phpVersion, webServer } }),
    removeSite: (id: number) => invoke<void>('remove_site', { id }),
    updateSitePhp: (id: number, phpVersion: string) =>
        invoke<Site>('update_site_php', { id, phpVersion }),
    updateSiteWebServer: (id: number, webServer: string) =>
        invoke<Site>('update_site_web_server', { id, webServer }),
    addSiteAlias: (id: number, domain: string) =>
        invoke<Site>('add_site_alias', { id, domain }),
    removeSiteAlias: (id: number, domain: string) =>
        invoke<Site>('remove_site_alias', { id, domain }),
    openSiteUrl: (id: number) => invoke<void>('open_site_url', { id }),
    revealSitePath: (id: number) => invoke<void>('reveal_site_path', { id }),
    openSiteInEditor: (id: number) => invoke<void>('open_site_in_editor', { id }),
    openSiteTerminal: (id: number) => invoke<void>('open_site_terminal', { id }),
    tailSiteLogs: (id: number) => invoke<SiteLogsTail>('tail_site_logs', { id }),
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
    mkcertStatus: () => invoke<MkcertStatus>('mkcert_status'),
    installMkcertCa: () => invoke<void>('install_mkcert_ca'),
    updateSiteHttps: (id: number, enabled: boolean) =>
        invoke<Site>('update_site_https', { id, enabled }),
};
