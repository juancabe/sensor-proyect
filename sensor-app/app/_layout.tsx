import { AppProvider } from '@/components/AppProvider';
import { Stack } from 'expo-router';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { useColorScheme } from 'react-native';
import { useMemo } from 'react';
import { makeRestyleTheme, Theme } from '@/ui/theme';
import { ThemeProvider } from '@shopify/restyle';

export default function RootLayout() {
    const cs = useColorScheme();

    const theme = useMemo(() => makeRestyleTheme(cs ? cs : 'dark'), [cs]) as Theme;
    return (
        <AppProvider>
            <SafeAreaProvider>
                <ThemeProvider theme={theme}>
                    <Stack>
                        <Stack.Screen name="index" options={{ headerShown: false }} />
                        <Stack.Screen name="login" options={{ headerShown: false }} />
                        <Stack.Screen name="home" options={{ headerShown: false }} />
                        <Stack.Screen
                            name="sensor_detail"
                            options={{
                                headerShown: false,
                            }}
                        />

                        <Stack.Screen
                            name="AddSensorScreen"
                            options={{
                                title: 'Add your sensor',
                                headerShown: false,
                            }}
                        />
                        <Stack.Screen
                            name="AddPlaceScreen"
                            options={{
                                title: 'Add a place',
                                headerShown: false,
                            }}
                        />
                    </Stack>
                </ThemeProvider>
            </SafeAreaProvider>
        </AppProvider>
    );
}
