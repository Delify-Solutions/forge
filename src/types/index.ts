export type WebServer = 'nginx' | 'apache' | 'openlitespeed';

export interface Site {
    id: number;
    name: string;
    path: string;
    domain: string;
    phpVersion: string;
    webServer: WebServer;
    createdAt: string;
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
