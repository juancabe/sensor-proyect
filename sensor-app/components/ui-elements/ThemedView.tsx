import { View, type ViewProps } from 'react-native';
import { useTheme, type Theme } from '@react-navigation/native';

export type ThemedViewProps = ViewProps & {
    theme?: Theme;
    style?: object;
    children?: React.ReactNode;
};

export function ThemedView({ theme, style, children, ...otherProps }: ThemedViewProps) {
    let hookedTheme = useTheme();
    theme = theme ? theme : hookedTheme;

    // Use theme colors if provided, otherwise fallback to default
    const backgroundColor = theme ? theme.colors.background : undefined;

    return (
        <View style={[{ backgroundColor }, style]} {...otherProps}>
            {children}
        </View>
    );
}
