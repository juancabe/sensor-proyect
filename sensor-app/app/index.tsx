import LoadingScreen from '@/components/LoadingScreen';
import { SessionData } from '@/persistence/SessionData';
import { useEffect, useState } from 'react';
import Login from './login';
import Home from './home';
import { Platform } from 'react-native';

export default function Index() {
    const [sessionData, setSessionData] = useState<SessionData | undefined>(undefined);
    useEffect(() => {
        const loadSession = async () => {
            const sd = await SessionData.create();
            console.debug('SessionData created: ', sd);
            setSessionData(sd);
        };

        loadSession();
    }, []);

    if (sessionData === undefined) {
        return <LoadingScreen />;
    }

    if (Platform.OS === 'web') {
        return <Home></Home>; // Web doesn't need persisted sessionData to have a session (JWT in headers)
    }

    if (sessionData.all_set()) {
        return <Home></Home>;
    } else {
        return <Login></Login>;
    }
}
