import React from 'react';
import { ScrollView, type ScrollViewProps } from 'react-native';
import { useTheme, type Theme } from '@react-navigation/native';

export type ThemedScrollViewProps = ScrollViewProps & {
    theme?: Theme;
    style?: object;
    children?: React.ReactNode;
};

export function ThemedScrollView({
    theme,
    style,
    children,
    ...otherProps
}: ThemedScrollViewProps) {
    const hookedTheme = useTheme();
    const resolvedTheme = theme ?? hookedTheme;

    const backgroundColor = resolvedTheme?.colors?.background ?? 'transparent';

    return (
        <ScrollView
            style={[{ backgroundColor }, style]}
            contentContainerStyle={{ padding: 10 }}
            {...otherProps}
        >
            {children}
        </ScrollView>
    );
}

