import { useTheme } from '@react-navigation/native';
import React from 'react';
import { StyleSheet, TextInput, View, type TextInputProps } from 'react-native';

export type FieldConfig = {
    placeholder: string;
    value: string;
    onChangeText: (text: string) => void;
    secureTextEntry?: boolean;
    inputProps?: Omit<
        TextInputProps,
        'placeholder' | 'value' | 'onChangeText' | 'secureTextEntry'
    >;
};

export type ThemedFormProps = {
    fields: FieldConfig[];
    style?: object;
};

export default function ThemedForm({ fields, style }: ThemedFormProps) {
    const theme = useTheme();
    const { colors } = theme;

    return (
        <View style={[styles.container, style]}>
            {fields.map((f, idx) => (
                <TextInput
                    key={idx}
                    style={[
                        styles.input,
                        {
                            backgroundColor: colors.card,
                            color: colors.text,
                            borderColor: colors.border,
                        },
                    ]}
                    placeholder={f.placeholder}
                    placeholderTextColor={colors.text + '99'}
                    value={f.value}
                    onChangeText={f.onChangeText}
                    secureTextEntry={f.secureTextEntry}
                    autoCapitalize="none"
                    {...f.inputProps}
                />
            ))}
        </View>
    );
}

const styles = StyleSheet.create({
    container: {
        width: '100%',
    },
    input: {
        height: 48,
        paddingHorizontal: 12,
        marginVertical: 8,
        borderWidth: 1,
        borderRadius: 6,
        fontSize: 16,
    },
});
