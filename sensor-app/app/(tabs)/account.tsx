import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { useRouter } from 'expo-router';
import { useState, useEffect } from 'react';
import { Button } from 'react-native';

export default function Account() {
    const router = useRouter();
    const ctx = useAppContext();
    const session = ctx.sessionData;
    const handleLogout = async () => {
        await ctx.logOut();
        router.replace('/');
    };

    const [username, setUsername] = useState<string | null | undefined>(undefined);

    useEffect(() => {
        const getUsername = async () => {
            if (!session?.all_set()) {
                setUsername(null);
                return;
            }
            setUsername(session.username);
        };
        getUsername();
    }, [session]);

    return (
        <BackgroundView secondaryColor="#007bff3f">
            <ThemedView style={{ backgroundColor: 'transparent' }}>
                <ThemedView
                    style={{
                        display: 'flex',
                        flexDirection: 'row',
                        justifyContent: 'flex-end',
                    }}
                >
                    <Button title="Log Out" onPress={handleLogout} />
                </ThemedView>
                <ThemedText>Hello {username} this is your account page!</ThemedText>
            </ThemedView>
        </BackgroundView>
    );
}
