import { HousePlus, ListRestart } from 'lucide-react-native';
import LoadingScreen from './LoadingScreen';
import PlaceCard from './PlaceCard';
import { Box, Text } from '@/ui/theme';
import { Card } from '@/ui/components/Card';
import { ScrollView } from 'react-native';
import { Button } from '@/ui/components/Button';
import { BoxV } from '@/ui/components/BoxV';
import { useRouter } from 'expo-router';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';

export interface PlacesScreenProps {
    reloadPlaces: () => void;
    isLoading: boolean;
    places?: ApiUserPlace[];
}

export default function PlacesScreen({
    reloadPlaces: reload,
    isLoading,
    places,
}: PlacesScreenProps) {
    const router = useRouter();

    return (
        <BoxV variant="field" padding="m" margin="m" gap="s" flex={1}>
            <Card variant="elevated" flexDirection="row" justifyContent="space-between">
                <Card variant="subtle">
                    <Text variant="heading">Places</Text>
                </Card>
                <BoxV variant="field" flexDirection="row" gap="l">
                    <Button
                        variant="positive"
                        icon={HousePlus}
                        onPress={() => {
                            router.push('/AddPlaceScreen');
                        }}
                    ></Button>
                    <Button
                        variant="warning"
                        icon={ListRestart}
                        onPress={reload}
                        disabled={isLoading}
                    ></Button>
                </BoxV>
            </Card>

            {isLoading ? (
                <LoadingScreen />
            ) : (
                <ScrollView style={{ flex: 1 }}>
                    <Card variant="subtle" flex={1}>
                        <Box
                            gap="l"
                            style={{
                                display: 'flex',
                                flexWrap: 'wrap',
                                flexDirection: 'row',
                                alignContent: 'center',
                                justifyContent: 'space-evenly',
                            }}
                        >
                            {places &&
                                places.map((place) => (
                                    <PlaceCard place={place} key={place.name} />
                                ))}
                        </Box>
                    </Card>
                </ScrollView>
            )}
        </BoxV>
    );
}

// const styles = StyleSheet.create({
//     headerContainer: {
//         display: 'flex',
//         flexDirection: 'row',
//         marginRight: 10,
//         marginLeft: 10,
//     },
//     buttonsContainer: {
//         flex: 1,
//         flexDirection: 'row',
//         justifyContent: 'flex-end',
//         alignItems: 'center',
//         gap: 20,
//     },
// });
