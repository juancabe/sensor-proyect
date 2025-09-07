import { useRouter } from 'expo-router';
import React, { useEffect } from 'react';
import { Button, StyleSheet, View } from 'react-native';
import { useAppContext } from './AppProvider';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
// import SensorCard from './SensorCard';
import { ThemedView } from './ui-elements/ThemedView';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import useApi from '@/hooks/useApi';
import { GetSensor } from '@/bindings/api/endpoints/sensor/GetSensor';
import { GetSensorResponse } from '@/bindings/api/endpoints/sensor/GetSensorResponse';
import SensorCard from './SensorCard';

export interface PlaceCardProps {
    place: ApiUserPlace;
}

export default function PlaceCard({ place }: PlaceCardProps) {
    const router = useRouter();
    const ctx = useAppContext();

    const handleAddSensorPress = () => {
        ctx.setActivePlace(place);
        router.navigate('/AddSensorScreen');
    };

    const getSensor: GetSensor = { 'FromPlaceName': place.name };

    const api = useApi(
        '/sensor?FromPlaceName=' + getSensor.FromPlaceName,
        undefined,
        'GET',
        false,
    );

    useEffect(() => {
        if (api.response) {
            let res = api.response as GetSensorResponse[];
            for (const { sensor, last_data } of res) {
                console.log('sensor: ', sensor);
                console.log('last_data: ', last_data);
            }
        }
    }, [api.response]);

    return (
        <ThemedView
            style={[
                styles.container,
                { backgroundColor: place.color.replace('HEX_', '#') + 'AA' },
            ]}
        >
            <ThemedText style={TEXT_STYLES.heading2}>{place.name}</ThemedText>
            <ThemedText style={TEXT_STYLES.label}>{place.description}</ThemedText>
            <Button title="Add Sensor" onPress={handleAddSensorPress} />
            <View
                style={{
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    justifyContent: 'center',
                }}
            >
                {api.response
                    ? (api.response as GetSensorResponse[]).map(
                          ({ sensor, last_data }) => (
                              <SensorCard
                                  key={sensor.device_id}
                                  sensor={sensor}
                                  data={last_data}
                              />
                          ),
                      )
                    : null}
            </View>
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    container: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        margin: 10,
        padding: 10,
        gap: 10,
    },
});
