// filepath: sensor-app/hooks/useAuth.ts
import { useCallback } from 'react';
import * as api from '@/api/auth';
import { useSession, SessionKeys } from './useSession';
import { ApiId } from '@/bindings/ApiId';
import { GetLoginResponseCode } from '@/bindings/endpoints/GetLogin';

export function useAuth() {
  const { setItem } = useSession();

  const login = useCallback(async (username: string, hashed_password: string) => {
    const login = await api.login({ username, hashed_password });
    if ((login as ApiId).id) {
      await setItem(SessionKeys.USERNAME, username);
      await setItem(SessionKeys.HASHED_PASSWORD, hashed_password);
      await setItem(SessionKeys.API_ID, (login as ApiId).id);
      return 'Ok';
    } else if (login === 'Unauthorized') {
      return 'Unauthorized';
    } else {
      console.warn("'Connection Error on login: '", login);
      return 'Connection Error';
    }
  }, []);

  const register = useCallback(
    async (username: string, email: string, hashed_password: string) => {
      const register = await api.register({
        username,
        hashed_password,
        email,
      });

      if ((register as ApiId).id) {
        await setItem(SessionKeys.USERNAME, username);
        await setItem(SessionKeys.HASHED_PASSWORD, hashed_password);
        await setItem(SessionKeys.API_ID, (register as ApiId).id);
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
    },
    []
  );

  return { login, register };
}
