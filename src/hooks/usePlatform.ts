import { useMemo } from 'react';

/**
 * Hook pour détecter la plateforme sur laquelle l'application s'exécute.
 *
 * @returns {Object} Objet contenant `isWindows` et `isLinux`
 */
export function usePlatform() {
    const isWindows = useMemo(() => {
        if (typeof window === 'undefined' || typeof navigator === 'undefined') {
            return false;
        }
        return navigator.userAgent.includes('Windows');
    }, []);

    const isLinux = useMemo(() => {
        if (typeof window === 'undefined' || typeof navigator === 'undefined') {
            return false;
        }
        return navigator.userAgent.includes('Linux');
    }, []);

    return { isWindows, isLinux };
}