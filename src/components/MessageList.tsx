import { formatMessage } from "../lib/i18n";
import { useLocale } from "../context/LocaleContext";
import { useScrollToBottom } from "../hooks/useScrollToBottom";
import { MessageContent } from "./MessageContent";
import { personalityDisplayName } from "./PersonalityCards";
import "./ChatScreen.css";
import "./MessageContent.css";

export interface Message {
  id: string;
  role: "user" | "companion";
  content: string;
  label?: string;
  className?: string;
  streaming?: boolean;
}

interface MessageListProps {
  messages: Message[];
  personalityId?: string;
}

export function MessageList({ messages, personalityId = "luna" }: MessageListProps) {
  const { locale, translate } = useLocale();
  const companionName = personalityDisplayName(locale, personalityId);
  const scrollKey = messages.reduce(
    (sum, message) => sum + message.content.length,
    messages.length,
  );
  const endRef = useScrollToBottom<HTMLDivElement>(scrollKey);

  return (
    <div
      className="message-list"
      role="log"
      aria-live="polite"
      aria-relevant="additions"
      aria-label={formatMessage(locale, "messageListLabel", {
        companion: companionName,
        you: translate("you"),
      })}
    >
      {messages.map((message) => (
        <article
          key={message.id}
          className={`message message--${message.role}${message.className ? ` message--${message.className}` : ""}`}
          data-personality={message.role === "companion" ? personalityId : undefined}
        >
          {message.label && (
            <span className="message__label">{message.label}</span>
          )}
          <MessageContent
            content={message.content}
            plain={message.role === "user" || message.className === "typing"}
            streaming={message.streaming || message.className === "streaming"}
          />
        </article>
      ))}
      <div ref={endRef} className="message-list__anchor" aria-hidden="true" />
    </div>
  );
}
