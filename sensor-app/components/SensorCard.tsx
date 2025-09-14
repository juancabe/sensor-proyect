import React, { useEffect, useState } from 'react';
import { Button, StyleSheet, TouchableOpacity, View } from 'react-native';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
import { ThemedView } from './ui-elements/ThemedView';
import { useAppContext } from './AppProvider';
import { useRouter } from 'expo-router';
import { timeDisplay } from '@/helpers/timeDisplay';
import { safeGet } from '@/helpers/objectWork';
import LabelValue from './ui-elements/LabelValue';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import { ApiSensorData } from '@/bindings/api/endpoints/sensor_data/ApiSensorData';
import SensorsModal from './FeedbackModal';
import useApi from '@/hooks/useApi';
import { DeleteSensor } from '@/bindings/api/endpoints/sensor/DeleteSensor';
import ErrorBox from './ui-elements/ErrorBox';
import ThemedForm, { FieldConfig } from './ui-elements/ThemedForm';
import { PutSensor } from '@/bindings/api/endpoints/sensor/PutSensor';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';

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

    const [apiMethod, setApiMethod] = useState<undefined | 'DELETE' | 'PUT'>(undefined);
    const [apiBody, setApiBody] = useState<undefined | DeleteSensor | PutSensor>();
    const api = useApi('/sensor', apiMethod, false, apiBody as any);

    useEffect(() => {
        if (api.response && api.returnedOk) {
            // TODO: Call reload on parent
            props.reloadSensorSource();
        }
    }, [api.response, api.returnedOk, props]);

    const apiName = useApiEntityName(props.sensor.name);
    const crudModalFormFields: FieldConfig[] = [
        {
            placeholder: 'Sensor Name',
            value: apiName.name,
            onChangeText: (name) => {
                apiName.setName(name);
            },
            error: apiName.error,
        },
    ];

    return (
        <View>
            <SensorsModal borderColor={sensor.color} visible={crudModalDisplayed}>
                <ThemedView style={styles.crudModal}>
                    <Button
                        title="Delete sensor"
                        onPress={() => {
                            const body: DeleteSensor = {
                                'FromSensorDeviceId': props.sensor.device_id,
                            };
                            setApiBody(body);
                            setApiMethod('DELETE');
                        }}
                    ></Button>
                    <Button
                        title="Close"
                        onPress={() => setCrudModalDisplayed(false)}
                    ></Button>
                    <ThemedText style={TEXT_STYLES.label}>Edit sensor name:</ThemedText>
                    <ThemedForm fields={crudModalFormFields}></ThemedForm>
                    <Button
                        title="Confirm edit"
                        onPress={() => {
                            const body: PutSensor = {
                                device_id: props.sensor.device_id,
                                change: { 'Name': apiName.name },
                            };
                            setApiBody(body);
                            setApiMethod('PUT');
                        }}
                        disabled={!apiName.isValid}
                    ></Button>
                    <ErrorBox error={api.formattedError}></ErrorBox>
                </ThemedView>
            </SensorsModal>
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
    crudModal: {
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
        backgroundColor: '#222',
        borderColor: '#222',
        borderRadius: 10,
        padding: 5,
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
