import { SensorSummary } from '@/bindings/SensorSummary';
import React from 'react';
import { StyleSheet, View } from 'react-native';
import SensorCard from './SensorCard';
import { TEXT_STYLES, ThemedText } from './ThemedText';
import { ThemedView } from './ThemedView';

export interface PlaceCardProps {
    place: [string, string | null];
    sensors: SensorSummary[];
    placeColor: string;
}

export default function PlaceCard({ place, sensors, placeColor }: PlaceCardProps) {
    console.log('placeColor: ', placeColor);

    return (
        <ThemedView style={styles.container}>
            <ThemedText style={TEXT_STYLES.heading2}>{place[0]}</ThemedText>
            <View style={{ flexDirection: 'row', flexWrap: 'wrap' }}>
                {sensors.map((sensor) => (
                    <SensorCard
                        key={sensor.api_id.id}
                        sensor={sensor}
                        placeColor={placeColor}
                    />
                ))}
            </View>
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    container: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        margin: 10,
        padding: 10,
    },
});
