import BindedColorPicker from '@/components/BindedColorPicker';
import { useAppContext } from '@/components/AppProvider';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { Button, StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useState } from 'react';

import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { useRouter } from 'expo-router';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import useApi, { errorText } from '@/hooks/useApi';
import { PostPlace } from '@/bindings/api/endpoints/place/PostPlace';
import { API_COLORS } from '@/constants/api_colors';

export default function AddPlaceScreen() {
    const ctx = useAppContext();
    const router = useRouter();

    const redirectToIndex = () => {
        ctx.reloadSummary();
        router.replace('/');
    };

    // TODO: Think about extracting these
    // -- API RELATED --
    const [name, setName] = useState<string>('');
    const [description, setDescription] = useState<string | null>(null);
    const [color, setColor] = useState<string>(API_COLORS[0]);
    const body: PostPlace = {
        name,
        description,
        color,
    };
    const [method, setMethod] = useState<'post' | undefined>(undefined);
    const postPlace = useApi('/place', body, method);

    if (postPlace.response) {
        // TODO: Add place to context
        redirectToIndex();
    }
    const [formErrorText, setFormErrorText] = useState<null | string>(null);
    const postErrorText = postPlace.error ? errorText(postPlace.error, true) : null;
    if (postErrorText) {
        setFormErrorText(postErrorText); // TODO: Different places
    }
    // -- API RELATED --

    const isAddable = name && color;

    const handleAdd = async () => {
        if (!name) {
            setFormErrorText(
                'You should have set a place name before clicking the add button',
            );
            return; // TODO Better error return
        }

        if (!color) {
            setFormErrorText(
                'You should have set a color before clicking the add button',
            );
            return; // TODO Better error return
        }

        setMethod('post');
    };

    let formFields: FieldConfig[] = [
        {
            placeholder: 'Name',
            onChangeText: setName,
            value: name ? name : '',
        },
        {
            placeholder: 'Description (optional)',
            onChangeText: setDescription,
            value: description ? description : '',
        },
    ];
    return (
        <BackgroundView secondaryColor="#ffd9009b">
            <SafeAreaView>
                <ThemedView style={[styles.mainContainer]}>
                    <ThemedText style={TEXT_STYLES.heading1}>Add place</ThemedText>
                    <ThemedForm fields={formFields}></ThemedForm>

                    <BindedColorPicker
                        selectedColor={color}
                        onColorChange={(color) => {
                            setColor(color);
                        }}
                        colorValues={API_COLORS}
                    ></BindedColorPicker>
                    <ErrorBox error={formErrorText}></ErrorBox>

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
