import { StyleSheet, View } from 'react-native';
import SensorsModal from './FeedbackModal';
import { ThemedView } from './ui-elements/ThemedView';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
import CloseButton from './CloseButton';
import ThemedButton from './ui-elements/ThemedButton';
import Form, { FieldConfig } from './ui-elements/ThemedForm';
import useLayerColor from '@/hooks/useLayerColor';
import useInvertedLayerColor from '@/hooks/useInvertedLayerColor';
import { useEffect, useState } from 'react';
import useApi from '@/hooks/useApi';
import { DeleteSensor } from '@/bindings/api/endpoints/sensor/DeleteSensor';
import { PutSensor } from '@/bindings/api/endpoints/sensor/PutSensor';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import ErrorBox from './ui-elements/ErrorBox';

export interface SensorCrudModalProps {
    reloadSensorSource: () => void;
    sensor: ApiUserSensor;
    displayed: boolean;
    setDisplayed: (value: boolean) => void;
}

export default function SensorCrudModal({
    reloadSensorSource,
    sensor,
    displayed,
    setDisplayed,
}: SensorCrudModalProps) {
    const [apiMethod, setApiMethod] = useState<undefined | 'DELETE' | 'PUT'>(undefined);
    const [apiBody, setApiBody] = useState<undefined | DeleteSensor | PutSensor>();
    const api = useApi('/sensor', apiMethod, false, apiBody as any);

    useEffect(() => {
        if (api.response && api.returnedOk) {
            reloadSensorSource();
        }
    }, [api.response, api.returnedOk, reloadSensorSource]);

    const apiName = useApiEntityName(sensor.name);
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

    const layerColor = useLayerColor();
    const invertedLayerColor = useInvertedLayerColor();
    const invertedLayerColor_2 = useInvertedLayerColor(2);

    const [deletePressed, setDeletePressed] = useState<number>(0);

    const handleClose = () => {
        setDeletePressed(0);
        setDisplayed(false);
    };

    return (
        <SensorsModal visible={displayed}>
            <ThemedView style={styles.crudModal}>
                <View
                    style={{
                        display: 'flex',
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        paddingLeft: 8,
                    }}
                >
                    <ThemedText
                        style={[
                            TEXT_STYLES.heading2,
                            {
                                backgroundColor: invertedLayerColor,
                                padding: 5,
                                borderRadius: 5,
                            },
                        ]}
                    >
                        {sensor.name}
                    </ThemedText>
                    <CloseButton onPress={() => handleClose()}></CloseButton>
                </View>
                {deletePressed === 0 ? (
                    <View
                        style={[
                            styles.modalLayer,
                            { backgroundColor: invertedLayerColor },
                        ]}
                    >
                        <View
                            style={{
                                backgroundColor: layerColor,
                                borderRadius: 5,
                                padding: 5,
                            }}
                        >
                            <ThemedText style={[TEXT_STYLES.body, { paddingLeft: 5 }]}>
                                Edit sensor name
                            </ThemedText>
                            <Form fields={crudModalFormFields}></Form>
                        </View>
                        <ThemedButton
                            title="Confirm edit"
                            onPress={() => {
                                const body: PutSensor = {
                                    device_id: sensor.device_id,
                                    change: { 'Name': apiName.name },
                                };
                                setApiBody(body);
                                setApiMethod('PUT');
                            }}
                            disabled={!apiName.isValid}
                        ></ThemedButton>
                    </View>
                ) : null}

                <View style={[styles.modalLayer, { backgroundColor: layerColor }]}>
                    {deletePressed > 0 ? (
                        <ThemedText
                            style={[
                                TEXT_STYLES.body,
                                {
                                    color: 'red',
                                    opacity: 1,
                                    backgroundColor: invertedLayerColor_2,
                                    padding: 5,
                                    borderRadius: 5,
                                },
                            ]}
                        >
                            Deleting a sensor involves deleting all sensor data collected
                            until now, you may want to save a copy before the operation is
                            done. Once done, the operation cannot be reverted.
                        </ThemedText>
                    ) : null}

                    <ThemedButton
                        title="Delete sensor"
                        onPress={() => {
                            if (deletePressed === 0) {
                                setDeletePressed(1);
                                return;
                            }
                            const body: DeleteSensor = {
                                'FromSensorDeviceId': sensor.device_id,
                            };
                            setApiBody(body);
                            setApiMethod('DELETE');
                        }}
                        style={{ backgroundColor: 'red' }}
                    ></ThemedButton>
                </View>
                {api.formattedError && <ErrorBox error={api.formattedError}></ErrorBox>}
            </ThemedView>
        </SensorsModal>
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
});
