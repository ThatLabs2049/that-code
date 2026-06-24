import { useEffect, useState } from "react";
import { clearCompletedTasks, listQueuedTasks, type QueuedTask } from "../lib/taskQueue";
import { useLocale } from "../context/LocaleContext";
import "./TaskQueuePanel.css";

interface TaskQueuePanelProps {
  conversationId: string | null;
  refreshKey?: number;
  sending?: boolean;
}

function statusLabel(
  status: string,
  translate: (key: import("../lib/i18n").MessageKey) => string,
): string {
  switch (status) {
    case "pending":
      return translate("taskQueuePending");
    case "running":
      return translate("taskQueueRunning");
    case "done":
      return translate("taskQueueDone");
    case "error":
      return translate("taskQueueError");
    default:
      return status;
  }
}

export function TaskQueuePanel({ conversationId, refreshKey = 0, sending = false }: TaskQueuePanelProps) {
  const { translate } = useLocale();
  const [tasks, setTasks] = useState<QueuedTask[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!conversationId) {
      setTasks([]);
      return;
    }

    let cancelled = false;
    setLoading(true);

    void listQueuedTasks(conversationId)
      .then((items) => {
        if (!cancelled) setTasks(items);
      })
      .catch(() => {
        if (!cancelled) setTasks([]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [conversationId, refreshKey, sending]);

  const visible = tasks.length > 0;
  if (!visible && !loading) return null;

  async function handleClear() {
    if (!conversationId) return;
    await clearCompletedTasks(conversationId);
    const items = await listQueuedTasks(conversationId);
    setTasks(items);
  }

  const hasCompleted = tasks.some((t) => t.status === "done" || t.status === "error");

  return (
    <section className="task-queue" aria-labelledby="task-queue-title">
      <div className="task-queue__header">
        <h3 id="task-queue-title" className="task-queue__title">
          {translate("taskQueueTitle")}
        </h3>
        {hasCompleted && (
          <button type="button" className="task-queue__clear" onClick={() => void handleClear()}>
            {translate("taskQueueClear")}
          </button>
        )}
      </div>
      {loading && tasks.length === 0 ? (
        <p className="task-queue__status" role="status">
          {translate("taskQueueLoading")}
        </p>
      ) : (
        <ol className="task-queue__list">
          {tasks.map((task) => (
            <li
              key={task.id}
              className={`task-queue__item task-queue__item--${task.status}`}
            >
              <span className="task-queue__status-badge">
                {statusLabel(task.status, translate)}
              </span>
              <span className="task-queue__objective" title={task.taskSpec.objective}>
                {task.taskSpec.objective}
              </span>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
