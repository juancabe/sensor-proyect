// app/index.tsx
import { useEffect, useState } from 'react';
import { Platform } from 'react-native';
import { Redirect } from 'expo-router';
import LoadingScreen from '@/components/LoadingScreen';
import { SessionData } from '@/persistence/SessionData';

export default function Index() {
    const [sessionData, setSessionData] = useState<SessionData | null>(null);
    const [ready, setReady] = useState(false);

    useEffect(() => {
        const load = async () => {
            const sd = await SessionData.create();
            setSessionData(sd);
            setReady(true);
        };
        load();
    }, []);

    if (!ready) return <LoadingScreen />;

    if (Platform.OS === 'web') {
        // On web you rely on headers/cookies; go straight to home
        return <Redirect href="/home" />;
    }

    return <Redirect href={sessionData?.all_set() ? '/home' : '/login'} />;
}
