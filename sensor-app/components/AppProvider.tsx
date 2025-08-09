import { fetchUserSummary } from '@/api/user_summary';
import type { PlaceSummary } from '@/bindings/PlaceSummary';
import type { SensorSummary } from '@/bindings/SensorSummary';
import {
    createContext,
    ReactNode,
    useCallback,
    useContext,
    useEffect,
    useState,
} from 'react';
import { SessionData } from '@/persistence/SessionData';
import { reload } from 'expo-router/build/global-state/routing';

export type SummaryMap = Map<string, [PlaceSummary, SensorSummary[]]>;
type SummaryState = SummaryMap | 'Unauthorized' | 'Connection Error' | undefined;

interface AppContextType {
    summary: SummaryState;
    reloadSummary: () => Promise<void>;
    sessionData: SessionData | undefined;
    activePlace: PlaceSummary | undefined;
    setActivePlace: (place_id: string) => boolean | undefined;
    logOut: () => Promise<void>;
}

const AppContext = createContext<AppContextType | null>(null);
export function AppProvider({ children }: { children: ReactNode }) {
    console.debug('[AppContext] mounted');

    const [summary, setSummary] = useState<SummaryState>(undefined);
    const [activePlace, setActivePlace] = useState<PlaceSummary | undefined>(undefined);
    const [sessionData, setSessionData] = useState<SessionData | undefined>(undefined);

    useEffect(() => {
        const setSessionDataState = async () => {
            let sd = await SessionData.create();
            setSessionData(sd);
        };

        setSessionDataState();
    }, []);

    const reloadSummary = useCallback(async () => {
        if (!sessionData) {
            return;
        }

        if (!sessionData.all_set()) {
            console.warn('[reloadSummary] session.sessionData was not set');
            setSummary('Unauthorized');
            return;
        }

        const username = sessionData.username!;
        const api_id = sessionData.api_id!;

        let res;

        try {
            res = await fetchUserSummary(username, { id: api_id });
        } catch (e) {
            console.error(`Unexpected error thrown on [reloadSummary]: ${e}`);
            return;
        }

        if (!res) {
            console.error(`[reloadSummary] res was nullish: (${res})`);
            setSummary('Connection Error');
            return;
        }

        console.log('Res: ', res);
        if (typeof res === 'object' && 'summary' in res) {
            let result = res.summary;
            let places = result.places;
            let sensors = result.sensors;
            let map = new Map<string, [PlaceSummary, SensorSummary[]]>();

            places.forEach((place) => {
                map.set(place.place_id.id, [place, []]);
            });
            sensors.forEach((sensor) => {
                let opt = map.get(sensor.place_id.id);
                if (!opt) {
                    console.warn('Sensor without corresponding place');
                    console.warn('sensor: ', sensor);
                    return;
                }
                let [_place, sens_arr] = opt;
                sens_arr.push(sensor);
            });

            setSummary(map);
        } else {
            console.error(`[reloadSummary] Unexpected situation, res: (${res})`);
            setSummary('Unauthorized');
        }
    }, [sessionData]);

    useEffect(() => {
        reloadSummary();
    }, [reloadSummary]);

    const publicSetActivePlace = (place_id: string): boolean | undefined => {
        if (typeof summary !== 'object') {
            return undefined;
        }

        let place_summary = summary.get(place_id);
        if (place_summary) {
            setActivePlace(place_summary[0]);
            return true;
        } else {
            return false;
        }
    };

    const logOut = async () => {
        if (!sessionData) {
            return;
        }

        setSummary(undefined);
        setSummary(undefined);
        setActivePlace(undefined);

        console.debug(`summary: ${summary}, activePlace: ${activePlace}`);
        await sessionData.deleteSession();
        await reloadSummary();
    };

    return (
        <AppContext.Provider
            value={{
                summary,
                reloadSummary,
                sessionData: sessionData,
                activePlace,
                setActivePlace: publicSetActivePlace,
                logOut,
            }}
        >
            {children}
        </AppContext.Provider>
    );
}

export function useAppContext() {
    const ctx = useContext(AppContext);
    if (!ctx) throw new Error('useAppContext must be inside AppProvider');
    return ctx;
}
