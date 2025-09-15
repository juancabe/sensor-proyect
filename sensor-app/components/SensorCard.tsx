import React, { useState } from 'react';
import { StyleSheet, TouchableOpacity, View } from 'react-native';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
import { ThemedView } from './ui-elements/ThemedView';
import { useAppContext } from './AppProvider';
import { useRouter } from 'expo-router';
import { timeDisplay } from '@/helpers/timeDisplay';
import { safeGet } from '@/helpers/objectWork';
import LabelValue from './ui-elements/LabelValue';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import { ApiSensorData } from '@/bindings/api/endpoints/sensor_data/ApiSensorData';
import SensorCrudModal from './SensorCrudModal';

export interface SensorCardProps {
    sensor: ApiUserSensor;
    data: ApiSensorData | null;
    reloadSensorSource: () => void;
}

export default function SensorCard(props: SensorCardProps) {
    const sensor = props.sensor;
    const data = props.data;
    const ctx = useAppContext();
    const router = useRouter();

    const lastData: [string, string][] | undefined = (() => {
        if (!data) return undefined;

        const parsed = JSON.parse(JSON.parse(data.data));
        const numberKeys = Object.entries(parsed)
            .filter(([_, v]) => typeof v === 'number')
            .map(([k]) => k);

        return numberKeys.map((key) => [key, safeGet(parsed, key)]);
    })();

    const backgroundColor = props.sensor.color.replace('HEX_', '#') + '99';

    const [crudModalDisplayed, setCrudModalDisplayed] = useState<boolean>(false);

    return (
        <View>
            <SensorCrudModal
                reloadSensorSource={props.reloadSensorSource}
                sensor={props.sensor}
                displayed={crudModalDisplayed}
                setDisplayed={setCrudModalDisplayed}
            ></SensorCrudModal>
            <TouchableOpacity
                onPress={() => {
                    ctx.setActiveSensor(sensor);
                    router.navigate('/SensorDetail');
                }}
                onLongPress={() => setCrudModalDisplayed(true)}
            >
                <ThemedView
                    style={[{ backgroundColor: backgroundColor }, styles.mainContainer]}
                >
                    <LabelValue label="Name" horizontal={true}>
                        <ThemedText style={TEXT_STYLES.heading2}>
                            {sensor.name}
                        </ThemedText>
                    </LabelValue>
                    {data && (
                        <View>
                            {lastData ? (
                                <LabelValue
                                    label={timeDisplay(new Date(data.added_at * 1000))}
                                >
                                    {lastData.map(([label, value], index) => (
                                        <LabelValue label={label} horizontal key={value}>
                                            <ThemedText
                                                key={index}
                                                style={[styles.value, TEXT_STYLES.label]}
                                            >
                                                {value}
                                            </ThemedText>
                                        </LabelValue>
                                    ))}
                                </LabelValue>
                            ) : null}
                        </View>
                    )}
                    <LabelValue label="Last sensor change">
                        <ThemedText>
                            {timeDisplay(new Date(sensor.updated_at * 1000))}
                        </ThemedText>
                    </LabelValue>
                </ThemedView>
            </TouchableOpacity>
        </View>
    );
}

const styles = StyleSheet.create({
    modalLayer: {
        padding: 10,
        margin: 5,
        borderRadius: 10,
        gap: 10,
    },
    crudModal: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        gap: 20,
        borderRadius: 10,
        padding: 15,
        width: '80%',
    },
    value: {
        backgroundColor: '#00000040',
        padding: 10,
        borderRadius: 10,
    },
    sensorName: {
        padding: 10,
        borderRadius: 10,
    },
    mainContainer: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        padding: 10,
        gap: 10,
        width: '100%',
    },
});
