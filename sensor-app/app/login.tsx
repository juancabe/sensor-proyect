import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { useState } from 'react';
import { Button, Keyboard } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
// import * as auth from '@/helpers/auth';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import useApi from '@/hooks/useApi';
import useRedirect from '@/hooks/useRedirect';
import { useApiUsername } from '@/hooks/api/useApiUsername';
import { useApiRawPassword } from '@/hooks/api/useApiRawPassword';
import { useApiEmail } from '@/hooks/api/useApiEmail';
import { PostUser } from '@/bindings/api/endpoints/user/PostUser';
import { PostSession } from '@/bindings/api/endpoints/session/PostSession';

export default function Login() {
    const redirect = useRedirect();

    const [type, setType] = useState<'register' | 'login'>('login');

    const username = useApiUsername();
    const password = useApiRawPassword();
    const repeatPassword = useApiRawPassword();
    const email = useApiEmail();

    const registerBody: PostUser = {
        username: username.username,
        raw_password: password.password,
        email: email.email,
    };

    const loginBody: PostSession = {
        username: username.username,
        raw_password: password.password,
    };

    const [method, setMethod] = useState<'POST' | undefined>(undefined);
    const api = useApi(
        type === 'register' ? '/users' : `/session`,
        type === 'register' ? registerBody : loginBody,
        method,
    );

    if (api.response) {
        redirect.redirectToIndex();
    }

    function isSubmissionDisabled(): boolean {
        if (api.loading) return true;

        const isLoginValid = username.isValid && password.isValid;
        const isRegisterValid =
            isLoginValid &&
            repeatPassword.isValid &&
            email.isValid &&
            password.password === repeatPassword.password;

        return type === 'login' ? !isLoginValid : !isRegisterValid;
    }

    const oppositeType = () => {
        if (type === 'login') {
            return 'register';
        } else {
            return 'login';
        }
    };

    const handleSubmission = async () => {
        Keyboard.dismiss();
        setMethod('POST');
    };

    const loginFields: FieldConfig[] = [
        {
            placeholder: 'Username',
            value: username.username,
            onChangeText: username.setUsername,
        },
        {
            placeholder: 'Password',
            value: password.password,
            onChangeText: password.setPassword,
            secureTextEntry: true,
        },
    ];

    const registerFields: FieldConfig[] = [
        {
            placeholder: 'Email',
            value: email.email,
            onChangeText: email.setEmail,
        },
        ...loginFields,
        {
            placeholder: 'Repeat Password',
            value: repeatPassword.password,
            onChangeText: repeatPassword.setPassword,
            secureTextEntry: true,
        },
    ];

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
                    <ThemedForm
                        fields={type === 'login' ? loginFields : registerFields}
                    />
                    <Button
                        title={type}
                        disabled={isSubmissionDisabled()}
                        onPress={handleSubmission}
                    />
                </ThemedView>
                <ErrorBox error={api.formattedError}></ErrorBox>
                <Button title={oppositeType()} onPress={() => setType(oppositeType())} />
            </ThemedView>
        </SafeAreaView>
    );
}
