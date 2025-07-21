import BackgroundView from '@/components/BackgroundView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { useSession } from '@/hooks/useSession';
import { useTheme } from '@react-navigation/native';
import { useRouter } from 'expo-router';
import { Button } from 'react-native';

export default function Account() {
  const theme = useTheme();
  const session = useSession();
  const router = useRouter();

  const handleLogout = () => {
    session.deleteSession();
    router.replace('/');
  };

  return (
    <BackgroundView secondaryColor="#007bff3f">
      <ThemedView style={{ backgroundColor: 'transparent' }}>
        <ThemedView
          style={{ display: 'flex', flexDirection: 'row', justifyContent: 'flex-end' }}
        >
          <Button title="Log Out" onPress={handleLogout} />
        </ThemedView>
        <ThemedText theme={theme}>
          Edit app/(tabs)/account.tsx to edit this screen.
        </ThemedText>
      </ThemedView>
    </BackgroundView>
  );
}
