import { invoke } from "@tauri-apps/api/core";

export interface AppError {
  type: string;
  message: string;
}

export function isAppError(err: unknown): err is AppError {
  return (
    typeof err === "object" &&
    err !== null &&
    "type" in err &&
    typeof (err as AppError).type === "string" &&
    ("message" in err ? typeof (err as AppError).message === "string" : true)
  );
}

function normalizeError(err: unknown): AppError {
  if (isAppError(err)) {
    return {
      type: err.type,
      message: err.message || "操作失败",
    };
  }
  return {
    type: "unknown",
    message: typeof err === "string" ? err : String(err),
  };
}

export async function invokeCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    throw normalizeError(err);
  }
}
