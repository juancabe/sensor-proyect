import { fetchUserSummary } from '@/api/user_summary';
import { ApiId } from '@/bindings/ApiId';
import { UserSummary } from '@/bindings/UserSummary';
import { useCallback } from 'react';
import { SessionKeys, useSession } from './useSession';

export function useSummary() {
  const { getItem } = useSession();

  const fetchSummary = useCallback<
    () => Promise<UserSummary | 'Unauthorized' | 'Connection Error'>
  >(async () => {
    const username = await getItem(SessionKeys.USERNAME);
    const apiId = await getItem(SessionKeys.API_ID);

    if (!username || !apiId) {
      return 'Unauthorized';
    }

    const result = await fetchUserSummary(username, { id: apiId } as ApiId);

    if (typeof result === 'object' && 'summary' in result) {
      return result.summary;
    } else if (result === 'Unauthorized') {
      return 'Unauthorized';
    } else {
      console.warn('Connection Error on fetching summary:', result);
      return 'Connection Error';
    }
  }, [getItem]);

  return { fetchSummary };
}
