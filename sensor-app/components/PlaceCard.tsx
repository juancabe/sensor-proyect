import { useRouter } from 'expo-router';
import React from 'react';
import { Button, StyleSheet, View } from 'react-native';
import { useAppContext } from './AppProvider';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
// import SensorCard from './SensorCard';
import { ThemedView } from './ui-elements/ThemedView';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';

export interface PlaceCardProps {
    place: ApiUserPlace;
}

export default function PlaceCard({ place }: PlaceCardProps) {
    const router = useRouter();
    const ctx = useAppContext();

    const handleAddSensorPress = () => {
        ctx.setActivePlace(place);
        router.navigate('/AddSensorScreen');
    };

    // TODO: Call get sensors api

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
                {/* {sensors.map(([sensor, data]) => ( */}
                {/*     <SensorCard key={sensor.device_id} sensor={sensor} data={data} /> */}
                {/* ))} */}
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
