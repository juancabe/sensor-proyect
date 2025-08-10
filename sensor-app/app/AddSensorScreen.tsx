import BindedColorPicker from '@/components/BindedColorPicker';
import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { newUserSensor } from '@/api/sensor_crud';
import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import useBLE from '@/hooks/useBLE';
import React, { useEffect, useState } from 'react';
import { Button, StyleSheet } from 'react-native';
import { Device } from 'react-native-ble-plx';
import { SafeAreaView } from 'react-native-safe-area-context';

import { PostSensorResponseBody } from '@/bindings/endpoints/PostSensor';
import { SensorColor } from '@/bindings/SensorColor';
import { useRouter } from 'expo-router';
import ErrorBox from '@/components/ui-elements/ErrorBox';

const sensorColorValues: Record<SensorColor, string> = {
    HEX_DB2122: '#DB2122',
    HEX_F0D16F: '#F0D16F',
    HEX_21DB55: '#21DB55',
    HEX_2132DB: '#2132DB',
    HEX_6FF0D1: '#6FF0D1',
    HEX_DB21A0: '#DB21A0',
    HEX_DB8F21: '#DB8F21',
};

export default function AddSensorScreen() {
    const ble = useBLE();
    const ctx = useAppContext();
    const router = useRouter();

    const [sensorName, setSensorName] = useState<string | undefined>(undefined);
    const [sensorDescription, setSensorDescription] = useState<string | null>(null);
    const [selectedColor, setSelectedColor] = useState<SensorColor | undefined>(
        undefined,
    );

    const [errorText, setErrorText] = useState<string | null>(null);

    const redirectToIndex = () => {
        ctx.reloadSummary();
        router.replace('/');
    };

    const allSet = sensorName && selectedColor;

    const handleConnect = async (dev: Device) => {
        if (!ctx.sessionData?.all_set()) {
            console.error('[handleConnect] sessionData not set');
            setErrorText(
                'Your login data is incorrectly set somehow, please log out and try again!',
            );
            return;
        }
        let user_api_id = ctx.sessionData!.api_id;

        let place_id = ctx.activePlace?.place_id;
        if (!place_id) {
            console.error('[handleConnect] No place id found');
            setErrorText(
                "I don't know what place does this sensor belong to, please select the place again!",
            );
            return;
        }

        const sensorApiIdFetch = async (device_id: string) => {
            const response = await newUserSensor(
                { id: user_api_id! },
                place_id,
                { id: device_id },
                'Scd4x',
                sensorName!,
                sensorDescription,
                selectedColor!,
            );

            if (response && (response as PostSensorResponseBody).sensor_api_id) {
                return (response as PostSensorResponseBody).sensor_api_id;
            } else {
                throw response;
            }
        };

        try {
            const res = await ble.connectToDeviceAndConfigure(
                dev,
                user_api_id!,
                sensorApiIdFetch,
            );
            if (res) {
                redirectToIndex();
            } else {
                console.error(
                    '[handleConnect] connectToDeviceAndConfigure returned false',
                );
            }
        } catch (e) {
            console.error(
                '[handleConnect] connectToDeviceAndConfigure threw and error: ',
                e,
            );
        }
    };

    useEffect(() => {
        const init_ble = async () => {
            try {
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
            value: sensorName ? sensorName : '',
            onChangeText: setSensorName,
        },
        {
            placeholder: 'Description (optional)',
            value: sensorDescription ? sensorDescription : '',
            onChangeText: setSensorDescription,
        },
    ];

    return (
        <BackgroundView secondaryColor="#ffd9009b">
            <SafeAreaView>
                <ThemedView style={[styles.mainContainer]}>
                    <ThemedText style={TEXT_STYLES.heading1}>
                        Add sensor to {ctx.activePlace.name}
                    </ThemedText>
                    <ThemedForm fields={formFields}></ThemedForm>
                    <BindedColorPicker
                        colorValues={sensorColorValues}
                        selectedColor={selectedColor}
                        onColorChange={(color) => {
                            console.log('Setting color to: ', color);
                            setSelectedColor(color as SensorColor);
                        }}
                    ></BindedColorPicker>
                    <ErrorBox error={errorText}></ErrorBox>
                    {ble.allDevices.map((dev) => {
                        return (
                            <ThemedView style={[styles.deviceContainer]} key={dev.id}>
                                <ThemedText>{dev.id}</ThemedText>
                                <Button
                                    disabled={!allSet}
                                    title="Connect"
                                    onPress={() => handleConnect(dev)}
                                />
                            </ThemedView>
                        );
                    })}
                </ThemedView>
            </SafeAreaView>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    deviceContainer: {
        padding: 20,
        backgroundColor: '#d2ac00ff',
        borderRadius: 10,
        margin: 20,
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
    },
    mainContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        padding: 20,
    },
});
