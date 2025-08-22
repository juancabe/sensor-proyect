import { useAppContext } from '@/components/AppProvider';
import { useRouter } from 'expo-router';

export default function useRedirect() {
    const ctx = useAppContext();
    const router = useRouter();

    const redirectToIndex = () => {
        ctx.reloadSummary();
        router.replace('/');
    };

    return { redirectToIndex };
}
