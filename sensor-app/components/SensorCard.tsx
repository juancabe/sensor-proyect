import { SensorSummary } from '@/bindings/SensorSummary';
import React from 'react';
import { StyleSheet, TouchableOpacity } from 'react-native';
import { ThemedText } from './ui-elements/ThemedText';
import { ThemedView } from './ui-elements/ThemedView';
import { useAppContext } from './AppProvider';
import { useRouter } from 'expo-router';
import { timeDisplay } from '@/helpers/timeDisplay';
import { safeGet } from '@/helpers/objectWork';
import LabelValue from './ui-elements/LabelValue';

export interface SensorCardProps {
    sensor: SensorSummary;
}

export default function SensorCard(props: SensorCardProps) {
    const sensor = props.sensor;
    const ctx = useAppContext();
    const router = useRouter();

    let lastData: [string, string][] | undefined = undefined;
    if (sensor.last_serialized_data) {
        const parsed = JSON.parse(sensor.last_serialized_data[0]);

        const numberKeys = Object.entries(parsed)
            .filter(([, v]) => typeof v === 'number')
            .map(([k]) => k as string);
        lastData = [];

        for (const key of numberKeys) {
            lastData.push([key, safeGet(parsed, key)]);
        }
    }

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
                <LabelValue label="Last Update">
                    <ThemedText key={'formattedTime'} style={styles.value}>
                        {timeDisplay(new Date(sensor.last_update * 1000)) + ' ago'}
                    </ThemedText>
                    {lastData ? (
                        <LabelValue label="Information">
                            {lastData.map(([label, value], index) => (
                                <LabelValue label={label} horizontal key={value}>
                                    <ThemedText key={index} style={styles.value}>
                                        {value}
                                    </ThemedText>
                                </LabelValue>
                            ))}
                        </LabelValue>
                    ) : null}
                </LabelValue>
                <LabelValue label="Sensor Kind">
                    <ThemedText style={styles.value}>{sensor.kind}</ThemedText>
                </LabelValue>
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
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        margin: 10,
        padding: 20,
        gap: 10,
    },
});
