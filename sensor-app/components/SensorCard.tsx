import { SensorSummary } from '@/bindings/SensorSummary';
import React from 'react';
import { StyleSheet, TouchableOpacity } from 'react-native';
import { ThemedText } from './ui-elements/ThemedText';
import { ThemedView } from './ui-elements/ThemedView';
import { useAppContext } from './AppProvider';
import { useRouter } from 'expo-router';
import { timeDisplay } from '@/helpers/timeDisplay';

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

export default function SensorCard(props: SensorCardProps) {
    const sensor = props.sensor;
    const ctx = useAppContext();
    const router = useRouter();

    return (
        <TouchableOpacity
            onPress={() => {
                ctx.setActiveSensor(sensor);
                router.navigate('/SensorDetail');
            }}
        >
            <ThemedView
                style={[
                    { backgroundColor: props.sensor.color.replace('HEX_', '#') },
                    styles.mainContainer,
                ]}
            >
                <ThemedText style={styles.sensorName}>{sensor.name}</ThemedText>
                <LabelValue
                    label="Last Update"
                    value={timeDisplay(new Date(sensor.last_update * 1000))}
                />
                <LabelValue label="Sensor Kind" value={sensor.kind} />
            </ThemedView>
        </TouchableOpacity>
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
