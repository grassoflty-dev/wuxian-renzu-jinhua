import { ClockworksEnemyModel, clockworksEnemyForEvent, isClockworksEnemyCue, isClockworksEnemyMotion } from "../renderer/ClockworksEnemyModel.js";
import { ambientAudioCueForWorld, type AudioCueDefinition, type AudioTone, audioCueForPresentationEvent, AUDIO_CUE_LIBRARY, type AudioCueId } from "./AudioEventMap.js";
import type { PresentationEvent, WorldSnapshotEnvelope, Vec3 } from "../protocol/types.js";

export const AUDIO_PREFERENCES_STORAGE_KEY = "wuxian.audio-preferences.v1";
const DEFAULT_VOLUME = 0.35;
const MAX_CUE_GAIN = 0.22;

export interface AudioPreferenceState {
  muted: boolean;
  volume: number;
}

export interface AudioPreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export type AudioRuntimeState = "locked" | "running" | "unavailable";

export interface AudioCueRuntime {
  state(): AudioRuntimeState;
  unlock(): Promise<boolean>;
  play(cue: AudioCueDefinition, masterVolume: number, pan?: number, actorId?: string): boolean;
  stopActorVoices?(actorId: string): void;
  startAmbient(cue: AudioCueDefinition, masterVolume: number): boolean;
  stopVoices(): void;
  stopAmbient(): void;
  close(): void | Promise<void>;
}

export interface AudioCuePlayerOptions {
  getStorage?: () => AudioPreferenceStorage | null;
  createRuntime?: () => AudioCueRuntime | null;
}

type AudioObserver = () => void;
interface Voice {
  actorId?: string;
  sources: OscillatorNode[];
  gains: GainNode[];
  remaining: number;
  panner?: StereoPannerNode;
}

function localStorageOrNull(): AudioPreferenceStorage | null {
  try { return typeof window === "undefined" ? null : window.localStorage; }
  catch { return null; }
}

function defaultRuntime(): AudioCueRuntime | null {
  const globals = globalThis as typeof globalThis & { webkitAudioContext?: typeof AudioContext };
  const AudioContextType = globalThis.AudioContext ?? globals.webkitAudioContext;
  if (!AudioContextType) return null;
  return new WebAudioCueRuntime(new AudioContextType());
}

function clampVolume(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_VOLUME;
  return Math.min(1, Math.max(0, Math.round(value * 100) / 100));
}

function decodePreferences(raw: string): AudioPreferenceState | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== "object" || Array.isArray(value)) return null;
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    if (keys.join(",") !== "muted,schemaVersion,volume" || record.schemaVersion !== 1 ||
        typeof record.muted !== "boolean" || typeof record.volume !== "number" ||
        !Number.isFinite(record.volume) || record.volume < 0 || record.volume > 1) return null;
    return { muted: record.muted, volume: record.volume };
  } catch {
    return null;
  }
}

/** Local-only synthesized cues. Every recipe stays marked temporary until approved clips replace it. */
export class AudioCuePlayer {
  private preferences: AudioPreferenceState = { muted: false, volume: DEFAULT_VOLUME };
  private canPersist = false;
  private restored = false;
  private runtime: AudioCueRuntime | null = null;
  private runtimeStatus: "locked" | "ready" | "unsupported" | "failed" = "locked";
  private currentEpoch: number | null = null;
  private lastEventId = 0;
  private enemySnapshot: WorldSnapshotEnvelope | null = null;
  private readonly enemyCues = new ClockworksEnemyModel();
  private listener: { epoch: number; position: Vec3 } | null = null;
  private scene: { worldId: string; sceneId: string } | null = null;
  private active = true;
  private disposed = false;
  private readonly observers = new Set<AudioObserver>();

  constructor(private readonly options: AudioCuePlayerOptions = {}) {
    this.restore();
  }

  get temporary_audio(): true { return true; }

  get settings(): AudioPreferenceState { return { ...this.preferences }; }

  get statusText(): string {
    const audioState = this.runtime?.state();
    const output = this.preferences.muted ? "声音已静音。"
      : audioState === "unavailable" ? "音频输出不可用；请检查系统声音设备后重试。"
        : audioState === "locked" && this.runtimeStatus === "ready" ? "浏览器已暂停音频；再次操作以恢复。"
          : this.runtimeStatus === "ready" ? "临时合成音频已就绪。"
        : this.runtimeStatus === "unsupported" ? "当前环境不支持 Web Audio；游戏仍可继续。"
          : this.runtimeStatus === "failed" ? "音频输出不可用；请检查系统声音设备后重试。"
            : "首次操作后启用浏览器音频。";
    return this.canPersist ? output : `${output} 设置仅保存在本次运行。`;
  }

  subscribe(observer: AudioObserver): () => void {
    this.observers.add(observer);
    observer();
    return () => this.observers.delete(observer);
  }

  setMuted(muted: boolean): void {
    if (typeof muted !== "boolean" || muted === this.preferences.muted) return;
    this.preferences = { ...this.preferences, muted };
    if (muted) {
      this.runtime?.stopVoices();
      this.runtime?.stopAmbient();
    } else {
      this.refreshAmbient();
    }
    this.persist();
    this.notify();
  }

  setVolume(volume: number): void {
    const next = clampVolume(volume);
    if (next === this.preferences.volume) return;
    this.preferences = { ...this.preferences, volume: next };
    this.persist();
    this.refreshAmbient();
    this.notify();
  }

  async unlock(): Promise<boolean> {
    if (this.disposed || this.preferences.muted) return false;
    if (!this.runtime && this.runtimeStatus === "locked") {
      try {
        this.runtime = (this.options.createRuntime ?? defaultRuntime)();
        if (!this.runtime) {
          this.runtimeStatus = "unsupported";
          this.notify();
          return false;
        }
      } catch {
        this.runtimeStatus = "failed";
        this.notify();
        return false;
      }
    }
    if (!this.runtime) return false;
    try {
      const running = await this.runtime.unlock();
      this.runtimeStatus = running && this.runtime.state() === "running" ? "ready" : "failed";
      if (running) this.refreshAmbient();
      this.notify();
      return running;
    } catch {
      this.runtimeStatus = "failed";
      this.notify();
      return false;
    }
  }

  playCue(id: AudioCueId, pan = 0, actorId?: string, remainingMs?: number): boolean {
    if (!this.canPlay()) return false;
    try {
      const recipe = AUDIO_CUE_LIBRARY[id];
      const cue = remainingMs === undefined ? recipe : { ...recipe, tones: recipe.tones
        .map(tone => ({ ...tone, durationMs: Math.min(tone.durationMs, remainingMs - tone.offsetMs) }))
        .filter(tone => tone.durationMs >= 12) };
      if (!cue.tones.length) return false;
      const played = this.runtime!.play(cue, this.preferences.volume, Number.isFinite(pan) ? Math.max(-1,Math.min(1,pan)) : 0, actorId);
      if (!played) this.failOutput();
      return played;
    } catch {
      this.failOutput();
      return false;
    }
  }

  playUiConfirm(): boolean { return this.playCue("audio.ui.confirm"); }
  playUiError(): boolean { return this.playCue("audio.ui.error"); }
  playUiCancel(): boolean { return this.playCue("audio.ui.cancel"); }

  setEpoch(epoch: number): void {
    if (!Number.isSafeInteger(epoch) || epoch < 0 || (this.currentEpoch !== null && epoch <= this.currentEpoch)) return;
    this.currentEpoch = epoch;
    this.enemySnapshot = null;
    this.enemyCues.reset();
    this.listener = null;
    this.lastEventId = 0;
    this.runtime?.stopVoices();
  }

  setListenerSnapshot(snapshot: WorldSnapshotEnvelope): void {
    const prior = this.enemySnapshot;
    if (snapshot.protocolVersion === 3 && Number.isSafeInteger(snapshot.worldEpoch) &&
        (this.currentEpoch === null || snapshot.worldEpoch >= this.currentEpoch)) {
      this.setEpoch(snapshot.worldEpoch);
      const sameContext = prior?.protocolVersion === 3 && prior.worldEpoch === snapshot.worldEpoch &&
        prior.worldId === snapshot.worldId && prior.sceneId === snapshot.sceneId;
      if (sameContext && snapshot.serverTick < prior.serverTick) return;
      if (prior?.protocolVersion === 3) {
        for (const actor of Array.isArray(prior.actors) ? prior.actors : []) {
          if (!actor) continue;
          const current = (Array.isArray(snapshot.actors) ? snapshot.actors : []).filter(item => item?.entityId === actor.entityId);
          if (!sameContext || current.length !== 1 || current[0]?.entityType !== actor.entityType || current[0]?.active !== actor.active) {
            this.runtime?.stopActorVoices?.(actor.entityId);
          }
        }
      }
      this.enemySnapshot = snapshot;
      this.enemyCues.project(snapshot, true);
    } else {
      this.enemySnapshot = null;
      this.enemyCues.reset();
      if (prior) this.runtime?.stopVoices();
    }
    if(snapshot.protocolVersion!==3||snapshot.worldId!=="mist_harbor"||snapshot.sceneId!=="mh_warden_arena") { this.listener=null;return; }
    if(this.currentEpoch!==null&&snapshot.worldEpoch<this.currentEpoch){this.listener=null;return;}
    this.setEpoch(snapshot.worldEpoch);
    const p=snapshot.player.transform.positionM;
    this.listener=Number.isSafeInteger(snapshot.worldEpoch)&&[p.xM,p.yM,p.zM].every(Number.isFinite)
      ? {epoch:snapshot.worldEpoch,position:{...p}}:null;
  }

  handlePresentationEvents(events: readonly PresentationEvent[]): number {
    if (this.disposed || !Array.isArray(events)) return 0;
    let played = 0;
    const authorizedEnemyIds = new Set(this.enemySnapshot ? this.enemyCues.accept(events, this.enemySnapshot) : []);
    for (const event of events) {
      if (!event || event.protocolVersion !== 2 || !Number.isSafeInteger(event.worldEpoch) || event.worldEpoch < 0 ||
          !Number.isSafeInteger(event.eventId) || event.eventId <= 0) continue;
      if (isClockworksEnemyCue(event.kind) && (!this.enemySnapshot || !clockworksEnemyForEvent(event, this.enemySnapshot))) continue;
      if (this.currentEpoch === null || event.worldEpoch > this.currentEpoch) this.setEpoch(event.worldEpoch);
      if (event.worldEpoch !== this.currentEpoch || event.eventId <= this.lastEventId) continue;
      this.lastEventId = event.eventId;
      let remainingMs: number | undefined;
      if (isClockworksEnemyCue(event.kind)) {
        if (!this.enemySnapshot || !clockworksEnemyForEvent(event, this.enemySnapshot)) continue;
        if (!authorizedEnemyIds.has(event.eventId)) continue;
        if (event.actorId && event.kind !== "EnemyAlert") this.runtime?.stopActorVoices?.(event.actorId);
        if (isClockworksEnemyMotion(event.kind)) continue;
        remainingMs = event.durationMs! - (this.enemySnapshot.serverTick - event.serverTick) * 1000 / 60;
      }
      const cue = audioCueForPresentationEvent(event);
      let pan=0;
      if(event.kind.startsWith("ResonanceWarden")){
        const listener=this.listener;
        if(!listener||listener.epoch!==event.worldEpoch||!event.positionM||
          ![event.positionM.xM,event.positionM.yM,event.positionM.zM].every(Number.isFinite)) continue;
        const dx=event.positionM.xM-listener.position.xM,dz=event.positionM.zM-listener.position.zM;
        const distance=Math.hypot(dx,dz);pan=distance>0?(dx-dz)/(Math.SQRT2*distance):0;
      }
      if (cue && this.playCue(cue.id, pan, event.actorId, remainingMs)) played++;
    }
    return played;
  }

  setScene(worldId: string, sceneId: string): void {
    if (!worldId || !sceneId || (this.scene?.worldId === worldId && this.scene.sceneId === sceneId)) return;
    this.runtime?.stopVoices();
    this.scene = { worldId, sceneId };
    this.refreshAmbient();
  }

  clearScene(): void {
    if (!this.scene) return;
    this.scene = null;
    this.enemySnapshot = null;
    this.enemyCues.reset();
    this.runtime?.stopVoices();
    this.runtime?.stopAmbient();
  }

  suspend(): void {
    if (!this.active) return;
    this.active = false;
    this.runtime?.stopVoices();
    this.runtime?.stopAmbient();
    this.notify();
  }

  resume(): void {
    if (this.disposed || this.active) return;
    this.active = true;
    this.refreshAmbient();
    this.notify();
  }

  stop(): void {
    this.enemySnapshot = null;
    this.enemyCues.reset();
    this.runtime?.stopVoices();
    this.runtime?.stopAmbient();
  }

  async dispose(): Promise<void> {
    if (this.disposed) return;
    this.disposed = true;
    this.active = false;
    const runtime = this.runtime;
    this.runtime = null;
    try { await runtime?.close(); } catch { /* Audio teardown must not block page exit. */ }
    this.notify();
  }

  private restore(): void {
    this.restored = true;
    const getStorage = this.options.getStorage ?? localStorageOrNull;
    try {
      const storage = getStorage();
      if (!storage) return;
      const raw = storage.getItem(AUDIO_PREFERENCES_STORAGE_KEY);
      if (raw === null) {
        this.canPersist = true;
        return;
      }
      const saved = decodePreferences(raw);
      if (!saved) return;
      this.preferences = saved;
      this.canPersist = true;
    } catch {
      this.canPersist = false;
    }
  }

  private persist(): void {
    if (!this.restored || !this.canPersist) return;
    try {
      const storage = (this.options.getStorage ?? localStorageOrNull)();
      if (!storage) { this.canPersist = false; return; }
      storage.setItem(AUDIO_PREFERENCES_STORAGE_KEY, JSON.stringify({
        schemaVersion: 1,
        muted: this.preferences.muted,
        volume: this.preferences.volume,
      }));
    } catch {
      this.canPersist = false;
    }
  }

  private canPlay(): boolean {
    return !this.disposed && this.active && !this.preferences.muted &&
      this.preferences.volume > 0 && this.runtimeStatus === "ready" && this.runtime?.state() === "running";
  }

  private refreshAmbient(): void {
    if (!this.runtime) return;
    this.runtime.stopAmbient();
    if (!this.active || this.preferences.muted || this.preferences.volume <= 0 ||
        this.runtimeStatus !== "ready" || this.runtime.state() !== "running" || !this.scene) return;
    const { worldId, sceneId } = this.scene;
    const exactHub = worldId === "return_station" && sceneId === "rs_core_room";
    const greyHiveScene = worldId === "grey_hive" && sceneId.startsWith("gh_");
    if (!exactHub && !greyHiveScene) return;
    const cue = ambientAudioCueForWorld(worldId);
    if (!cue) return;
    try {
      if (!this.runtime.startAmbient(cue, this.preferences.volume)) this.failOutput();
    } catch {
      this.failOutput();
    }
  }

  private failOutput(): void {
    this.runtimeStatus = "failed";
    this.runtime?.stopVoices();
    this.runtime?.stopAmbient();
    this.notify();
  }

  private notify(): void {
    for (const observer of this.observers) observer();
  }
}

class WebAudioCueRuntime implements AudioCueRuntime {
  private readonly voices = new Set<Voice>();
  private ambient: Voice | null = null;

  constructor(private readonly context: AudioContext) {
  }

  state(): AudioRuntimeState {
    return this.context.state === "running" ? "running"
      : this.context.state === "suspended" || this.context.state === "interrupted" ? "locked" : "unavailable";
  }

  async unlock(): Promise<boolean> {
    if (this.context.state !== "running") await this.context.resume();
    return this.context.state === "running";
  }

  play(cue: AudioCueDefinition, masterVolume: number, pan = 0, actorId?: string): boolean {
    if (this.context.state !== "running" || cue.temporary_audio !== true || !cue.tones.length) return false;
    const voice: Voice = { sources: [], gains: [], remaining: cue.tones.length, ...(actorId ? { actorId } : {}) };
    const now = this.context.currentTime;
    if(typeof this.context.createStereoPanner === "function"){
      voice.panner=this.context.createStereoPanner();voice.panner.pan.setValueAtTime(Math.max(-1,Math.min(1,pan)),now);voice.panner.connect(this.context.destination);
    }
    for (const item of cue.tones) this.addTone(voice, item, now, cue.peakGain * masterVolume, false);
    this.voices.add(voice);
    return true;
  }

  startAmbient(cue: AudioCueDefinition, masterVolume: number): boolean {
    if (this.context.state !== "running" || cue.temporary_audio !== true || !cue.tones.length) return false;
    this.stopAmbient();
    const voice: Voice = { sources: [], gains: [], remaining: cue.tones.length };
    const now = this.context.currentTime;
    for (const item of cue.tones) this.addTone(voice, item, now, cue.peakGain * masterVolume, true);
    this.ambient = voice;
    return true;
  }

  stopActorVoices(actorId: string): void {
    for (const voice of this.voices) {
      if (voice.actorId !== actorId) continue;
      this.fadeAndStop(voice, 0.025);
      this.voices.delete(voice);
    }
  }

  stopVoices(): void {
    for (const voice of this.voices) this.fadeAndStop(voice, 0.045);
    this.voices.clear();
  }

  stopAmbient(): void {
    if (!this.ambient) return;
    this.fadeAndStop(this.ambient, 0.2);
    this.ambient = null;
  }

  async close(): Promise<void> {
    this.stopVoices();
    this.stopAmbient();
    if (this.context.state !== "closed") await this.context.close();
  }

  private addTone(voice: Voice, tone: AudioTone, now: number, peak: number, loop: boolean): void {
    const oscillator = this.context.createOscillator();
    const gain = this.context.createGain();
    oscillator.type = tone.waveform;
    oscillator.frequency.setValueAtTime(tone.frequencyHz, now + tone.offsetMs / 1000);
    if (tone.endFrequencyHz) oscillator.frequency.exponentialRampToValueAtTime(tone.endFrequencyHz, now + (tone.offsetMs + tone.durationMs) / 1000);
    gain.gain.setValueAtTime(0.0001, now);
    if (loop) {
      gain.gain.setTargetAtTime(Math.min(MAX_CUE_GAIN, Math.max(0, peak)), now + 0.015, 0.35);
    } else {
      const start = now + tone.offsetMs / 1000;
      const end = start + tone.durationMs / 1000;
      const safePeak = Math.min(MAX_CUE_GAIN, Math.max(0.0002, peak));
      gain.gain.setValueAtTime(0.0001, start);
      gain.gain.linearRampToValueAtTime(safePeak, start + 0.009);
      gain.gain.setValueAtTime(safePeak, Math.max(start + 0.01, end - 0.018));
      gain.gain.exponentialRampToValueAtTime(0.0001, end);
    }
    oscillator.connect(gain);
    gain.connect(voice.panner??this.context.destination);
    oscillator.onended = () => {
      voice.remaining--;
      oscillator.disconnect();gain.disconnect();
      if (voice.remaining <= 0) {this.voices.delete(voice);voice.panner?.disconnect();}
    };
    oscillator.start(now);
    if (!loop) oscillator.stop(now + (tone.offsetMs + tone.durationMs) / 1000 + 0.025);
    voice.sources.push(oscillator);
    voice.gains.push(gain);
  }

  private fadeAndStop(voice: Voice, fadeSeconds: number): void {
    const now = this.context.currentTime;
    for (let index = 0; index < voice.sources.length; index++) {
      const source = voice.sources[index];
      const gain = voice.gains[index];
      if (!source || !gain) continue;
      try {
        gain.gain.cancelScheduledValues(now);
        gain.gain.setTargetAtTime(0.0001, now, Math.max(0.008, fadeSeconds / 4));
        source.stop(now + fadeSeconds);
      } catch { /* A source can finish between a lifecycle event and this stop. */ }
    }
  }
}
