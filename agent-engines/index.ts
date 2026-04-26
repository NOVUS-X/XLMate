import { SecurityGuard } from "./security_guard";
import { JobQueue } from "./job_queue";

const guard = new SecurityGuard();

export function processPrompt(input: string) {
  if (!guard.validatePrompt(input)) {
    throw new Error("Unsafe prompt detected");
  }

  const sanitized = guard.sanitize(input);

  return {
    processed: sanitized,
  };
}


const queue = new JobQueue();

export function runAIAnalysis(payload: any) {
  queue.enqueue({
    id: Date.now().toString(),
    payload,
  });

  return {
    status: "queued",
  };
}