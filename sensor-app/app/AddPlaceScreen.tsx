import BindedColorPicker from '@/components/BindedColorPicker';
import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { Button, StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useState } from 'react';

import { newUserPlace } from '@/api/place_crud';
import { PlaceColor } from '@/bindings/PlaceColor';
import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { useRouter } from 'expo-router';
import ErrorBox from '@/components/ui-elements/ErrorBox';

const placeColorValues: Record<PlaceColor, string> = {
    HEX_403E2A: '#403E2A',
    HEX_807895: '#807895',
    HEX_2A4039: '#2A4039',
    HEX_402E2A: '#402E2A',
    HEX_957E78: '#957E78',
    HEX_302A40: '#302A40',
    HEX_807E71: '#807E71',
    HEX_78958B: '#78958B',
    HEX_BFBA7A: '#BFBA7A',
    HEX_EA937D: '#EA937D',

    // HEX_DB2122: '#DB2122',
    // HEX_F0D16F: '#F0D16F',
    // HEX_21DB55: '#21DB55',
    // HEX_2132DB: '#2132DB',
    // HEX_6FF0D1: '#6FF0D1',
    // HEX_DB21A0: '#DB21A0',
    // HEX_DB8F21: '#DB8F21',
};

export default function AddPlaceScreen() {
    const ctx = useAppContext();
    const router = useRouter();

    const [placeName, setPlaceName] = useState<string | undefined>(undefined);
    const [placeDescription, setPlaceDescription] = useState<string | null>(null);
    const [placeColor, setPlaceColor] = useState<PlaceColor | undefined>(undefined);

    const [errorText, setErrorText] = useState<null | string>(null);

    const isAddable = placeName && placeColor;

    const redirectToIndex = () => {
        ctx.reloadSummary();
        router.replace('/');
    };

    const handleAdd = async () => {
        if (!ctx.sessionData?.all_set()) {
            console.warn('[AddPlaceScreen] sessionData was not set');
            setErrorText(
                'Your login data is incorrectly set somehow, please log out and try again!',
            );
            return;
        }

        if (!placeName) {
            setErrorText(
                'You should have set a place name before clicking the add button',
            );
            return; // TODO Better error return
        }

        if (!placeColor) {
            setErrorText('You should have set a color before clicking the add button');
            return; // TODO Better error return
        }

        const response = await newUserPlace(
            ctx.sessionData.username!,
            {
                id: ctx.sessionData.api_id!,
            },
            placeName,
            placeDescription,
            placeColor,
        );

        if (response && !(typeof response === 'number')) {
            redirectToIndex();
        } else {
            setErrorText(
                'There was an error when sending the place to the server, please log out and try again',
            );
        }
    };

    let formFields: FieldConfig[] = [
        {
            placeholder: 'Name',
            onChangeText: setPlaceName,
            value: placeName ? placeName : '',
        },
        {
            placeholder: 'Description (optional)',
            onChangeText: setPlaceDescription,
            value: placeDescription ? placeDescription : '',
        },
    ];
    return (
        <BackgroundView secondaryColor="#ffd9009b">
            <SafeAreaView>
                <ThemedView style={[styles.mainContainer]}>
                    <ThemedText style={TEXT_STYLES.heading1}>Add place</ThemedText>
                    <ThemedForm fields={formFields}></ThemedForm>

                    <BindedColorPicker
                        selectedColor={placeColor}
                        onColorChange={(color) => {
                            setPlaceColor(color as PlaceColor);
                        }}
                        colorValues={placeColorValues}
                    ></BindedColorPicker>
                    <ErrorBox error={errorText}></ErrorBox>

                    <Button
                        title="Add Place"
                        onPress={handleAdd}
                        disabled={!isAddable}
                    ></Button>
                </ThemedView>
            </SafeAreaView>
        </BackgroundView>
    );
}

const styles = StyleSheet.create({
    deviceContainer: {
        padding: 20,
        backgroundColor: '#d2ac00ff',
        borderRadius: 10,
        margin: 20,
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
    },
    mainContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        padding: 20,
        gap: 10,
    },
});
