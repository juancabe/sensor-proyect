import { AppProvider } from '@/components/AppProvider';
import { Stack } from 'expo-router';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { ThemeProvider, DefaultTheme, DarkTheme } from '@react-navigation/native';
import { useColorScheme } from 'react-native';

export default function RootLayout() {
    const cs = useColorScheme();

    return (
        <AppProvider>
            <SafeAreaProvider>
                <ThemeProvider value={cs === 'light' ? DefaultTheme : DarkTheme}>
                    <Stack>
                        <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
                        <Stack.Screen name="index" options={{ headerShown: false }} />
                        <Stack.Screen name="login" options={{ headerShown: false }} />
                        <Stack.Screen
                            name="AddSensorScreen"
                            options={{ headerShown: false }}
                        />
                    </Stack>
                </ThemeProvider>
            </SafeAreaProvider>
        </AppProvider>
    );
}
