import { useState } from "react";

/** The browser's notification permission, or `unsupported` without the API. */
function permission(): NotificationPermission | "unsupported" {
  return typeof Notification === "undefined" ? "unsupported" : Notification.permission;
}

/**
 * Asks for notifications, as wallago/todo does. On Android they are what puts
 * the unread count on the app's icon. Asking needs a tap, so it's a button,
 * not a prompt on start.
 */
export function NotifyRow({ onNotify }: { onNotify: () => void }) {
  const [state, setState] = useState(permission);
  if (state === "unsupported" || state === "granted") return null;
  if (state === "denied") {
    return (
      <p className="muted notify">
        Notifications are blocked: allow them in the browser's site settings to see new briefings
        there.
      </p>
    );
  }
  return (
    <p className="notify">
      <button
        type="button"
        onClick={async () => {
          const answer = await Notification.requestPermission();
          setState(answer);
          if (answer === "granted") onNotify();
        }}
      >
        Show new briefings as notifications
      </button>
    </p>
  );
}
