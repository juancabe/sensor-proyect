import BindedColorPicker from '@/components/BindedColorPicker';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useEffect, useMemo, useState } from 'react';

import Form, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import useApi from '@/hooks/useApi';
import { PostPlace } from '@/bindings/api/endpoints/place/PostPlace';
import { useApiEntityName } from '@/hooks/api/useApiEntityName';
import { useApiDescription } from '@/hooks/api/useApiDescription';
import { useApiColor } from '@/hooks/api/useApiColor';
import { Redirect } from 'expo-router';
import { Box, Text } from '@/ui/theme';
import { Button } from '@/ui/components/Button';
import { Card } from '@/ui/components/Card';
import LabelValue from '@/components/ui-elements/LabelValue';

const secondaryColor = '#ffd9009b';

export default function AddPlaceScreen() {
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

    if (postPlace.returnedOk) {
        return <Redirect href={'/home'} />;
    }

    return (
        <BackgroundView>
            <Box>
                <Card
                    variant="elevated"
                    flexDirection="column"
                    alignItems="center"
                    gap="l"
                >
                    <Text variant="heading">Add place</Text>
                    <Form fields={formFields}></Form>
                    <LabelValue label="Representative color">
                        <BindedColorPicker
                            selectedColor={color.color}
                            onColorChange={(new_color) => {
                                color.setColor(new_color);
                            }}
                            colorValues={color.API_COLORS}
                        ></BindedColorPicker>
                    </LabelValue>
                    {postPlace.formattedError && (
                        <ErrorBox error={postPlace.formattedError}></ErrorBox>
                    )}
                    <Button
                        variant="positive"
                        label="Add Place"
                        onPress={handleAdd}
                        disabled={!isAddable}
                    ></Button>
                </Card>
            </Box>
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
