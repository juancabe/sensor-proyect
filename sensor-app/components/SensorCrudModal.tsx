import SensorsModal from './FeedbackModal';
import CloseButton from './CloseButton';
import Form, { FieldConfig } from './ui-elements/ThemedForm';
import { useEffect, useState } from 'react';
import useApi from '@/hooks/useApi';
import { DeleteSensor } from '@/bindings/api/endpoints/sensor/DeleteSensor';
import { PutSensor } from '@/bindings/api/endpoints/sensor/PutSensor';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import ErrorBox from './ui-elements/ErrorBox';
import { Card } from '@/ui/components/Card';
import { Text } from '@/ui/theme';
import { Button } from '@/ui/components/Button';
import { safeGet } from '@/helpers/objectWork';

export interface SensorCrudModalProps {
    gotoSensorSource: () => void;
    sensor: ApiUserSensor;
    displayed: boolean;
    setDisplayed: (value: boolean) => void;
    setSensor: (sensor: ApiUserSensor) => void;
}

export default function SensorCrudModal({
    gotoSensorSource: gotoSensorSource,
    sensor,
    displayed,
    setDisplayed,
    setSensor,
}: SensorCrudModalProps) {
    const [apiMethod, setApiMethod] = useState<undefined | 'DELETE' | 'PUT'>(undefined);
    const [apiBody, setApiBody] = useState<undefined | DeleteSensor | PutSensor>();
    const api = useApi('/sensor', apiMethod, false, apiBody as any);

    useEffect(() => {
        if (api.response && api.returnedOk) {
            if (apiMethod === 'DELETE') {
                gotoSensorSource();
            } else {
                const sensor = api.response as ApiUserSensor;
                if (sensor && safeGet(sensor, 'device_id')) setSensor(sensor);
                setDisplayed(false);
            }
        }
    }, [
        api.response,
        api.returnedOk,
        gotoSensorSource,
        apiMethod,
        setSensor,
        setDisplayed,
    ]);

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

    const [deletePressed, setDeletePressed] = useState<number>(0);

    const handleClose = () => {
        setDeletePressed(0);
        setDisplayed(false);
    };

    return (
        <SensorsModal visible={displayed}>
            <Card variant="elevated" gap="m">
                <Card
                    variant="subtle"
                    flexDirection="row"
                    justifyContent="space-between"
                    alignItems="center"
                >
                    <Text variant="heading">{sensor.name}</Text>
                    <CloseButton onPress={() => handleClose()}></CloseButton>
                </Card>
                {deletePressed === 0 ? (
                    <Card variant="subtle" gap="m">
                        <Card variant="elevated" gap="s">
                            <Text variant="body">Edit sensor name</Text>
                            <Form fields={crudModalFormFields}></Form>
                        </Card>
                        <Button
                            label="Confirm edit"
                            onPress={() => {
                                const body: PutSensor = {
                                    device_id: sensor.device_id,
                                    change: { 'Name': apiName.name },
                                };
                                setApiBody(body);
                                setApiMethod('PUT');
                            }}
                            disabled={!apiName.isValid}
                        ></Button>
                    </Card>
                ) : null}

                <Card variant="subtle">
                    {deletePressed > 0 ? (
                        <Text variant="body" color="warning">
                            Deleting a sensor involves deleting all sensor data collected
                            until now, you may want to save a copy before the operation is
                            done. Once done, the operation cannot be reverted.
                        </Text>
                    ) : null}

                    <Button
                        variant="negative"
                        label="Delete sensor"
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
                    ></Button>
                </Card>
                {api.formattedError && <ErrorBox error={api.formattedError}></ErrorBox>}
            </Card>
        </SensorsModal>
    );
}
