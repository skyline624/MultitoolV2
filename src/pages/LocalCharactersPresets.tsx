import { motion } from 'framer-motion';
import { useState, useCallback, useEffect, useMemo } from 'react';
import { columns } from "@/components/custom/local-characters-presets/columns";
import { DataTable } from "@/components/custom/local-characters-presets/data-table";
import { useToast } from "@/hooks/use-toast";
import { invoke } from "@tauri-apps/api/core";
import { isGamePaths, GamePaths } from "@/types/translation";
import { LocalCharactersResult } from "@/types/charactersList";
import logger from "@/utils/logger";
import { isProtectedPath } from "@/utils/fs-permissions";
import PageHeader from '@/components/custom/PageHeader';
import { IconUsers } from '@tabler/icons-react';
import { usePlatform } from "@/hooks/usePlatform";
import { useGamePaths } from "@/hooks/useGamePaths";
import { FolderOpen, Plus, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

/**
 * Page de gestion des presets de personnages locaux.
 * Permet de visualiser, dupliquer et gérer les personnages sauvegardés entre différentes versions du jeu.
 */
function LocalCharactersPresets() {
    type CharacterRow = {
        name: string;
        versions: { version: string; path: string }[];
    };
    const [localCharacters, setLocalCharacters] = useState<CharacterRow[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [loadingDot, setLoadingDot] = useState(0);
    const [gamePaths, setGamePaths] = useState<GamePaths | null>(null);
    const [customPathInput, setCustomPathInput] = useState("");
    const [isAddingPath, setIsAddingPath] = useState(false);
    const { toast } = useToast();
    const { isWindows } = usePlatform();
    const {
        customPaths,
        addPath,
        removePath,
        clearPaths,
        validatePath,
        gameVersions: customGameVersions
    } = useGamePaths();

    // On regroupe les personnages par identifiant unique (ex: path ou name)
    // Regroupe les personnages par nom et stocke les versions et chemins
    const scanLocalCharacters = useCallback(async (gamePath: string) => {
        try {
            const result: LocalCharactersResult = JSON.parse(
                await invoke("get_character_informations", { path: gamePath }),
            );

            setLocalCharacters(prev => {
                const allVersions = Object.keys(gamePaths?.versions || {});
                const map = new Map<string, CharacterRow>();
                // Ajoute les anciens
                prev.forEach(char => {
                    // On clone pour ne pas muter l'état
                    map.set(char.name, {
                        name: char.name,
                        versions: [...char.versions],
                    });
                });
                result.characters.forEach(newChar => {
                    const key = newChar.name;
                    if (!map.has(key)) {
                        map.set(key, {
                            name: newChar.name,
                            versions: allVersions.map(version => ({
                                version,
                                path: version === newChar.version ? newChar.path : ""
                            }))
                        });
                    } else {
                        const existing = map.get(key)!;
                        const idx = existing.versions.findIndex(v => v.version === newChar.version);
                        if (idx !== -1) {
                            if (!existing.versions[idx].path) {
                                existing.versions[idx].path = newChar.path;
                            }
                        }
                    }
                });
                return Array.from(map.values());
            });
        } catch (error) {
            /**
             * Affiche un toast d'erreur si la récupération des informations des personnage échoue.
             */
            toast({
                title: "Erreur",
                description: "Impossible de récupérer les informations des personnages",
                variant: "destructive",
            });
        }
    }, [toast, gamePaths]);

    /**
     * Fonction pour rafraîchir complètement les données.
     * @async
     */
    const refreshLocalCharacters = useCallback(async () => {
        if (!gamePaths) return;

        setIsLoading(true);
        setLocalCharacters([]);

        const entries = Object.entries(gamePaths.versions)
            .filter(([_, version]) => version?.path)
            .map(([versionName, version]) => ({ versionName, path: version!.path }));

        await Promise.all(entries.map(({ path }) => scanLocalCharacters(path)));
        setIsLoading(false);
    }, [gamePaths, scanLocalCharacters]);

    // Charger les versions du jeu (détection auto ou chemins personnalisés)
    useEffect(() => {
        const getGameVersions = async () => {
            try {
                // Si des chemins personnalisés existent, les utiliser
                if (customPaths.length > 0) {
                    const versions = customGameVersions;
                    if (Object.keys(versions.versions).length > 0) {
                        logger.log("Versions du jeu (chemins personnalisés):", versions);
                        setGamePaths(versions);
                    } else {
                        // Aucun chemin valide trouvé
                        setGamePaths({ versions: {} });
                    }
                } else {
                    // Détection automatique
                    const versions = await invoke("get_star_citizen_versions");
                    if (isGamePaths(versions)) {
                        logger.log("Versions du jeu reçues:", versions);
                        setGamePaths(versions);
                    }
                }
            } catch (error) {
                logger.error("Erreur lors de la récupération des versions:", error);
                toast({
                    title: "Erreur",
                    description: "Impossible de récupérer les versions de Star Citizen",
                    variant: "destructive",
                });
            }
        };

        getGameVersions();
    }, [toast, customPaths, customGameVersions]);

    useEffect(() => {
        if (!gamePaths) return;

        // Si aucune version trouvée, pas besoin de scanner
        if (Object.keys(gamePaths.versions).length === 0) {
            setIsLoading(false);
            return;
        }

        const scanAllPaths = async () => {
            const entries = Object.entries(gamePaths.versions)
                .filter(([_, version]) => version?.path)
                .map(([versionName, version]) => ({ versionName, path: version!.path }));

            for (const { path } of entries) {
                if (isProtectedPath(path)) {
                    toast({
                        title: "Chemin protégé",
                        description: "Le jeu est dans Program Files. En cas d'erreur, relancez en administrateur ou installez-le ailleurs.",
                        variant: "warning",
                        duration: 4000,
                    });
                }
                await scanLocalCharacters(path);
            }
            setIsLoading(false);
        };
        scanAllPaths();
    }, [gamePaths, scanLocalCharacters]);

    useEffect(() => {
        if (!isLoading) return;

        const interval = setInterval(() => {
            setLoadingDot(prev => prev === 3 ? 0 : prev + 1);
        }, 500);

        return () => clearInterval(interval);
    }, [isLoading]);

    const availableVersions = useMemo(() => {
        const versions = localCharacters.flatMap(char => char.versions.map(v => v.version));
        return Array.from(new Set(versions)).sort();
    }, [localCharacters]);

    // Gestion de l'ajout d'un chemin personnalisé
    const handleAddCustomPath = useCallback(async () => {
        const path = customPathInput.trim();
        if (!path) return;

        setIsAddingPath(true);
        try {
            const isValid = await validatePath(path);
            if (!isValid) {
                toast({
                    title: "Chemin invalide",
                    description: "Ce dossier ne contient pas d'installation valide de Star Citizen",
                    variant: "destructive",
                });
                return;
            }

            await addPath(path);
            setCustomPathInput("");
            toast({
                title: "Chemin ajouté",
                description: "Le chemin a été ajouté avec succès",
            });
        } catch (error) {
            toast({
                title: "Erreur",
                description: String(error),
                variant: "destructive",
            });
        } finally {
            setIsAddingPath(false);
        }
    }, [customPathInput, validatePath, addPath, toast]);

    // Aucune installation trouvée - permettre d'ajouter un chemin manuel
    if (!gamePaths) {
        return (
            <div className="flex h-screen w-full items-center justify-center">
                <p>Recherche des installations de Star Citizen...</p>
            </div>
        );
    }

    if (Object.keys(gamePaths.versions).length === 0 && !isLoading) {
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
                <PageHeader
                    icon={<IconUsers className="h-6 w-6" />}
                    title="Gestionnaire de presets de Personnages"
                    description="Gérez vos presets de personnages locaux"
                />

                <div className="flex flex-1 items-center justify-center">
                    <div className="flex flex-col items-center gap-6 text-center p-8 max-w-lg">
                        <FolderOpen className="h-16 w-16 text-muted-foreground" />
                        <h2 className="text-xl font-semibold">Aucune installation détectée</h2>
                        <p className="text-muted-foreground">
                            {!isWindows
                                ? "Sur Linux, vous devez spécifier le chemin de votre installation Star Citizen (via Proton/Wine)."
                                : "Aucune installation de Star Citizen n'a été détectée automatiquement."}
                        </p>
                        <p className="text-muted-foreground text-sm">
                            Vous pouvez spécifier manuellement le chemin d'installation.
                        </p>

                        {/* Liste des chemins personnalisés */}
                        {customPaths.length > 0 && (
                            <div className="w-full space-y-2">
                                <p className="text-sm font-medium text-left w-full">Chemins configurés :</p>
                                <div className="space-y-2">
                                    {customPaths.map((path, index) => (
                                        <div key={index} className="flex items-center gap-2 bg-muted p-2 rounded-md">
                                            <span className="flex-1 text-sm truncate">{path}</span>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => removePath(path)}
                                            >
                                                <X className="h-4 w-4" />
                                            </Button>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* Formulaire d'ajout */}
                        <div className="w-full space-y-4">
                            <div className="flex gap-2">
                                <Input
                                    placeholder={isWindows
                                        ? "Ex: C:\\Program Files\\Roberts Space Industries\\StarCitizen\\LIVE"
                                        : "Ex: /home/user/.wine/drive_c/Program Files/Roberts Space Industries/StarCitizen/LIVE"}
                                    value={customPathInput}
                                    onChange={(e) => setCustomPathInput(e.target.value)}
                                    className="flex-1"
                                />
                                <Button
                                    onClick={handleAddCustomPath}
                                    disabled={isAddingPath || !customPathInput.trim()}
                                >
                                    <Plus className="h-4 w-4 mr-2" />
                                    Ajouter
                                </Button>
                            </div>
                            <p className="text-xs text-muted-foreground">
                                Le chemin doit contenir un dossier StarCitizen avec les fichiers Bin64 et Data.p4k
                            </p>
                        </div>

                        {customPaths.length > 0 && (
                            <Button
                                variant="outline"
                                onClick={clearPaths}
                                className="text-destructive"
                            >
                                Réinitialiser les chemins personnalisés
                            </Button>
                        )}
                    </div>
                </div>
            </motion.div>
        );
    }

    if (isLoading) {
        return (
            <div className="flex h-screen w-full items-center justify-center">
                <p>
                    Récupération des données{" "}
                    {Array.from({ length: loadingDot }).map((_, i) => (
                        <span key={i}>.</span>
                    ))}
                </p>
            </div>
        );
    }

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
            <PageHeader
                icon={<IconUsers className="h-6 w-6" />}
                title="Gestionnaire de presets de Personnages"
                description="Gérez vos presets de personnages locaux"
            />

            <DataTable
                columns={columns(toast, refreshLocalCharacters, availableVersions)}
                data={localCharacters} // Utiliser les données filtrées
            />
        </motion.div>
    );
}

export default LocalCharactersPresets;