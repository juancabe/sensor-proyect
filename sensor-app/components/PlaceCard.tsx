import { Redirect, useRouter } from 'expo-router';
import React, { useEffect, useMemo, useState } from 'react';
import { Platform, ScrollView, StyleSheet } from 'react-native';
import { useAppContext } from './AppProvider';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import useApi from '@/hooks/useApi';
import { GetSensor } from '@/bindings/api/endpoints/sensor/GetSensor';
import { GetSensorResponse } from '@/bindings/api/endpoints/sensor/GetSensorResponse';
import LabelValue from './ui-elements/LabelValue';
import ThemedButton from './ui-elements/ThemedButton';
import { Circle, ListPlus } from 'lucide-react-native';
import useLayerColor from '@/hooks/useLayerColor';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { Button } from '@/ui/components/Button';

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

    return (
        <Card
            variant="elevated"
            gap="s"
            style={{
                display: 'flex',
                flexDirection: 'column',
                borderColor: place.color,
                borderWidth: 2,
            }}
        >
            <Box flexDirection="row">
                <LabelValue label="Name" horizontal={true}>
                    <Text variant="heading">{place.name}</Text>
                </LabelValue>
                {Platform.OS !== 'web' && (
                    <ThemedButton
                        icon={ListPlus}
                        title="Add Sensor"
                        onPress={handleAddSensorPress}
                    />
                )}
            </Box>
            {place.description ? (
                <Text variant="caption">{place.description}</Text>
            ) : null}

            <Text variant="body">Sensors</Text>
            {api.response && (api.response as any).length ? (
                <Box flexDirection="column">
                    <ScrollView style={{}}>
                        {(api.response as GetSensorResponse[]).map(
                            ({ sensor, last_data }) => (
                                <Button
                                    variant="primary"
                                    key={sensor.device_id}
                                    label={sensor.name}
                                    icon={Circle}
                                    iconColor={sensor.color}
                                    onPress={() => {
                                        ctx.setActiveSensor({ sensor, data: last_data });
                                        router.push('/sensor_detail');
                                    }}
                                ></Button>
                            ),
                        )}
                    </ScrollView>
                </Box>
            ) : (
                <Card variant="elevated">
                    <Text variant="body">No sensors available</Text>
                </Card>
            )}
        </Card>
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
