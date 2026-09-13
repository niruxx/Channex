import { useEffect, useState } from "react";
import { fetchMediaBytes } from "../lib/api";
import { mimeForExt } from "../lib/format";

interface MediaBlobState {
  url: string | null;
  loading: boolean;
  error: string | null;
}

/**
 * Fetches a remote media file through the Rust backend and exposes it as a
 * local blob: URL, revoking the previous one whenever the source changes or
 * the component unmounts. See fetchMediaBytes for why this is necessary
 * instead of pointing an <img>/<video> straight at the remote URL.
 */
export function useMediaBlob(sourceUrl: string | null, ext: string | null): MediaBlobState {
  const [state, setState] = useState<MediaBlobState>({ url: null, loading: false, error: null });

  useEffect(() => {
    if (!sourceUrl || !ext) {
      setState({ url: null, loading: false, error: null });
      return;
    }

    let cancelled = false;
    let objectUrl: string | null = null;
    setState({ url: null, loading: true, error: null });

    fetchMediaBytes(sourceUrl)
      .then((bytes) => {
        if (cancelled) return;
        const blob = new Blob([bytes], { type: mimeForExt(ext) });
        objectUrl = URL.createObjectURL(blob);
        setState({ url: objectUrl, loading: false, error: null });
      })
      .catch((err) => {
        if (cancelled) return;
        setState({ url: null, loading: false, error: String(err) });
      });

    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [sourceUrl, ext]);

  return state;
}
