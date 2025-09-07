import BindedColorPicker from '@/components/BindedColorPicker';
import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import useBLE from '@/hooks/useBLE';
import React, { useEffect, useState } from 'react';
import { Button, StyleSheet } from 'react-native';
import { Device } from 'react-native-ble-plx';

import ErrorBox from '@/components/ui-elements/ErrorBox';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { useApiDescription } from '@/hooks/api/useApiDescription';
import { useApiColor } from '@/hooks/api/useApiColor';
import useApi from '@/hooks/useApi';
import { ThemedScrollView } from '@/components/ui-elements/ThemedScrollView';
import { PostSensor } from '@/bindings/api/endpoints/sensor/PostSensor';
import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import FeedbackModal from '@/components/FeedbackModal';
import useRedirect from '@/hooks/useRedirect';

const secondaryColor = '#58a4b0';

export default function AddSensorScreen() {
    const redirect = useRedirect();

    const ble = useBLE();
    const ctx = useAppContext();

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

    const api = useApi<PostSensor | undefined, ApiUserSensor, unknown>(
        '/sensor',
        sensorsApiBody,
        sensorsApiMethod,
        false,
    );

    const handleConnect = async (dev: Device) => {
        setSelectedDevice(dev);
        let device_info;
        try {
            device_info = await ble.connectToDevice(dev);
        } catch (e) {
            // TODO: Good error
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
        if (api.returnedOk && api.response) {
            console.log('api returned ok: ', api.response);
            if (!selectedDevice) {
                console.error('API returned ok and no device was previously selected');
                return;
            }
            try {
                ble.configureSensor(selectedDevice, wifiSsid!, wifiPass!);
                setModalVisible(true);
            } catch (e) {
                setError('An error occured while configuring the sensor via BLE');
                console.error('Error on configure sensor: ', e);
            }
        }

        if (api.error) {
            console.error('api error: ', api.error);
        }
    }, [api, selectedDevice, wifiSsid, wifiPass, ble]);

    useEffect(() => {
        const init_ble = async () => {
            try {
                if (!ble.stopScanForPeripherals) {
                    throw 'No ble device';
                }
                await ble.stopScanForPeripherals();
            } catch (e) {
                console.log('Error while trying to stop scan: ', e);
            }
            await ble.requestPermissions();
            await ble.scanForPeripherals();
        };
        init_ble();
    }, []);

    if (!ctx.activePlace) {
        console.error('No active place set when on addSensorScreen');
        return <ThemedView></ThemedView>;
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
        <BackgroundView secondaryColor={secondaryColor}>
            <FeedbackModal borderColor={secondaryColor} visible={modalVisible}>
                <ThemedView style={[styles.feedbackContainer]}>
                    <ThemedText>Sensor correctly configured</ThemedText>
                    <ThemedView style={[styles.feedbackButtonsContainer]}>
                        <Button
                            title="Go back to places"
                            onPress={() => redirect.redirectToIndex()}
                        ></Button>
                    </ThemedView>
                </ThemedView>
            </FeedbackModal>
            <ThemedView style={[styles.mainContainer]}>
                <ThemedText style={TEXT_STYLES.heading1}>
                    Add sensor to {ctx.activePlace.name}
                </ThemedText>
                <ThemedText style={[TEXT_STYLES.body, styles.screenDescription]}>
                    Fill the form and select the sensor from the sensors discovered via
                    Bluethooth
                </ThemedText>
                <ThemedForm fields={formFields}></ThemedForm>
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
                <ThemedScrollView>
                    {ble.allDevices.map((dev) => {
                        return (
                            <ThemedView style={[styles.deviceContainer]} key={dev.id}>
                                <ThemedText>{dev.id}</ThemedText>
                                <Button
                                    disabled={!allSet}
                                    title="Configure"
                                    onPress={() => handleConnect(dev)}
                                />
                            </ThemedView>
                        );
                    })}
                </ThemedScrollView>
            </ThemedView>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    feedbackContainer: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        borderWidth: 3,
        padding: 20,
        gap: 20,
        borderColor: secondaryColor,
    },
    feedbackButtonsContainer: {
        display: 'flex',
        flexDirection: 'row',
        padding: 10,
        gap: 10,
        justifyContent: 'center',
        alignItems: 'center',
    },
    screenDescription: {
        backgroundColor: '#3883',
        padding: 5,
        borderRadius: 5,
    },
    deviceContainer: {
        padding: 10,
        backgroundColor: '#d2ac00ff',
        borderRadius: 10,
        margin: 10,
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
    },
    mainContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        borderWidth: 3,
        borderColor: secondaryColor,
        padding: 20,
        gap: 10,
    },
});
