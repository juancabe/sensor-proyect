import { useState, useEffect, useCallback } from 'react';
import * as SecureStore from 'expo-secure-store';

export enum SessionKeys {
  USERNAME = 'USERNAME',
  HASHED_PASSWORD = 'HASHED_PASSWORD',
  API_ID = 'API_ID',
}

export const useSession = () => {
  const [sessionData, setSessionData] = useState<
    Record<string, string> | undefined | null
  >(undefined);

  const loadSessionData = async () => {
    const keys = Object.values(SessionKeys);
    const data: Record<string, string> = {};
    for (const key of keys) {
      const dataGot = await SecureStore.getItemAsync(key);
      if (dataGot !== null) {
        data[key] = dataGot;
      } else {
        setSessionData(null);
        return;
      }
    }
    setSessionData(data);
  };

  // Load initial session data from SecureStore
  useEffect(() => {
    loadSessionData();
  }, []);

  // Function to update a session key
  const setItem = useCallback(async (key: SessionKeys, value: string) => {
    await SecureStore.setItemAsync(key, value);
    loadSessionData();
  }, []);

  // Function to delete sessionData
  const deleteSession = useCallback(async () => {
    const keys = Object.values(SessionKeys);
    for (const key of keys) {
      await SecureStore.deleteItemAsync(key);
    }
    setSessionData(null);
  }, []);

  // Function to get a session key if session data is set
  const getItem = useCallback(
    (key: SessionKeys): string | null | undefined => {
      if (sessionData) return sessionData[key];
      else return sessionData;
    },
    [sessionData]
  );

  return { sessionData, setItem, getItem, deleteSession };
};
