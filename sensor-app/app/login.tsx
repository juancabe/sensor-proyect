import Form, { FieldConfig } from '@/components/ui-elements/ThemedForm';
import { useEffect, useMemo, useState } from 'react';
import { Keyboard } from 'react-native';
// import * as auth from '@/helpers/auth';
import ErrorBox from '@/components/ui-elements/ErrorBox';
import useApi from '@/hooks/useApi';
import { useApiUsername } from '@/hooks/api/useApiUsername';
import { useApiRawPassword } from '@/hooks/api/useApiRawPassword';
import { useApiEmail } from '@/hooks/api/useApiEmail';
import { PostUser } from '@/bindings/api/endpoints/user/PostUser';
import { PostSession } from '@/bindings/api/endpoints/session/PostSession';
import { useAppContext } from '@/components/AppProvider';
import { Card } from '@/ui/components/Card';
import { Box, Text } from '@/ui/theme';
import { Button } from '@/ui/components/Button';
import BackgroundView from '@/components/ui-elements/BackgroundView';
import { ApiRawPassword } from '@/bindings/api/types/ApiRawPassword';
import { ApiUsername } from '@/bindings/api/types/ApiUsername';
import { Redirect, useRouter } from 'expo-router';

export default function Login() {
    const ctx = useAppContext();

    const [type, setType] = useState<'Register' | 'Login'>('Login');

    const username = useApiUsername();
    const password = useApiRawPassword();
    const repeatPassword = useApiRawPassword();
    const email = useApiEmail();

    const [triedInvalidField, setTriedInvalidField] = useState<boolean>(false);

    const [holeFormError, setHoleFormError] = useState<string | null>(null);

    const body = useMemo(() => {
        if (type === 'Register') {
            let body: PostUser = {
                username: username.username,
                raw_password: password.password,
                email: email.email,
            };
            return body;
        } else {
            let body: PostSession = {
                'User': {
                    username: username.username,
                    raw_password: password.password,
                },
            };
            return body;
        }
    }, [type, email.email, password.password, username.username]);

    const endpoint = useMemo(() => {
        return type === 'Register' ? '/user' : `/session`;
    }, [type]);

    const [method, setMethod] = useState<'POST' | undefined>(undefined);
    const api = useApi(endpoint, method, false, body);

    const router = useRouter();

    useEffect(() => {
        const fn = async () => {
            if (api.returnedOk === true) {
                console.log(
                    'setting session data to: ',
                    username.username,
                    password.password,
                );
                await ctx.sessionData?.setSession(username.username, password.password);
                router.replace('/');
                return;
            } else if (api.returnedOk === false) {
                setMethod(undefined);
            }
        };
        fn();
    }, [api.returnedOk, username.username, password.password, ctx.sessionData, router]);

    useEffect(() => {
        if (
            !(type === 'Login') &&
            password.isValid &&
            (repeatPassword.password.length > 0 || triedInvalidField) &&
            !(password.password === repeatPassword.password)
        ) {
            setHoleFormError("Passwords don't match");
        } else if (
            type === 'Login' ||
            password.password === repeatPassword.password ||
            (repeatPassword.password.length <= 0 && !triedInvalidField)
        ) {
            setHoleFormError(null);
        }
    }, [type, password, repeatPassword, triedInvalidField]);

    const validFormInput = useMemo(() => {
        const loginValid = username.isValid && password.isValid;
        const registerValid =
            loginValid &&
            repeatPassword.isValid &&
            email.isValid &&
            password.password === repeatPassword.password;

        return type === 'Login' ? loginValid : registerValid;
    }, [
        email.isValid,
        password.isValid,
        password.password,
        repeatPassword.isValid,
        repeatPassword.password,
        type,
        username.isValid,
    ]);

    const isSubmissionDisabled = useMemo(() => {
        console.log('api.loading: ', api.loading);
        if (api.loading) return true;
        if (!triedInvalidField) return false;
        return !validFormInput;
    }, [api.loading, validFormInput, triedInvalidField]);

    const oppositeType = () => {
        if (type === 'Login') {
            return 'Register';
        } else {
            return 'Login';
        }
    };

    const handleSubmission = () => {
        if (!triedInvalidField) {
            setTriedInvalidField(true);
            if (!validFormInput) {
                console.log('the form is invalid');
                return;
            } else {
                console.log('the form is valid');
            }
        }

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
            error: triedInvalidField || type === 'Register' ? username.error : undefined,
        },
        {
            placeholder: 'Password',
            value: password.password,
            onChangeText: password.setPassword,
            secureTextEntry: true,
            error: triedInvalidField || type === 'Register' ? password.error : undefined,
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
        },
    ];

    return (
        <BackgroundView>
            <Box
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
                <Text variant="title">Sensor App</Text>
                <Box
                    flex={1}
                    alignContent="center"
                    justifyContent="space-between"
                    flexDirection="column"
                    style={{ marginTop: '7%', marginBottom: '7%' }}
                >
                    <Card variant="elevated" style={{ minWidth: 300 }}>
                        <Form fields={type === 'Login' ? loginFields : registerFields} />
                        <Button
                            variant="positive"
                            label={type}
                            disabled={isSubmissionDisabled}
                            onPress={handleSubmission}
                        />
                    </Card>
                    {api.error && (
                        <ErrorBox
                            error={
                                api.error.error?.status === 401
                                    ? 'Invalid Credentials'
                                    : api.formattedError
                            }
                        ></ErrorBox>
                    )}
                    {holeFormError && <ErrorBox error={holeFormError}></ErrorBox>}
                </Box>
                <Button
                    variant="warning"
                    label={oppositeType()}
                    onPress={() => setType(oppositeType())}
                />
            </Box>
        </BackgroundView>
    );
}
