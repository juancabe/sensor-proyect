import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import PlaceCard from '@/components/PlaceCard';
import { ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedScrollView } from '@/components/ui-elements/ThemedScrollView';
import { useTheme } from '@react-navigation/native';
import { Button, View } from 'react-native';
import { useRouter } from 'expo-router';

export default function Home() {
    const theme = useTheme();
    const ctx = useAppContext();
    const router = useRouter();

    return (
        <BackgroundView secondaryColor="#ff00003f">
            <ThemedScrollView style={{ backgroundColor: 'transparent' }}>
                <Button
                    title="Add Place"
                    onPress={() => {
                        router.navigate('/AddPlaceScreen');
                    }}
                />
                {ctx.summary === undefined && <ThemedText>Loading summary…</ThemedText>}
                {typeof ctx.summary === 'object' &&
                    Array.from(ctx.summary.entries()).map(
                        ([placeId, [place, sensors]]) => (
                            <View key={placeId}>
                                <PlaceCard place={place} sensors={sensors} />
                            </View>
                        ),
                    )}
                {ctx.summary === 'Unauthorized' && (
                    <ThemedText style={{ color: theme.colors.notification }}>
                        Unauthorized. Please log in again.
                    </ThemedText>
                )}
                {ctx.summary === 'Connection Error' && (
                    <ThemedText>
                        Could not fetch summary. Check your connection.
                    </ThemedText>
                )}
            </ThemedScrollView>
        </BackgroundView>
    );
}
