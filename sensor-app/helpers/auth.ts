import { SessionData } from '@/persistence/SessionData';
import * as api from '@/api/auth';
import { ApiId } from '@/bindings/ApiId';

export const register = async (
    username: string,
    email: string,
    hashed_password: string,
) => {
    const register = await api.register({
        username,
        hashed_password,
        email,
    });

    if ((register as ApiId).id) {
        const sessionData = await SessionData.create();
        await sessionData.setSession((register as ApiId).id, hashed_password, username);
        return 'Ok';
    } else if (
        register === 'EmailUsed' ||
        register === 'UsernameUsed' ||
        register === 'HashInvalid'
    ) {
        return register;
    } else {
        console.warn("'Connection Error on register: '", register);
        return 'Connection Error';
    }
};

export const login = async (username: string, hashed_password: string) => {
    const login = await api.login({ username, hashed_password });

    if ((login as ApiId).id) {
        const sessionData = await SessionData.create();
        await sessionData.setSession((login as ApiId).id, hashed_password, username);
        console.debug(`all login session items set`);
        return 'Ok';
    } else if (login === 'Unauthorized') {
        return 'Unauthorized';
    } else {
        console.warn("'Connection Error on login: '", login);
        return 'Connection Error';
    }
};
