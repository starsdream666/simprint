import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import {
  getServerEndpointState,
  resetServerBaseUrl,
  setServerBaseUrl,
  type ServerEndpointState,
} from '../../../../services/store/src';

interface UseServerEndpointReturn {
  endpointInput: string;
  setEndpointInput: (value: string) => void;
  endpointState: ServerEndpointState | null;
  isLoading: boolean;
  isSaving: boolean;
  hasUnsavedChanges: boolean;
  applyEndpoint: () => Promise<ServerEndpointState>;
  resetEndpoint: () => Promise<ServerEndpointState>;
}

export function useServerEndpoint(): UseServerEndpointReturn {
  const { t } = useTranslation('auth');
  const [endpointState, setEndpointState] = useState<ServerEndpointState | null>(null);
  const [endpointInput, setEndpointInput] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  const loadState = useCallback(async () => {
    setIsLoading(true);
    try {
      const state = await getServerEndpointState();
      setEndpointState(state);
      setEndpointInput(state.customBaseUrl || state.defaultBaseUrl);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t('login.server.loadFailed');
      toast.error(message);
    } finally {
      setIsLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void loadState();
  }, [loadState]);

  const hasUnsavedChanges = useMemo(() => {
    if (!endpointState) {
      return false;
    }

    const baseline = (endpointState.customBaseUrl || endpointState.defaultBaseUrl).trim();
    return endpointInput.trim() !== baseline;
  }, [endpointInput, endpointState]);

  const applyEndpoint = useCallback(async () => {
    setIsSaving(true);
    try {
      const nextState = await setServerBaseUrl(endpointInput);
      setEndpointState(nextState);
      setEndpointInput(nextState.customBaseUrl || nextState.defaultBaseUrl);
      toast.success(t('login.server.saveSuccess'));
      return nextState;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t('login.server.saveFailed');
      toast.error(message);
      throw error;
    } finally {
      setIsSaving(false);
    }
  }, [endpointInput, t]);

  const resetEndpoint = useCallback(async () => {
    setIsSaving(true);
    try {
      const nextState = await resetServerBaseUrl();
      setEndpointState(nextState);
      setEndpointInput(nextState.defaultBaseUrl);
      toast.success(t('login.server.resetSuccess'));
      return nextState;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t('login.server.resetFailed');
      toast.error(message);
      throw error;
    } finally {
      setIsSaving(false);
    }
  }, [t]);

  return {
    endpointInput,
    setEndpointInput,
    endpointState,
    isLoading,
    isSaving,
    hasUnsavedChanges,
    applyEndpoint,
    resetEndpoint,
  };
}
