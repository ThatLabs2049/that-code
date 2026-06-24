import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useLocale } from "../context/LocaleContext";
import { MessageItem } from "./MessageItem";
import "./ChatScreen.css";
import "./MessageContent.css";

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  className?: string;
  streaming?: boolean;
}

interface MessageListProps {
  messages: Message[];
}

const VIRTUALIZE_THRESHOLD = 48;
const MESSAGE_GAP_PX = 16;
const ESTIMATED_MESSAGE_HEIGHT = 96;

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function scrollContentLength(messages: Message[]): number {
  return messages.reduce((sum, message) => sum + message.content.length, messages.length);
}

function StaticMessageList({
  messages,
  scrollKey,
}: {
  messages: Message[];
  scrollKey: number;
}) {
  const { translate } = useLocale();
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({
      behavior: prefersReducedMotion() ? "auto" : "smooth",
      block: "end",
    });
  }, [scrollKey]);

  return (
    <div
      className="message-list"
      role="log"
      aria-live="polite"
      aria-relevant="additions"
      aria-label={translate("messageListLabel")}
    >
      {messages.map((message) => (
        <MessageItem key={message.id} message={message} />
      ))}
      <div ref={endRef} className="message-list__anchor" aria-hidden="true" />
    </div>
  );
}

function VirtualizedMessageList({
  messages,
  scrollKey,
}: {
  messages: Message[];
  scrollKey: number;
}) {
  const { translate } = useLocale();
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ESTIMATED_MESSAGE_HEIGHT,
    gap: MESSAGE_GAP_PX,
    overscan: 8,
  });
  const virtualizerRef = useRef(virtualizer);
  virtualizerRef.current = virtualizer;

  useEffect(() => {
    if (messages.length === 0) return;
    virtualizerRef.current.scrollToIndex(messages.length - 1, {
      align: "end",
      behavior: prefersReducedMotion() ? "auto" : "smooth",
    });
  }, [scrollKey, messages.length]);

  return (
    <div
      ref={parentRef}
      className="message-list message-list--virtual"
      role="log"
      aria-live="polite"
      aria-relevant="additions"
      aria-label={translate("messageListLabel")}
    >
      <div
        className="message-list__virtual-track"
        style={{ height: `${virtualizer.getTotalSize()}px` }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const message = messages[virtualRow.index];
          return (
            <div
              key={message.id}
              ref={virtualizer.measureElement}
              data-index={virtualRow.index}
              className="message-list__virtual-row"
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              <MessageItem message={message} />
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function MessageList({ messages }: MessageListProps) {
  const scrollKey = scrollContentLength(messages);
  const useVirtual = messages.length >= VIRTUALIZE_THRESHOLD;

  if (useVirtual) {
    return <VirtualizedMessageList messages={messages} scrollKey={scrollKey} />;
  }

  return <StaticMessageList messages={messages} scrollKey={scrollKey} />;
}
