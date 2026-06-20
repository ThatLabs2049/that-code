import { useEffect, useRef } from "react";

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function useScrollToBottom<T extends HTMLElement>(dependency: unknown) {
  const endRef = useRef<T>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({
      behavior: prefersReducedMotion() ? "auto" : "smooth",
      block: "end",
    });
  }, [dependency]);

  return endRef;
}
