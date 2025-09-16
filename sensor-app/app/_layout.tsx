import { AppProvider } from '@/components/AppProvider';
import { Stack } from 'expo-router';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { ThemeProvider, DefaultTheme, DarkTheme, Theme } from '@react-navigation/native';
import { useColorScheme } from 'react-native';

export default function RootLayout() {
    const cs = useColorScheme();

    const lightTheme: Theme = {
        ...DefaultTheme,
    };

    const darkTheme: Theme = {
        ...DarkTheme,
    };

    return (
        <AppProvider>
            <SafeAreaProvider>
                <ThemeProvider value={cs === 'light' ? lightTheme : darkTheme}>
                    <Stack>
                        <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
                        <Stack.Screen name="index" options={{ headerShown: false }} />
                        <Stack.Screen name="login" options={{ headerShown: false }} />
                        <Stack.Screen
                            name="AddSensorScreen"
                            options={{
                                title: 'Add your sensor',
                            }}
                        />
                        <Stack.Screen
                            name="AddPlaceScreen"
                            options={{
                                title: 'Add a place',
                            }}
                        />
                        <Stack.Screen
                            name="SensorDetail"
                            options={{
                                headerShown: false,
                            }}
                        />
                    </Stack>
                </ThemeProvider>
            </SafeAreaProvider>
        </AppProvider>
    );
}
