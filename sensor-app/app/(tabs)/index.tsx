import { Image } from 'expo-image';
import { Button, Platform, StyleSheet, TextInput } from 'react-native';

import { HelloWave } from '@/components/HelloWave';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { useEffect, useState } from 'react';
import { useSession, SessionKeys } from '@/hooks/useSession';

export default function HomeScreen() {
  const { sessionData, setItem } = useSession();
  const [apiUserId, setApiUserId] = useState(
    sessionData[SessionKeys.API_USER_ID] || ''
  );
  const [sensorApiId, setSensorApiId] = useState(
    sessionData[SessionKeys.API_SENSOR_ID] || ''
  );

  const [apiUserIdSaved, setApiUserIdSaved] = useState(true);
  const [sensorApiIdSaved, setSensorApiIdSaved] = useState(true);

  const handleApiUserIdChange = (value: string) => {
    setApiUserIdSaved(false);
    setApiUserId(value);
  };

  const handleSensorApiIdChange = (value: string) => {
    setSensorApiIdSaved(false);
    setSensorApiId(value);
  };

  const handleApiUserIdSave = () => {
    setItem(SessionKeys.API_USER_ID, apiUserId);
    setApiUserIdSaved(true);
  };

  const handleSensorApiIdSave = () => {
    setItem(SessionKeys.API_SENSOR_ID, sensorApiId);
    setSensorApiIdSaved(true);
  };

  useEffect(() => {
    if (apiUserId !== sessionData[SessionKeys.API_USER_ID]) {
      setApiUserId(sessionData[SessionKeys.API_USER_ID] || '');
      setApiUserIdSaved(true);
    }
    if (sensorApiId !== sessionData[SessionKeys.API_SENSOR_ID]) {
      setSensorApiId(sessionData[SessionKeys.API_SENSOR_ID] || '');
      setSensorApiIdSaved(true);
    }
  }, [sessionData]);

  return (
    <ParallaxScrollView
      headerBackgroundColor={{ light: '#A1CEDC', dark: '#1D3D47' }}
      headerImage={
        <Image
          source={require('@/assets/images/partial-react-logo.png')}
          style={styles.reactLogo}
        />
      }
    >
      <ThemedView style={styles.titleContainer}>
        <ThemedText type="title">Hello User!</ThemedText>
        <HelloWave />
      </ThemedView>
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">User API Id</ThemedText>
        <ThemedView style={styles.inputContainer}>
          <ThemedText>
            <TextInput
              placeholder="Enter your API ID"
              value={apiUserId}
              style={styles.textInput}
              onChangeText={handleApiUserIdChange}
            />
          </ThemedText>
          <Button
            title={'Save'}
            disabled={apiUserIdSaved}
            onPress={handleApiUserIdSave}
          />
        </ThemedView>
      </ThemedView>
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Sensor API Id</ThemedText>
        <ThemedView style={styles.inputContainer}>
          <ThemedText>
            <TextInput
              placeholder="Enter your sensor API ID"
              value={sensorApiId}
              style={styles.textInput}
              onChangeText={handleSensorApiIdChange}
            />
          </ThemedText>
          {/* <Button
            title={'Save'}
            disabled={sensorApiIdSaved}
            onPress={handleSensorApiIdSave}
          /> */}
        </ThemedView>
      </ThemedView>
    </ParallaxScrollView>
  );
}

const styles = StyleSheet.create({
  titleContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  stepContainer: {
    gap: 8,
    marginBottom: 8,
  },
  reactLogo: {
    height: 178,
    width: 290,
    bottom: 0,
    left: 0,
    position: 'absolute',
  },
  textInput: {
    color: 'white',
    width: Platform.OS === 'web' ? 300 : 200,
    alignItems: 'center',
    height: 40,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: 8,
    marginBottom: 8,
  },
});
