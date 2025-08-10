import { Buffer } from 'buffer';
import { useState } from 'react';
import { PermissionsAndroid, Platform } from 'react-native';

import * as ExpoDevice from 'expo-device';

import base64 from 'react-native-base64';
import { BleManager, Device } from 'react-native-ble-plx';

const CFG_SERVICE_UUID = '4b80ba9d-64fd-4ffa-86fb-544e73d26ed1';
const SENSOR_API_ID_CHAR_UUID = '8c680060-22b7-45b8-b325-f7b1b102d80f';
const API_ACCOUNT_ID_CHAR_UUID = 'e11ca181-20c9-4675-b6f3-3f9fb91d1dc1';
const SENSOR_UUID_CHAR_UUID = '333cad84-ceb5-4e18-bfcf-6147987c6733';

const bleManager = new BleManager();

function useBLE() {
    const [allDevices, setAllDevices] = useState<Device[]>([]);
    const [connectedDevice, setConnectedDevice] = useState<Device | null>(null);

    const requestAndroid31Permissions = async () => {
        const bluetoothScanPermission = await PermissionsAndroid.request(
            PermissionsAndroid.PERMISSIONS.BLUETOOTH_SCAN,
            {
                title: 'Location Permission',
                message: 'Bluetooth Low Energy requires Location',
                buttonPositive: 'OK',
            },
        );
        const bluetoothConnectPermission = await PermissionsAndroid.request(
            PermissionsAndroid.PERMISSIONS.BLUETOOTH_CONNECT,
            {
                title: 'Location Permission',
                message: 'Bluetooth Low Energy requires Location',
                buttonPositive: 'OK',
            },
        );
        const fineLocationPermission = await PermissionsAndroid.request(
            PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION,
            {
                title: 'Location Permission',
                message: 'Bluetooth Low Energy requires Location',
                buttonPositive: 'OK',
            },
        );

        return (
            bluetoothScanPermission === 'granted' &&
            bluetoothConnectPermission === 'granted' &&
            fineLocationPermission === 'granted'
        );
    };

    const requestPermissions = async () => {
        if (Platform.OS === 'android') {
            if ((ExpoDevice.platformApiLevel ?? -1) < 31) {
                const granted = await PermissionsAndroid.request(
                    PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION,
                    {
                        title: 'Location Permission',
                        message: 'Bluetooth Low Energy requires Location',
                        buttonPositive: 'OK',
                    },
                );
                return granted === PermissionsAndroid.RESULTS.GRANTED;
            } else {
                const isAndroid31PermissionsGranted = await requestAndroid31Permissions();

                return isAndroid31PermissionsGranted;
            }
        } else {
            return true;
        }
    };

    const connectToDeviceAndConfigure = async (
        device: Device,
        accountIdHEX: string,
        sensorApiIdFetch: (
            user_api_id: string,
            user_place_id: number,
            device_id: string,
            sensor_kind: string,
        ) => Promise<string>,
    ): Promise<string | void> => {
        try {
            const deviceConnection = await bleManager.connectToDevice(device.id);
            setConnectedDevice(deviceConnection);
            await deviceConnection.discoverAllServicesAndCharacteristics();
            bleManager.stopDeviceScan();

            const device_uuid = await getDeviceUUIDHex(device);
            if (!device_uuid) {
                console.log('No device_uuid read from characteristic');
                return 'No device_uuid read from characteristic';
            } else {
                console.debug('device_uuid: ', device_uuid);
            }

            let sensorApiIdHEX;

            try {
                sensorApiIdHEX = await sensorApiIdFetch(
                    accountIdHEX,
                    1,
                    device_uuid!,
                    'Scd4x',
                );
            } catch (e) {
                console.error(
                    '[connectToDeviceAndConfigure] an error occurred on sensorApiIdFetch: ',
                    e,
                );
                return 'Error requesting the server to add our new sensor to the account';
            }

            await configureSensor(deviceConnection, sensorApiIdHEX, accountIdHEX);
        } catch (e) {
            console.log('FAILED TO CONNECT:', e);
            return 'Unexpected error';
        }
        // No errors, we return void
        return;
    };

    const isDuplicteDevice = (devices: Device[], nextDevice: Device) =>
        devices.findIndex((device) => nextDevice.id === device.id) > -1;

    const scanForPeripherals = () =>
        bleManager.startDeviceScan(null, null, (error, device) => {
            if (error) {
                console.log(error);
                return;
            }

            if (
                device &&
                (device.localName === 'esp32-sensor' || device.name === 'esp32-sensor')
            ) {
                setAllDevices((prevState: Device[]) => {
                    if (!isDuplicteDevice(prevState, device)) {
                        return [...prevState, device];
                    }
                    return prevState;
                });
            }
        });

    const stopScanForPeripherals = bleManager.stopDeviceScan;

    // hexStr must be 40 hex chars (20 bytes)
    function encodeHexId20(hexStr: string): string {
        if (hexStr.length !== 40)
            throw new Error('Hex string must be 40 characters (20 bytes)');
        const bytes = new Uint8Array(20);
        for (let i = 0; i < 20; i++) {
            bytes[i] = parseInt(hexStr.substr(i * 2, 2), 16);
        }
        return Buffer.from(bytes).toString('base64');
    }

    // const onDataUpdate = (
    //   error: BleError | null,
    //   characteristic: Characteristic | null
    // ) => {
    //   if (error) {
    //     console.log(error);
    //     return;
    //   } else if (!characteristic?.value) {
    //     console.log("No Data was received");
    //     return;
    //   }

    //   const colorCode = base64.decode(characteristic.value);

    //   let color = "white";
    //   if (colorCode === "B") {
    //     color = "blue";
    //   } else if (colorCode === "R") {
    //     color = "red";
    //   } else if (colorCode === "G") {
    //     color = "green";
    //   }

    //   setColor(color);
    // };

    // const startStreamingData = async (device: Device) => {
    //   if (device) {
    //     device.monitorCharacteristicForService(
    //       DATA_SERVICE_UUID,
    //       SENSOR_AUTHENTICATION_CHARACTERISTIC_UUID,
    //       onDataUpdate
    //     );
    //   } else {
    //     console.log("No Device Connected");
    //   }
    // };

    const getDeviceUUIDHex = async (device: Device): Promise<string | null> => {
        try {
            const uuid = (
                await device.readCharacteristicForService(
                    CFG_SERVICE_UUID,
                    SENSOR_UUID_CHAR_UUID,
                )
            ).value;

            if (!uuid) {
                throw Error('device.value was null');
            }

            const arr = base64
                .decode(uuid)
                .split('')
                .map((char) => char.charCodeAt(0));

            const hexString = arr
                .map((byte) => byte.toString(16).padStart(2, '0'))
                .join('');

            return hexString;
        } catch (e) {
            console.log('Exception occured on getDeviceUUID: ', e);
            return null;
        }
    };

    // Void means no error
    const configureSensor = async (
        device: Device,
        sensorApiIdHEX: string,
        accountIdHEX: string,
    ): Promise<string | void> => {
        if (!device) {
            console.error('No device connected');
            return 'Device disconnected while configuring...';
        }
        const sensorApiId = encodeHexId20(sensorApiIdHEX);
        const accountId = encodeHexId20(accountIdHEX);

        // Set the sensor data using the provided API ID and account ID
        try {
            await device.writeCharacteristicWithResponseForService(
                CFG_SERVICE_UUID,
                SENSOR_API_ID_CHAR_UUID,
                sensorApiId,
            );
            await device.writeCharacteristicWithResponseForService(
                CFG_SERVICE_UUID,
                API_ACCOUNT_ID_CHAR_UUID,
                accountId,
            );
            console.log('Sensor data set successfully');
            return;
        } catch (error) {
            console.error('Failed to set sensor data', error);
            return 'Failed to set sensor data through BLE';
        }
    };

    return {
        connectToDeviceAndConfigure,
        allDevices,
        connectedDevice,
        requestPermissions,
        scanForPeripherals,
        stopScanForPeripherals,
        configureSensor,
    };
}

export default useBLE;
