import BackgroundView from '@/components/ui-elements/BackgroundView';
import PlaceCard from '@/components/PlaceCard';
import { ThemedScrollView } from '@/components/ui-elements/ThemedScrollView';
import { StyleSheet, View } from 'react-native';
import { useRouter } from 'expo-router';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import useApi from '@/hooks/useApi';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import { useMemo, useState } from 'react';
import LoadingScreen from '@/components/LoadingScreen';
import ThemedHeader from '@/components/ThemedHeader';
import ThemedButton from '@/components/ui-elements/ThemedButton';
import { HousePlus, ListRestart } from 'lucide-react-native';
import useLayerColor from '@/hooks/useLayerColor';

const secondaryColor = '#ff00003f';

export default function Home() {
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

    const reloadApi = () => {
        setMethod(undefined);
        setTimeout(() => {
            // Using setTimeout so that it runs in next React cycle
            setMethod('GET');
        }, 0);
    };

    const layerColor = useLayerColor();

    const isLoading = placeApi.loading || (!placeApi.response && !placeApi.error);
    return (
        <BackgroundView secondaryColor={secondaryColor}>
            <ThemedView style={[styles.placesContainer, { backgroundColor: layerColor }]}>
                <View style={styles.headerContainer}>
                    <ThemedHeader>Places</ThemedHeader>
                    <View style={[styles.buttonsContainer]}>
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
                    </View>
                </View>

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
            </ThemedView>
        </BackgroundView>
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
