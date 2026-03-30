import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface VersionInfo {
    path: string;
    translated: boolean;
    up_to_date: boolean;
}

export interface VersionPaths {
    versions: Record<string, VersionInfo>;
}

/**
 * Hook pour obtenir les versions de Star Citizen détectées automatiquement.
 */
export function useGamePaths() {
    const [gameVersions, setGameVersions] = useState<VersionPaths>({ versions: {} });
    const [isLoading, setIsLoading] = useState(true);

    const load = useCallback(async () => {
        setIsLoading(true);
        try {
            const versions = await invoke<VersionPaths>('get_star_citizen_versions');
            setGameVersions(versions);
        } catch (err) {
            console.error('Erreur chargement versions Star Citizen:', err);
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => { load(); }, [load]);

    return { gameVersions, isLoading, refreshVersions: load };
}
