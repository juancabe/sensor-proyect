import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import PlaceCard from '@/components/PlaceCard';
import { ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedScrollView } from '@/components/ui-elements/ThemedScrollView';
import { useTheme } from '@react-navigation/native';
import { Button, View } from 'react-native';
import { useRouter } from 'expo-router';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import useApi from '@/hooks/useApi';
import { GetPlace } from '@/bindings/api/endpoints/place/GetPlace';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';

export default function Home() {
    const theme = useTheme();
    const ctx = useAppContext();
    const router = useRouter();

    let placeQuery: GetPlace = 'UserPlaces';
    const placeApi = useApi<undefined, ApiUserPlace[], undefined>(
        '/place?kind=' + placeQuery,
        undefined,
        'GET',
        false,
    );

    console.debug('placeApi.response: ', placeApi.response);

    let response: ApiUserPlace[] | undefined = placeApi.response;

    return (
        <BackgroundView secondaryColor="#ff00003f">
            <ThemedView style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                <Button
                    title="Add Place"
                    onPress={() => {
                        router.navigate('/AddPlaceScreen');
                    }}
                />
            </ThemedView>
            <ThemedScrollView
                style={{
                    backgroundColor: 'transparent',
                }}
            >
                {placeApi.loading && <ThemedText>Loading summary…</ThemedText>}
                {placeApi.response &&
                    placeApi.response.map((place) => (
                        <View key={place.name}>
                            <PlaceCard place={place} />
                        </View>
                    ))}
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
