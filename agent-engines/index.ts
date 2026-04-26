import { SecurityGuard } from "./security_guard";
import { JobQueue } from "./job_queue";
import { trackActivity, checkUser } from "./anomaly_service";
import { DashboardService } from "./monitoring/dashboard_service";
import { OrchestratorService, NodeInfo } from "./orchestrator_service";

const guard = new SecurityGuard();
const orchestrator = new OrchestratorService();

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


export function handleUserAction(userId: string, action: string) {
  trackActivity(userId, action);

  const analysis = checkUser(userId);

  if (analysis.isBot) {
    return {
      blocked: true,
      reasons: analysis.reasons,
    };
  }

  return {
    blocked: false,
  };
}


const dashboard = new DashboardService();

export function trackAIRequest(duration: number, success: boolean) {
  dashboard.recordLatency(duration);
  dashboard.recordThroughput(1);

  if (!success) {
    dashboard.recordError();
  }
}

export function getAIHealthDashboard() {
  return dashboard.getDashboard();
}

export function registerOrchestratorNode(node: NodeInfo) {
  orchestrator.registerNode(node);
}

export function getClusterHealth() {
  return orchestrator.getClusterState();
}

export async function dispatchAITask(taskId: string, payload: any) {
  return orchestrator.dispatchTask(taskId, payload);
}