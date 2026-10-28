import { GoogleGenAI } from "@google/genai";
import { parseLlmError } from "./errors.ts";

export interface CacheKeyParams {
  model: string;
  systemInstruction?: string;
  toolDeclarations?: unknown[];
  brainContent?: string;
}

export class ContextCacheManager {
  private client: GoogleGenAI;
  private cacheStore = new Map<string, { cacheName: string; expiresAt: number }>();

  constructor(client: GoogleGenAI) {
    this.client = client;
  }

  async getOrOrCreateCache(params: CacheKeyParams): Promise<string | undefined> {
    const combinedContent = [
      params.systemInstruction || "",
      params.brainContent || "",
      params.toolDeclarations ? JSON.stringify(params.toolDeclarations) : "",
    ].join("\n\n");

    // Only cache if stable prefix has significant length
    if (combinedContent.length < 1000) {
      return undefined;
    }

    const key = `${params.model}:${combinedContent.length}:${combinedContent.slice(0, 100)}`;
    const existing = this.cacheStore.get(key);
    if (existing && existing.expiresAt > Date.now()) {
      return existing.cacheName;
    }

    try {
      if (!this.client.caches || typeof this.client.caches.create !== "function") {
        return undefined;
      }

      const created = await this.client.caches.create({
        model: params.model,
        config: {
          displayName: `forge-prefix-${Date.now()}`,
          ttl: "300s",
          systemInstruction: params.systemInstruction
            ? { role: "system", parts: [{ text: params.systemInstruction }] }
            : undefined,
          contents: params.brainContent
            ? [{ role: "user", parts: [{ text: params.brainContent }] }]
            : undefined,
        },
      });

      if (created && created.name) {
        this.cacheStore.set(key, {
          cacheName: created.name,
          expiresAt: Date.now() + 290 * 1000,
        });
        return created.name;
      }
    } catch (err) {
      // Fall back silently if context caching is not supported for this model or environment
      return undefined;
    }

    return undefined;
  }
}
