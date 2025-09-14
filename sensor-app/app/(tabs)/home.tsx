import BackgroundView from '@/components/ui-elements/BackgroundView';
import PlaceCard from '@/components/PlaceCard';
import { ThemedScrollView } from '@/components/ui-elements/ThemedScrollView';
import { Button, StyleSheet, View } from 'react-native';
import { useRouter } from 'expo-router';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import useApi from '@/hooks/useApi';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import { useMemo, useState } from 'react';
import LoadingScreen from '@/components/LoadingScreen';

const secondaryColor = '#ff00003f';

export default function Home() {
    const router = useRouter();
    console.log('home reloaded');
    const [method, setMethod] = useState<'GET' | undefined>('GET');
    // let placeQuery: GetPlace = { kind: 'UserPlaces' };
    const queryParams = useMemo<[string, string][]>(() => [['kind', 'UserPlaces']], []);
    const placeApi = useApi<undefined, ApiUserPlace[], undefined>(
        '/place',
        method,
        false,
        undefined,
        queryParams,
    );

    const reloadApi = () => {
        setMethod(undefined);
        setTimeout(() => {
            // Using setTimeout so that it runs in next React cycle
            setMethod('GET');
        }, 0);
    };

    const isLoading = placeApi.loading || (!placeApi.response && !placeApi.error);

    return (
        <BackgroundView secondaryColor={secondaryColor}>
            <ThemedView style={[styles.buttonsContainer]}>
                <Button
                    title="Add Place"
                    onPress={() => {
                        router.navigate('/AddPlaceScreen');
                    }}
                ></Button>
                <Button
                    disabled={placeApi.loading}
                    onPress={reloadApi}
                    title="Reload Places"
                ></Button>
            </ThemedView>
            {isLoading ? (
                <LoadingScreen />
            ) : (
                <ThemedScrollView
                    style={{
                        backgroundColor: 'transparent',
                    }}
                >
                    {placeApi.response &&
                        placeApi.response.map((place) => (
                            <View key={place.name}>
                                <PlaceCard place={place} />
                            </View>
                        ))}
                    <ErrorBox error={placeApi.formattedError} />
                </ThemedScrollView>
            )}
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    buttonsContainer: {
        display: 'flex',
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingHorizontal: 40,
        backgroundColor: 'transparent',
        margin: 10,
    },
});
