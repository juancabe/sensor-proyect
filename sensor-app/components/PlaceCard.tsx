import { useRouter } from 'expo-router';
import React, { useEffect, useMemo, useState } from 'react';
import { Button, Platform, ScrollView, StyleSheet, View } from 'react-native';
import { useAppContext } from './AppProvider';
import { TEXT_STYLES, ThemedText } from './ui-elements/ThemedText';
// import SensorCard from './SensorCard';
import { ThemedView } from './ui-elements/ThemedView';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import useApi from '@/hooks/useApi';
import { GetSensor } from '@/bindings/api/endpoints/sensor/GetSensor';
import { GetSensorResponse } from '@/bindings/api/endpoints/sensor/GetSensorResponse';
import SensorCard from './SensorCard';
import LabelValue from './ui-elements/LabelValue';
import ThemedButton from './ui-elements/ThemedButton';
import { ListPlus } from 'lucide-react-native';
import useLayerColor from '@/hooks/useLayerColor';

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
    const apiParams = useMemo(
        () => [['FromPlaceName', getSensor.FromPlaceName]],
        [getSensor.FromPlaceName],
    );
    const [apiMethod, setApiMethod] = useState<undefined | 'GET'>('GET');
    const api = useApi('/sensor', apiMethod, false, undefined, apiParams);

    useEffect(() => {
        if (api.response) {
            let res = api.response as GetSensorResponse[];
            for (const { sensor, last_data } of res) {
                console.log('sensor: ', sensor);
                console.log('last_data: ', last_data);
            }
        }
    }, [api.response]);

    const reloadApi = () => {
        setApiMethod(undefined);
        setTimeout(() => {
            // Using setTimeout so that it runs in next React cycle
            setApiMethod('GET');
        }, 0);
    };

    const layerBg = useLayerColor();

    return (
        <ThemedView
            style={[
                styles.container,
                { backgroundColor: place.color.replace('HEX_', '#') + '77' },
            ]}
        >
            <View style={styles.headerContainer}>
                <LabelValue label="Name" horizontal={true}>
                    <ThemedText style={TEXT_STYLES.heading3}>{place.name}</ThemedText>
                </LabelValue>
                {Platform.OS !== 'web' && (
                    <ThemedButton
                        icon={ListPlus}
                        title="Add Sensor"
                        onPress={handleAddSensorPress}
                    />
                )}
            </View>
            {place.description ? (
                <ThemedText style={TEXT_STYLES.label}>{place.description}</ThemedText>
            ) : null}

            <View
                style={[
                    styles.layer,
                    { backgroundColor: layerBg, width: '100%', padding: 5 },
                ]}
            >
                <ThemedText style={[TEXT_STYLES.heading2, { padding: 5 }]}>
                    Sensors
                </ThemedText>
                {api.response && (api.response as any).length ? (
                    <ScrollView style={{}}>
                        {(api.response as GetSensorResponse[]).map(
                            ({ sensor, last_data }) => (
                                <SensorCard
                                    key={sensor.device_id}
                                    sensor={sensor}
                                    data={last_data}
                                    reloadSensorSource={() => {
                                        reloadApi();
                                    }}
                                />
                            ),
                        )}
                    </ScrollView>
                ) : (
                    <View
                        style={{
                            display: 'flex',
                            flexDirection: 'row',
                            justifyContent: 'center',
                            padding: 10,
                            backgroundColor: layerBg,
                            borderRadius: 10,
                        }}
                    >
                        <ThemedText style={TEXT_STYLES.heading3}>
                            No sensors available
                        </ThemedText>
                    </View>
                )}
            </View>
        </ThemedView>
    );
}

const styles = StyleSheet.create({
    layer: {
        padding: 0,
        borderRadius: 10,
    },
    headerContainer: {
        flex: 1,
        flexDirection: 'row',
        gap: 20,
        justifyContent: 'space-between',
    },
    container: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        padding: 10,
        gap: 10,
        marginBottom: 20,
    },
});
