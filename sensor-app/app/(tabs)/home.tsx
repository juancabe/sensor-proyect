import { SensorSummary } from '@/bindings/SensorSummary';
import BackgroundView from '@/components/BackgroundView';
import PlaceCard from '@/components/PlaceCard';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { useSummary } from '@/hooks/useSummary';
import { useTheme } from '@react-navigation/native';
import { useEffect, useState } from 'react';
import { Button, View } from 'react-native';

type Place = [string, string | null];
type Summary = Map<number, [Place, SensorSummary[]]>;

export default function Home() {
    const theme = useTheme();
    const { fetchSummary } = useSummary();
    const [summary, setSummary] = useState<
        Summary | 'Unauthorized' | 'Connection Error' | undefined
    >(undefined);

    useEffect(() => {
        async function load() {
            let result = await fetchSummary();
            if (typeof result !== 'string') {
                let places = result.places;
                let sensors = result.sensors;
                let map = new Map<number, [Place, SensorSummary[]]>();

                places.forEach((place) => {
                    map.set(place[0], [[place[1], place[2]], []]);
                });
                sensors.forEach((sensor) => {
                    let opt = map.get(sensor.place);
                    if (!opt) {
                        console.warn('Sensor without corresponding place');
                        return;
                    }
                    let [place, sum] = opt;
                    sum.push(sensor);
                });

                setSummary(map);
            } else {
                setSummary(result);
            }
        }
        load();
    }, [fetchSummary]);

    return (
        <BackgroundView secondaryColor="#ff00003f">
            <ThemedView style={{ backgroundColor: 'transparent' }}>
                <Button title="Add Sensor" />

                {summary === undefined && <ThemedText>Loading summary…</ThemedText>}
                {typeof summary === 'object' &&
                    Array.from(summary.entries()).map(([placeId, [place, sensors]]) => (
                        <View key={placeId}>
                            <PlaceCard
                                place={place}
                                sensors={sensors}
                                placeColor={`#${String(placeId)
                                    .charAt(0)
                                    .repeat(2)}77ff44`}
                            />
                        </View>
                    ))}
                {summary === 'Unauthorized' && (
                    <ThemedText style={{ color: theme.colors.notification }}>
                        Unauthorized. Please log in again.
                    </ThemedText>
                )}
                {summary === 'Connection Error' && (
                    <ThemedText>
                        Could not fetch summary. Check your connection.
                    </ThemedText>
                )}
            </ThemedView>
        </BackgroundView>
    );
}
