import { PlaceSummary } from '@/bindings/PlaceSummary';
import { SensorSummary } from '@/bindings/SensorSummary';
import { useRouter } from 'expo-router';
import React from 'react';
import { Button, StyleSheet, View } from 'react-native';
import { useAppContext } from './AppProvider';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
import SensorCard from './SensorCard';
import { ThemedView } from './ui-elements/ThemedView';

export interface PlaceCardProps {
    place: PlaceSummary;
    sensors: SensorSummary[];
}

export default function PlaceCard({ place, sensors }: PlaceCardProps) {
    const router = useRouter();
    const ctx = useAppContext();

    const handleAddSensorPress = () => {
        if (ctx.setActivePlace(place.place_id.id)) {
            console.warn('TODO: navigate to AddSensorScreen');
            // router.navigate('/AddSensorScreen');
        } else {
            // TODO: Display error
        }
    };

    return (
        <ThemedView
            style={[
                styles.container,
                { backgroundColor: place.color.replace('HEX_', '#') },
            ]}
        >
            <ThemedText style={TEXT_STYLES.heading2}>{place.name}</ThemedText>
            <Button title="Add Sensor" onPress={handleAddSensorPress} />
            <View
                style={{
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    justifyContent: 'center',
                }}
            >
                {sensors.map((sensor) => (
                    <SensorCard key={sensor.api_id.id} sensor={sensor} />
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
        gap: 10,
    },
});
