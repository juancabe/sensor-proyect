import ThemedForm, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { TEXT_STYLES, ThemedText } from '@/components/ui-elements/ThemedText';
import { ThemedView } from '@/components/ui-elements/ThemedView';
import { useEffect, useState } from 'react';
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
import { useAppContext } from '@/components/AppProvider';

export default function Login() {
    const redirect = useRedirect();
    const ctx = useAppContext();

    const [type, setType] = useState<'register' | 'login'>('login');

    const username = useApiUsername();
    const password = useApiRawPassword();
    const repeatPassword = useApiRawPassword();
    const email = useApiEmail();

    const [holeFormError, setHoleFormError] = useState<string | null>(null);

    const registerBody: PostUser = {
        username: username.username,
        raw_password: password.password,
        email: email.email,
    };

    const loginBody: PostSession = {
        'User': {
            username: username.username,
            raw_password: password.password,
        },
    };

    const [method, setMethod] = useState<'POST' | undefined>(undefined);
    const api = useApi(
        type === 'register' ? '/user' : `/session`,
        type === 'register' ? registerBody : loginBody,
        method,
        false,
    );

    useEffect(() => {
        if (api.returnedOk) {
            console.debug(
                'setting session data to: ',
                username.username,
                password.password,
            );
            ctx.sessionData?.setSession(username.username, password.password);
            redirect.redirectToIndex();
        }
    }, [api.returnedOk, redirect, username.username, password.password, ctx.sessionData]);

    useEffect(() => {
        if (api.error) {
            setMethod(undefined);
        }
    }, [api.error]);

    useEffect(() => {
        if (
            password.isValid &&
            repeatPassword.isValid &&
            !(password.password === repeatPassword.password)
        ) {
            api.clearError();
            setHoleFormError("Passwords don't match");
        } else if (password.password === repeatPassword.password) {
            setHoleFormError(null);
        }
    }, [password, repeatPassword, api]);

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
        setHoleFormError(null);
        api.clearError();
        Keyboard.dismiss();
        setMethod('POST');
    };

    const loginFields: FieldConfig[] = [
        {
            placeholder: 'Username',
            value: username.username,
            onChangeText: username.setUsername,
            error: username.error,
        },
        {
            placeholder: 'Password',
            value: password.password,
            onChangeText: password.setPassword,
            secureTextEntry: true,
            error: password.error,
        },
    ];

    const registerFields: FieldConfig[] = [
        {
            placeholder: 'Email',
            value: email.email,
            onChangeText: email.setEmail,
            error: email.error,
        },
        ...loginFields,
        {
            placeholder: 'Repeat Password',
            value: repeatPassword.password,
            onChangeText: repeatPassword.setPassword,
            secureTextEntry: true,
            error: repeatPassword.error,
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
                <ErrorBox
                    error={
                        api.error
                            ? api.error[1]?.status === 401
                                ? 'Invalid Credentials'
                                : api.formattedError
                            : null
                    }
                ></ErrorBox>
                <ErrorBox error={holeFormError}></ErrorBox>
                <Button title={oppositeType()} onPress={() => setType(oppositeType())} />
            </ThemedView>
        </SafeAreaView>
    );
}
