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
    source?: 'brew' | 'system' | 'mise';
}

export interface SystemReport {
    homebrew: { installed: boolean; prefix?: string };
    nginx: EngineStatus;
    php: EngineStatus;
    phpFpm: EngineStatus;
    ports: PortStatus[];
    resolver: { exists: boolean; correct: boolean };
}

export interface ProcessStatus {
    name: string;
    state: 'stopped' | 'running' | 'crashed';
    pid?: number;
}
