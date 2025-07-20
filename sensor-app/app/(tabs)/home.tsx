import BackgroundView from '@/components/BackgroundView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { useTheme } from '@react-navigation/native';
import { SafeAreaView, Text, View } from 'react-native';

export default function Home() {
  const theme = useTheme();

  return (
    <BackgroundView secondaryColor="#ff00003f">
      <ThemedView theme={theme} style={{ backgroundColor: 'transparent' }}>
        <ThemedText theme={theme}>
          Edit app/(home)/index.tsx to edit this screen.
        </ThemedText>
      </ThemedView>
    </BackgroundView>
  );
}
