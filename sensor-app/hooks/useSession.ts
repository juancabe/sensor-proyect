import { useState, useEffect, useCallback } from 'react';
import * as SecureStore from 'expo-secure-store';

export enum SessionKeys {
  API_USER_ID = 'API_USER_ID',
  API_SENSOR_ID = 'API_SENSOR_ID',
}

export const useSession = () => {
  const [sessionData, setSessionData] = useState<Record<string, string | null>>(
    {}
  );

  // Load initial session data from SecureStore
  useEffect(() => {
    const loadSessionData = async () => {
      const keys = Object.values(SessionKeys);
      const data: Record<string, string | null> = {};
      for (const key of keys) {
        data[key] = await SecureStore.getItemAsync(key);
      }
      setSessionData(data);
    };

    loadSessionData();
  }, []);

  // Function to update a session key
  const setItem = useCallback(
    async (key: SessionKeys, value: string | null) => {
      if (value === null) {
        await SecureStore.deleteItemAsync(key);
      } else {
        await SecureStore.setItemAsync(key, value);
      }

      // Update the internal state
      setSessionData((prev) => ({
        ...prev,
        [key]: value,
      }));
    },
    []
  );

  // Function to get a session key
  const getItem = useCallback(
    (key: SessionKeys): string | null => {
      return sessionData[key] || null;
    },
    [sessionData]
  );

  return { sessionData, setItem, getItem };
};
