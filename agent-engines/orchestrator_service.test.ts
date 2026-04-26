import { expect } from "chai";
import { OrchestratorService, NodeInfo } from "./orchestrator_service";

describe("OrchestratorService", () => {
  let service: OrchestratorService;

  beforeEach(() => {
    service = new OrchestratorService();
  });

  it("should register and unregister nodes", () => {
    const node: NodeInfo = {
      nodeId: "node-1",
      address: "localhost",
      status: "online",
      load: 0.5,
      lastSeen: new Date().toISOString(),
    };

    service.registerNode(node);
    let state = service.getClusterState();
    expect(state.nodes).to.have.lengthOf(1);
    expect(state.activeNodes).to.equal(1);

    service.unregisterNode("node-1");
    state = service.getClusterState();
    expect(state.nodes).to.have.lengthOf(0);
  });

  it("should calculate cluster state correctly", () => {
    service.registerNode({
      nodeId: "n1",
      address: "addr1",
      status: "online",
      load: 0.2,
      lastSeen: "",
    });
    service.registerNode({
      nodeId: "n2",
      address: "addr2",
      status: "online",
      load: 0.8,
      lastSeen: "",
    });

    const state = service.getClusterState();
    expect(state.totalLoad).to.equal(1.0);
    expect(state.activeNodes).to.equal(2);
  });

  it("should dispatch tasks to the least loaded node", async () => {
    service.registerNode({
      nodeId: "busy",
      address: "addr1",
      status: "online",
      load: 0.9,
      lastSeen: "",
    });
    service.registerNode({
      nodeId: "idle",
      address: "addr2",
      status: "online",
      load: 0.1,
      lastSeen: "",
    });

    const result = await service.dispatchTask("task-1", {});
    expect(result.nodeId).to.equal("idle");
    expect(result.status).to.equal("dispatched");
  });

  it("should throw error if no active nodes", async () => {
    try {
      await service.dispatchTask("t1", {});
      expect.fail("Should have thrown error");
    } catch (e: any) {
      expect(e.message).to.equal("No active nodes in cluster");
    }
  });
});
