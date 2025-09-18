import { createContext, ReactNode, useContext, useEffect, useState } from 'react';
import { SessionData } from '@/persistence/SessionData';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import { ApiSensorData } from '@/bindings/api/endpoints/sensor_data/ApiSensorData';

export interface AppContextType {
    sessionData: SessionData | undefined;
    activePlace: ApiUserPlace | undefined;
    activeSensor: ApiUserSensor | undefined;
    activeSensorData: ApiSensorData | undefined;
    jwt: string | undefined;
    setJwt: (jwt: string | undefined) => void;
    setActivePlace: (place: ApiUserPlace | undefined) => void;
    setActiveSensor: (
        value: { sensor: ApiUserSensor; data: ApiSensorData | null } | undefined,
    ) => void;
    logOut: () => Promise<void>;
}

const AppContext = createContext<AppContextType | null>(null);
export function AppProvider({ children }: { children: ReactNode }) {
    const [activePlace, setActivePlace] = useState<ApiUserPlace | undefined>(undefined);
    const [activeSensor, setActiveSensor] = useState<ApiUserSensor | undefined>(
        undefined,
    );
    const [sessionData, setSessionData] = useState<SessionData | undefined>(undefined);
    const [activeSensorData, setactiveSensorData] = useState<ApiSensorData | undefined>(
        undefined,
    );
    const [jwt, setJwt] = useState<string | undefined>(undefined);

    useEffect(() => {
        const setSessionDataState = async () => {
            let sd = await SessionData.create();
            setSessionData(sd);
        };

        setSessionDataState();
    }, []);

    const publicSetActivePlace = (place: ApiUserPlace | undefined) => {
        setActivePlace(place);
    };

    const logOut = async () => {
        if (!sessionData) {
            return;
        }

        setActivePlace(undefined);

        await sessionData.deleteSession();
    };

    function publicSetActiveSensor(
        value: { sensor: ApiUserSensor; data: ApiSensorData } | undefined,
    ) {
        setActiveSensor(value?.sensor);
        setactiveSensorData(value?.data);
    }

    return (
        <AppContext.Provider
            value={{
                sessionData: sessionData,
                activePlace,
                activeSensor,
                activeSensorData,
                jwt,
                setJwt,
                setActivePlace: publicSetActivePlace,
                setActiveSensor: publicSetActiveSensor,
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
