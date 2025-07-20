import LoadingScreen from '@/components/LoadingScreen';
import ThemedForm, { FieldConfig } from '@/components/ThemedForm';
import { TEXT_STYLES, ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { useAuth } from '@/hooks/useAuth';
import { useSession } from '@/hooks/useSession';
import { useTheme } from '@react-navigation/native';
import { Redirect } from 'expo-router';
import { useState } from 'react';
import { Button } from 'react-native';
import { SafeAreaProvider, SafeAreaView } from 'react-native-safe-area-context';
import * as Crypto from 'expo-crypto';

export default function Login() {
  const { sessionData } = useSession();
  const theme = useTheme();
  const auth = useAuth();

  const [type, setType] = useState<'register' | 'login'>('login');
  const [username, setUsername] = useState<string>('');
  const [password, setPassword] = useState<string>('');
  const [repeatPassword, setRepeatPassword] = useState<string>('');
  const [email, setEmail] = useState<string>('');
  const [working, setWorking] = useState<boolean>(false);
  const [workingError, setWorkingError] = useState<string | null>(null);

  function isSubmissionDisabled(): boolean {
    if (working) return true;
    if (type === 'login') {
      return !username || !password;
    } else {
      return (
        !username || !email || !password || !repeatPassword || password !== repeatPassword
      );
    }
  }

  const loginFields: FieldConfig[] = [
    {
      placeholder: 'Username',
      value: username,
      onChangeText: setUsername,
    },
    {
      placeholder: 'Password',
      value: password,
      onChangeText: setPassword,
      secureTextEntry: true,
    },
  ];

  const registerFields: FieldConfig[] = [
    {
      placeholder: 'Username',
      value: username,
      onChangeText: setUsername,
    },
    {
      placeholder: 'Email',
      value: email,
      onChangeText: setEmail,
    },
    {
      placeholder: 'Password',
      value: password,
      onChangeText: setPassword,
      secureTextEntry: true,
    },
    {
      placeholder: 'Repeat Password',
      value: repeatPassword,
      onChangeText: setRepeatPassword,
      secureTextEntry: true,
    },
  ];

  const oppositeType = () => {
    if (type === 'login') {
      return 'register';
    } else {
      return 'login';
    }
  };

  async function hash_password(password: string): Promise<string> {
    return await Crypto.digestStringAsync(Crypto.CryptoDigestAlgorithm.SHA256, password);
  }

  const handleSubmission = async () => {
    if (working) return;
    else setWorking(true);

    const hashed_password = await hash_password(password);

    if (type === 'login') {
      console.log('called login');
      let res = await auth.login(username, hashed_password);
      console.log('returned login');
      if (res === 'Ok') {
        // Index should handle this
      } else {
        console.log('login error: ', res);
        setWorkingError(res);
      }
    }

    if (type === 'register') {
      let res = await auth.register(username, email, hashed_password);
      if (res === 'Ok') {
        // Index should handle this
      } else if (res == 'Connection Error') {
        setWorkingError(res);
      } else {
        switch (res) {
          case 'EmailUsed': {
            setWorkingError('The email you tried to use is already in use');
            break;
          }
          case 'UsernameUsed': {
            setWorkingError('The username you tried to use is already in use');
            break;
          }
          case 'HashInvalid': {
            setWorkingError('Unexpected Error happened');
            console.error('UNEXPECTED HASH INVALID');
            break;
          }
          default:
            throw Error('UNEXPECTED CASE'); // TODO: Get rid of this assertion
        }
      }
    }

    setWorking(false);
  };

  return (
    <SafeAreaView>
      <ThemedView
        style={{
          padding: 20,
          paddingTop: 60,
          paddingBottom: 60,
          alignContent: 'center',
          justifyContent: 'space-between',
          alignItems: 'center',
          height: '100%',
        }}
      >
        <ThemedText style={TEXT_STYLES.heading1}>Sensor App</ThemedText>
        <ThemedView style={{ width: '100%', gap: 30 }}>
          <ThemedForm fields={type === 'login' ? loginFields : registerFields} />
          <Button
            title={type}
            disabled={isSubmissionDisabled()}
            onPress={handleSubmission}
          />
        </ThemedView>
        <Button title={oppositeType()} onPress={() => setType(oppositeType())} />
      </ThemedView>
    </SafeAreaView>
  );
}
