import React from 'react';
import { StyleSheet } from 'react-native';
import { LinearGradient } from 'expo-linear-gradient';
import { useTheme } from '@react-navigation/native';

type BackgroundViewProps = {
    children: React.ReactNode;
    secondaryColor: string;
    style?: object;
};

export default function BackgroundView({
    children,
    secondaryColor,
    style,
}: BackgroundViewProps) {
    const theme = useTheme();
    return (
        <LinearGradient
            colors={[secondaryColor, theme.colors.background]}
            start={{ x: 0.9, y: 0.1 }}
            end={{ x: 0.9, y: 0.5 }}
            style={[StyleSheet.absoluteFill, { padding: 15 }, style]}
        >
            {children}
        </LinearGradient>
    );
}
