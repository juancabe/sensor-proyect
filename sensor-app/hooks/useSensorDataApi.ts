import { ApiUserSensor } from '@/bindings/api/endpoints/sensor/ApiUserSensor';
import { GetSensorData } from '@/bindings/api/endpoints/sensor_data/GetSensorData';
import { SensorDataLoader } from '@/helpers/SensorDataLoader';
import { useEffect, useMemo, useState } from 'react';
import useApi from './useApi';
import { ApiSensorData } from '@/bindings/api/endpoints/sensor_data/ApiSensorData';
import { DataShape } from '@/model/ShapedData';

function fabGetSensorData(sensor: ApiUserSensor, offsetMillis: number): GetSensorData {
    return {
        device_id: sensor.device_id,
        lowest_added_at: ~~((Date.now() - offsetMillis) / 1000),
        upper_added_at: null,
    };
}

const computeParams = (apiBody: GetSensorData | undefined) => {
    if (!apiBody) return;
    let params_arr = [['device_id', apiBody.device_id]];
    console.log('apiBody.lowest_added_at', apiBody.lowest_added_at);
    if (apiBody.lowest_added_at) {
        params_arr.push(['lowest_added_at', '' + apiBody.lowest_added_at]);
    }
    if (apiBody.upper_added_at) {
        params_arr.push(['upper_added_at', '' + apiBody.upper_added_at]);
    }
    return params_arr;
};

export default function useSensorDataApi(
    offsetMillis: number,
    sensor?: ApiUserSensor,
    maxLabels: number = 4,
) {
    const [dataLoader, setDataLoader] = useState<SensorDataLoader | undefined>(undefined);
    const data = useMemo(() => {
        return dataLoader?.getData();
    }, [dataLoader]);
    const lastData = useMemo(() => dataLoader?.getLastData(), [dataLoader]);
    const availableKeys = useMemo(() => dataLoader?.getKeys(), [dataLoader]);

    const defaultApiBody: GetSensorData | undefined = sensor
        ? fabGetSensorData(sensor, offsetMillis)
        : undefined;
    const [apiBody, setApiBody] = useState<GetSensorData | undefined>(defaultApiBody);
    useEffect(() => {
        if (!sensor) return;
        setApiBody((prev) =>
            prev
                ? {
                      ...prev,
                      lowest_added_at: ~~((Date.now() - offsetMillis) / 1000),
                  }
                : fabGetSensorData(sensor, offsetMillis),
        );
    }, [sensor, offsetMillis]);

    const apiParams = useMemo(() => computeParams(apiBody), [apiBody]);

    const [apiMethod, setApiMethod] = useState<'GET' | undefined>(
        sensor ? 'GET' : undefined,
    );
    const api = useApi('/sensor_data', apiMethod, false, undefined, apiParams);

    const reload = () => {
        setApiMethod(undefined);
        setTimeout(() => {
            setApiMethod('GET');
        }, 0);
    };

    // work with received data
    useEffect(() => {
        if (!(api.returnedOk && api.response)) return;

        let cancelled = false;
        const load = async () => {
            const data = api.response as ApiSensorData[];
            const dataShape: DataShape = { maxLabels, maxPoints: 1000 };
            const loader = await SensorDataLoader.load(data, dataShape);
            if (!cancelled) {
                setDataLoader(loader);
                console.warn('availableKeys: ', loader.getKeys());
            }
        };

        load();
        return () => {
            cancelled = true;
        };
    }, [api.response, api.returnedOk, maxLabels]);

    return { reload, availableKeys, data, lastData };
}
