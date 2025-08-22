import BindedColorPicker from '@/components/BindedColorPicker';
import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import useBLE from '@/hooks/useBLE';
import React, { useEffect } from 'react';
import { Button, StyleSheet } from 'react-native';
import { Device } from 'react-native-ble-plx';
import { SafeAreaView } from 'react-native-safe-area-context';

import ErrorBox from '@/components/ui-elements/ErrorBox';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { useApiDescription } from '@/hooks/api/useApiDescription';
import { useApiColor } from '@/hooks/api/useApiColor';
import useRedirect from '@/hooks/useRedirect';
import useApi from '@/hooks/useApi';

export default function AddSensorScreen() {
    const ble = useBLE();
    const ctx = useAppContext();
    const redirect = useRedirect();

    const sensorName = useApiEntityName();
    const sensorDescription = useApiDescription();
    const color = useApiColor();

    const allSet = sensorName.isValid && sensorDescription.isValid && color.isValid;

    const api = useApi('/sensors', {}, undefined); // TODO: Complete

    const handleConnect = async (dev: Device) => {
        // TODO: Call api /sensors/post, if OK, configure
        // try {
        //     const res = await ble.connectToDeviceAndConfigure(
        //         dev,
        //         username!,
        //         password,
        //     );
        //     redirectToIndex();
        // } catch (e) {
        //     console.error(
        //         '[handleConnect] connectToDeviceAndConfigure threw and error: ',
        //         e,
        //     );
        //     setErrorText('Error while connecting to device, try again');
        // }

        redirect.redirectToIndex();
    };

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
        },
        {
            placeholder: 'Description (optional)',
            value: sensorDescription.description ? sensorDescription.description : '',
            onChangeText: sensorDescription.setDescription,
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
                        colorValues={color.API_COLORS}
                        selectedColor={color.color}
                        onColorChange={(color_new) => {
                            console.log('Setting color to: ', color_new);
                            color.setColor(color_new);
                        }}
                    ></BindedColorPicker>
                    <ErrorBox error={api.formattedError}></ErrorBox>
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
