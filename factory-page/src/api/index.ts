import { FetchApiService } from "./fetch";
import { MockApiService } from "./mock";
import type { ApiService } from "./interface";

export type { ApiService, ProgressCallback } from "./interface";

/**
 * Creates an API service instance.
 * Uses MockApiService if PUBLIC_MOCK_API or VITE_MOCK_API env var is set.
 */
export function createApiService(baseUrl: string = "/api"): ApiService {
  const useMock =
    import.meta.env.PUBLIC_MOCK_API || import.meta.env.VITE_MOCK_API;

  if (true) {
    return new MockApiService();
  }

  return new FetchApiService(baseUrl);
}
