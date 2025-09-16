import { StyleSheet } from 'react-native';
import { ThemedView } from './ui-elements/ThemedView';
import ThemedHeader from './ThemedHeader';
import ThemedButton from './ui-elements/ThemedButton';
import { HousePlus, ListRestart } from 'lucide-react-native';
import LoadingScreen from './LoadingScreen';
import { ThemedScrollView } from './ui-elements/ThemedScrollView';
import ErrorBox from './ui-elements/ErrorBox';
import useLayerColor from '@/hooks/useLayerColor';
import { useRouter } from 'expo-router';
import { useEffect, useMemo, useState } from 'react';
import useApi from '@/hooks/useApi';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import useRedirect from '@/hooks/useRedirect';
import PlaceCard from './PlaceCard';

export default function PlacesScreen() {
    const router = useRouter();
    const [method, setMethod] = useState<'GET' | undefined>('GET');
    const queryParams = useMemo<[string, string][]>(() => [['kind', 'UserPlaces']], []);
    const placeApi = useApi<undefined, ApiUserPlace[], undefined>(
        '/place',
        method,
        false,
        undefined,
        queryParams,
    );
    const redirectInvalidSession = useRedirect().redirectToLogin;

    useEffect(() => {
        if (placeApi.error?.error && placeApi.error?.error.status === 401) {
            redirectInvalidSession();
        } else {
            console.log('error changed: ', placeApi.error);
        }
    }, [placeApi.error, redirectInvalidSession]);

    const layerColor = useLayerColor();

    const reloadApi = () => {
        setMethod(undefined);
        setTimeout(() => {
            // Using setTimeout so that it runs in next React cycle
            setMethod('GET');
        }, 0);
    };

    const isLoading = placeApi.loading || (!placeApi.response && !placeApi.error);

    return (
        <ThemedView style={[styles.placesContainer, { backgroundColor: layerColor }]}>
            <ThemedView style={styles.headerContainer}>
                <ThemedHeader>Places</ThemedHeader>
                <ThemedView style={[styles.buttonsContainer]} noBackground={true}>
                    <ThemedButton
                        icon={HousePlus}
                        onPress={() => {
                            router.navigate('/AddPlaceScreen');
                        }}
                    ></ThemedButton>
                    <ThemedButton
                        icon={ListRestart}
                        onPress={reloadApi}
                        disabled={isLoading}
                    ></ThemedButton>
                </ThemedView>
            </ThemedView>

            {isLoading ? (
                <LoadingScreen />
            ) : (
                <ThemedScrollView
                    style={{
                        backgroundColor: 'transparent',
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 20,
                    }}
                >
                    {placeApi.response &&
                        placeApi.response.map((place) => (
                            <ThemedView key={place.name} noBackground={true}>
                                <PlaceCard place={place} />
                            </ThemedView>
                        ))}
                    <ErrorBox error={placeApi.formattedError} />
                </ThemedScrollView>
            )}
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    headerContainer: {
        display: 'flex',
        flexDirection: 'row',
        marginRight: 10,
        marginLeft: 10,
    },
    placesContainer: {
        flex: 1,
        margin: 10,
        borderRadius: 10,
        padding: 10,
    },
    buttonsContainer: {
        flex: 1,
        flexDirection: 'row',
        justifyContent: 'flex-end',
        alignItems: 'center',
        gap: 20,
    },
});
