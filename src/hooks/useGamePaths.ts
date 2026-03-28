import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface CustomGamePaths {
    paths: string[];
}

export interface VersionInfo {
    path: string;
    translated: boolean;
    up_to_date: boolean;
}

export interface VersionPaths {
    versions: Record<string, VersionInfo>;
}

/**
 * Hook pour gérer les chemins personnalisés de Star Citizen.
 *
 * Permet de:
 * - Charger les chemins personnalisés sauvegardés
 * - Sauvegarder de nouveaux chemins
 * - Valider un chemin de jeu
 * - Obtenir les versions détectées
 */
export function useGamePaths() {
    const [customPaths, setCustomPaths] = useState<string[]>([]);
    const [gameVersions, setGameVersions] = useState<VersionPaths>({ versions: {} });
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Charger les chemins personnalisés au démarrage
    useEffect(() => {
        loadCustomPaths();
    }, []);

    const loadCustomPaths = useCallback(async () => {
        setIsLoading(true);
        setError(null);

        try {
            // Charger les chemins personnalisés
            const customPathsResult: CustomGamePaths = await invoke('load_custom_game_paths');
            setCustomPaths(customPathsResult.paths || []);

            // Si des chemins personnalisés existent, les utiliser
            if (customPathsResult.paths && customPathsResult.paths.length > 0) {
                const versions: VersionPaths = await invoke('get_custom_game_versions', {
                    paths: customPathsResult.paths
                });
                setGameVersions(versions);
            } else {
                // Sinon, détecter automatiquement (Windows only)
                const versions: VersionPaths = await invoke('get_star_citizen_versions');
                setGameVersions(versions);
            }
        } catch (err) {
            console.error('Erreur lors du chargement des chemins de jeu:', err);
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    }, []);

    const savePaths = useCallback(async (paths: string[]) => {
        try {
            await invoke('save_custom_game_paths', { paths });
            setCustomPaths(paths);

            // Recharger les versions avec les nouveaux chemins
            const versions: VersionPaths = await invoke('get_custom_game_versions', { paths });
            setGameVersions(versions);
        } catch (err) {
            console.error('Erreur lors de la sauvegarde des chemins:', err);
            throw err;
        }
    }, []);

    const addPath = useCallback(async (path: string) => {
        const newPath = path.trim().replace(/\\/g, '/').replace(/\/+$/, '');
        if (!newPath) return;

        // Vérifier si le chemin existe déjà
        if (customPaths.some(p => p.replace(/\\/g, '/').replace(/\/+$/, '') === newPath)) {
            throw new Error('Ce chemin existe déjà dans la liste');
        }

        const updatedPaths = [...customPaths, newPath];
        await savePaths(updatedPaths);
    }, [customPaths, savePaths]);

    const removePath = useCallback(async (path: string) => {
        const normalizedPath = path.replace(/\\/g, '/').replace(/\/+$/, '');
        const updatedPaths = customPaths.filter(p =>
            p.replace(/\\/g, '/').replace(/\/+$/, '') !== normalizedPath
        );
        await savePaths(updatedPaths);
    }, [customPaths, savePaths]);

    const clearPaths = useCallback(async () => {
        try {
            await invoke('clear_custom_game_paths');
            setCustomPaths([]);
            // Recharger les versions détectées automatiquement
            const versions: VersionPaths = await invoke('get_star_citizen_versions');
            setGameVersions(versions);
        } catch (err) {
            console.error('Erreur lors de la suppression des chemins:', err);
            throw err;
        }
    }, []);

    const validatePath = useCallback(async (path: string): Promise<boolean> => {
        try {
            return await invoke('validate_game_path', { path });
        } catch (err) {
            console.error('Erreur lors de la validation du chemin:', err);
            return false;
        }
    }, []);

    const refreshVersions = useCallback(async () => {
        setIsLoading(true);
        try {
            if (customPaths.length > 0) {
                const versions: VersionPaths = await invoke('get_custom_game_versions', {
                    paths: customPaths
                });
                setGameVersions(versions);
            } else {
                const versions: VersionPaths = await invoke('get_star_citizen_versions');
                setGameVersions(versions);
            }
        } catch (err) {
            console.error('Erreur lors du rafraîchissement des versions:', err);
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    }, [customPaths]);

    return {
        customPaths,
        gameVersions,
        isLoading,
        error,
        addPath,
        removePath,
        clearPaths,
        validatePath,
        refreshVersions,
        loadCustomPaths,
    };
}