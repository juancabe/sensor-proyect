import React, { useMemo, useState } from 'react';
import { StyleSheet } from 'react-native';
import { Redirect, useRouter } from 'expo-router';
import { timeDisplay } from '@/helpers/timeDisplay';
import { safeGet } from '@/helpers/objectWork';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { useAppContext } from '@/components/AppProvider';
import SensorCrudModal from '@/components/SensorCrudModal';
import LabelValue from '@/components/ui-elements/LabelValue';
import { Button } from '@/ui/components/Button';
import { Eye } from 'lucide-react-native';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { BoxV } from '@/ui/components/BoxV';
import { useFrameSize } from '@react-navigation/elements';
import { useSafeAreaFrame } from 'react-native-safe-area-context';

export default function SensorCard() {
    const ctx = useAppContext();
    const sensor = ctx.activeSensor;
    const data = ctx.activeSensorData;
    const router = useRouter();
    const frame = useSafeAreaFrame();

    const lastData: [string, string][] | undefined = (() => {
        if (!data) return undefined;

        const parsed = JSON.parse(JSON.parse(data.data));
        const numberKeys = Object.entries(parsed)
            .filter(([_, v]) => typeof v === 'number')
            .map(([k]) => k);

        return numberKeys.map((key) => [key, safeGet(parsed, key)]);
    })();

    // const backgroundColor = props.sensor.color.replace('HEX_', '#') + '99';
    const [crudModalDisplayed, setCrudModalDisplayed] = useState<boolean>(false);

    if (!sensor || !data) {
        return <Redirect href={'/home'} />;
    }

    return (
        <BackgroundView>
            <SensorCrudModal
                reloadSensorSource={() => {
                    console.warn('TODO: reload x place when needed, ctx export');
                }}
                sensor={sensor}
                displayed={crudModalDisplayed}
                setDisplayed={setCrudModalDisplayed}
            ></SensorCrudModal>
            <Box flex={1} justifyContent="space-between">
                <Box flexDirection="row" flexWrap="wrap" gap="l">
                    <Card variant="elevated" flexDirection="column" gap="l">
                        <LabelValue label="Sensor name" horizontal={true}>
                            <Text variant="subTitle">{sensor.name}</Text>
                        </LabelValue>
                        <LabelValue label="Last sensor change">
                            <Text variant="caption">
                                {timeDisplay(new Date(sensor.updated_at * 1000))}
                            </Text>
                        </LabelValue>
                    </Card>
                    {lastData ? (
                        <Card
                            variant="elevated"
                            gap="m"
                            justifyContent="space-between"
                            flexDirection="column"
                            style={{ maxWidth: frame.width * 0.9 }}
                        >
                            <Text variant="heading">Last data</Text>
                            <Text variant="body">
                                {timeDisplay(new Date(data.added_at * 1000))}
                            </Text>
                            <Box gap="m" flexDirection="row" flexWrap="wrap">
                                {lastData.map(([label, value]) => (
                                    <Box key={value} flexDirection="row">
                                        <LabelValue label={label} horizontal>
                                            <Text variant="body">{value}</Text>
                                        </LabelValue>
                                    </Box>
                                ))}
                            </Box>

                            {/* <Button */}
                            {/*     label="More detail" */}
                            {/*     icon={Eye} */}
                            {/*     onPress={() => router.navigate('/SensorData')} */}
                            {/* ></Button> */}
                        </Card>
                    ) : null}
                </Box>
            </Box>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    modalLayer: {
        padding: 10,
        margin: 5,
        borderRadius: 10,
        gap: 10,
    },
    crudModal: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        gap: 20,
        borderRadius: 10,
        padding: 15,
        width: '80%',
    },
    value: {
        backgroundColor: '#00000040',
        padding: 10,
        borderRadius: 10,
    },
    sensorName: {
        padding: 10,
        borderRadius: 10,
    },
    mainContainer: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        padding: 10,
        gap: 10,
        width: '100%',
    },
});
