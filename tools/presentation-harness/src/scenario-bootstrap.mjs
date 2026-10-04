export function scenarioBootstrapScript(fixture) {
  return `(() => {
    const fixture = ${JSON.stringify(fixture)};
    const scenarios = fixture.scenarios;
    let activeScenario = "powerBefore";
    let pendingScenario = null;
    const calls = [];
    const invoke = async (command, args = {}) => {
      calls.push({ command, args });
      if (command === "formal_snapshot") return fixture.ipc.formalSnapshot;
      if (command === "formal_list_save_slots") return fixture.ipc.formalListSaveSlots || [];
      if (command === "formal_new") { activeScenario = "powerBefore"; return fixture.ipc.formalNew; }
      if (command === "formal_interact") { activeScenario = "powerAfter"; return fixture.ipc.formalInteractPower; }
      if (command === "formal_submit_input") {
        const selected = pendingScenario || activeScenario;
        pendingScenario = null;
        activeScenario = selected;
        const receipt = fixture.ipc.formalSubmitInput?.[selected];
        if (!receipt) throw new Error("E_FIXTURE_INPUT_OUTCOME_MISSING:" + selected);
        window.__PRESENTATION_HARNESS__.lastAppliedScenario = selected;
        return receipt;
      }
      throw new Error("E_FIXTURE_IPC_UNMAPPED:" + command);
    };
    window.__PRESENTATION_SCENARIO_FIXTURE__ = fixture;
    window.__PRESENTATION_HARNESS__ = {
      source: "committed-scenario-runner-fixture",
      calls,
      renderRecords: [],
      lastAppliedScenario: null,
      stageScenario(name) {
        if (!["gateBefore", "gateAfter"].includes(name)) throw new Error("E_FIXTURE_STAGE_UNSUPPORTED:" + name);
        if (!scenarios[name]?.view) throw new Error("E_FIXTURE_SCENARIO_MISSING:" + name);
        pendingScenario = name;
      }
    };
    window.__TAURI__ = { core: { invoke } };
  })();`;
}
