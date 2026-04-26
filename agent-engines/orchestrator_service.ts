export interface NodeInfo {
  nodeId: string;
  address: string;
  status: string;
  load: number;
  lastSeen: string;
}

export interface ClusterState {
  nodes: NodeInfo[];
  totalLoad: number;
  activeNodes: number;
}

export class OrchestratorService {
  private nodes: Map<string, NodeInfo> = new Map();

  constructor() {
    // In a real app, this might connect to a discovery service or a backend API
  }

  registerNode(node: NodeInfo): void {
    this.nodes.set(node.nodeId, node);
  }

  unregisterNode(nodeId: string): void {
    this.nodes.delete(nodeId);
  }

  getClusterState(): ClusterState {
    const nodes = Array.from(this.nodes.values());
    const totalLoad = nodes.reduce((sum, node) => sum + node.load, 0);
    const activeNodes = nodes.filter((n) => n.status === "online").length;

    return {
      nodes,
      totalLoad,
      activeNodes,
    };
  }

  async dispatchTask(taskId: string, payload: any): Promise<{ nodeId: string; status: string }> {
    const state = this.getClusterState();
    if (state.activeNodes === 0) {
      throw new Error("No active nodes in cluster");
    }

    // Find least loaded node
    const bestNode = state.nodes.sort((a, b) => a.load - b.load)[0];

    // Simulate task dispatch
    console.log(`Dispatching task ${taskId} to node ${bestNode.nodeId}`);
    
    return {
      nodeId: bestNode.nodeId,
      status: "dispatched",
    };
  }
}
