import ThemedBackgroundView from '@/components/ui-elements/BackgroundView';
import PlacesScreen from '@/components/PlacesScreen';
import { useMemo, useState } from 'react';
import useApi from '@/hooks/useApi';
import { ApiUserPlace } from '@/bindings/api/endpoints/place/ApiUserPlace';
import { Redirect } from 'expo-router';
import ErrorBox from '@/components/ui-elements/ErrorBox';

export default function Home() {
    const [method, setMethod] = useState<'GET' | undefined>('GET');
    const queryParams = useMemo<[string, string][]>(() => [['kind', 'UserPlaces']], []);
    const placeApi = useApi<undefined, ApiUserPlace[], undefined>(
        '/place',
        method,
        false,
        undefined,
        queryParams,
    );

    function reloadApi() {
        setMethod(undefined);
        setTimeout(() => {
            // Using setTimeout so that it runs in next React cycle
            setMethod('GET');
        }, 0);
    }

    const isLoading = placeApi.loading || (!placeApi.response && !placeApi.error);

    if (placeApi.error?.error?.status === 401) {
        return <Redirect href={'/login'} />;
    } else {
        console.log('not 401', placeApi.error?.error?.status);
    }

    return (
        <ThemedBackgroundView>
            <PlacesScreen
                places={placeApi.response}
                reloadPlaces={reloadApi}
                isLoading={isLoading}
            />
            {placeApi.formattedError && <ErrorBox error={placeApi.formattedError} />}
        </ThemedBackgroundView>
    );
}
