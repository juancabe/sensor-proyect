import LoadingScreen from '@/components/LoadingScreen';
import { Redirect } from 'expo-router';
import { SessionData } from '@/persistence/SessionData';
import { useEffect, useState } from 'react';
import Login from './login';

export default function Index() {
    const [sessionData, setSessionData] = useState<SessionData | undefined>(undefined);

    useEffect(() => {
        const loadSession = async () => {
            const sd = await SessionData.create();
            setSessionData(sd);
        };

        loadSession();
    }, []);

    if (sessionData === undefined) {
        return <LoadingScreen />;
    }

    if (sessionData.all_set()) {
        return <Redirect href={'/(tabs)/home'}></Redirect>;
    } else {
        return <Login></Login>;
    }
}
