import { SensorSummary } from '@/bindings/SensorSummary';
import React from 'react';
import { StyleSheet } from 'react-native';
import { ThemedText } from './ui-elements/ThemedText';
import { ThemedView } from './ui-elements/ThemedView';

export interface SensorCardProps {
    sensor: SensorSummary;
}

export interface LabelValueProps {
    label: string;
    value: string;
}

function LabelValue(props: LabelValueProps) {
    return (
        <ThemedView style={styles.labelValueContainer}>
            <ThemedText style={styles.label}>{props.label}</ThemedText>
            <ThemedText style={styles.value}>{props.value}</ThemedText>
        </ThemedView>
    );
}

function getFormattedLastUpdate(date: Date) {
    const diff = new Date().getTime() - date.getTime();
    const MILLIS = 1000;
    const SECONDS = 60;
    const MINUTES_30 = MILLIS * SECONDS * 30;
    const MINUTES_2 = MILLIS * SECONDS * 2;

    if (Math.abs(diff) > MINUTES_30) {
        return date.toUTCString();
    } else {
        if (diff > MINUTES_2) {
            return `${~~(diff / (MILLIS / SECONDS))} minutes ago`;
        } else {
            return `${~~(diff / MILLIS)} seconds ago`;
        }
    }
}

export default function SensorCard(props: SensorCardProps) {
    const sensor = props.sensor;

    return (
        <ThemedView
            style={[
                { backgroundColor: props.sensor.color.replace('HEX_', '#') },
                styles.mainContainer,
            ]}
        >
            <ThemedText style={styles.sensorName}>{sensor.name}</ThemedText>
            <LabelValue
                label="Last Update"
                value={getFormattedLastUpdate(new Date(sensor.last_update * 1000))}
            />
            <LabelValue label="Sensor Kind" value={sensor.kind} />
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    value: {
        backgroundColor: '#00000040',
        padding: 10,
        borderRadius: 10,
    },
    sensorName: {
        backgroundColor: '#00000040',
        padding: 10,
        fontSize: 20,
        borderRadius: 10,
    },
    mainContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        margin: 10,
        padding: 20,
        gap: 10,
    },
    labelValueContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        backgroundColor: '#00000040',
        padding: 10,
        borderRadius: 10,
        gap: 10,
    },
    label: {
        fontSize: 15,
        fontWeight: 'bold',
    },
});
