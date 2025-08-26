import { useTheme } from '@react-navigation/native';
import React from 'react';
import { StyleSheet, TextInput, View, type TextInputProps } from 'react-native';
import ErrorBox from './ErrorBox';
import { TEXT_STYLES } from './ThemedText';

export type FieldConfig = {
    placeholder: string;
    value: string;
    onChangeText: (text: string) => void;
    secureTextEntry?: boolean;
    inputProps?: Omit<
        TextInputProps,
        'placeholder' | 'value' | 'onChangeText' | 'secureTextEntry'
    >;
    error?: string;
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
                <View key={idx}>
                    <TextInput
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
                    <ErrorBox
                        error={f.error ? f.error : null}
                        style={{ padding: 5, borderWidth: 1, margin: 10, marginTop: 0 }}
                    />
                </View>
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
