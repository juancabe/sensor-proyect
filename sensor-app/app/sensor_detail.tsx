import React, { useMemo, useState } from 'react';
import { Redirect } from 'expo-router';
import { timeDisplay } from '@/helpers/timeDisplay';
import { objectNumberKeysToArray } from '@/helpers/objectWork';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { useAppContext } from '@/components/AppProvider';
import SensorCrudModal from '@/components/SensorCrudModal';
import LabelValue from '@/components/ui-elements/LabelValue';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { useSafeAreaFrame } from 'react-native-safe-area-context';
import { Button } from '@/ui/components/Button';
import { Pen, RefreshCcw } from 'lucide-react-native';
import DataChart, { calculateParams, DataChartProps } from '@/components/DataChart';
import useSensorDataApi from '@/hooks/useSensorDataApi';
import { ScrollView } from 'react-native';
import LoadingScreen from '@/components/LoadingScreen';

export default function SensorCard() {
    const ctx = useAppContext();
    const sensor = ctx.activeSensor;
    const globalData = ctx.activeSensorData;
    const frame = useSafeAreaFrame();

    const MIN30 = 30 * 60 * 1000;
    const {
        reload,
        data,
        lastData: hookedLastData,
    } = useSensorDataApi(MIN30, globalData ? sensor : undefined);
    const chartProps: { chartProps: DataChartProps; key: string }[] | undefined =
        useMemo(() => {
            if (!data) {
                return undefined;
            }
            const chartProps: { chartProps: DataChartProps; key: string }[] = [];
            for (const datum of data.data) {
                const params = calculateParams(datum, frame.width * 0.67);

                chartProps.push({
                    key: datum.key,
                    chartProps: {
                        data: datum,
                        params,
                    },
                });
            }
            return chartProps;
        }, [data, frame.width]);

    const lastData = useMemo(() => {
        if (hookedLastData) {
            return objectNumberKeysToArray(hookedLastData[0]);
        } else if (globalData) {
            const parsed = JSON.parse(JSON.parse(globalData.data));
            return objectNumberKeysToArray(parsed);
        } else {
            return undefined;
        }
    }, [globalData, hookedLastData]);
    const lastDataAddedAt = useMemo(() => {
        if (hookedLastData) {
            return new Date(hookedLastData[1] * 1000);
        } else if (globalData) {
            return new Date(globalData.added_at * 1000);
        } else {
            return undefined;
        }
    }, [globalData, hookedLastData]);

    // const backgroundColor = props.sensor.color.replace('HEX_', '#') + '99';
    const [crudModalDisplayed, setCrudModalDisplayed] = useState<boolean>(false);

    if (!sensor) {
        return <Redirect href={'/home'} />;
    }

    return (
        <BackgroundView>
            {crudModalDisplayed && (
                <SensorCrudModal
                    gotoSensorSource={() => {
                        console.warn('TODO: reload x place when needed, ctx export');
                    }}
                    sensor={sensor}
                    displayed={crudModalDisplayed}
                    setDisplayed={setCrudModalDisplayed}
                    setSensor={ctx.setActiveSensor}
                ></SensorCrudModal>
            )}
            <ScrollView>
                <Box flex={1} justifyContent="space-between">
                    <Box
                        flexDirection="row"
                        flexWrap="wrap"
                        gap="m"
                        justifyContent="space-around"
                        padding="l"
                    >
                        <Card
                            variant="elevated"
                            flexDirection="column"
                            gap="l"
                            style={{ maxWidth: frame.width * 0.6 }}
                        >
                            <LabelValue label="Sensor name" horizontal={true}>
                                <Text variant="subTitle">{sensor.name}</Text>
                            </LabelValue>
                            <LabelValue label="Last sensor change">
                                <Text
                                    variant="body"
                                    color="mainText"
                                    style={{ flexWrap: 'wrap' }}
                                >
                                    {timeDisplay(new Date(sensor.updated_at * 1000))}
                                </Text>
                            </LabelValue>
                        </Card>
                        <Button
                            iconPosition="up"
                            icon={RefreshCcw}
                            iconSize={40}
                            label="Reload"
                            onPress={() => {
                                console.log('reload called');
                                reload();
                            }}
                            style={{ maxWidth: frame.width * 0.2 }}
                        ></Button>

                        <Button
                            variant="warning"
                            iconPosition="up"
                            icon={Pen}
                            iconSize={40}
                            label="Edit"
                            onPress={() => {
                                setCrudModalDisplayed(true);
                            }}
                            style={{ maxWidth: frame.width * 0.2 }}
                        ></Button>
                        {lastData && lastDataAddedAt ? (
                            <Card
                                variant="elevated"
                                gap="m"
                                justifyContent="space-between"
                                flexDirection="column"
                                style={{ maxWidth: frame.width * 0.6 }}
                                alignItems="center"
                            >
                                <Text variant="heading">Last data</Text>
                                <Text variant="body">{timeDisplay(lastDataAddedAt)}</Text>
                                <Box
                                    gap="m"
                                    flexDirection="row"
                                    flexWrap="wrap"
                                    justifyContent="center"
                                >
                                    {lastData.map(([label, value]) => (
                                        <Box key={value} flexDirection="row">
                                            <LabelValue label={label} horizontal>
                                                <Text variant="body" color="mainText">
                                                    {value}
                                                </Text>
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
                    {lastData && (
                        <Box
                            flexDirection="row"
                            flexWrap="wrap"
                            gap="m"
                            justifyContent="space-around"
                            padding="l"
                        >
                            {chartProps ? (
                                chartProps.map(({ chartProps, key }) => {
                                    return (
                                        <Card key={key} variant="elevated" gap="m">
                                            <Text variant="heading">{key}</Text>
                                            <DataChart
                                                data={chartProps.data}
                                                params={chartProps.params}
                                            />
                                        </Card>
                                    );
                                })
                            ) : (
                                <LoadingScreen />
                            )}
                        </Box>
                    )}
                </Box>
            </ScrollView>
        </BackgroundView>
    );
}
