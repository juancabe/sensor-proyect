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

export default function TabTwoScreen() {
  useFocusEffect(
    useCallback(() => {
      startSearch();
    }, [])
  );

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
              onPress={() => {
                console.log('Connecting...');
                connectToDeviceAndConfigure(
                  device as Device,
                  '1'.repeat(40),
                  '2'.repeat(40)
                );
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
