import { useRouter } from 'expo-router';
import React, { useMemo } from 'react';
import { Platform, ScrollView } from 'react-native';
import { useAppContext } from './AppProvider';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import useApi from '@/hooks/useApi';
import { GetSensor } from '@/bindings/api/endpoints/sensor/GetSensor';
import { GetSensorResponse } from '@/bindings/api/endpoints/sensor/GetSensorResponse';
import LabelValue from './ui-elements/LabelValue';
import { Circle, ListPlus } from 'lucide-react-native';
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
    const api = useApi('/sensor', 'GET', false, undefined, apiParams);

    return (
        <Card
            variant="elevated"
            gap="l"
            style={{
                display: 'flex',
                flexDirection: 'column',
                borderColor: place.color,
                borderWidth: 2,
            }}
        >
            <Box flexDirection="column" gap="m">
                <LabelValue label="Name" horizontal={true}>
                    <Text variant="heading">{place.name}</Text>
                </LabelValue>
                {Platform.OS !== 'web' && (
                    <Button
                        icon={ListPlus}
                        label="Add Sensor"
                        onPress={handleAddSensorPress}
                    />
                )}
                {place.description ? (
                    <Text variant="caption">{place.description}</Text>
                ) : null}
            </Box>

            <Box gap="m">
                <Text variant="body">Sensors</Text>
                {api.response && (api.response as any).length ? (
                    <Box flexDirection="column">
                        <ScrollView>
                            {(api.response as GetSensorResponse[]).map(
                                ({ sensor, last_data }) => (
                                    <Box key={sensor.device_id} margin="xs">
                                        <Button
                                            variant="primary"
                                            label={sensor.name}
                                            icon={Circle}
                                            iconColor={sensor.color}
                                            onPress={() => {
                                                ctx.setActiveSensor(sensor);
                                                ctx.setActiveSensorData(last_data);
                                                router.push('/sensor_detail');
                                            }}
                                        ></Button>
                                    </Box>
                                ),
                            )}
                        </ScrollView>
                    </Box>
                ) : (
                    <Card variant="elevated">
                        <Text variant="body" color="warning">
                            No sensors added
                        </Text>
                    </Card>
                )}
            </Box>
        </Card>
    );
}
