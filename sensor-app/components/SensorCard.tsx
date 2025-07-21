import { SensorSummary } from '@/bindings/SensorSummary';
import React from 'react';
import { StyleSheet } from 'react-native';
import { ThemedText } from './ThemedText';
import { ThemedView } from './ThemedView';

export interface SensorCardProps {
    sensor: SensorSummary;
    placeColor: string;
}

export default function SensorCard(props: SensorCardProps) {
    const sensor = props.sensor;
    console.log('loaded sensor: ', sensor.api_id);
    return (
        <ThemedView style={styles.container}>
            <ThemedText>{sensor.kind}</ThemedText>
            <ThemedText>{sensor.last_update}</ThemedText>
            <ThemedText>{sensor.device_id.id}</ThemedText>
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    container: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        backgroundColor: '#11551160',
        borderRadius: 10,
        margin: 10,
        padding: 10,
    },
});
