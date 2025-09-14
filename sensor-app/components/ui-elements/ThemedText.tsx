import React from 'react';
import { StyleSheet, Text, type TextProps } from 'react-native';
import { useTheme, type Theme } from '@react-navigation/native';

export type ThemedTextProps = TextProps & {
    theme?: Theme;
    style?: object;
    children?: React.ReactNode;
};

export function ThemedText({ theme, style, children, ...otherProps }: ThemedTextProps) {
    // Use provided theme or fallback to current app theme
    const appTheme = useTheme();
    const activeTheme = theme ?? appTheme;
    const color = activeTheme.colors.text;

    return (
        <Text style={[{ color }, style]} {...otherProps}>
            {children}
        </Text>
    );
}

export const TEXT_STYLES = StyleSheet.create({
    heading1: {
        fontSize: 32,
        fontWeight: '700',
        lineHeight: 40,
    },
    heading2: {
        fontSize: 24,
        fontWeight: '600',
        lineHeight: 32,
    },
    heading3: {
        fontSize: 18,
        fontWeight: '500',
        lineHeight: 26,
    },
    body: {
        fontSize: 16,
        fontWeight: '400',
        lineHeight: 24,
    },
    label: {
        fontSize: 14,
        fontWeight: '500',
        lineHeight: 20,
    },
    caption: {
        fontSize: 12,
        fontWeight: '300',
        lineHeight: 16,
    },
});
