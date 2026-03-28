"use client";
import { motion } from "framer-motion";
import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { CacheInfos, columns, Folder } from "@/components/custom/clear-cache/columns";
import { DataTable } from "@/components/custom/clear-cache/data-table";
import ActionsMenu from "@/components/custom/clear-cache/actions";
import { useToast } from "@/hooks/use-toast";
import PageHeader from "@/components/custom/PageHeader";
import { BrushCleaning, MonitorSmartphone } from "lucide-react";
import { usePlatform } from "@/hooks/usePlatform";
import { useGamePaths } from "@/hooks/useGamePaths";

export default function ClearCache() {
    const [cacheInfos, setCacheInfos] = useState<Folder[] | null>(null);
    const { toast } = useToast();
    const [loadingDot, setLoadingDot] = useState(0);
    const { isLinux } = usePlatform();
    const { customPaths, isLoading: pathsLoading } = useGamePaths();

    const ScanCache = useCallback(async () => {
        const result: CacheInfos = JSON.parse(
            await invoke("get_cache_informations"),
        );
        setCacheInfos(result.folders);
    }, [setCacheInfos]);

    const updateCacheInfos = (path: string) => {
        setCacheInfos(
            (prev) => prev?.filter((folder) => folder.path !== path) || null,
        );
    };

    useEffect(() => {
        if (!cacheInfos) {
            ScanCache();
        }
    }, [cacheInfos, ScanCache]);

    useEffect(() => {
        if (cacheInfos) return;
        const interval = setInterval(() => {
            if (loadingDot === 3) {
                setLoadingDot(0);
            }
            setLoadingDot((prev) => prev + 1);
        }, 500);

        return () => clearInterval(interval);
    }, [cacheInfos, loadingDot]);

    // Sur Linux, si aucun chemin de jeu n'est configuré, afficher un message d'aide
    if (isLinux && !pathsLoading && customPaths.length === 0) {
        return (
            <motion.div
                initial={{ opacity: 0, x: 100 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{
                    duration: 0.8,
                    delay: 0.2,
                    ease: [0, 0.71, 0.2, 1.01],
                }}
                className="flex flex-col w-full max-h-[calc(100vh-50px)]"
            >
                <div className="flex items-center justify-between">
                    <PageHeader
                        icon={<BrushCleaning className="h-6 w-6" />}
                        title="Gestionnaire du cache"
                        description="Gérez et nettoyez les fichiers en cache de Star Citizen"
                    />
                </div>

                <div className="flex flex-1 items-center justify-center">
                    <div className="flex flex-col items-center gap-4 text-center p-8">
                        <MonitorSmartphone className="h-16 w-16 text-muted-foreground" />
                        <h2 className="text-xl font-semibold">Aucun chemin de jeu configuré</h2>
                        <p className="text-muted-foreground max-w-md">
                            Pour gérer le cache sur Linux, configurez d'abord le chemin
                            d'installation de Star Citizen (Proton/Wine) dans les préférences.
                        </p>
                    </div>
                </div>
            </motion.div>
        );
    }

    return cacheInfos ? (
        <motion.div
            initial={{ opacity: 0, x: 100 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{
                duration: 0.8,
                delay: 0.2,
                ease: [0, 0.71, 0.2, 1.01],
            }}
            className="flex flex-col w-full max-h-[calc(100vh-50px)]"
        >
            <div className="flex items-center justify-between">
                <PageHeader
                    icon={<BrushCleaning className="h-6 w-6" />}
                    title="Gestionnaire du cache"
                    description="Gérez et nettoyez les fichiers en cache de Star Citizen"
                />
                <ActionsMenu setCacheInfos={setCacheInfos} />
            </div>

            <DataTable
                columns={columns(toast, updateCacheInfos)}
                data={cacheInfos}
            />
        </motion.div>
    ) : (
        <div className="flex h-screen w-full flex-row gap-3 items-center justify-center">
            <p>
                Récupération des données{" "}
                {Array.from({ length: loadingDot }).map(() => ".")}
            </p>
        </div>
    );
}
