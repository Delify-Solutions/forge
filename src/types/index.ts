export interface Site {
    id: number;
    name: string;
    path: string;
    domain: string;
    phpVersion: string;
    webServer: 'nginx' | 'apache' | 'openlitespeed';
    createdAt: string;
}

export interface PortStatus {
    port: number;
    inUse: boolean;
    usedBy?: string;
}

export interface EngineStatus {
    found: boolean;
    binary?: string;
    version?: string;
    source?: 'brew' | 'system' | 'mise' | 'forge';
}

export interface SystemReport {
    homebrew: { installed: boolean; prefix?: string };
    nginx: EngineStatus;
    php: EngineStatus;
    phpFpm: EngineStatus;
    ports: PortStatus[];
    resolver: { exists: boolean; correct: boolean };
    installedPhpVersions: string[];
    installedPhpLines: string[];
}

export interface AddSiteRequest {
    name: string;
    path: string;
    phpVersion?: string;
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
