export type WebServer = 'nginx' | 'apache' | 'openlitespeed';

export type ProjectTemplate = 'none' | 'plainPhp' | 'static' | 'laravel';

export interface Site {
    id: number;
    name: string;
    path: string;
    domain: string;
    aliases: string[];
    phpVersion: string;
    webServer: WebServer;
    httpsEnabled: boolean;
    createdAt: string;
}

export interface MkcertStatus {
    found: boolean;
    version?: string;
    caInstalled: boolean;
}

export interface ComposerStatus {
    found: boolean;
    version?: string;
}

export interface PortStatus {
    port: number;
    inUse: boolean;
    usedBy?: string;
    ownedByForge: boolean;
}

export interface EngineStatus {
    found: boolean;
    binary?: string;
    version?: string;
    source?: 'brew' | 'system' | 'mise' | 'forge';
}

export interface SystemReport {
    homebrew: { installed: boolean; prefix?: string };
    dnsmasq: EngineStatus;
    nginx: EngineStatus;
    php: EngineStatus;
    phpFpm: EngineStatus;
    dnsPort: number;
    ports: PortStatus[];
    resolver: { exists: boolean; correct: boolean };
    installedPhpVersions: string[];
    installedPhpLines: string[];
}

export interface AddSiteRequest {
    name: string;
    path: string;
    phpVersion?: string;
    webServer?: WebServer;
}

export interface ScaffoldAndAddSiteRequest extends AddSiteRequest {
    template: ProjectTemplate;
}

export interface ProcessStatus {
    name: string;
    state: 'stopped' | 'running' | 'crashed';
    pid?: number;
}

export interface BundleEntry {
    engine: string;
    version: string;
    displayName: string;
    url: string;
    sha256?: string;
    sizeBytes?: number;
    binSubpath: string;
    installed: boolean;
    installPath?: string;
}

export type InstallProgress =
    | { kind: 'started'; totalBytes?: number }
    | { kind: 'downloading'; downloaded: number; total?: number }
    | { kind: 'verifying' }
    | { kind: 'extracting' }
    | { kind: 'done'; installPath: string }
    | { kind: 'failed'; message: string };

export interface SiteLogsTail {
    error: string[];
    access: string[];
    errorMissing: boolean;
    accessMissing: boolean;
}

export type ToolKind = 'editor' | 'terminal';

export type ToolSlug = string;

export interface ToolCatalogEntry {
    slug: ToolSlug;
    label: string;
    cli: string | null;
    bundle: string | null;
    installed: boolean;
}

export interface ToolCatalog {
    editors: ToolCatalogEntry[];
    terminals: ToolCatalogEntry[];
}

export interface PreferredTools {
    editor: ToolSlug;
    terminal: ToolSlug;
}
