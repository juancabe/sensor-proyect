import { Image } from 'expo-image';
import { Button, Platform, StyleSheet } from 'react-native';

import { Collapsible } from '@/components/Collapsible';
import { ExternalLink } from '@/components/ExternalLink';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { IconSymbol } from '@/components/ui/IconSymbol';
import useBLE from '@/hooks/useBLE';
import { useState } from 'react';

export default function TabTwoScreen() {

  const {
    allDevices,
    connectedDevice,
    connectToDevice,
    color,
    requestPermissions,
    scanForPeripherals,
  } = useBLE();

  const [isModalVisible, setIsModalVisible] = useState<boolean>(false);

  const scanForDevices = async () => {
    const isPermissionsEnabled = await requestPermissions();
    if (isPermissionsEnabled) {
      console.log("Permissions granted, scanning...");
      scanForPeripherals();
    }
  };

  const startSearch = async () => {
    scanForDevices();
    setIsModalVisible(true);
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
      }>
      <ThemedView style={styles.titleContainer}>
        <ThemedText type="title">Explore Devices</ThemedText>
      </ThemedView>
      <ThemedText>Here you can scan for new devices.</ThemedText>
      {isModalVisible ? null : <Button title="Scan for Devices" onPress={startSearch}></Button>}
        {
          allDevices.map((device, index) => {
            return (<ThemedView key={index} style={styles.deviceContainer}>
              <ThemedText>{device.name}</ThemedText>
            </ThemedView>)
          })
        }
      <ThemedView style={styles.titleContainer}>

      </ThemedView>
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
    gap: 8,
  },
  deviceContainer: {
    flexDirection: 'column',
    gap: 10,
  }
});
