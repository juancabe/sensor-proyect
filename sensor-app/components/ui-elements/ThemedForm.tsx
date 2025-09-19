import React from 'react';
import { StyleSheet, type TextInputProps } from 'react-native';
import { TextInput } from 'react-native';
import ErrorBox from './ErrorBox';
import { Box, Theme } from '@/ui/theme';
import { useTheme } from '@shopify/restyle';

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

export type FormProps = {
    fields: FieldConfig[];
    style?: object;
};

export default function Form({ fields, style }: FormProps) {
    const theme = useTheme<Theme>();

    console.log('theme colors text: ', theme.colors.text);

    console.log(
        'theme.colors.cardBackground,:',
        theme.colors.cardBackground,
        'theme.colors.mainText,:',
        theme.colors.mainText,
        'theme.colors.borderOfCard,:',
        theme.colors.borderOfCard,
        'theme.textVariants.body.lineHeight,:',
        theme.textVariants.body.lineHeight,
        'theme.spacing.m,:',
        theme.spacing.m,
        'theme.cardVariants.elevated.borderWidth,:',
        theme.cardVariants.elevated.borderWidth,
        'theme.borderRadii.m,:',
        theme.borderRadii.m,
        'theme.textVariants.body.fontSize,:',
        theme.textVariants.body.fontSize,
    );

    return (
        <Box gap="m" style={[style]}>
            {fields.map((f, idx) => (
                <Box key={idx} flexDirection="column" gap="s">
                    <TextInput
                        style={[
                            {
                                backgroundColor: theme.colors.cardBackground,
                                color: theme.colors.mainText,
                                borderColor: theme.colors.borderOfCard,
                                paddingHorizontal: theme.spacing.m,
                                paddingVertical: theme.spacing.s,
                                borderWidth: theme.cardVariants.elevated.borderWidth,
                                borderRadius: theme.borderRadii.m,
                                fontSize: theme.textVariants.body.fontSize,
                            },
                        ]}
                        placeholder={f.placeholder}
                        placeholderTextColor={theme.colors.text}
                        value={f.value}
                        onChangeText={f.onChangeText}
                        secureTextEntry={f.secureTextEntry}
                        autoCapitalize="none"
                        {...f.inputProps}
                    />
                    {f.error ? (
                        <ErrorBox
                            error={f.error}
                            style={{
                                padding: 5,
                                borderWidth: 1,
                                margin: 10,
                                marginTop: 0,
                            }}
                        />
                    ) : null}
                </Box>
            ))}
        </Box>
    );
}
