import LoadingScreen from '@/components/LoadingScreen';
import { useSession } from '@/hooks/useSession';
import { Redirect } from 'expo-router';
import Login from './login';

export default function Index() {
  const { sessionData } = useSession();

  if (sessionData === undefined) {
    return <LoadingScreen />;
  }
  if (sessionData === null) {
    return <Login />;
  }
  return <Redirect href="/(tabs)/home" />;
}
