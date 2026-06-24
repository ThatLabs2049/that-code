import { memo } from "react";
import { useLocale } from "../context/LocaleContext";
import { MessageContent } from "./MessageContent";
import type { Message } from "./MessageList";

interface MessageItemProps {
  message: Message;
}

export const MessageItem = memo(function MessageItem({ message }: MessageItemProps) {
  const { translate } = useLocale();
  const plain = message.role === "user" || message.className === "typing";
  const streaming = Boolean(message.streaming || message.className === "streaming");

  return (
    <article
      className={`message message--${message.role}${message.className ? ` message--${message.className}` : ""}`}
    >
      <span className="message__label">
        {message.role === "assistant" ? translate("assistant") : translate("you")}
      </span>
      <MessageContent content={message.content} plain={plain} streaming={streaming} />
    </article>
  );
});
