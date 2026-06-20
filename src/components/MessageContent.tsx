import type { ReactNode } from "react";
import { useLocale } from "../context/LocaleContext";
import "./MessageContent.css";

type Block =
  | { type: "paragraph"; content: InlineNode[] }
  | { type: "code"; language: string; content: string };

type InlineNode =
  | { type: "text"; value: string }
  | { type: "bold"; value: string }
  | { type: "code"; value: string }
  | { type: "link"; href: string; label: string };

function parseInline(text: string): InlineNode[] {
  const nodes: InlineNode[] = [];
  const pattern = /(\*\*(.+?)\*\*|`([^`]+)`|\[([^\]]+)\]\(([^)]+)\))/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      nodes.push({ type: "text", value: text.slice(lastIndex, match.index) });
    }

    if (match[2]) {
      nodes.push({ type: "bold", value: match[2] });
    } else if (match[3]) {
      nodes.push({ type: "code", value: match[3] });
    } else if (match[4] && match[5]) {
      const href = match[5].trim();
      if (/^https?:\/\//i.test(href)) {
        nodes.push({ type: "link", href, label: match[4] });
      } else {
        nodes.push({ type: "text", value: match[0] });
      }
    }

    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    nodes.push({ type: "text", value: text.slice(lastIndex) });
  }

  return nodes.length > 0 ? nodes : [{ type: "text", value: text }];
}

function parseBlocks(source: string): Block[] {
  const blocks: Block[] = [];
  const lines = source.split("\n");
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    const fenceMatch = line.match(/^```(\w*)/);

    if (fenceMatch) {
      const language = fenceMatch[1] || "";
      const codeLines: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].startsWith("```")) {
        codeLines.push(lines[i]);
        i += 1;
      }
      blocks.push({ type: "code", language, content: codeLines.join("\n") });
      i += 1;
      continue;
    }

    if (line.trim() === "") {
      i += 1;
      continue;
    }

    const paragraphLines: string[] = [line];
    i += 1;
    while (i < lines.length && lines[i].trim() !== "" && !lines[i].startsWith("```")) {
      paragraphLines.push(lines[i]);
      i += 1;
    }
    blocks.push({
      type: "paragraph",
      content: parseInline(paragraphLines.join("\n")),
    });
  }

  return blocks.length > 0 ? blocks : [{ type: "paragraph", content: parseInline(source) }];
}

async function openExternal(url: string) {
  try {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(url);
  } catch {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

function renderInline(nodes: InlineNode[]): ReactNode[] {
  return nodes.map((node, index) => {
    switch (node.type) {
      case "bold":
        return <strong key={index}>{node.value}</strong>;
      case "code":
        return (
          <code key={index} className="message-md__inline-code">
            {node.value}
          </code>
        );
      case "link":
        return (
          <a
            key={index}
            href={node.href}
            className="message-md__link"
            onClick={(event) => {
              event.preventDefault();
              void openExternal(node.href);
            }}
          >
            {node.label}
          </a>
        );
      default:
        return <span key={index}>{node.value}</span>;
    }
  });
}

interface MessageContentProps {
  content: string;
  plain?: boolean;
  streaming?: boolean;
}

export function MessageContent({ content, plain, streaming }: MessageContentProps) {
  const { translate } = useLocale();

  if (plain) {
    return (
      <p className="message__content" dir="auto">
        {content}
        {streaming && <span className="message-md__cursor" aria-hidden="true" />}
      </p>
    );
  }

  const blocks = parseBlocks(content);

  return (
    <div className="message-md" dir="auto">
      {blocks.map((block, index) => {
        if (block.type === "code") {
          return (
            <div key={index} className="message-md__code-wrap">
              <pre className="message-md__code" dir="ltr">
                <code>{block.content}</code>
              </pre>
              <button
                type="button"
                className="message-md__copy"
                aria-label={translate("copyCode")}
                onClick={() => void navigator.clipboard.writeText(block.content)}
              >
                {translate("copyCode")}
              </button>
            </div>
          );
        }

        return (
          <p key={index} className="message-md__paragraph">
            {renderInline(block.content)}
          </p>
        );
      })}
      {streaming && <span className="message-md__cursor" aria-hidden="true" />}
    </div>
  );
}
