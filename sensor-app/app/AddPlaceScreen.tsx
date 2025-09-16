import BindedColorPicker from '@/components/BindedColorPicker';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { Button, StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useEffect, useMemo, useState } from 'react';

import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import useApi from '@/hooks/useApi';
import { PostPlace } from '@/bindings/api/endpoints/place/PostPlace';
import useRedirect from '@/hooks/useRedirect';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { useApiDescription } from '@/hooks/api/useApiDescription';
import { useApiColor } from '@/hooks/api/useApiColor';

const secondaryColor = '#ffd9009b';

export default function AddPlaceScreen() {
    const { redirectToIndex } = useRedirect();

    // -- API RELATED --
    const name = useApiEntityName();
    const description = useApiDescription();
    const color = useApiColor();
    const isAddable = name.isValid && description.isValid && color;

    const body = useMemo(() => {
        const body: PostPlace = {
            name: name.name,
            description: description.description,
            color: color.color,
        };

        return body;
    }, [name.name, description.description, color.color]);

    const [method, setMethod] = useState<'POST' | undefined>(undefined);
    const postPlace = useApi('/place', method, false, body);

    useEffect(() => {
        if (postPlace.returnedOk) {
            redirectToIndex();
        }
    }, [postPlace.returnedOk, redirectToIndex]);

    useEffect(() => {
        if (postPlace.error) {
            setMethod(undefined);
        }
    }, [postPlace.error]);

    const handleAdd = async () => {
        setMethod('POST');
    };

    let formFields: FieldConfig[] = [
        {
            placeholder: 'Name',
            onChangeText: name.setName,
            value: name.name,
            error: name.error,
        },
        {
            placeholder: 'Description (optional)',
            onChangeText: description.setDescription,
            value: description.description ? description.description : '',
            error: description.error,
        },
    ];
    return (
        <BackgroundView secondaryColor={secondaryColor}>
            <SafeAreaView>
                <ThemedView style={[styles.mainContainer]}>
                    <ThemedText style={TEXT_STYLES.heading1}>Add place</ThemedText>
                    <ThemedForm fields={formFields}></ThemedForm>
                    <BindedColorPicker
                        selectedColor={color.color}
                        onColorChange={(new_color) => {
                            color.setColor(new_color);
                        }}
                        colorValues={color.API_COLORS}
                    ></BindedColorPicker>
                    <ErrorBox error={postPlace.formattedError}></ErrorBox>
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
    mainContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 10,
        padding: 20,
        gap: 10,
        borderWidth: 3,
        borderColor: secondaryColor,
    },
});
