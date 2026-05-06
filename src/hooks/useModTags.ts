import { useState, useCallback, useEffect } from 'react';

const STORAGE_KEY = 'svl-mod-tags';

type ModTagsMap = Record<string, string[]>;

function loadTags(): ModTagsMap {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return JSON.parse(raw);
    }
  } catch {}
  return {};
}

function saveTags(tags: ModTagsMap) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags));
}

export function useModTags() {
  const [tagsMap, setTagsMap] = useState<ModTagsMap>(loadTags);

  useEffect(() => {
    saveTags(tagsMap);
  }, [tagsMap]);

  const getTags = useCallback(
    (uniqueId: string): string[] => {
      return tagsMap[uniqueId] || [];
    },
    [tagsMap],
  );

  const addTag = useCallback((uniqueId: string, tag: string) => {
    const trimmed = tag.trim();
    if (!trimmed) return;
    setTagsMap((prev) => {
      const current = prev[uniqueId] || [];
      if (current.includes(trimmed)) return prev;
      return { ...prev, [uniqueId]: [...current, trimmed] };
    });
  }, []);

  const removeTag = useCallback((uniqueId: string, tag: string) => {
    setTagsMap((prev) => {
      const current = prev[uniqueId];
      if (!current) return prev;
      const next = current.filter((t) => t !== tag);
      if (next.length === 0) {
        const { [uniqueId]: _, ...rest } = prev;
        return rest;
      }
      return { ...prev, [uniqueId]: next };
    });
  }, []);

  const getAllUniqueTags = useCallback((): string[] => {
    const allTags = new Set<string>();
    Object.values(tagsMap).forEach((tags) => {
      tags.forEach((t) => allTags.add(t));
    });
    return Array.from(allTags).sort();
  }, [tagsMap]);

  const searchByTag = useCallback(
    (query: string): string[] => {
      if (!query.trim()) return [];
      const lower = query.toLowerCase();
      const results: string[] = [];
      Object.entries(tagsMap).forEach(([uniqueId, tags]) => {
        if (tags.some((t) => t.toLowerCase().includes(lower))) {
          results.push(uniqueId);
        }
      });
      return results;
    },
    [tagsMap],
  );

  return { getTags, addTag, removeTag, getAllUniqueTags, searchByTag };
}
