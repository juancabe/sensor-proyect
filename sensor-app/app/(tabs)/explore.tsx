import { Image } from 'expo-image';
import { Button, Platform, StyleSheet, TouchableHighlight } from 'react-native';

import { Collapsible } from '@/components/Collapsible';
import { ExternalLink } from '@/components/ExternalLink';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { IconSymbol } from '@/components/ui/IconSymbol';
import useBLE from '@/hooks/useBLE';
import { useCallback, useEffect, useState } from 'react';
import { Colors } from '@/constants/Colors';

import { useColorScheme } from 'react-native';
import { RefreshButton } from '@/components/RefreshButton';
import { useFocusEffect } from 'expo-router';
import { ThemedList } from '@/components/ThemedList';
import { Device } from 'react-native-ble-plx';
import { SessionKeys, useSession } from '@/hooks/useSession';

export default function TabTwoScreen() {
  useFocusEffect(
    useCallback(() => {
      startSearch();
    }, [])
  );

  const { sessionData, setItem } = useSession();

  const {
    allDevices,
    connectedDevice,
    connectToDeviceAndConfigure,
    color,
    requestPermissions,
    scanForPeripherals,
    stopScanForPeripherals,
  } = useBLE();

  const [isScanning, setIsScanning] = useState<boolean>(false);

  const colorScheme = useColorScheme();

  const scanForDevices = async () => {
    const isPermissionsEnabled = await requestPermissions();
    if (isPermissionsEnabled) {
      console.log('Permissions granted, scanning...');
      scanForPeripherals();
    } else {
      console.log('Permissions not granted');
    }
  };

  const startSearch = async () => {
    if (isScanning) return;

    setIsScanning(true);
    await scanForDevices();
    setIsScanning(false);
  };

  return (
    <ParallaxScrollView
      headerBackgroundColor={{ light: '#D0D0D0', dark: '#353636' }}
      headerImage={
        <IconSymbol
          size={310}
          color="#808080"
          name="chevron.left.forwardslash.chevron.right"
          style={styles.headerImage}
        />
      }
    >
      <ThemedView style={styles.titleContainer}>
        <ThemedText type="title">Add Sensor</ThemedText>
        <TouchableHighlight onPress={startSearch} onLongPress={startSearch}>
          <RefreshButton />
        </TouchableHighlight>
      </ThemedView>

      <ThemedList
        data={allDevices}
        items={allDevices}
        title="Available Devices"
        renderItem={(device) => (
          <ThemedView style={styles.deviceContainer}>
            <ThemedText>
              {(device as Device).name || 'Unnamed Device'}
            </ThemedText>
            <Button
              title="Connect"
              disabled={!sessionData[SessionKeys.API_USER_ID]}
              onPress={() => {
                console.log('Connecting...');
                connectToDeviceAndConfigure(
                  device as Device,
                  sessionData[SessionKeys.API_USER_ID]!,
                  sensorApiIdFetch
                ).then((connected) => {
                  if (connected) {
                    console.log('Connected to device:', device);
                    setItem(SessionKeys.API_SENSOR_ID, (device as Device).id);
                  } else {
                    console.log('Failed to connect to device:', device);
                  }
                });
              }}
            />
          </ThemedView>
        )}
        onItemPress={(device) => {
          console.log('device: ', device);
        }}
      />
      <ThemedView style={styles.titleContainer}></ThemedView>
    </ParallaxScrollView>
  );
}

async function sensorApiIdFetch(
  accountIdHEX: string,
  user_place_id: number,
  ble_mac: string,
  sensor_kind: string
): Promise<string> {
  // const api_path = 'http://192.168.1.133:3000/api/v0/post_sensor';
  const api_path = 'http://sensor-server.juancb.ftp.sh:3000/api/v0/post_sensor';
  const user_uuid = accountIdHEX;

  const response = await fetch(api_path, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      user_uuid,
      user_place_id,
      sensor_mac: ble_mac,
      sensor_kind,
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to fetch sensor API ID: ' + response.statusText);
  }
  const data = await response.json();
  return data.sensor_api_id;
}

const styles = StyleSheet.create({
  headerImage: {
    color: '#808080',
    bottom: -90,
    left: -35,
    position: 'absolute',
  },
  titleContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    gap: 8,
  },
  deviceContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
});
