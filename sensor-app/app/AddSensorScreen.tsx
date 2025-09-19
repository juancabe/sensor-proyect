import BindedColorPicker from '@/components/BindedColorPicker';
import Form, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import useBLE from '@/hooks/useBLE';
import React, { useEffect, useState } from 'react';
import { ScrollView } from 'react-native';
import { Device } from 'react-native-ble-plx';

import ErrorBox from '@/components/ui-elements/ErrorBox';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { useApiDescription } from '@/hooks/api/useApiDescription';
import { useApiColor } from '@/hooks/api/useApiColor';
import useApi from '@/hooks/useApi';
import { PostSensor } from '@/bindings/api/endpoints/sensor/PostSensor';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import SensorsModal from '@/components/FeedbackModal';
import { Redirect, useRouter } from 'expo-router';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { Button } from '@/ui/components/Button';

export default function AddSensorScreen() {
    const ble = useBLE();
    const ctx = useAppContext();
    const router = useRouter();

    const sensorName = useApiEntityName();
    const sensorDescription = useApiDescription();
    const color = useApiColor();

    const [error, setError] = useState<string | undefined>(undefined);

    const [wifiSsid, setWifiSsid] = useState<string | undefined>();
    const [wifiSsidError, setWifiSsidError] = useState<string | undefined>(
        'Wifi SSID is mandatory',
    );
    useEffect(() => {
        if (wifiSsid === undefined) {
            setWifiSsidError('Wifi SSID is mandatory');
            return;
        }
        if (wifiSsid.length > 32) {
            setWifiSsidError('Wifi SSID too long, max 32 characters');
            return;
        }
        if (wifiSsid.length < 1) {
            setWifiSsidError('Wifi SSID too short');
            return;
        }
        if (!/^[\x20-\x7E]*$/.test(wifiSsid)) {
            setWifiSsidError('Wifi SSID contains non ASCII characters');
            return;
        }
        setWifiSsidError(undefined);
    }, [wifiSsid, setWifiSsidError]);

    const [wifiPass, setWifiPass] = useState<string | undefined>(undefined);
    const [wifiPassError, setWifiPassError] = useState<string | undefined>(undefined);
    useEffect(() => {
        if (wifiPass === undefined) {
            setWifiPassError('Wifi password is mandatory');
            return;
        }
        if (wifiPass.length > 63) {
            setWifiPassError('Wifi password too long, max 63 characters');
            return;
        }
        if (wifiPass.length < 8) {
            setWifiPassError('Wifi password too short');
            return;
        }
        if (!/^[\x20-\x7E]*$/.test(wifiPass)) {
            setWifiPassError('Wifi password contains non ASCII characters');
            return;
        }
        setWifiPassError(undefined);
    }, [wifiPass, setWifiPassError]);

    const allSet =
        sensorName.isValid &&
        sensorDescription.isValid &&
        color.isValid &&
        !wifiPassError &&
        !wifiSsidError;

    const [sensorsApiBody, setSensorsApiBody] = useState<undefined | PostSensor>(
        undefined,
    );
    const [sensorsApiMethod, setSensorApiMethod] = useState<'POST' | undefined>(
        undefined,
    );

    const [selectedDevice, setSelectedDevice] = useState<Device | undefined>(undefined);
    const [configuring, setConfiguring] = useState<boolean>(false);

    const api = useApi<PostSensor | undefined, ApiUserSensor, unknown>(
        '/sensor',
        sensorsApiMethod,
        false,
        sensorsApiBody,
    );

    const handleConnect = async (dev: Device) => {
        setSelectedDevice(dev);
        setConfiguring(true);
        let device_info;
        try {
            device_info = await ble.connectToDevice(dev);
        } catch (e) {
            setError("Couldn't connect to the device");
            setConfiguring(false);
            console.error('connect error: ', e);
        }

        if (!ctx.activePlace || !device_info) {
            return;
        }

        const postSensorBody: PostSensor = {
            place_name: ctx.activePlace?.name,
            device_id: device_info.sensorDeviceId,
            pub_key: device_info.sensorPubKey,
            name: sensorName.name,
            description: sensorDescription.description,
            color: color.color,
        };

        setSensorsApiBody(postSensorBody);
        setSensorApiMethod('POST');

        return;
    };

    const [modalVisible, setModalVisible] = useState(false);

    useEffect(() => {
        const fn = async (dev: Device) => {
            try {
                await ble.configureSensor(dev, wifiSsid!, wifiPass!);
                setModalVisible(true);
            } catch (e) {
                setConfiguring(false);
                setError('An error occured while configuring the sensor via BLE');
                console.error('Error on configure sensor: ', e);
            }
        };

        if (api.returnedOk && api.response) {
            console.log('api returned ok: ', api.response);
            if (!selectedDevice) {
                console.error('API returned ok and no device was previously selected');
                return;
            }
            fn(selectedDevice);
        }

        if (api.error) {
            setConfiguring(false);
            console.error('api error: ', api.error);
        }
    }, [api, selectedDevice, wifiSsid, wifiPass, ble]);

    const requestPermissions = ble.requestPermissions;
    const scanForPeripherals = ble.scanForPeripherals;
    const stopScanForPeripherals = ble.stopScanForPeripherals;

    useEffect(() => {
        const init_ble = async () => {
            try {
                if (!stopScanForPeripherals) {
                    throw 'No ble device';
                }
                await stopScanForPeripherals();
            } catch (e) {
                console.log('Error while trying to stop scan: ', e);
            }
            await requestPermissions();
            await scanForPeripherals();
        };
        init_ble();
    }, [requestPermissions, scanForPeripherals, stopScanForPeripherals]);

    if (!ctx.activePlace) {
        console.error('No active place set when on addSensorScreen');
        return <Redirect href={'/home'}></Redirect>;
    }

    const formFields: FieldConfig[] = [
        {
            placeholder: 'Name',
            value: sensorName.name,
            onChangeText: sensorName.setName,
            error: sensorName.error,
        },
        {
            placeholder: 'Description (optional)',
            value: sensorDescription.description ? sensorDescription.description : '',
            onChangeText: sensorDescription.setDescription,
            error: sensorDescription.error,
        },
        {
            placeholder: 'Wifi SSID',
            value: wifiSsid ? wifiSsid : '',
            onChangeText: setWifiSsid,
            error: wifiSsidError,
        },
        {
            placeholder: 'Wifi Password',
            value: wifiPass ? wifiPass : '',
            onChangeText: setWifiPass,
            error: wifiPassError,
        },
    ];

    return (
        <BackgroundView>
            <SensorsModal visible={modalVisible}>
                <Card variant="subtle" gap="l">
                    <Text variant="heading">Sensor correctly configured</Text>
                    <Button
                        label="Go back to home"
                        onPress={() => router.push('/home')}
                    ></Button>
                </Card>
            </SensorsModal>
            <Card variant="subtle" flex={1}>
                <ScrollView>
                    <Box gap="m" alignItems="center">
                        <Box flexDirection="row" alignItems="flex-end" gap="s">
                            <Text variant="heading">Add sensor to</Text>
                            <Text variant="subTitle">{ctx.activePlace.name}</Text>
                        </Box>
                        <Text variant="body">
                            Fill the form and select the sensor from the sensors
                            discovered via Bluethooth
                        </Text>
                        <Box width={'100%'}>
                            <Form fields={formFields}></Form>
                        </Box>
                        <BindedColorPicker
                            colorValues={color.API_COLORS}
                            selectedColor={color.color}
                            onColorChange={(color_new) => {
                                console.log('Setting color to: ', color_new);
                                color.setColor(color_new);
                            }}
                        ></BindedColorPicker>
                        {api.formattedError ? (
                            <ErrorBox error={api.formattedError}></ErrorBox>
                        ) : null}
                        {error ? <ErrorBox error={error}></ErrorBox> : null}
                        {ble.allDevices.map((dev) => {
                            return (
                                <Card variant="elevated" key={dev.id}>
                                    <Text variant="heading">{dev.id}</Text>
                                    <Button
                                        disabled={!allSet || configuring}
                                        label="Configure"
                                        onPress={() => handleConnect(dev)}
                                    />
                                </Card>
                            );
                        })}
                    </Box>
                </ScrollView>
            </Card>
        </BackgroundView>
    );
}
